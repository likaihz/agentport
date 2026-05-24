use crate::core::{
    agentport_env,
    error::AppError,
    scenario_service,
    skill_store::{ScenarioRecord, SkillStore},
};
use serde::Serialize;
use std::sync::Arc;
use tauri::State;

#[derive(Debug, Clone, Serialize)]
pub struct AgentPortEnvStatus {
    pub doctor: agentport_env::EnvDoctorReport,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diff: Option<agentport_env::EnvDiffReport>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diff_error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentPortEnvApplyReport {
    pub ok: bool,
    pub profile_id: String,
    pub profile_name: String,
    pub dry_run: bool,
    pub missing_skills: Vec<String>,
    pub targets: Vec<scenario_service::SyncPreviewTarget>,
    pub resources: Vec<agentport_env::EnvResourceActionItem>,
    pub applied: bool,
}

pub fn agentport_env_status_for_store(
    store: &SkillStore,
    profile: Option<&str>,
) -> anyhow::Result<AgentPortEnvStatus> {
    let doctor = agentport_env::doctor(store)?;
    let (diff, diff_error) = if doctor.manifest_exists {
        match agentport_env::diff_current_environment(store, profile) {
            Ok(report) => (Some(report), None),
            Err(err) => (None, Some(err.to_string())),
        }
    } else {
        (None, None)
    };

    Ok(AgentPortEnvStatus {
        doctor,
        diff,
        diff_error,
    })
}

pub fn agentport_env_init_for_store(
    store: &SkillStore,
    overwrite: bool,
) -> anyhow::Result<agentport_env::EnvWriteReport> {
    agentport_env::write_current_environment(store, overwrite)
}

pub fn agentport_env_export_resources_for_store(
    store: &SkillStore,
    overwrite: bool,
    dry_run: bool,
) -> anyhow::Result<agentport_env::EnvResourceActionReport> {
    let report = agentport_env::export_resource_artifacts(store, overwrite, dry_run)?;
    if !dry_run && report.ok {
        agentport_env::write_current_environment(store, true)?;
    }
    Ok(report)
}

pub fn agentport_env_apply_resources_for_store(
    store: &SkillStore,
    dry_run: bool,
) -> anyhow::Result<agentport_env::EnvResourceActionReport> {
    let manifest = agentport_env::read_manifest()?;
    agentport_env::apply_resource_artifacts(store, &manifest, dry_run)
}

pub fn agentport_env_apply_for_store(
    store: &SkillStore,
    profile_ref: Option<&str>,
    dry_run: bool,
) -> anyhow::Result<AgentPortEnvApplyReport> {
    let manifest = agentport_env::read_manifest()?;
    let profile = agentport_env::select_profile(&manifest, profile_ref)
        .ok_or_else(|| anyhow::anyhow!("profile not found in agentport.yaml"))?;
    let diff = agentport_env::diff_current_environment(store, Some(&profile.id))?;
    let preset =
        resolve_scenario(store, &profile.id).or_else(|_| resolve_scenario(store, &profile.name))?;
    let targets = scenario_service::preview_scenario_sync(store, &preset.id).map_err(app_error)?;
    let planned_resources = agentport_env::apply_resource_artifacts(store, &manifest, true)?;

    if !diff.missing_skills.is_empty() {
        return Ok(AgentPortEnvApplyReport {
            ok: false,
            profile_id: profile.id.clone(),
            profile_name: profile.name.clone(),
            dry_run,
            missing_skills: diff.missing_skills,
            targets,
            resources: planned_resources.items,
            applied: false,
        });
    }

    let resources = if dry_run {
        planned_resources
    } else {
        scenario_service::apply_scenario_to_default(store, &preset.id).map_err(app_error)?;
        agentport_env::apply_resource_artifacts(store, &manifest, false)?
    };

    Ok(AgentPortEnvApplyReport {
        ok: resources.ok,
        profile_id: profile.id.clone(),
        profile_name: profile.name.clone(),
        dry_run,
        missing_skills: Vec::new(),
        targets,
        resources: resources.items,
        applied: !dry_run,
    })
}

#[tauri::command]
pub async fn agentport_env_status(
    store: State<'_, Arc<SkillStore>>,
    profile: Option<String>,
) -> Result<AgentPortEnvStatus, AppError> {
    let store = store.inner().clone();
    tokio::task::spawn_blocking(move || {
        agentport_env_status_for_store(&store, profile.as_deref()).map_err(AppError::internal)
    })
    .await?
}

#[tauri::command]
pub async fn agentport_env_init(
    store: State<'_, Arc<SkillStore>>,
    overwrite: bool,
) -> Result<agentport_env::EnvWriteReport, AppError> {
    let store = store.inner().clone();
    tokio::task::spawn_blocking(move || {
        agentport_env_init_for_store(&store, overwrite).map_err(AppError::internal)
    })
    .await?
}

#[tauri::command]
pub async fn agentport_env_export_resources(
    store: State<'_, Arc<SkillStore>>,
    overwrite: bool,
    dry_run: bool,
) -> Result<agentport_env::EnvResourceActionReport, AppError> {
    let store = store.inner().clone();
    tokio::task::spawn_blocking(move || {
        agentport_env_export_resources_for_store(&store, overwrite, dry_run)
            .map_err(AppError::internal)
    })
    .await?
}

#[tauri::command]
pub async fn agentport_env_apply_resources(
    store: State<'_, Arc<SkillStore>>,
    dry_run: bool,
) -> Result<agentport_env::EnvResourceActionReport, AppError> {
    let store = store.inner().clone();
    tokio::task::spawn_blocking(move || {
        agentport_env_apply_resources_for_store(&store, dry_run).map_err(AppError::internal)
    })
    .await?
}

#[tauri::command]
pub async fn agentport_env_apply(
    store: State<'_, Arc<SkillStore>>,
    profile: Option<String>,
    dry_run: bool,
) -> Result<AgentPortEnvApplyReport, AppError> {
    let store = store.inner().clone();
    tokio::task::spawn_blocking(move || {
        agentport_env_apply_for_store(&store, profile.as_deref(), dry_run)
            .map_err(AppError::internal)
    })
    .await?
}

fn resolve_scenario(store: &SkillStore, reference: &str) -> anyhow::Result<ScenarioRecord> {
    let scenarios = store.get_all_scenarios()?;
    if reference == "current" {
        let active = store
            .get_active_scenario_id()?
            .ok_or_else(|| anyhow::anyhow!("no active preset"))?;
        return scenarios
            .into_iter()
            .find(|scenario| scenario.id == active)
            .ok_or_else(|| anyhow::anyhow!("active preset not found"));
    }

    let matches: Vec<_> = scenarios
        .into_iter()
        .filter(|scenario| scenario.id == reference || scenario.name == reference)
        .collect();
    match matches.len() {
        1 => Ok(matches.into_iter().next().unwrap()),
        0 => Err(anyhow::anyhow!("preset not found: {reference}")),
        _ => Err(anyhow::anyhow!(
            "preset reference is ambiguous: {reference}"
        )),
    }
}

fn app_error(err: AppError) -> anyhow::Error {
    anyhow::anyhow!(err.message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{agentport_env, central_repo, skill_store::SkillStore};

    #[test]
    fn status_reports_doctor_even_before_manifest_exists() {
        let _guard = central_repo::test_base_dir_lock();
        let tmp = tempfile::tempdir().unwrap();
        central_repo::set_test_base_dir_override(Some(tmp.path().join("state")));

        let store = SkillStore::new(&tmp.path().join("test.db")).unwrap();
        let status = agentport_env_status_for_store(&store, None).unwrap();

        assert!(!status.doctor.manifest_exists);
        assert!(status.diff.is_none());
        assert!(status.diff_error.is_none());
        assert_eq!(
            status.doctor.manifest_path,
            agentport_env::manifest_path().to_string_lossy()
        );

        central_repo::set_test_base_dir_override(None);
    }

    #[test]
    fn init_writes_environment_files_and_status_reads_them() {
        let _guard = central_repo::test_base_dir_lock();
        let tmp = tempfile::tempdir().unwrap();
        central_repo::set_test_base_dir_override(Some(tmp.path().join("state")));

        let store = SkillStore::new(&tmp.path().join("test.db")).unwrap();
        let report = agentport_env_init_for_store(&store, true).unwrap();
        let status = agentport_env_status_for_store(&store, None).unwrap();

        assert_eq!(
            report.manifest_path,
            agentport_env::manifest_path().to_string_lossy()
        );
        assert!(status.doctor.manifest_exists);
        assert!(status.doctor.lock_exists);
        assert!(status.diff.is_some());
        assert!(status.diff_error.is_none());

        central_repo::set_test_base_dir_override(None);
    }
}
