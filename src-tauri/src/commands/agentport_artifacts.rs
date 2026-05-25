use crate::core::{agentport_env, error::AppError, skill_store::SkillStore};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use tauri::State;

#[derive(Debug, Clone, Serialize)]
pub struct AgentPortArtifactsStatus {
    pub manifest_path: String,
    pub manifest_exists: bool,
    pub artifact_count: usize,
    pub skill_count: usize,
    pub resource_count: usize,
    pub package_owned_count: usize,
    pub drift_count: usize,
    pub artifacts: Vec<AgentPortArtifactSummary>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentPortArtifactSummary {
    pub id: String,
    pub kind: String,
    pub name: String,
    pub path: String,
    pub status: String,
    pub source_type: String,
    pub source_confidence: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_hash: Option<String>,
    pub target_count: usize,
    pub targets: Vec<AgentPortArtifactTargetSummary>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentPortArtifactTargetSummary {
    pub tool: String,
    pub path: String,
    pub mode: String,
    pub status: String,
}

pub fn agentport_artifacts_for_store(
    store: &SkillStore,
) -> anyhow::Result<AgentPortArtifactsStatus> {
    let manifest_path = agentport_env::manifest_path();
    let manifest_exists = manifest_path.exists();
    let expected_manifest = if manifest_exists {
        agentport_env::read_manifest()?
    } else {
        agentport_env::build_manifest_from_store(store)?
    };
    let current_manifest = agentport_env::build_manifest_from_store(store)?;
    let current_by_id = current_manifest
        .artifacts
        .into_iter()
        .map(|artifact| (artifact.id.clone(), artifact))
        .collect::<BTreeMap<_, _>>();
    let diff = if manifest_exists {
        Some(agentport_env::diff_current_environment(store, None)?)
    } else {
        None
    };
    let missing = diff
        .as_ref()
        .map(|diff| {
            diff.missing_artifacts
                .iter()
                .cloned()
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    let unmanaged = diff
        .as_ref()
        .map(|diff| {
            diff.unmanaged_artifacts
                .iter()
                .cloned()
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    let changed = diff
        .as_ref()
        .map(|diff| {
            diff.changed_artifacts
                .iter()
                .map(|drift| (drift.id.clone(), drift.clone()))
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();

    let mut artifacts = Vec::new();
    let mut seen = BTreeSet::new();
    for expected in expected_manifest.artifacts {
        let id = expected.id.clone();
        let current = current_by_id.get(&id);
        let display = current.unwrap_or(&expected);
        let drift = changed.get(&id);
        let status = if missing.contains(&id) {
            "missing"
        } else if drift.is_some() {
            "drifted"
        } else {
            "synced"
        };
        artifacts.push(summary_from_artifact(display, status, drift));
        seen.insert(id);
    }

    for id in unmanaged {
        if seen.contains(&id) {
            continue;
        }
        if let Some(artifact) = current_by_id.get(&id) {
            artifacts.push(summary_from_artifact(artifact, "unmanaged", None));
            seen.insert(id);
        }
    }

    artifacts.sort_by(|a, b| a.id.cmp(&b.id));
    let skill_count = artifacts
        .iter()
        .filter(|artifact| artifact.kind == "skill")
        .count();
    let package_owned_count = artifacts
        .iter()
        .filter(|artifact| artifact.owner_type.as_deref() == Some("package"))
        .count();
    let drift_count = artifacts
        .iter()
        .filter(|artifact| artifact.status != "synced")
        .count();

    Ok(AgentPortArtifactsStatus {
        manifest_path: manifest_path.to_string_lossy().to_string(),
        manifest_exists,
        artifact_count: artifacts.len(),
        skill_count,
        resource_count: artifacts.len().saturating_sub(skill_count),
        package_owned_count,
        drift_count,
        artifacts,
    })
}

fn summary_from_artifact(
    artifact: &agentport_env::EnvArtifact,
    status: &str,
    drift: Option<&agentport_env::EnvArtifactDrift>,
) -> AgentPortArtifactSummary {
    AgentPortArtifactSummary {
        id: artifact.id.clone(),
        kind: artifact.kind.clone(),
        name: artifact.name.clone(),
        path: artifact.path.clone(),
        status: status.to_string(),
        source_type: artifact.source.source_type.clone(),
        source_confidence: artifact.source.confidence.clone(),
        package_id: artifact.source.package.clone(),
        owner_type: artifact
            .owner
            .as_ref()
            .map(|owner| owner.owner_type.clone()),
        owner_id: artifact.owner.as_ref().map(|owner| owner.id.clone()),
        expected_hash: drift.and_then(|drift| drift.expected_hash.clone()),
        current_hash: drift.and_then(|drift| drift.current_hash.clone()),
        target_count: artifact.deployed_to.len(),
        targets: artifact
            .deployed_to
            .iter()
            .map(|target| AgentPortArtifactTargetSummary {
                tool: target.tool.clone(),
                path: target.path.clone(),
                mode: target.mode.clone(),
                status: target.status.clone(),
            })
            .collect(),
    }
}

#[tauri::command]
pub async fn agentport_artifacts_status(
    store: State<'_, Arc<SkillStore>>,
) -> Result<AgentPortArtifactsStatus, AppError> {
    let store = store.inner().clone();
    tokio::task::spawn_blocking(move || {
        agentport_artifacts_for_store(&store).map_err(AppError::internal)
    })
    .await?
}

#[cfg(test)]
mod tests {
    use crate::commands::agentport_artifacts::agentport_artifacts_for_store;
    use crate::core::{
        agentport_env, central_repo,
        skill_store::{SkillRecord, SkillStore, SkillTargetRecord},
    };
    use std::fs;

    fn sample_skill(
        id: &str,
        name: &str,
        source_type: &str,
        central_path: &std::path::Path,
    ) -> SkillRecord {
        SkillRecord {
            id: id.to_string(),
            name: name.to_string(),
            description: None,
            source_type: source_type.to_string(),
            source_ref: Some("https://github.com/example/pack.git".to_string()),
            source_ref_resolved: Some("https://github.com/example/pack.git".to_string()),
            source_subpath: Some(format!("skills/{name}")),
            source_branch: Some("main".to_string()),
            source_revision: Some("abc123".to_string()),
            remote_revision: None,
            central_path: central_path.to_string_lossy().to_string(),
            content_hash: Some(format!("hash-{id}")),
            enabled: true,
            created_at: 1,
            updated_at: 1,
            status: "ok".to_string(),
            update_status: "up_to_date".to_string(),
            last_checked_at: None,
            last_check_error: None,
        }
    }

    #[test]
    fn artifact_status_merges_expected_missing_and_unmanaged_current_artifacts() {
        let _guard = central_repo::test_base_dir_lock();
        let tmp = tempfile::tempdir().unwrap();
        central_repo::set_test_base_dir_override(Some(tmp.path().join("state")));

        let store = SkillStore::new(&tmp.path().join("test.db")).unwrap();
        let alpha_dir = tmp.path().join("central/alpha");
        fs::create_dir_all(&alpha_dir).unwrap();
        fs::write(alpha_dir.join("SKILL.md"), "---\nname: alpha\n---\n").unwrap();
        store
            .insert_skill(&sample_skill("skill-alpha", "alpha", "git", &alpha_dir))
            .unwrap();
        store
            .insert_target(&SkillTargetRecord {
                id: "target-alpha".to_string(),
                skill_id: "skill-alpha".to_string(),
                tool: "codex".to_string(),
                target_path: tmp
                    .path()
                    .join("codex/skills/alpha")
                    .to_string_lossy()
                    .to_string(),
                mode: "copy".to_string(),
                status: "synced".to_string(),
                synced_at: Some(1),
                last_error: None,
                source_hash: Some("hash-skill-alpha".to_string()),
            })
            .unwrap();

        let manifest = agentport_env::build_manifest_from_store(&store).unwrap();
        fs::create_dir_all(central_repo::skills_dir()).unwrap();
        fs::write(
            agentport_env::manifest_path(),
            serde_yaml::to_string(&manifest).unwrap(),
        )
        .unwrap();
        store.delete_skill("skill-alpha").unwrap();

        let beta_dir = tmp.path().join("central/beta");
        fs::create_dir_all(&beta_dir).unwrap();
        fs::write(beta_dir.join("SKILL.md"), "---\nname: beta\n---\n").unwrap();
        store
            .insert_skill(&sample_skill(
                "skill-beta",
                "beta",
                "local_created",
                &beta_dir,
            ))
            .unwrap();

        let status = agentport_artifacts_for_store(&store).unwrap();
        let alpha = status
            .artifacts
            .iter()
            .find(|artifact| artifact.id == "skill:skill-alpha")
            .unwrap();
        let beta = status
            .artifacts
            .iter()
            .find(|artifact| artifact.id == "skill:skill-beta")
            .unwrap();

        assert!(status.manifest_exists);
        assert!(status.artifact_count >= 2);
        assert!(status.skill_count >= 2);
        assert_eq!(status.package_owned_count, 1);
        assert!(status.drift_count >= 2);
        assert_eq!(alpha.status, "missing");
        assert_eq!(alpha.source_type, "package_artifact");
        assert_eq!(
            alpha.package_id.as_deref(),
            Some("git:https://github.com/example/pack")
        );
        assert_eq!(alpha.target_count, 1);
        assert_eq!(beta.status, "unmanaged");
        assert_eq!(beta.source_type, "local_created");

        central_repo::set_test_base_dir_override(None);
    }
}
