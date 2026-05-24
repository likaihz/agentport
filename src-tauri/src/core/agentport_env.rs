use anyhow::{bail, Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use super::{
    central_repo, content_hash,
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub resources: Vec<EnvToolResource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvToolResource {
    pub kind: String,
    pub scope: String,
    pub path: String,
    pub deploy: String,
    pub scan: String,
    pub exists: bool,
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
    pub missing_artifacts: Vec<String>,
    pub unmanaged_artifacts: Vec<String>,
    pub changed_artifacts: Vec<EnvArtifactDrift>,
    pub missing_profile: bool,
    pub ok: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct EnvArtifactDrift {
    pub id: String,
    pub kind: String,
    pub expected_hash: Option<String>,
    pub current_hash: Option<String>,
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MachineLocalOverlay {
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub origins: BTreeMap<String, MachineLocalOrigin>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineLocalOrigin {
    pub path: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct EnvResourceActionReport {
    pub ok: bool,
    pub dry_run: bool,
    pub items: Vec<EnvResourceActionItem>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EnvResourceActionItem {
    pub id: String,
    pub tool: String,
    pub kind: String,
    pub source_path: String,
    pub target_path: String,
    pub deploy: String,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backup_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

pub fn manifest_path() -> PathBuf {
    central_repo::skills_dir().join(MANIFEST_FILE)
}

pub fn lock_path() -> PathBuf {
    central_repo::skills_dir().join(LOCK_FILE)
}

pub fn machine_dir() -> PathBuf {
    central_repo::skills_dir().join("machine")
}

pub fn machine_local_path() -> PathBuf {
    machine_dir().join("machine.local.yaml")
}

pub fn secrets_local_path() -> PathBuf {
    machine_dir().join("secrets.local.yaml")
}

pub fn machine_backups_dir() -> PathBuf {
    machine_dir().join("backups")
}

pub fn read_machine_local_overlay() -> Result<MachineLocalOverlay> {
    ensure_machine_local_overlay()?;
    let path = machine_local_path();
    if !path.exists() {
        return Ok(MachineLocalOverlay::default());
    }
    let raw =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    if raw.trim().is_empty() {
        return Ok(MachineLocalOverlay::default());
    }
    serde_yaml::from_str(&raw).with_context(|| format!("failed to parse {}", path.display()))
}

pub fn record_linked_origin_path(origin_id: &str, path: &Path) -> Result<()> {
    let mut overlay = read_machine_local_overlay()?;
    overlay.origins.insert(
        origin_id.to_string(),
        MachineLocalOrigin {
            path: path.to_string_lossy().to_string(),
            updated_at: Utc::now().to_rfc3339(),
        },
    );
    write_yaml(&machine_local_path(), &overlay)
}

pub fn linked_origin_path(origin_id: &str) -> Result<Option<PathBuf>> {
    Ok(read_machine_local_overlay()?
        .origins
        .get(origin_id)
        .map(|origin| PathBuf::from(&origin.path)))
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

fn build_current_machine_lock_from_store(store: &SkillStore) -> Result<EnvLock> {
    let mut lock = build_lock_from_store(store)?;
    for artifact in &mut lock.artifacts {
        artifact.content_hash = current_artifact_content_hash(store, &artifact.id)
            .ok()
            .flatten();
    }
    Ok(lock)
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
    ensure_machine_local_overlay()?;
    export_resource_artifacts_internal(store, false, false)?;

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

fn ensure_machine_local_overlay() -> Result<()> {
    fs::create_dir_all(machine_dir())?;
    fs::create_dir_all(machine_backups_dir())?;
    ensure_gitignore_entries(&[
        "# AgentPort machine-local state",
        "/machine/machine.local.yaml",
        "/machine/secrets.local.yaml",
        "/machine/backups/",
    ])
}

fn ensure_gitignore_entries(entries: &[&str]) -> Result<()> {
    let path = central_repo::skills_dir().join(".gitignore");
    let mut content = fs::read_to_string(&path).unwrap_or_default();
    let mut changed = false;
    for entry in entries {
        if content.lines().any(|line| line.trim() == *entry) {
            continue;
        }
        if !content.is_empty() && !content.ends_with('\n') {
            content.push('\n');
        }
        content.push_str(entry);
        content.push('\n');
        changed = true;
    }
    if changed || !path.exists() {
        fs::write(&path, content)?;
    }
    Ok(())
}

pub fn read_manifest() -> Result<EnvManifest> {
    let path = manifest_path();
    let raw =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_yaml::from_str(&raw).with_context(|| format!("failed to parse {}", path.display()))
}

pub fn read_lock() -> Result<EnvLock> {
    let path = lock_path();
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
    let manifest_artifact_ids: BTreeSet<String> = manifest
        .artifacts
        .iter()
        .map(|artifact| artifact.id.clone())
        .collect();
    let current_artifact_ids: BTreeSet<String> = build_artifacts(store)?
        .into_iter()
        .map(|artifact| artifact.id)
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
    let missing_artifacts = manifest_artifact_ids
        .difference(&current_artifact_ids)
        .cloned()
        .collect::<Vec<_>>();
    let unmanaged_artifacts = current_artifact_ids
        .difference(&manifest_artifact_ids)
        .cloned()
        .collect::<Vec<_>>();
    let changed_artifacts = if lock_path().exists() {
        changed_artifacts_from_locks(
            &read_lock()?,
            &build_current_machine_lock_from_store(store)?,
        )
    } else {
        Vec::new()
    };
    let missing_profile = profile_ref.is_some() && profile.is_none();

    Ok(EnvDiffReport {
        manifest_path: manifest_path().to_string_lossy().to_string(),
        profile: profile.map(|profile| EnvProfileRef {
            id: profile.id.clone(),
            name: profile.name.clone(),
        }),
        ok: missing_skills.is_empty()
            && unmanaged_skills.is_empty()
            && missing_artifacts.is_empty()
            && unmanaged_artifacts.is_empty()
            && changed_artifacts.is_empty()
            && !missing_profile,
        missing_skills,
        unmanaged_skills,
        missing_artifacts,
        unmanaged_artifacts,
        changed_artifacts,
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

pub fn export_resource_artifacts(
    store: &SkillStore,
    overwrite: bool,
    dry_run: bool,
) -> Result<EnvResourceActionReport> {
    export_resource_artifacts_internal(store, overwrite, dry_run)
}

fn export_resource_artifacts_internal(
    store: &SkillStore,
    overwrite: bool,
    dry_run: bool,
) -> Result<EnvResourceActionReport> {
    if !dry_run {
        ensure_machine_local_overlay()?;
    }
    let mut items = Vec::new();

    for tool in tool_service::list_tool_info(store) {
        for resource in tool.resources {
            if resource.kind == "skill" || !resource.exists {
                continue;
            }
            let id = resource_artifact_id(&tool.key, &resource);
            let source_path = PathBuf::from(&resource.path);
            let target_path = resource_repository_path(&tool.key, &resource);
            items.push(export_resource_path(
                &id,
                &tool.key,
                &resource.kind,
                &source_path,
                &target_path,
                &resource.deploy,
                overwrite,
                dry_run,
            )?);
        }
    }

    Ok(EnvResourceActionReport {
        ok: items.iter().all(|item| item.error.is_none()),
        dry_run,
        items,
    })
}

pub fn apply_resource_artifacts(
    store: &SkillStore,
    manifest: &EnvManifest,
    dry_run: bool,
) -> Result<EnvResourceActionReport> {
    if !dry_run {
        ensure_machine_local_overlay()?;
    }
    let resources = current_resource_lookup(store);
    let mut items = Vec::new();

    for artifact in manifest.artifacts.iter().filter(|artifact| {
        artifact.kind != "skill" && artifact.source.source_type == "tool_resource"
    }) {
        let tool_key = artifact
            .owner
            .as_ref()
            .filter(|owner| owner.owner_type == "tool")
            .map(|owner| owner.id.as_str())
            .or_else(|| {
                artifact
                    .deployed_to
                    .first()
                    .map(|target| target.tool.as_str())
            })
            .unwrap_or("unknown");
        let source_path = resolve_repo_relative_path(&artifact.path);
        let resource = resources.get(artifact.id.as_str());
        let target_path = resource
            .map(|resource| PathBuf::from(&resource.path))
            .or_else(|| {
                artifact
                    .deployed_to
                    .first()
                    .map(|target| expand_portable_path(&target.path))
            })
            .unwrap_or_else(|| PathBuf::from(&artifact.path));
        let deploy = resource
            .map(|resource| resource.deploy.as_str())
            .or_else(|| {
                artifact
                    .deployed_to
                    .first()
                    .map(|target| target.mode.as_str())
            })
            .unwrap_or("copy");

        if !source_path.exists() {
            items.push(EnvResourceActionItem {
                id: artifact.id.clone(),
                tool: tool_key.to_string(),
                kind: artifact.kind.clone(),
                source_path: source_path.to_string_lossy().to_string(),
                target_path: target_path.to_string_lossy().to_string(),
                deploy: deploy.to_string(),
                status: "missing_source".to_string(),
                backup_path: None,
                error: Some("artifact source is missing from the AgentPort repo".to_string()),
            });
            continue;
        }

        match deploy_resource_path(
            &artifact.id,
            tool_key,
            &artifact.kind,
            &source_path,
            &target_path,
            deploy,
            dry_run,
        ) {
            Ok(item) => items.push(item),
            Err(err) => items.push(EnvResourceActionItem {
                id: artifact.id.clone(),
                tool: tool_key.to_string(),
                kind: artifact.kind.clone(),
                source_path: source_path.to_string_lossy().to_string(),
                target_path: target_path.to_string_lossy().to_string(),
                deploy: deploy.to_string(),
                status: "error".to_string(),
                backup_path: None,
                error: Some(err.to_string()),
            }),
        }
    }

    Ok(EnvResourceActionReport {
        ok: items.iter().all(|item| item.error.is_none()),
        dry_run,
        items,
    })
}

fn build_tools(store: &SkillStore) -> BTreeMap<String, EnvTool> {
    tool_service::list_tool_info(store)
        .into_iter()
        .map(|tool| {
            let resources = env_resources_for_tool(&tool);
            (
                tool.key,
                EnvTool {
                    display_name: tool.display_name,
                    installed: tool.installed,
                    enabled: tool.enabled,
                    skills_dir: portable_home_path(&tool.skills_dir),
                    project_relative_skills_dir: tool.project_relative_skills_dir,
                    is_custom: tool.is_custom,
                    resources,
                },
            )
        })
        .collect()
}

fn env_resources_for_tool(tool: &tool_service::ToolInfo) -> Vec<EnvToolResource> {
    tool.resources
        .iter()
        .map(|resource| EnvToolResource {
            kind: resource.kind.clone(),
            scope: resource.scope.clone(),
            path: portable_home_path(&resource.path),
            deploy: resource.deploy.clone(),
            scan: resource.scan.clone(),
            exists: resource.exists,
        })
        .collect()
}

fn current_resource_lookup(store: &SkillStore) -> BTreeMap<String, tool_service::ToolResourceInfo> {
    let mut resources = BTreeMap::new();
    for tool in tool_service::list_tool_info(store) {
        for resource in tool.resources {
            if resource.kind == "skill" {
                continue;
            }
            resources.insert(resource_artifact_id(&tool.key, &resource), resource);
        }
    }
    resources
}

fn build_packages(store: &SkillStore) -> Result<Vec<EnvPackage>> {
    let mut packages = BTreeMap::<String, EnvPackage>::new();
    for skill in store.get_all_skills()? {
        let Some(package_id) = package_id_for_skill_record(&skill) else {
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
        let package_id = package_id_for_skill_record(&skill);
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
    artifacts.extend(build_resource_artifacts(store)?);
    artifacts.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(artifacts)
}

fn build_resource_artifacts(store: &SkillStore) -> Result<Vec<EnvArtifact>> {
    let mut artifacts = Vec::new();
    for tool in tool_service::list_tool_info(store) {
        for resource in &tool.resources {
            if let Some(artifact) = resource_artifact_for_tool_resource(&tool.key, resource) {
                artifacts.push(artifact);
            }
        }
    }
    Ok(artifacts)
}

fn resource_artifact_for_tool_resource(
    tool_key: &str,
    resource: &tool_service::ToolResourceInfo,
) -> Option<EnvArtifact> {
    if !resource.exists || resource.kind == "skill" {
        return None;
    }
    Some(EnvArtifact {
        id: resource_artifact_id(tool_key, resource),
        kind: resource.kind.clone(),
        name: format!("{tool_key}:{}:{}", resource.scope, resource.kind),
        path: resource_repository_relative_path(tool_key, resource),
        source: EnvArtifactSource {
            source_type: "tool_resource".to_string(),
            confidence: "exact".to_string(),
            package: None,
            path: Some(resource.path_template.clone()),
            revision: None,
        },
        owner: Some(EnvArtifactOwner {
            owner_type: "tool".to_string(),
            id: tool_key.to_string(),
        }),
        deployed_to: vec![EnvDeployTarget {
            tool: tool_key.to_string(),
            path: portable_home_path(&resource.path),
            mode: resource.deploy.clone(),
            status: "active".to_string(),
        }],
    })
}

fn resource_artifact_id(tool_key: &str, resource: &tool_service::ToolResourceInfo) -> String {
    format!(
        "resource:{tool_key}:{}:{}:{}",
        resource.scope, resource.kind, resource.path_template
    )
}

fn resource_repository_relative_path(
    tool_key: &str,
    resource: &tool_service::ToolResourceInfo,
) -> String {
    let path = PathBuf::from("artifacts")
        .join(tool_key)
        .join(&resource.scope)
        .join(&resource.kind)
        .join(safe_relative_path(&resource.path_template));
    path.to_string_lossy().replace('\\', "/")
}

fn resource_repository_path(tool_key: &str, resource: &tool_service::ToolResourceInfo) -> PathBuf {
    central_repo::skills_dir().join(resource_repository_relative_path(tool_key, resource))
}

fn safe_relative_path(path: &str) -> PathBuf {
    let trimmed = path.trim_start_matches('/').trim_start_matches("~/");
    let mut out = PathBuf::new();
    for component in Path::new(trimmed).components() {
        if let std::path::Component::Normal(part) = component {
            out.push(part);
        }
    }
    out
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
    let local_linked = skill.source_type == "local_linked";
    EnvSource {
        source_type: skill.source_type.clone(),
        mode: local_source.then(|| match skill.source_type.as_str() {
            "local_created" => "created".to_string(),
            "local_linked" => "linked".to_string(),
            _ => "vendored".to_string(),
        }),
        confidence: "exact".to_string(),
        reference: if local_linked {
            skill.source_ref.clone()
        } else if local_source {
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
    if let Some(skill_id) = artifact_id.strip_prefix("skill:") {
        return Ok(store
            .get_skill_by_id(skill_id)?
            .and_then(|skill| skill.content_hash));
    }

    for tool in tool_service::list_tool_info(store) {
        for resource in &tool.resources {
            if resource.kind == "skill" || resource_artifact_id(&tool.key, resource) != artifact_id
            {
                continue;
            }
            let path = resource_repository_path(&tool.key, resource);
            if path.exists() {
                return Ok(content_hash::hash_path(&path).ok());
            }
        }
    }
    Ok(None)
}

fn current_artifact_content_hash(store: &SkillStore, artifact_id: &str) -> Result<Option<String>> {
    if let Some(skill_id) = artifact_id.strip_prefix("skill:") {
        return Ok(store
            .get_skill_by_id(skill_id)?
            .and_then(|skill| skill.content_hash));
    }

    for tool in tool_service::list_tool_info(store) {
        for resource in &tool.resources {
            if resource.kind == "skill" || resource_artifact_id(&tool.key, resource) != artifact_id
            {
                continue;
            }
            let path = Path::new(&resource.path);
            if path.exists() {
                return Ok(content_hash::hash_path(path).ok());
            }
        }
    }
    Ok(None)
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
                match skill.source_type.as_str() {
                    "local_created" => "local_created".to_string(),
                    "local_linked" => "local_linked".to_string(),
                    _ => "local_vendored".to_string(),
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

pub fn package_id_for_skill_record(skill: &SkillRecord) -> Option<String> {
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
        "local" | "import" | "local_created" | "local_linked" | "local_vendored" | "local_package"
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

fn changed_artifacts_from_locks(expected: &EnvLock, current: &EnvLock) -> Vec<EnvArtifactDrift> {
    let current_by_id: BTreeMap<&str, &EnvLockedArtifact> = current
        .artifacts
        .iter()
        .map(|artifact| (artifact.id.as_str(), artifact))
        .collect();
    let mut changed = Vec::new();

    for expected_artifact in &expected.artifacts {
        let Some(current_artifact) = current_by_id.get(expected_artifact.id.as_str()) else {
            continue;
        };
        if expected_artifact.content_hash != current_artifact.content_hash {
            changed.push(EnvArtifactDrift {
                id: expected_artifact.id.clone(),
                kind: expected_artifact.kind.clone(),
                expected_hash: expected_artifact.content_hash.clone(),
                current_hash: current_artifact.content_hash.clone(),
            });
        }
    }

    changed
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

fn expand_portable_path(path: &str) -> PathBuf {
    if path == "~" {
        return dirs::home_dir().unwrap_or_else(|| PathBuf::from(path));
    }
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest);
        }
    }
    PathBuf::from(path)
}

fn resolve_repo_relative_path(path: &str) -> PathBuf {
    let candidate = expand_portable_path(path);
    if candidate.is_absolute() {
        candidate
    } else {
        central_repo::skills_dir().join(safe_relative_path(path))
    }
}

#[allow(clippy::too_many_arguments)]
fn export_resource_path(
    id: &str,
    tool: &str,
    kind: &str,
    source: &Path,
    target: &Path,
    deploy: &str,
    overwrite: bool,
    dry_run: bool,
) -> Result<EnvResourceActionItem> {
    let status = if target.exists() {
        if paths_have_same_hash(source, target) {
            "unchanged"
        } else if overwrite {
            if dry_run {
                "would_update"
            } else {
                remove_path_if_exists(target)?;
                copy_path(source, target)?;
                "updated"
            }
        } else {
            "skipped_existing"
        }
    } else if dry_run {
        "would_create"
    } else {
        copy_path(source, target)?;
        "created"
    };

    Ok(EnvResourceActionItem {
        id: id.to_string(),
        tool: tool.to_string(),
        kind: kind.to_string(),
        source_path: source.to_string_lossy().to_string(),
        target_path: target.to_string_lossy().to_string(),
        deploy: deploy.to_string(),
        status: status.to_string(),
        backup_path: None,
        error: None,
    })
}

fn deploy_resource_path(
    id: &str,
    tool: &str,
    kind: &str,
    source: &Path,
    target: &Path,
    deploy: &str,
    dry_run: bool,
) -> Result<EnvResourceActionItem> {
    match deploy {
        "merge_toml" => deploy_merge_toml(id, tool, kind, source, target, deploy, dry_run),
        "merge_json" => deploy_merge_json(id, tool, kind, source, target, deploy, dry_run),
        _ => deploy_copy(id, tool, kind, source, target, deploy, dry_run),
    }
}

fn deploy_copy(
    id: &str,
    tool: &str,
    kind: &str,
    source: &Path,
    target: &Path,
    deploy: &str,
    dry_run: bool,
) -> Result<EnvResourceActionItem> {
    let unchanged = target.exists() && paths_have_same_hash(source, target);
    if unchanged {
        return Ok(resource_action_item(
            id,
            tool,
            kind,
            source,
            target,
            deploy,
            "unchanged",
            None,
            None,
        ));
    }

    let status = if target.exists() {
        if dry_run {
            "would_update"
        } else {
            let backup = backup_existing_path(target)?;
            remove_path_if_exists(target)?;
            copy_path(source, target)?;
            return Ok(resource_action_item(
                id, tool, kind, source, target, deploy, "updated", backup, None,
            ));
        }
    } else if dry_run {
        "would_create"
    } else {
        copy_path(source, target)?;
        "created"
    };

    Ok(resource_action_item(
        id, tool, kind, source, target, deploy, status, None, None,
    ))
}

fn deploy_merge_toml(
    id: &str,
    tool: &str,
    kind: &str,
    source: &Path,
    target: &Path,
    deploy: &str,
    dry_run: bool,
) -> Result<EnvResourceActionItem> {
    let shared = fs::read_to_string(source)
        .with_context(|| format!("failed to read {}", source.display()))?;
    let current = fs::read_to_string(target).unwrap_or_default();
    let merged = merge_toml_documents(&current, &shared)?;
    deploy_generated_file(
        id,
        tool,
        kind,
        source,
        target,
        deploy,
        merged.into_bytes(),
        dry_run,
    )
}

fn deploy_merge_json(
    id: &str,
    tool: &str,
    kind: &str,
    source: &Path,
    target: &Path,
    deploy: &str,
    dry_run: bool,
) -> Result<EnvResourceActionItem> {
    if source.is_dir() {
        let unchanged = target.exists() && paths_have_same_hash(source, target);
        if unchanged {
            return Ok(resource_action_item(
                id,
                tool,
                kind,
                source,
                target,
                deploy,
                "unchanged",
                None,
                None,
            ));
        }
        if dry_run {
            return Ok(resource_action_item(
                id,
                tool,
                kind,
                source,
                target,
                deploy,
                if target.exists() {
                    "would_update"
                } else {
                    "would_create"
                },
                None,
                None,
            ));
        }
        let backup = if target.exists() {
            backup_existing_path(target)?
        } else {
            None
        };
        merge_json_dir(source, target)?;
        return Ok(resource_action_item(
            id,
            tool,
            kind,
            source,
            target,
            deploy,
            if backup.is_some() {
                "updated"
            } else {
                "created"
            },
            backup,
            None,
        ));
    }

    let shared = fs::read_to_string(source)
        .with_context(|| format!("failed to read {}", source.display()))?;
    let current = fs::read_to_string(target).unwrap_or_default();
    let merged = merge_json_documents(&current, &shared)?;
    deploy_generated_file(
        id,
        tool,
        kind,
        source,
        target,
        deploy,
        merged.into_bytes(),
        dry_run,
    )
}

fn deploy_generated_file(
    id: &str,
    tool: &str,
    kind: &str,
    source: &Path,
    target: &Path,
    deploy: &str,
    content: Vec<u8>,
    dry_run: bool,
) -> Result<EnvResourceActionItem> {
    if target.exists() && fs::read(target).unwrap_or_default() == content {
        return Ok(resource_action_item(
            id,
            tool,
            kind,
            source,
            target,
            deploy,
            "unchanged",
            None,
            None,
        ));
    }

    if dry_run {
        return Ok(resource_action_item(
            id,
            tool,
            kind,
            source,
            target,
            deploy,
            if target.exists() {
                "would_update"
            } else {
                "would_create"
            },
            None,
            None,
        ));
    }

    let backup = if target.exists() {
        backup_existing_path(target)?
    } else {
        None
    };
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(target, content).with_context(|| format!("failed to write {}", target.display()))?;
    Ok(resource_action_item(
        id,
        tool,
        kind,
        source,
        target,
        deploy,
        if backup.is_some() {
            "updated"
        } else {
            "created"
        },
        backup,
        None,
    ))
}

#[allow(clippy::too_many_arguments)]
fn resource_action_item(
    id: &str,
    tool: &str,
    kind: &str,
    source: &Path,
    target: &Path,
    deploy: &str,
    status: &str,
    backup_path: Option<PathBuf>,
    error: Option<String>,
) -> EnvResourceActionItem {
    EnvResourceActionItem {
        id: id.to_string(),
        tool: tool.to_string(),
        kind: kind.to_string(),
        source_path: source.to_string_lossy().to_string(),
        target_path: target.to_string_lossy().to_string(),
        deploy: deploy.to_string(),
        status: status.to_string(),
        backup_path: backup_path.map(|path| path.to_string_lossy().to_string()),
        error,
    }
}

fn merge_toml_documents(current: &str, shared: &str) -> Result<String> {
    let mut base = if current.trim().is_empty() {
        toml::Value::Table(Default::default())
    } else {
        toml::from_str(current).context("failed to parse existing TOML")?
    };
    let overlay: toml::Value = toml::from_str(shared).context("failed to parse shared TOML")?;
    merge_toml_values(&mut base, overlay);
    let mut rendered = toml::to_string_pretty(&base)?;
    if !rendered.ends_with('\n') {
        rendered.push('\n');
    }
    Ok(rendered)
}

fn merge_toml_values(base: &mut toml::Value, overlay: toml::Value) {
    match (base, overlay) {
        (toml::Value::Table(base_table), toml::Value::Table(overlay_table)) => {
            for (key, value) in overlay_table {
                if let Some(existing) = base_table.get_mut(&key) {
                    merge_toml_values(existing, value);
                } else {
                    base_table.insert(key, value);
                }
            }
        }
        (base_value, overlay_value) => *base_value = overlay_value,
    }
}

fn merge_json_documents(current: &str, shared: &str) -> Result<String> {
    let mut base = if current.trim().is_empty() {
        serde_json::Value::Object(Default::default())
    } else {
        serde_json::from_str(current).context("failed to parse existing JSON")?
    };
    let overlay: serde_json::Value =
        serde_json::from_str(shared).context("failed to parse shared JSON")?;
    merge_json_values(&mut base, overlay);
    let mut rendered = serde_json::to_string_pretty(&base)?;
    rendered.push('\n');
    Ok(rendered)
}

fn merge_json_values(base: &mut serde_json::Value, overlay: serde_json::Value) {
    match (base, overlay) {
        (serde_json::Value::Object(base_map), serde_json::Value::Object(overlay_map)) => {
            for (key, value) in overlay_map {
                if let Some(existing) = base_map.get_mut(&key) {
                    merge_json_values(existing, value);
                } else {
                    base_map.insert(key, value);
                }
            }
        }
        (base_value, overlay_value) => *base_value = overlay_value,
    }
}

fn merge_json_dir(source: &Path, target: &Path) -> Result<()> {
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            merge_json_dir(&source_path, &target_path)?;
        } else if source_path.extension().is_some_and(|ext| ext == "json") {
            let shared = fs::read_to_string(&source_path)?;
            let current = fs::read_to_string(&target_path).unwrap_or_default();
            let merged = merge_json_documents(&current, &shared)?;
            if let Some(parent) = target_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(target_path, merged)?;
        } else {
            copy_path(&source_path, &target_path)?;
        }
    }
    Ok(())
}

fn backup_existing_path(path: &Path) -> Result<Option<PathBuf>> {
    if !path.exists() {
        return Ok(None);
    }
    fs::create_dir_all(machine_backups_dir())?;
    let file_name = path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "resource".to_string());
    let backup_name = format!(
        "{}-{}",
        Utc::now().format("%Y%m%d-%H%M%S%.3f"),
        file_name.replace('/', "_")
    );
    let backup_path = machine_backups_dir().join(backup_name);
    copy_path(path, &backup_path)?;
    Ok(Some(backup_path))
}

fn paths_have_same_hash(left: &Path, right: &Path) -> bool {
    if !left.exists() || !right.exists() {
        return false;
    }
    match (
        content_hash::hash_path(left),
        content_hash::hash_path(right),
    ) {
        (Ok(left_hash), Ok(right_hash)) => left_hash == right_hash,
        _ => false,
    }
}

fn copy_path(source: &Path, target: &Path) -> Result<()> {
    if source.is_dir() {
        copy_dir_recursive(source, target)
    } else {
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(source, target).with_context(|| {
            format!(
                "failed to copy {} to {}",
                source.display(),
                target.display()
            )
        })?;
        Ok(())
    }
}

fn copy_dir_recursive(source: &Path, target: &Path) -> Result<()> {
    fs::create_dir_all(target)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_recursive(&source_path, &target_path)?;
        } else {
            copy_path(&source_path, &target_path)?;
        }
    }
    Ok(())
}

fn remove_path_if_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }
    if path.is_dir() {
        fs::remove_dir_all(path)?;
    } else {
        fs::remove_file(path)?;
    }
    Ok(())
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
        let package_id = package_id_for_skill_record(&skill).unwrap();
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

        assert!(package_id_for_skill_record(&skill).is_none());
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
        assert!(package_id_for_skill_record(&skill).is_none());
    }

    #[test]
    fn local_linked_skill_keeps_origin_id_and_redacts_path() {
        let mut skill = skill("local_linked");
        skill.source_ref = Some("personal-skills/review".to_string());
        skill.source_ref_resolved = Some("/Users/example/dev/review".to_string());
        let source = env_source_for_skill(&skill, true);
        let artifact_source = artifact_source_for_skill(&skill, None);

        assert_eq!(source.source_type, "local_linked");
        assert_eq!(source.mode.as_deref(), Some("linked"));
        assert_eq!(source.reference.as_deref(), Some("personal-skills/review"));
        assert!(source.resolved_reference.is_none());
        assert_eq!(artifact_source.source_type, "local_linked");
        assert!(package_id_for_skill_record(&skill).is_none());
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

    #[test]
    fn changed_artifacts_from_locks_reports_hash_drift() {
        let expected = EnvLock {
            version: 1,
            generated_at: "now".to_string(),
            created_by: "test".to_string(),
            packages: Vec::new(),
            artifacts: vec![EnvLockedArtifact {
                id: "skill:review".to_string(),
                kind: "skill".to_string(),
                owner_package: None,
                content_hash: Some("old".to_string()),
            }],
            skills: Vec::new(),
        };
        let current = EnvLock {
            artifacts: vec![EnvLockedArtifact {
                id: "skill:review".to_string(),
                kind: "skill".to_string(),
                owner_package: None,
                content_hash: Some("new".to_string()),
            }],
            ..expected.clone()
        };

        let changed = changed_artifacts_from_locks(&expected, &current);

        assert_eq!(changed.len(), 1);
        assert_eq!(changed[0].id, "skill:review");
        assert_eq!(changed[0].expected_hash.as_deref(), Some("old"));
        assert_eq!(changed[0].current_hash.as_deref(), Some("new"));
    }

    #[test]
    fn tool_resources_are_written_with_portable_paths() {
        let tool = tool_service::ToolInfo {
            key: "codex".to_string(),
            display_name: "Codex".to_string(),
            installed: true,
            skills_dir: dirs::home_dir()
                .unwrap()
                .join(".agents")
                .join("skills")
                .to_string_lossy()
                .to_string(),
            enabled: true,
            is_custom: false,
            has_path_override: false,
            project_relative_skills_dir: Some(".codex/skills".to_string()),
            category: crate::core::tool_adapters::ToolCategory::Coding,
            resources: vec![tool_service::ToolResourceInfo {
                kind: "config".to_string(),
                scope: "global".to_string(),
                path: dirs::home_dir()
                    .unwrap()
                    .join(".codex")
                    .join("config.toml")
                    .to_string_lossy()
                    .to_string(),
                path_template: ".codex/config.toml".to_string(),
                deploy: "merge_toml".to_string(),
                scan: "file".to_string(),
                exists: false,
            }],
        };

        let resources = env_resources_for_tool(&tool);

        assert_eq!(resources.len(), 1);
        assert_eq!(resources[0].kind, "config");
        assert_eq!(resources[0].scope, "global");
        assert_eq!(resources[0].path, "~/.codex/config.toml");
        assert_eq!(resources[0].deploy, "merge_toml");
        assert_eq!(resources[0].scan, "file");
    }

    #[test]
    fn existing_non_skill_resource_becomes_artifact() {
        let resource = tool_service::ToolResourceInfo {
            kind: "config".to_string(),
            scope: "global".to_string(),
            path: dirs::home_dir()
                .unwrap()
                .join(".codex")
                .join("config.toml")
                .to_string_lossy()
                .to_string(),
            path_template: ".codex/config.toml".to_string(),
            deploy: "merge_toml".to_string(),
            scan: "file".to_string(),
            exists: true,
        };

        let artifact = resource_artifact_for_tool_resource("codex", &resource).unwrap();

        assert_eq!(
            artifact.id,
            "resource:codex:global:config:.codex/config.toml"
        );
        assert_eq!(artifact.kind, "config");
        assert_eq!(
            artifact.path,
            "artifacts/codex/global/config/.codex/config.toml"
        );
        assert_eq!(artifact.source.source_type, "tool_resource");
        assert_eq!(artifact.source.path.as_deref(), Some(".codex/config.toml"));
        assert_eq!(artifact.owner.as_ref().unwrap().owner_type, "tool");
        assert_eq!(artifact.owner.as_ref().unwrap().id, "codex");
        assert_eq!(artifact.deployed_to.len(), 1);
        assert_eq!(artifact.deployed_to[0].path, "~/.codex/config.toml");
        assert_eq!(artifact.deployed_to[0].mode, "merge_toml");
    }

    #[test]
    fn machine_local_overlay_records_linked_origin_paths() {
        let _guard = central_repo::test_base_dir_lock();
        let tmp = tempfile::tempdir().unwrap();
        let base = tmp.path().join("repo");
        central_repo::set_test_base_dir_override(Some(base.clone()));
        std::fs::create_dir_all(central_repo::skills_dir()).unwrap();
        let linked_path = tmp.path().join("dev").join("review");
        std::fs::create_dir_all(&linked_path).unwrap();

        record_linked_origin_path("personal/review", &linked_path).unwrap();

        let overlay = read_machine_local_overlay().unwrap();
        assert_eq!(
            overlay.origins["personal/review"].path,
            linked_path.to_string_lossy()
        );
        assert_eq!(
            linked_origin_path("personal/review").unwrap().as_deref(),
            Some(linked_path.as_path())
        );

        central_repo::set_test_base_dir_override(None);
    }

    #[test]
    fn merge_toml_documents_preserves_local_keys_and_applies_shared_keys() {
        let current = r#"
model = "gpt-5"
approval_policy = "on-request"

[profiles.personal]
temperature = 0.2
machine_only = true
"#;
        let shared = r#"
model = "gpt-5.1"

[profiles.personal]
temperature = 0.1
"#;

        let merged = merge_toml_documents(current, shared).unwrap();

        assert!(merged.contains("model = \"gpt-5.1\""));
        assert!(merged.contains("approval_policy = \"on-request\""));
        assert!(merged.contains("temperature = 0.1"));
        assert!(merged.contains("machine_only = true"));
    }

    #[test]
    fn deploy_merge_toml_backs_up_current_file() {
        let _guard = central_repo::test_base_dir_lock();
        let tmp = tempfile::tempdir().unwrap();
        let base = tmp.path().join("repo");
        central_repo::set_test_base_dir_override(Some(base.clone()));
        std::fs::create_dir_all(central_repo::skills_dir()).unwrap();
        let source = tmp.path().join("source.toml");
        let target = tmp.path().join("target.toml");
        std::fs::write(&source, "model = \"gpt-5.1\"\n").unwrap();
        std::fs::write(
            &target,
            "model = \"gpt-5\"\napproval_policy = \"on-request\"\n",
        )
        .unwrap();

        let item = deploy_resource_path(
            "resource:codex:global:config:.codex/config.toml",
            "codex",
            "config",
            &source,
            &target,
            "merge_toml",
            false,
        )
        .unwrap();

        let written = std::fs::read_to_string(&target).unwrap();
        assert_eq!(item.status, "updated");
        assert!(item.backup_path.is_some());
        assert!(written.contains("model = \"gpt-5.1\""));
        assert!(written.contains("approval_policy = \"on-request\""));

        central_repo::set_test_base_dir_override(None);
    }

    #[test]
    fn writing_environment_protects_machine_local_overlay() {
        let _guard = central_repo::test_base_dir_lock();
        let tmp = tempfile::tempdir().unwrap();
        let base = tmp.path().join("repo");
        central_repo::set_test_base_dir_override(Some(base.clone()));
        std::fs::create_dir_all(central_repo::skills_dir()).unwrap();
        let store = SkillStore::new(&base.join("test.db")).unwrap();

        write_current_environment(&store, true).unwrap();

        let gitignore = std::fs::read_to_string(central_repo::skills_dir().join(".gitignore"))
            .expect(".gitignore should be written");
        assert!(gitignore.contains("/machine/machine.local.yaml"));
        assert!(gitignore.contains("/machine/secrets.local.yaml"));
        assert!(gitignore.contains("/machine/backups/"));
        assert!(machine_dir().is_dir());

        central_repo::set_test_base_dir_override(None);
    }

    #[test]
    fn resource_export_dry_run_does_not_create_machine_overlay() {
        let _guard = central_repo::test_base_dir_lock();
        let tmp = tempfile::tempdir().unwrap();
        let base = tmp.path().join("repo");
        central_repo::set_test_base_dir_override(Some(base.clone()));
        std::fs::create_dir_all(central_repo::skills_dir()).unwrap();
        let store = SkillStore::new(&base.join("test.db")).unwrap();

        export_resource_artifacts(&store, false, true).unwrap();

        assert!(!machine_dir().exists());
        assert!(!central_repo::skills_dir().join(".gitignore").exists());

        central_repo::set_test_base_dir_override(None);
    }
}
