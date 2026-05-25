use crate::commands::skills as skill_commands;
use crate::core::{
    agentport_env, content_hash,
    error::AppError,
    installer,
    repo_lock::RepoLock,
    skill_store::{SkillRecord, SkillStore, SkillTargetRecord},
    sync_engine, sync_metadata,
};
use anyhow::{bail, Context};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::State;

#[derive(Debug, Clone, Serialize)]
pub struct AgentPortTargetDriftStatus {
    pub target_count: usize,
    pub drift_count: usize,
    pub targets: Vec<AgentPortTargetDrift>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentPortTargetDrift {
    pub skill_id: String,
    pub skill_name: String,
    pub tool: String,
    pub target_path: String,
    pub mode: String,
    pub status: String,
    pub central_hash: Option<String>,
    pub target_hash: Option<String>,
    pub last_synced_hash: Option<String>,
    pub can_pull: bool,
    pub can_discard: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentPortTargetActionReport {
    pub ok: bool,
    pub action: String,
    pub skill_id: String,
    pub skill_name: String,
    pub tool: String,
    pub target_path: String,
    pub mode: String,
}

pub fn agentport_target_drifts_for_store(
    store: &SkillStore,
) -> anyhow::Result<AgentPortTargetDriftStatus> {
    let skills = store
        .get_all_skills()?
        .into_iter()
        .map(|skill| (skill.id.clone(), skill))
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut target_count = 0usize;
    let mut targets = Vec::new();

    for target in store.get_all_targets()? {
        if target.mode != "copy" {
            continue;
        }
        target_count += 1;
        let Some(skill) = skills.get(&target.skill_id) else {
            continue;
        };
        if let Some(drift) = classify_copy_target(skill, &target) {
            targets.push(drift);
        }
    }

    targets.sort_by(|a, b| {
        a.skill_name
            .cmp(&b.skill_name)
            .then_with(|| a.tool.cmp(&b.tool))
            .then_with(|| a.target_path.cmp(&b.target_path))
    });

    Ok(AgentPortTargetDriftStatus {
        target_count,
        drift_count: targets.len(),
        targets,
    })
}

fn classify_copy_target(
    skill: &SkillRecord,
    target: &SkillTargetRecord,
) -> Option<AgentPortTargetDrift> {
    let central_hash = skill.content_hash.clone();
    let last_synced_hash = target.source_hash.clone();
    let target_path = PathBuf::from(&target.target_path);
    let target_hash = if target_path.exists() {
        content_hash::hash_path(&target_path).ok()
    } else {
        None
    };

    let status = match (
        central_hash.as_deref(),
        target_hash.as_deref(),
        last_synced_hash.as_deref(),
    ) {
        (_, None, _) => "missing",
        (Some(central), Some(target), _) if central == target => "synced",
        (Some(central), Some(target), Some(last)) if central == last && target != last => {
            "target_newer"
        }
        (Some(central), Some(target), Some(last)) if target == last && central != last => {
            "central_newer"
        }
        (Some(central), Some(target), Some(last)) if central != last && target != last => {
            "conflict"
        }
        _ => "drifted",
    };

    if status == "synced" {
        return None;
    }

    Some(AgentPortTargetDrift {
        skill_id: skill.id.clone(),
        skill_name: skill.name.clone(),
        tool: target.tool.clone(),
        target_path: target.target_path.clone(),
        mode: target.mode.clone(),
        status: status.to_string(),
        central_hash,
        target_hash,
        last_synced_hash,
        can_pull: target_path.is_dir() && matches!(status, "target_newer" | "conflict" | "drifted"),
        can_discard: Path::new(&skill.central_path).is_dir(),
    })
}

pub fn agentport_pull_target_for_store(
    store: &SkillStore,
    skill_id: &str,
    tool: &str,
) -> anyhow::Result<AgentPortTargetActionReport> {
    let skill = resolve_skill(store, skill_id)?;
    let target = resolve_skill_target(store, &skill.id, tool)?;
    if target.mode != "copy" {
        bail!(
            "pull-target only applies to copy targets; {} is {}",
            tool,
            target.mode
        );
    }

    let target_path = PathBuf::from(&target.target_path);
    if !target_path.is_dir() {
        bail!("target path does not exist: {}", target_path.display());
    }

    let _lock = RepoLock::acquire("agentport pull target skill")?;
    let staged_path = skill_commands::staged_path_for(&skill.central_path);
    let install_result = installer::install_from_local_to_destination(
        &target_path,
        Some(&skill.name),
        &staged_path,
    )?;
    skill_commands::swap_skill_directory(&staged_path, Path::new(&skill.central_path))
        .map_err(app_error)?;
    store.update_skill_after_install(
        &skill.id,
        &skill.name,
        install_result.description.as_deref(),
        skill.source_revision.as_deref(),
        skill.remote_revision.as_deref(),
        Some(&install_result.content_hash),
        &skill.update_status,
    )?;
    skill_commands::resync_copy_targets(store, &skill.id).map_err(app_error)?;
    sync_metadata::write_all_from_db_unlocked(store)?;
    refresh_agentport_environment_if_present(store)?;

    Ok(action_report("pull-target", skill, target))
}

pub fn agentport_discard_target_for_store(
    store: &SkillStore,
    skill_id: &str,
    tool: &str,
) -> anyhow::Result<AgentPortTargetActionReport> {
    let skill = resolve_skill(store, skill_id)?;
    let target = resolve_skill_target(store, &skill.id, tool)?;
    let source = PathBuf::from(&skill.central_path);
    let target_path = PathBuf::from(&target.target_path);
    let desired_mode = if target.mode == "copy" {
        sync_engine::SyncMode::Copy
    } else {
        sync_engine::SyncMode::Symlink
    };

    let _lock = RepoLock::acquire("agentport discard target skill")?;
    let actual_mode = sync_engine::sync_skill(&source, &target_path, desired_mode)
        .with_context(|| format!("failed to sync target {}", target_path.display()))?;
    let updated_target = SkillTargetRecord {
        mode: actual_mode.as_str().to_string(),
        status: "ok".to_string(),
        synced_at: Some(chrono::Utc::now().timestamp_millis()),
        last_error: None,
        source_hash: skill.content_hash.clone(),
        ..target
    };
    store.insert_target(&updated_target)?;
    refresh_agentport_environment_if_present(store)?;

    Ok(action_report("discard-target", skill, updated_target))
}

fn resolve_skill(store: &SkillStore, reference: &str) -> anyhow::Result<SkillRecord> {
    let matches = store
        .get_all_skills()?
        .into_iter()
        .filter(|skill| skill.id == reference || skill.name == reference)
        .collect::<Vec<_>>();
    match matches.len() {
        1 => Ok(matches.into_iter().next().unwrap()),
        0 => bail!("skill not found: {reference}"),
        _ => bail!("skill reference is ambiguous: {reference}"),
    }
}

fn resolve_skill_target(
    store: &SkillStore,
    skill_id: &str,
    tool: &str,
) -> anyhow::Result<SkillTargetRecord> {
    let matches = store
        .get_targets_for_skill(skill_id)?
        .into_iter()
        .filter(|target| target.tool == tool)
        .collect::<Vec<_>>();
    match matches.len() {
        1 => Ok(matches.into_iter().next().unwrap()),
        0 => bail!("no synced target for tool: {tool}"),
        _ => bail!("multiple targets for tool: {tool}"),
    }
}

fn refresh_agentport_environment_if_present(store: &SkillStore) -> anyhow::Result<()> {
    if agentport_env::manifest_path().exists() {
        agentport_env::write_current_environment(store, true)?;
    }
    Ok(())
}

fn action_report(
    action: &str,
    skill: SkillRecord,
    target: SkillTargetRecord,
) -> AgentPortTargetActionReport {
    AgentPortTargetActionReport {
        ok: true,
        action: action.to_string(),
        skill_id: skill.id,
        skill_name: skill.name,
        tool: target.tool,
        target_path: target.target_path,
        mode: target.mode,
    }
}

fn app_error(err: AppError) -> anyhow::Error {
    anyhow::anyhow!(err.message)
}

#[tauri::command]
pub async fn agentport_target_drifts(
    store: State<'_, Arc<SkillStore>>,
) -> Result<AgentPortTargetDriftStatus, AppError> {
    let store = store.inner().clone();
    tokio::task::spawn_blocking(move || {
        agentport_target_drifts_for_store(&store).map_err(AppError::internal)
    })
    .await?
}

#[tauri::command]
pub async fn agentport_pull_target(
    store: State<'_, Arc<SkillStore>>,
    skill_id: String,
    tool: String,
) -> Result<AgentPortTargetActionReport, AppError> {
    let store = store.inner().clone();
    tokio::task::spawn_blocking(move || {
        agentport_pull_target_for_store(&store, &skill_id, &tool).map_err(AppError::internal)
    })
    .await?
}

#[tauri::command]
pub async fn agentport_discard_target(
    store: State<'_, Arc<SkillStore>>,
    skill_id: String,
    tool: String,
) -> Result<AgentPortTargetActionReport, AppError> {
    let store = store.inner().clone();
    tokio::task::spawn_blocking(move || {
        agentport_discard_target_for_store(&store, &skill_id, &tool).map_err(AppError::internal)
    })
    .await?
}

#[cfg(test)]
mod tests {
    use crate::commands::agentport_targets::{
        agentport_discard_target_for_store, agentport_target_drifts_for_store,
    };
    use crate::core::{
        central_repo, content_hash,
        skill_store::{SkillRecord, SkillStore, SkillTargetRecord},
    };
    use std::fs;

    fn sample_skill(id: &str, central_path: &std::path::Path, hash: String) -> SkillRecord {
        SkillRecord {
            id: id.to_string(),
            name: id.to_string(),
            description: None,
            source_type: "local_created".to_string(),
            source_ref: None,
            source_ref_resolved: None,
            source_subpath: None,
            source_branch: None,
            source_revision: None,
            remote_revision: None,
            central_path: central_path.to_string_lossy().to_string(),
            content_hash: Some(hash),
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
    fn target_drifts_classifies_copy_target_newer() {
        let _guard = central_repo::test_base_dir_lock();
        let tmp = tempfile::tempdir().unwrap();
        central_repo::set_test_base_dir_override(Some(tmp.path().join("state")));

        let store = SkillStore::new(&tmp.path().join("test.db")).unwrap();
        let central = tmp.path().join("central/review");
        let target = tmp.path().join("codex/review");
        fs::create_dir_all(&central).unwrap();
        fs::create_dir_all(&target).unwrap();
        fs::write(
            central.join("SKILL.md"),
            "---\nname: review\n---\ncentral\n",
        )
        .unwrap();
        fs::write(target.join("SKILL.md"), "---\nname: review\n---\ncentral\n").unwrap();
        let central_hash = content_hash::hash_path(&central).unwrap();
        store
            .insert_skill(&sample_skill("review", &central, central_hash.clone()))
            .unwrap();
        fs::write(
            target.join("SKILL.md"),
            "---\nname: review\n---\ntarget edit\n",
        )
        .unwrap();
        store
            .insert_target(&SkillTargetRecord {
                id: "target-1".to_string(),
                skill_id: "review".to_string(),
                tool: "codex".to_string(),
                target_path: target.to_string_lossy().to_string(),
                mode: "copy".to_string(),
                status: "ok".to_string(),
                synced_at: Some(1),
                last_error: None,
                source_hash: Some(central_hash),
            })
            .unwrap();

        let status = agentport_target_drifts_for_store(&store).unwrap();

        assert_eq!(status.drift_count, 1);
        assert_eq!(status.targets[0].status, "target_newer");
        assert!(status.targets[0].can_pull);
        assert!(status.targets[0].can_discard);

        central_repo::set_test_base_dir_override(None);
    }

    #[test]
    fn discard_target_restores_copy_target_from_central() {
        let _guard = central_repo::test_base_dir_lock();
        let tmp = tempfile::tempdir().unwrap();
        central_repo::set_test_base_dir_override(Some(tmp.path().join("state")));

        let store = SkillStore::new(&tmp.path().join("test.db")).unwrap();
        let central = tmp.path().join("central/review");
        let target = tmp.path().join("codex/review");
        fs::create_dir_all(&central).unwrap();
        fs::create_dir_all(&target).unwrap();
        fs::write(
            central.join("SKILL.md"),
            "---\nname: review\n---\ncentral\n",
        )
        .unwrap();
        fs::write(
            target.join("SKILL.md"),
            "---\nname: review\n---\ntarget edit\n",
        )
        .unwrap();
        let central_hash = content_hash::hash_path(&central).unwrap();
        store
            .insert_skill(&sample_skill("review", &central, central_hash.clone()))
            .unwrap();
        store
            .insert_target(&SkillTargetRecord {
                id: "target-1".to_string(),
                skill_id: "review".to_string(),
                tool: "codex".to_string(),
                target_path: target.to_string_lossy().to_string(),
                mode: "copy".to_string(),
                status: "ok".to_string(),
                synced_at: Some(1),
                last_error: None,
                source_hash: Some(central_hash.clone()),
            })
            .unwrap();

        let report = agentport_discard_target_for_store(&store, "review", "codex").unwrap();
        let updated_target = store.get_targets_for_skill("review").unwrap().remove(0);

        assert!(report.ok);
        assert_eq!(
            fs::read_to_string(target.join("SKILL.md")).unwrap(),
            fs::read_to_string(central.join("SKILL.md")).unwrap()
        );
        assert_eq!(
            updated_target.source_hash.as_deref(),
            Some(central_hash.as_str())
        );

        central_repo::set_test_base_dir_override(None);
    }
}
