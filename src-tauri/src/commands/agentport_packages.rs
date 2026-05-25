use crate::core::{
    agentport_env,
    error::AppError,
    skill_store::{SkillRecord, SkillStore},
};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use tauri::State;

#[derive(Debug, Clone, Serialize)]
pub struct AgentPortPackagesStatus {
    pub package_count: usize,
    pub unpackaged_skill_count: usize,
    pub packages: Vec<AgentPortPackageSummary>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentPortPackageSummary {
    pub id: String,
    pub source: agentport_env::EnvPackageSource,
    pub artifact_ids: Vec<String>,
    pub artifact_count: usize,
    pub skill_count: usize,
    pub update_available_count: usize,
    pub managed_skill_names: Vec<String>,
}

pub fn agentport_packages_for_store(store: &SkillStore) -> anyhow::Result<AgentPortPackagesStatus> {
    let packages = agentport_env::packages_from_manifest_or_store(store)?;
    let skills = store.get_all_skills()?;
    let mut skills_by_package: BTreeMap<String, Vec<&SkillRecord>> = BTreeMap::new();
    let mut unpackaged_skill_count = 0usize;

    for skill in &skills {
        if let Some(package_id) = agentport_env::package_id_for_skill_record(skill) {
            skills_by_package.entry(package_id).or_default().push(skill);
        } else {
            unpackaged_skill_count += 1;
        }
    }

    let listed_package_ids: BTreeSet<String> =
        packages.iter().map(|package| package.id.clone()).collect();
    let unmanaged_packaged_skill_count = skills_by_package
        .iter()
        .filter(|(package_id, _)| !listed_package_ids.contains(*package_id))
        .map(|(_, skills)| skills.len())
        .sum::<usize>();

    let summaries = packages
        .into_iter()
        .map(|package| {
            let package_skills = skills_by_package.remove(&package.id).unwrap_or_default();
            let update_available_count = package_skills
                .iter()
                .filter(|skill| skill.update_status == "update_available")
                .count();
            let managed_skill_names = package_skills
                .iter()
                .map(|skill| skill.name.clone())
                .collect::<Vec<_>>();
            let artifact_count = package.artifacts.len();

            AgentPortPackageSummary {
                id: package.id,
                source: package.source,
                artifact_ids: package.artifacts,
                artifact_count,
                skill_count: package_skills.len(),
                update_available_count,
                managed_skill_names,
            }
        })
        .collect::<Vec<_>>();

    Ok(AgentPortPackagesStatus {
        package_count: summaries.len(),
        unpackaged_skill_count: unpackaged_skill_count + unmanaged_packaged_skill_count,
        packages: summaries,
    })
}

#[tauri::command]
pub async fn agentport_packages_status(
    store: State<'_, Arc<SkillStore>>,
) -> Result<AgentPortPackagesStatus, AppError> {
    let store = store.inner().clone();
    tokio::task::spawn_blocking(move || {
        agentport_packages_for_store(&store).map_err(AppError::internal)
    })
    .await?
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{
        central_repo,
        skill_store::{SkillRecord, SkillStore},
    };
    use std::fs;

    fn sample_git_skill(id: &str, name: &str, central_path: &std::path::Path) -> SkillRecord {
        SkillRecord {
            id: id.to_string(),
            name: name.to_string(),
            description: None,
            source_type: "git".to_string(),
            source_ref: Some("https://github.com/example/pack.git".to_string()),
            source_ref_resolved: Some("https://github.com/example/pack.git".to_string()),
            source_subpath: Some("skills/demo".to_string()),
            source_branch: Some("main".to_string()),
            source_revision: Some("abc123".to_string()),
            remote_revision: Some("def456".to_string()),
            central_path: central_path.to_string_lossy().to_string(),
            content_hash: Some("hash-1".to_string()),
            enabled: true,
            created_at: 1,
            updated_at: 1,
            status: "ok".to_string(),
            update_status: "update_available".to_string(),
            last_checked_at: None,
            last_check_error: None,
        }
    }

    #[test]
    fn package_status_groups_store_skills_when_manifest_is_missing() {
        let _guard = central_repo::test_base_dir_lock();
        let tmp = tempfile::tempdir().unwrap();
        central_repo::set_test_base_dir_override(Some(tmp.path().join("state")));

        let store = SkillStore::new(&tmp.path().join("test.db")).unwrap();
        let skill_dir = tmp.path().join("central/demo");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(skill_dir.join("SKILL.md"), "---\nname: demo\n---\n").unwrap();
        store
            .insert_skill(&sample_git_skill("skill-1", "demo", &skill_dir))
            .unwrap();

        let status = agentport_packages_for_store(&store).unwrap();

        assert_eq!(status.package_count, 1);
        assert_eq!(status.unpackaged_skill_count, 0);
        assert_eq!(status.packages[0].id, "git:https://github.com/example/pack");
        assert_eq!(status.packages[0].skill_count, 1);
        assert_eq!(status.packages[0].update_available_count, 1);
        assert_eq!(
            status.packages[0].source.revision.as_deref(),
            Some("abc123")
        );
        assert_eq!(
            status.packages[0].source.remote_revision.as_deref(),
            Some("def456")
        );

        central_repo::set_test_base_dir_override(None);
    }
}
