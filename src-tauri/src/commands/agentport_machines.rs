use crate::core::{agentport_env, error::AppError, skill_store::SkillStore};
use anyhow::Context;
use serde::Serialize;
use std::fs;
use std::sync::Arc;
use tauri::State;

#[derive(Debug, Clone, Serialize)]
pub struct AgentPortMachineStatus {
    pub machine_dir: String,
    pub machine_local_path: String,
    pub secrets_local_path: String,
    pub backups_dir: String,
    pub machine_dir_exists: bool,
    pub machine_local_exists: bool,
    pub secrets_local_exists: bool,
    pub backups_dir_exists: bool,
    pub gitignore_protected: bool,
    pub origin_count: usize,
    pub secret_count: usize,
    pub missing_secret_count: usize,
    pub backup_count: usize,
    pub origins: Vec<AgentPortMachineOriginSummary>,
    pub secret_ids: Vec<String>,
    pub missing_secret_ids: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentPortMachineOriginSummary {
    pub id: String,
    pub path: String,
    pub updated_at: String,
}

pub fn agentport_machine_status_for_store(
    store: &SkillStore,
) -> anyhow::Result<AgentPortMachineStatus> {
    let machine_dir = agentport_env::machine_dir();
    let machine_local_path = agentport_env::machine_local_path();
    let secrets_local_path = agentport_env::secrets_local_path();
    let backups_dir = agentport_env::machine_backups_dir();
    let local_overlay = read_machine_local_overlay_if_present()?;
    let secrets_overlay = read_machine_secrets_overlay_if_present()?;
    let mut origins = local_overlay
        .origins
        .into_iter()
        .map(|(id, origin)| AgentPortMachineOriginSummary {
            id,
            path: origin.path,
            updated_at: origin.updated_at,
        })
        .collect::<Vec<_>>();
    origins.sort_by(|a, b| a.id.cmp(&b.id));

    let mut secret_ids = secrets_overlay.secrets.into_keys().collect::<Vec<_>>();
    secret_ids.sort();

    let doctor = agentport_env::doctor(store)?;
    let mut missing_secret_ids = doctor
        .warnings
        .iter()
        .filter_map(|warning| warning.strip_prefix("missing machine-local secret: "))
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    missing_secret_ids.sort();

    Ok(AgentPortMachineStatus {
        machine_dir: machine_dir.to_string_lossy().to_string(),
        machine_local_path: machine_local_path.to_string_lossy().to_string(),
        secrets_local_path: secrets_local_path.to_string_lossy().to_string(),
        backups_dir: backups_dir.to_string_lossy().to_string(),
        machine_dir_exists: machine_dir.is_dir(),
        machine_local_exists: machine_local_path.is_file(),
        secrets_local_exists: secrets_local_path.is_file(),
        backups_dir_exists: backups_dir.is_dir(),
        gitignore_protected: machine_gitignore_protected(),
        origin_count: origins.len(),
        secret_count: secret_ids.len(),
        missing_secret_count: missing_secret_ids.len(),
        backup_count: backup_count(&backups_dir)?,
        origins,
        secret_ids,
        missing_secret_ids,
        warnings: doctor.warnings,
    })
}

fn read_machine_local_overlay_if_present() -> anyhow::Result<agentport_env::MachineLocalOverlay> {
    let path = agentport_env::machine_local_path();
    read_overlay_file(&path)
}

fn read_machine_secrets_overlay_if_present() -> anyhow::Result<agentport_env::MachineSecretsOverlay>
{
    let path = agentport_env::secrets_local_path();
    read_overlay_file(&path)
}

fn read_overlay_file<T>(path: &std::path::Path) -> anyhow::Result<T>
where
    T: Default + serde::de::DeserializeOwned,
{
    if !path.exists() {
        return Ok(T::default());
    }
    let raw =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    if raw.trim().is_empty() {
        return Ok(T::default());
    }
    serde_yaml::from_str(&raw).with_context(|| format!("failed to parse {}", path.display()))
}

fn machine_gitignore_protected() -> bool {
    let path = crate::core::central_repo::skills_dir().join(".gitignore");
    let Ok(content) = fs::read_to_string(path) else {
        return false;
    };
    [
        "/machine/machine.local.yaml",
        "/machine/secrets.local.yaml",
        "/machine/backups/",
    ]
    .iter()
    .all(|entry| content.lines().any(|line| line.trim() == *entry))
}

fn backup_count(backups_dir: &std::path::Path) -> anyhow::Result<usize> {
    if !backups_dir.is_dir() {
        return Ok(0);
    }
    Ok(fs::read_dir(backups_dir)?.filter_map(Result::ok).count())
}

#[tauri::command]
pub async fn agentport_machine_status(
    store: State<'_, Arc<SkillStore>>,
) -> Result<AgentPortMachineStatus, AppError> {
    let store = store.inner().clone();
    tokio::task::spawn_blocking(move || {
        agentport_machine_status_for_store(&store).map_err(AppError::internal)
    })
    .await?
}

#[cfg(test)]
mod tests {
    use crate::commands::agentport_machines::agentport_machine_status_for_store;
    use crate::core::{
        agentport_env, central_repo,
        skill_store::{SkillRecord, SkillStore},
    };
    use std::fs;

    fn sample_skill(id: &str, central_path: &std::path::Path) -> SkillRecord {
        SkillRecord {
            id: id.to_string(),
            name: id.to_string(),
            description: None,
            source_type: "local_linked".to_string(),
            source_ref: Some("personal/review".to_string()),
            source_ref_resolved: None,
            source_subpath: None,
            source_branch: None,
            source_revision: None,
            remote_revision: None,
            central_path: central_path.to_string_lossy().to_string(),
            content_hash: Some("hash-1".to_string()),
            enabled: true,
            created_at: 1,
            updated_at: 1,
            status: "ok".to_string(),
            update_status: "local_only".to_string(),
            last_checked_at: None,
            last_check_error: None,
        }
    }

    #[test]
    fn machine_status_counts_overlays_without_exposing_secret_values() {
        let _guard = central_repo::test_base_dir_lock();
        let tmp = tempfile::tempdir().unwrap();
        central_repo::set_test_base_dir_override(Some(tmp.path().join("state")));

        let store = SkillStore::new(&tmp.path().join("test.db")).unwrap();
        let skill_dir = tmp.path().join("central/review");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(skill_dir.join("SKILL.md"), "---\nname: review\n---\n").unwrap();
        store
            .insert_skill(&sample_skill("review", &skill_dir))
            .unwrap();

        agentport_env::record_linked_origin_path("personal/review", &tmp.path().join("origin"))
            .unwrap();
        fs::write(
            agentport_env::secrets_local_path(),
            serde_yaml::to_string(&agentport_env::MachineSecretsOverlay {
                secrets: [(
                    "secret/api_key".to_string(),
                    agentport_env::MachineSecretValue {
                        value: "super-secret-token".to_string(),
                        updated_at: "2026-05-25T00:00:00Z".to_string(),
                    },
                )]
                .into_iter()
                .collect(),
            })
            .unwrap(),
        )
        .unwrap();
        fs::create_dir_all(agentport_env::machine_backups_dir()).unwrap();
        fs::write(
            agentport_env::machine_backups_dir().join("one.bak"),
            "backup",
        )
        .unwrap();

        let status = agentport_machine_status_for_store(&store).unwrap();
        let rendered = serde_json::to_string(&status).unwrap();

        assert!(status.machine_dir_exists);
        assert!(status.machine_local_exists);
        assert!(status.secrets_local_exists);
        assert!(status.backups_dir_exists);
        assert!(status.gitignore_protected);
        assert_eq!(status.origin_count, 1);
        assert_eq!(status.secret_count, 1);
        assert_eq!(status.backup_count, 1);
        assert!(status
            .origins
            .iter()
            .any(|origin| origin.id == "personal/review"));
        assert!(status.secret_ids.iter().any(|id| id == "secret/api_key"));
        assert!(!rendered.contains("super-secret-token"));

        central_repo::set_test_base_dir_override(None);
    }
}
