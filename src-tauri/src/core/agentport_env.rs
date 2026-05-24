use anyhow::{bail, Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

use super::{
    central_repo,
    skill_store::{ScenarioRecord, SkillStore},
    tool_service,
};

pub const MANIFEST_FILE: &str = "agentport.yaml";
pub const LOCK_FILE: &str = "agentport.lock";
const MANIFEST_VERSION: u32 = 1;
const CREATED_BY: &str = "skills-manager/agentport-mvp";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvManifest {
    pub version: u32,
    pub generated_at: String,
    pub created_by: String,
    pub active_profile: Option<String>,
    pub repo: EnvRepo,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub tools: BTreeMap<String, EnvTool>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skills: Vec<EnvSkill>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub profiles: Vec<EnvProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvRepo {
    pub layout: String,
    pub manifest_path: String,
    pub lock_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvTool {
    pub display_name: String,
    pub installed: bool,
    pub enabled: bool,
    pub skills_dir: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_relative_skills_dir: Option<String>,
    pub is_custom: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvSkill {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub path: String,
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    pub source: EnvSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvSource {
    #[serde(rename = "type")]
    pub source_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_reference: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subpath: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote_revision: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvProfile {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub active: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skills: Vec<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub tools_by_skill: BTreeMap<String, BTreeMap<String, bool>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvLock {
    pub version: u32,
    pub generated_at: String,
    pub created_by: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skills: Vec<EnvLockedSkill>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvLockedSkill {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,
    pub source_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_revision: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote_revision: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EnvWriteReport {
    pub manifest_path: String,
    pub lock_path: String,
    pub skill_count: usize,
    pub profile_count: usize,
    pub tool_count: usize,
    pub overwritten: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct EnvDiffReport {
    pub manifest_path: String,
    pub profile: Option<EnvProfileRef>,
    pub missing_skills: Vec<String>,
    pub unmanaged_skills: Vec<String>,
    pub missing_profile: bool,
    pub ok: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct EnvProfileRef {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct EnvDoctorReport {
    pub manifest_path: String,
    pub lock_path: String,
    pub manifest_exists: bool,
    pub lock_exists: bool,
    pub skills_dir_exists: bool,
    pub skill_count: usize,
    pub profile_count: usize,
    pub installed_tool_count: usize,
    pub warnings: Vec<String>,
}

pub fn manifest_path() -> PathBuf {
    central_repo::skills_dir().join(MANIFEST_FILE)
}

pub fn lock_path() -> PathBuf {
    central_repo::skills_dir().join(LOCK_FILE)
}

pub fn build_manifest_from_store(store: &SkillStore) -> Result<EnvManifest> {
    let generated_at = Utc::now().to_rfc3339();
    let active_profile = store.get_active_scenario_id()?;
    let tools = build_tools(store);
    let skills = build_skills(store)?;
    let profiles = build_profiles(store, active_profile.as_deref())?;

    Ok(EnvManifest {
        version: MANIFEST_VERSION,
        generated_at,
        created_by: CREATED_BY.to_string(),
        active_profile,
        repo: EnvRepo {
            layout: "skills-root-v1".to_string(),
            manifest_path: MANIFEST_FILE.to_string(),
            lock_path: LOCK_FILE.to_string(),
        },
        tools,
        skills,
        profiles,
    })
}

pub fn build_lock_from_store(store: &SkillStore) -> Result<EnvLock> {
    let mut skills: Vec<EnvLockedSkill> = store
        .get_all_skills()?
        .into_iter()
        .map(|skill| EnvLockedSkill {
            id: skill.id,
            name: skill.name,
            content_hash: skill.content_hash,
            source_type: skill.source_type,
            source_revision: skill.source_revision,
            remote_revision: skill.remote_revision,
        })
        .collect();
    skills.sort_by(|a, b| a.id.cmp(&b.id));

    Ok(EnvLock {
        version: MANIFEST_VERSION,
        generated_at: Utc::now().to_rfc3339(),
        created_by: CREATED_BY.to_string(),
        skills,
    })
}

pub fn write_current_environment(store: &SkillStore, overwrite: bool) -> Result<EnvWriteReport> {
    let manifest_path = manifest_path();
    let lock_path = lock_path();
    if !overwrite && manifest_path.exists() {
        bail!(
            "{} already exists; pass --overwrite to replace it",
            manifest_path.display()
        );
    }
    if let Some(parent) = manifest_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let manifest = build_manifest_from_store(store)?;
    let lock = build_lock_from_store(store)?;
    write_yaml(&manifest_path, &manifest)?;
    write_yaml(&lock_path, &lock)?;

    Ok(EnvWriteReport {
        manifest_path: manifest_path.to_string_lossy().to_string(),
        lock_path: lock_path.to_string_lossy().to_string(),
        skill_count: manifest.skills.len(),
        profile_count: manifest.profiles.len(),
        tool_count: manifest.tools.len(),
        overwritten: overwrite,
    })
}

pub fn read_manifest() -> Result<EnvManifest> {
    let path = manifest_path();
    let raw =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_yaml::from_str(&raw).with_context(|| format!("failed to parse {}", path.display()))
}

pub fn diff_current_environment(
    store: &SkillStore,
    profile_ref: Option<&str>,
) -> Result<EnvDiffReport> {
    let manifest = read_manifest()?;
    let profile = select_profile(&manifest, profile_ref);
    let manifest_skill_ids: BTreeSet<String> = manifest
        .skills
        .iter()
        .map(|skill| skill.id.clone())
        .collect();
    let local_skill_ids: BTreeSet<String> = store
        .get_all_skills()?
        .into_iter()
        .map(|skill| skill.id)
        .collect();

    let expected_skill_ids: BTreeSet<String> = profile
        .as_ref()
        .map(|profile| profile.skills.iter().cloned().collect())
        .unwrap_or_else(|| manifest_skill_ids.clone());

    let missing_skills = expected_skill_ids
        .difference(&local_skill_ids)
        .cloned()
        .collect::<Vec<_>>();
    let unmanaged_skills = local_skill_ids
        .difference(&manifest_skill_ids)
        .cloned()
        .collect::<Vec<_>>();
    let missing_profile = profile_ref.is_some() && profile.is_none();

    Ok(EnvDiffReport {
        manifest_path: manifest_path().to_string_lossy().to_string(),
        profile: profile.map(|profile| EnvProfileRef {
            id: profile.id.clone(),
            name: profile.name.clone(),
        }),
        ok: missing_skills.is_empty() && unmanaged_skills.is_empty() && !missing_profile,
        missing_skills,
        unmanaged_skills,
        missing_profile,
    })
}

pub fn doctor(store: &SkillStore) -> Result<EnvDoctorReport> {
    let manifest_path = manifest_path();
    let lock_path = lock_path();
    let manifest_exists = manifest_path.exists();
    let lock_exists = lock_path.exists();
    let skills_dir_exists = central_repo::skills_dir().is_dir();
    let skill_count = store.get_all_skills()?.len();
    let profile_count = store.get_all_scenarios()?.len();
    let installed_tool_count = tool_service::list_tool_info(store)
        .into_iter()
        .filter(|tool| tool.installed)
        .count();

    let mut warnings = Vec::new();
    if !manifest_exists {
        warnings.push(format!(
            "{} is missing; run env init --from-current-machine",
            MANIFEST_FILE
        ));
    }
    if manifest_exists && !lock_exists {
        warnings.push(format!(
            "{} is missing; run env init --overwrite",
            LOCK_FILE
        ));
    }
    if skill_count == 0 {
        warnings.push("no managed skills found".to_string());
    }
    if installed_tool_count == 0 {
        warnings.push("no installed agent tools detected".to_string());
    }

    Ok(EnvDoctorReport {
        manifest_path: manifest_path.to_string_lossy().to_string(),
        lock_path: lock_path.to_string_lossy().to_string(),
        manifest_exists,
        lock_exists,
        skills_dir_exists,
        skill_count,
        profile_count,
        installed_tool_count,
        warnings,
    })
}

pub fn select_profile<'a>(
    manifest: &'a EnvManifest,
    profile_ref: Option<&str>,
) -> Option<&'a EnvProfile> {
    let requested = profile_ref.or(manifest.active_profile.as_deref());
    match requested {
        Some(reference) => manifest
            .profiles
            .iter()
            .find(|profile| profile.id == reference || profile.name == reference),
        None => manifest.profiles.first(),
    }
}

fn build_tools(store: &SkillStore) -> BTreeMap<String, EnvTool> {
    tool_service::list_tool_info(store)
        .into_iter()
        .map(|tool| {
            (
                tool.key,
                EnvTool {
                    display_name: tool.display_name,
                    installed: tool.installed,
                    enabled: tool.enabled,
                    skills_dir: portable_home_path(&tool.skills_dir),
                    project_relative_skills_dir: tool.project_relative_skills_dir,
                    is_custom: tool.is_custom,
                },
            )
        })
        .collect()
}

fn build_skills(store: &SkillStore) -> Result<Vec<EnvSkill>> {
    let tags_map = store.get_tags_map()?;
    let mut skills = store
        .get_all_skills()?
        .into_iter()
        .map(|skill| {
            let tags = tags_map.get(&skill.id).cloned().unwrap_or_default();
            let local_source = matches!(skill.source_type.as_str(), "local" | "import");
            EnvSkill {
                id: skill.id,
                name: skill.name,
                description: skill.description,
                path: relative_skill_path(&skill.central_path),
                enabled: skill.enabled,
                tags,
                source: EnvSource {
                    source_type: skill.source_type,
                    reference: if local_source { None } else { skill.source_ref },
                    resolved_reference: if local_source {
                        None
                    } else {
                        skill.source_ref_resolved
                    },
                    subpath: skill.source_subpath,
                    branch: skill.source_branch,
                    revision: skill.source_revision,
                    remote_revision: skill.remote_revision,
                },
            }
        })
        .collect::<Vec<_>>();
    for skill in &mut skills {
        skill.tags.sort();
        skill.tags.dedup();
    }
    skills.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(skills)
}

fn build_profiles(store: &SkillStore, active_id: Option<&str>) -> Result<Vec<EnvProfile>> {
    let mut scenarios = store.get_all_scenarios()?;
    scenarios.sort_by(compare_scenarios);
    let mut profiles = Vec::with_capacity(scenarios.len());

    for scenario in scenarios {
        let skills = store.get_skill_ids_for_scenario(&scenario.id)?;
        let mut tools_by_skill = BTreeMap::new();
        for skill_id in &skills {
            let toggles = store.get_scenario_skill_tool_toggles(&scenario.id, skill_id)?;
            let tool_map = toggles
                .into_iter()
                .map(|toggle| (toggle.tool, toggle.enabled))
                .collect::<BTreeMap<_, _>>();
            if !tool_map.is_empty() {
                tools_by_skill.insert(skill_id.clone(), tool_map);
            }
        }

        profiles.push(EnvProfile {
            active: active_id == Some(scenario.id.as_str()),
            id: scenario.id,
            name: scenario.name,
            description: scenario.description,
            skills,
            tools_by_skill,
        });
    }

    Ok(profiles)
}

fn compare_scenarios(a: &ScenarioRecord, b: &ScenarioRecord) -> std::cmp::Ordering {
    a.sort_order
        .cmp(&b.sort_order)
        .then_with(|| a.created_at.cmp(&b.created_at))
        .then_with(|| a.id.cmp(&b.id))
}

fn relative_skill_path(path: &str) -> String {
    let path = PathBuf::from(path);
    path.strip_prefix(central_repo::skills_dir())
        .map(|relative| relative.to_string_lossy().to_string())
        .unwrap_or_else(|_| portable_home_path(&path.to_string_lossy()))
}

fn portable_home_path(path: &str) -> String {
    let path_buf = PathBuf::from(path);
    if let Some(home) = dirs::home_dir() {
        if let Ok(relative) = path_buf.strip_prefix(&home) {
            let rel = relative.to_string_lossy();
            if rel.is_empty() {
                return "~".to_string();
            }
            return format!("~/{}", rel);
        }
    }
    path.to_string()
}

fn write_yaml<T: Serialize>(path: &PathBuf, value: &T) -> Result<()> {
    let mut yaml = serde_yaml::to_string(value)?;
    if !yaml.ends_with('\n') {
        yaml.push('\n');
    }
    fs::write(path, yaml).with_context(|| format!("failed to write {}", path.display()))
}
