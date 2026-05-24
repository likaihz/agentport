use anyhow::{bail, Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use super::{
    central_repo,
    skill_store::{ScenarioRecord, SkillRecord, SkillStore},
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub packages: Vec<EnvPackage>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artifacts: Vec<EnvArtifact>,
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
    pub mode: Option<String>,
    pub confidence: String,
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
pub struct EnvPackage {
    pub id: String,
    pub source: EnvPackageSource,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artifacts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvPackageSource {
    #[serde(rename = "type")]
    pub source_type: String,
    pub confidence: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_reference: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote_revision: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvArtifact {
    pub id: String,
    pub kind: String,
    pub name: String,
    pub path: String,
    pub source: EnvArtifactSource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<EnvArtifactOwner>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub deployed_to: Vec<EnvDeployTarget>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvArtifactSource {
    #[serde(rename = "type")]
    pub source_type: String,
    pub confidence: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvArtifactOwner {
    #[serde(rename = "type")]
    pub owner_type: String,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvDeployTarget {
    pub tool: String,
    pub path: String,
    pub mode: String,
    pub status: String,
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
    pub packages: Vec<EnvLockedPackage>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artifacts: Vec<EnvLockedArtifact>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skills: Vec<EnvLockedSkill>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvLockedPackage {
    pub id: String,
    pub source_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_reference: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote_revision: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artifacts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvLockedArtifact {
    pub id: String,
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_package: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,
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
    pub package_count: usize,
    pub artifact_count: usize,
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
    pub package_count: usize,
    pub artifact_count: usize,
    pub skill_count: usize,
    pub profile_count: usize,
    pub installed_tool_count: usize,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EnvBootstrapSkillReport {
    pub destination: String,
    pub skill_file: String,
    pub profile: Option<String>,
    pub overwritten: bool,
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
    let packages = build_packages(store)?;
    let artifacts = build_artifacts(store)?;
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
        packages,
        artifacts,
        tools,
        skills,
        profiles,
    })
}

pub fn build_lock_from_store(store: &SkillStore) -> Result<EnvLock> {
    let packages = build_packages(store)?;
    let artifacts = build_artifacts(store)?;
    let locked_packages = packages
        .into_iter()
        .map(|package| EnvLockedPackage {
            id: package.id,
            source_type: package.source.source_type,
            resolved_reference: package.source.resolved_reference,
            revision: package.source.revision,
            remote_revision: package.source.remote_revision,
            artifacts: package.artifacts,
        })
        .collect();
    let locked_artifacts = artifacts
        .into_iter()
        .map(|artifact| EnvLockedArtifact {
            content_hash: artifact_content_hash(store, &artifact.id).ok().flatten(),
            id: artifact.id,
            kind: artifact.kind,
            owner_package: artifact
                .owner
                .as_ref()
                .and_then(|owner| (owner.owner_type == "package").then(|| owner.id.clone())),
        })
        .collect();
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
        packages: locked_packages,
        artifacts: locked_artifacts,
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
        package_count: manifest.packages.len(),
        artifact_count: manifest.artifacts.len(),
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
    let manifest = if manifest_exists {
        read_manifest().ok()
    } else {
        None
    };
    let skills_dir_exists = central_repo::skills_dir().is_dir();
    let package_count = manifest.as_ref().map(|m| m.packages.len()).unwrap_or(0);
    let artifact_count = manifest.as_ref().map(|m| m.artifacts.len()).unwrap_or(0);
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
        package_count,
        artifact_count,
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

pub fn packages_from_manifest_or_store(store: &SkillStore) -> Result<Vec<EnvPackage>> {
    if manifest_path().exists() {
        return Ok(read_manifest()?.packages);
    }
    build_packages(store)
}

pub fn discover_skill_dirs(root: &Path) -> Result<Vec<PathBuf>> {
    let mut candidates = Vec::new();
    for entry in WalkDir::new(root)
        .max_depth(6)
        .into_iter()
        .filter_entry(|entry| entry.file_name() != ".git")
    {
        let entry = entry?;
        if entry.file_type().is_dir() && super::skill_metadata::is_valid_skill_dir(entry.path()) {
            candidates.push(entry.path().to_path_buf());
        }
    }
    candidates.sort();

    let mut selected: Vec<PathBuf> = Vec::new();
    'outer: for candidate in candidates {
        for existing in &selected {
            if candidate.starts_with(existing) {
                continue 'outer;
            }
        }
        selected.push(candidate);
    }

    Ok(selected)
}

pub fn export_bootstrap_skill(
    profile: Option<&str>,
    destination: PathBuf,
    overwrite: bool,
) -> Result<EnvBootstrapSkillReport> {
    let skill_file = destination.join("SKILL.md");
    if skill_file.exists() && !overwrite {
        bail!(
            "{} already exists; pass --overwrite to replace it",
            skill_file.display()
        );
    }
    fs::create_dir_all(&destination)?;
    fs::write(&skill_file, bootstrap_skill_markdown(profile))
        .with_context(|| format!("failed to write {}", skill_file.display()))?;

    Ok(EnvBootstrapSkillReport {
        destination: destination.to_string_lossy().to_string(),
        skill_file: skill_file.to_string_lossy().to_string(),
        profile: profile.map(ToOwned::to_owned),
        overwritten: overwrite,
    })
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

fn build_packages(store: &SkillStore) -> Result<Vec<EnvPackage>> {
    let mut packages = BTreeMap::<String, EnvPackage>::new();
    for skill in store.get_all_skills()? {
        let Some(package_id) = package_id_for_skill(&skill) else {
            continue;
        };
        let artifact_id = artifact_id_for_skill(&skill);
        packages
            .entry(package_id.clone())
            .and_modify(|package| {
                if !package.artifacts.iter().any(|id| id == &artifact_id) {
                    package.artifacts.push(artifact_id.clone());
                    package.artifacts.sort();
                }
            })
            .or_insert_with(|| EnvPackage {
                id: package_id,
                source: package_source_for_skill(&skill),
                artifacts: vec![artifact_id],
            });
    }
    Ok(packages.into_values().collect())
}

fn build_artifacts(store: &SkillStore) -> Result<Vec<EnvArtifact>> {
    let mut artifacts = Vec::new();
    for skill in store.get_all_skills()? {
        let package_id = package_id_for_skill(&skill);
        let deployed_to = store
            .get_targets_for_skill(&skill.id)?
            .into_iter()
            .map(|target| EnvDeployTarget {
                tool: target.tool,
                path: portable_home_path(&target.target_path),
                mode: target.mode,
                status: target.status,
            })
            .collect();
        let artifact_id = artifact_id_for_skill(&skill);
        artifacts.push(EnvArtifact {
            id: artifact_id,
            kind: "skill".to_string(),
            name: skill.name.clone(),
            path: relative_skill_path(&skill.central_path),
            source: artifact_source_for_skill(&skill, package_id.as_deref()),
            owner: package_id.map(|id| EnvArtifactOwner {
                owner_type: "package".to_string(),
                id,
            }),
            deployed_to,
        });
    }
    artifacts.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(artifacts)
}

fn build_skills(store: &SkillStore) -> Result<Vec<EnvSkill>> {
    let tags_map = store.get_tags_map()?;
    let mut skills = store
        .get_all_skills()?
        .into_iter()
        .map(|skill| {
            let tags = tags_map.get(&skill.id).cloned().unwrap_or_default();
            let local_source = is_local_source_type(&skill.source_type);
            let source = env_source_for_skill(&skill, local_source);
            EnvSkill {
                id: skill.id,
                name: skill.name,
                description: skill.description,
                path: relative_skill_path(&skill.central_path),
                enabled: skill.enabled,
                tags,
                source,
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

fn env_source_for_skill(skill: &SkillRecord, local_source: bool) -> EnvSource {
    EnvSource {
        source_type: skill.source_type.clone(),
        mode: local_source.then(|| {
            if skill.source_type == "local_created" {
                "created".to_string()
            } else {
                "vendored".to_string()
            }
        }),
        confidence: "exact".to_string(),
        reference: if local_source {
            None
        } else {
            skill.source_ref.clone()
        },
        resolved_reference: if local_source {
            None
        } else {
            skill.source_ref_resolved.clone()
        },
        subpath: skill.source_subpath.clone(),
        branch: skill.source_branch.clone(),
        revision: skill.source_revision.clone(),
        remote_revision: skill.remote_revision.clone(),
    }
}

fn artifact_id_for_skill(skill: &SkillRecord) -> String {
    format!("skill:{}", skill.id)
}

fn artifact_content_hash(store: &SkillStore, artifact_id: &str) -> Result<Option<String>> {
    let Some(skill_id) = artifact_id.strip_prefix("skill:") else {
        return Ok(None);
    };
    Ok(store
        .get_skill_by_id(skill_id)?
        .and_then(|skill| skill.content_hash))
}

fn artifact_source_for_skill(skill: &SkillRecord, package_id: Option<&str>) -> EnvArtifactSource {
    if let Some(package_id) = package_id {
        EnvArtifactSource {
            source_type: "package_artifact".to_string(),
            confidence: "exact".to_string(),
            package: Some(package_id.to_string()),
            path: skill.source_subpath.clone(),
            revision: skill.source_revision.clone(),
        }
    } else {
        let local = is_local_source_type(&skill.source_type);
        EnvArtifactSource {
            source_type: if local {
                if skill.source_type == "local_created" {
                    "local_created".to_string()
                } else {
                    "local_vendored".to_string()
                }
            } else {
                skill.source_type.clone()
            },
            confidence: "exact".to_string(),
            package: None,
            path: Some(relative_skill_path(&skill.central_path)),
            revision: skill.source_revision.clone(),
        }
    }
}

fn package_id_for_skill(skill: &SkillRecord) -> Option<String> {
    match skill.source_type.as_str() {
        "git" => source_identity(skill).map(|identity| format!("git:{identity}")),
        "skillssh" => source_identity(skill).map(|identity| format!("skillssh:{identity}")),
        "local_package" => skill
            .source_ref
            .as_ref()
            .and_then(|source| Path::new(source).file_name())
            .map(|name| format!("local:{}", name.to_string_lossy())),
        _ => None,
    }
}

fn package_source_for_skill(skill: &SkillRecord) -> EnvPackageSource {
    let local = is_local_source_type(&skill.source_type);
    EnvPackageSource {
        source_type: skill.source_type.clone(),
        confidence: "exact".to_string(),
        reference: if local {
            None
        } else {
            skill.source_ref.clone()
        },
        resolved_reference: if local {
            None
        } else {
            skill.source_ref_resolved.clone()
        },
        branch: skill.source_branch.clone(),
        revision: skill.source_revision.clone(),
        remote_revision: skill.remote_revision.clone(),
    }
}

fn is_local_source_type(source_type: &str) -> bool {
    matches!(
        source_type,
        "local" | "import" | "local_created" | "local_package"
    )
}

fn source_identity(skill: &SkillRecord) -> Option<String> {
    skill
        .source_ref_resolved
        .as_ref()
        .or(skill.source_ref.as_ref())
        .map(|source| {
            source
                .trim_end_matches(".git")
                .trim_end_matches('/')
                .to_string()
        })
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

fn bootstrap_skill_markdown(profile: Option<&str>) -> String {
    let profile_arg = profile.unwrap_or("default");
    format!(
        r#"---
name: agentport-bootstrap
description: Bootstrap an AgentPort-managed AI coding agent environment on a new machine. Use this when the user wants to restore, verify, or apply their shared agent environment from agentport.yaml.
---

# AgentPort Bootstrap

Use this skill to restore a user's AgentPort environment on a new machine or in a new coding agent.

## Procedure

1. Check whether `agentport` is available. If it is not available, check whether `skills-manager-cli` is available.
2. If neither command exists, ask the user to install or build the AgentPort CLI before continuing.
3. Run `agentport env doctor` if `agentport` exists, otherwise run `skills-manager-cli env doctor`.
4. If the environment repo is missing, ask the user for its Git remote and clone or configure it using the CLI's git commands.
5. Run `agentport env diff {profile_arg}` if `agentport` exists, otherwise run `skills-manager-cli env diff {profile_arg}`.
6. Show the user the planned changes and ask for confirmation before applying them.
7. After confirmation, run `agentport env apply {profile_arg}` or `skills-manager-cli env apply {profile_arg}`.
8. Finish by running `agentport env doctor` or `skills-manager-cli env doctor` and report any warnings.

## Safety

- Do not edit secrets or API keys into `agentport.yaml`.
- Do not run package install scripts unless the user confirms the plan.
- Do not overwrite local customized skills unless the CLI reports it is safe.
- Prefer `--dry-run` before commands that modify files.
"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn skill(source_type: &str) -> SkillRecord {
        SkillRecord {
            id: "review".to_string(),
            name: "Review".to_string(),
            description: None,
            source_type: source_type.to_string(),
            source_ref: Some("/Users/example/dev/review".to_string()),
            source_ref_resolved: Some("https://github.com/acme/skills.git".to_string()),
            source_subpath: Some("skills/review".to_string()),
            source_branch: Some("main".to_string()),
            source_revision: Some("abc123".to_string()),
            remote_revision: Some("def456".to_string()),
            central_path: central_repo::skills_dir()
                .join("review")
                .to_string_lossy()
                .to_string(),
            content_hash: Some("hash".to_string()),
            enabled: true,
            created_at: 0,
            updated_at: 0,
            status: "active".to_string(),
            update_status: "current".to_string(),
            last_checked_at: None,
            last_check_error: None,
        }
    }

    #[test]
    fn local_source_is_vendored_and_redacts_original_path() {
        let skill = skill("local");
        let source = env_source_for_skill(&skill, true);

        assert_eq!(source.source_type, "local");
        assert_eq!(source.mode.as_deref(), Some("vendored"));
        assert_eq!(source.confidence, "exact");
        assert!(source.reference.is_none());
        assert!(source.resolved_reference.is_none());
    }

    #[test]
    fn git_skill_becomes_package_owned_artifact() {
        let skill = skill("git");
        let package_id = package_id_for_skill(&skill).unwrap();
        let artifact_source = artifact_source_for_skill(&skill, Some(&package_id));

        assert_eq!(package_id, "git:https://github.com/acme/skills");
        assert_eq!(artifact_id_for_skill(&skill), "skill:review");
        assert_eq!(artifact_source.source_type, "package_artifact");
        assert_eq!(
            artifact_source.package.as_deref(),
            Some(package_id.as_str())
        );
        assert_eq!(artifact_source.path.as_deref(), Some("skills/review"));
        assert_eq!(artifact_source.confidence, "exact");
    }

    #[test]
    fn local_skill_has_no_package_owner() {
        let skill = skill("local");

        assert!(package_id_for_skill(&skill).is_none());
        let artifact_source = artifact_source_for_skill(&skill, None);
        assert_eq!(artifact_source.source_type, "local_vendored");
        assert!(artifact_source.package.is_none());
    }

    #[test]
    fn local_created_skill_uses_created_mode() {
        let skill = skill("local_created");
        let source = env_source_for_skill(&skill, true);
        let artifact_source = artifact_source_for_skill(&skill, None);

        assert_eq!(source.mode.as_deref(), Some("created"));
        assert_eq!(artifact_source.source_type, "local_created");
        assert!(package_id_for_skill(&skill).is_none());
    }

    #[test]
    fn portable_home_path_replaces_home_prefix() {
        let Some(home) = dirs::home_dir() else {
            return;
        };
        let input = home.join(".codex").join("skills");

        assert_eq!(
            portable_home_path(&input.to_string_lossy()),
            "~/.codex/skills"
        );
    }

    #[test]
    fn bootstrap_skill_mentions_profile_and_confirmation() {
        let markdown = bootstrap_skill_markdown(Some("personal-default"));

        assert!(markdown.contains("agentport-bootstrap"));
        assert!(markdown.contains("env diff personal-default"));
        assert!(markdown.contains("ask for confirmation"));
        assert!(markdown.contains("Do not edit secrets"));
    }

    #[test]
    fn discover_skill_dirs_finds_multiple_skills_and_skips_nested_children() {
        let tmp = tempfile::tempdir().unwrap();
        let alpha = tmp.path().join("skills").join("alpha");
        let beta = tmp.path().join("skills").join("beta");
        let nested = alpha.join("nested");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::create_dir_all(&beta).unwrap();
        std::fs::write(alpha.join("SKILL.md"), "---\nname: alpha\n---\n").unwrap();
        std::fs::write(nested.join("SKILL.md"), "---\nname: nested\n---\n").unwrap();
        std::fs::write(beta.join("SKILL.md"), "---\nname: beta\n---\n").unwrap();

        let dirs = discover_skill_dirs(tmp.path()).unwrap();
        let names: Vec<_> = dirs
            .iter()
            .map(|path| path.file_name().unwrap().to_string_lossy().to_string())
            .collect();

        assert_eq!(names, vec!["alpha".to_string(), "beta".to_string()]);
    }
}
