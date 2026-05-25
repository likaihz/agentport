use crate::core::{agentport_env, error::AppError, skill_store::SkillStore};
use serde::Serialize;
use std::collections::BTreeSet;
use std::sync::Arc;
use tauri::State;

#[derive(Debug, Clone, Serialize)]
pub struct AgentPortProfilesStatus {
    pub profile_count: usize,
    pub active_profile_id: Option<String>,
    pub profiles: Vec<AgentPortProfileSummary>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentPortProfileSummary {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub active: bool,
    pub skill_count: usize,
    pub skills: Vec<String>,
    pub tool_toggle_count: usize,
    pub enabled_tool_count: usize,
    pub enabled_tools: Vec<String>,
}

pub fn agentport_profiles_for_store(store: &SkillStore) -> anyhow::Result<AgentPortProfilesStatus> {
    let manifest = agentport_env::build_manifest_from_store(store)?;
    let active_profile_id = manifest.active_profile.clone();
    let profiles = manifest
        .profiles
        .into_iter()
        .map(|profile| {
            let mut enabled_tools = BTreeSet::new();
            let mut tool_toggle_count = 0usize;
            for tools in profile.tools_by_skill.values() {
                for (tool, enabled) in tools {
                    tool_toggle_count += 1;
                    if *enabled {
                        enabled_tools.insert(tool.clone());
                    }
                }
            }

            AgentPortProfileSummary {
                id: profile.id,
                name: profile.name,
                description: profile.description,
                active: profile.active,
                skill_count: profile.skills.len(),
                skills: profile.skills,
                tool_toggle_count,
                enabled_tool_count: enabled_tools.len(),
                enabled_tools: enabled_tools.into_iter().collect(),
            }
        })
        .collect::<Vec<_>>();

    Ok(AgentPortProfilesStatus {
        profile_count: profiles.len(),
        active_profile_id,
        profiles,
    })
}

#[tauri::command]
pub async fn agentport_profiles_status(
    store: State<'_, Arc<SkillStore>>,
) -> Result<AgentPortProfilesStatus, AppError> {
    let store = store.inner().clone();
    tokio::task::spawn_blocking(move || {
        agentport_profiles_for_store(&store).map_err(AppError::internal)
    })
    .await?
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{
        central_repo,
        skill_store::{ScenarioRecord, SkillRecord, SkillStore},
    };
    use std::fs;

    fn sample_skill(id: &str, name: &str, central_path: &std::path::Path) -> SkillRecord {
        SkillRecord {
            id: id.to_string(),
            name: name.to_string(),
            description: None,
            source_type: "local_created".to_string(),
            source_ref: None,
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

    fn sample_scenario(id: &str, name: &str) -> ScenarioRecord {
        ScenarioRecord {
            id: id.to_string(),
            name: name.to_string(),
            description: Some("Personal profile".to_string()),
            icon: None,
            sort_order: 0,
            created_at: 1,
            updated_at: 1,
        }
    }

    #[test]
    fn profile_status_summarizes_scenarios_as_agentport_profiles() {
        let _guard = central_repo::test_base_dir_lock();
        let tmp = tempfile::tempdir().unwrap();
        central_repo::set_test_base_dir_override(Some(tmp.path().join("state")));

        let store = SkillStore::new(&tmp.path().join("test.db")).unwrap();
        let skill_dir = tmp.path().join("central/demo");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(skill_dir.join("SKILL.md"), "---\nname: demo\n---\n").unwrap();
        store
            .insert_skill(&sample_skill("skill-1", "demo", &skill_dir))
            .unwrap();
        store
            .insert_scenario(&sample_scenario("personal", "Personal"))
            .unwrap();
        store.add_skill_to_scenario("personal", "skill-1").unwrap();
        store.set_active_scenario("personal").unwrap();

        let status = agentport_profiles_for_store(&store).unwrap();

        assert_eq!(status.profile_count, 1);
        assert_eq!(status.active_profile_id.as_deref(), Some("personal"));
        assert_eq!(status.profiles[0].id, "personal");
        assert_eq!(status.profiles[0].name, "Personal");
        assert!(status.profiles[0].active);
        assert_eq!(status.profiles[0].skill_count, 1);
        assert_eq!(status.profiles[0].skills, vec!["skill-1"]);

        central_repo::set_test_base_dir_override(None);
    }
}
