<p align="center">
  <img src="assets/icon.png" width="80" />
</p>

<h1 align="center">AgentPort</h1>

<p align="center">
  A portable environment manager for AI coding agents, skills, plugins, and workspaces.
</p>

<p align="center">
  <a href="./README.zh-CN.md">中文说明</a>
  &nbsp;·&nbsp;
  <a href="https://x.com/JayTL00">@JayTL00 on X</a>
  &nbsp;·&nbsp;
  <a href="https://buymeacoffee.com/jaytl">Buy me a coffee</a>
</p>

<p align="center">
  <img src="assets/demo/library.png" width="800" alt="AgentPort Skills Library" />
</p>

<p align="center"><strong>Install Skills — Marketplace</strong></p>
<p align="center"><img src="assets/demo/install-skills.png" width="800" alt="Install Skills Marketplace" /></p>

<p align="center"><strong>Global Workspace</strong></p>
<p align="center"><img src="assets/demo/global-workspace.png" width="800" alt="Global Workspace" /></p>

<p align="center"><strong>Agent Workspace</strong></p>
<p align="center"><img src="assets/demo/agent-workspace.png" width="800" alt="Agent Workspace" /></p>

<p align="center"><strong>Project Workspace</strong></p>
<p align="center"><img src="assets/demo/project-workspace.png" width="800" alt="Project Workspace" /></p>

<p align="center"><strong>Settings</strong></p>
<p align="center"><img src="assets/demo/settings.png" width="800" alt="Settings" /></p>

## What AgentPort Manages

AgentPort is organized around three layers. The sidebar follows this same model.

1. **Library** — Your reusable skills live in one central library. Install skills from Git repos, local folders, archives, or the marketplace, then tag, preview, update, and back them up with Git.
2. **Environment** — AgentPort turns your agent setup into one portable environment declaration: profiles, package provenance, resource artifacts, machine-local overlays, and drift reports. The Environment is a singleton for the current central repo, not a list of separate environments.
3. **Workspaces** — Workspaces are where skills are applied. A global workspace changes an agent's global skills directory; a project workspace changes skills inside one project; presets are reusable groups you can apply to either.

## Sidebar Guide

| Sidebar section | Pages | What it means |
|-----------------|-------|---------------|
| **Home** | **Home** | Product-level command center. It summarizes the Library, the singleton Environment, and Workspaces, then points to the next action that needs attention. |
| **Library** | **Skills**, **Install Skills** | Manage the central reusable skill library. Use this when you are installing, tagging, deleting, updating, or inspecting skills. |
| **Environment** | **Overview**, **Profiles**, **Packages**, **Artifacts**, **Machines**, **Diff** | Inspect the singleton environment AgentPort will sync across machines. Overview initializes or rebuilds `agentport.yaml` / `agentport.lock`; Profiles describe desired agent state; Packages record plugin/source ownership; Artifacts are managed files/resources; Machines keeps local-only overlays and secret presence; Diff shows what changed on this machine. |
| **Workspaces** | **Presets**, **Global Workspace**, **Project Workspaces** | Apply skills to real agent locations. Presets are reusable groups, Global Workspace targets each agent's global skills folder, and Project Workspaces target per-project skill folders. |
| **Settings** | **Settings** | Configure agent paths, sync mode, Git remote, proxy, theme, language, updates, custom tools, and logs. |

If you only want to manage skills on one computer, you can mostly use **Library** and **Workspaces**. If you want the same agent environment on multiple computers or across tools like Claude Code, Codex, OpenCode, and Gemini CLI, use **Environment** as the source of truth.

## Features

- **Unified skill library** — Install skills from Git repos, local folders, `.zip` / `.skill` archives, or the [skills.sh](https://skills.sh) marketplace. Everything goes into one central repo, which defaults to `~/.skills-manager` and can be customized in **Settings**.
- **AgentPort environment sync** — Generate and apply portable environment metadata for skills, plugins/packages, managed artifacts, profiles, and machine-local overlays.
- **Marketplace + AI search** — Browse popular skills from the marketplace, run keyword search, or enable SkillsMP AI search with your API key.
- **Presets** — Group skills into named presets. In any workspace, click a preset pill to activate or deactivate all its skills for the current agent scope.
- **Global Workspace** — Each agent gets its own page listing every skill in its global folder, including skills installed outside AgentPort, so the view reflects what the agent actually sees.
- **Project Workspaces** — View and manage project-local skill folders for supported agents, compare them with your central library, and sync changes in either direction.
- **Multi-tool sync** — Sync skills to supported tools via symlink or copy. Skill cards show agent badges that can install or remove that skill for an agent in one click.
- **Packages and provenance** — Track where skills and artifacts came from, including plugin-owned packages and local skills that have been vendored into the central library.
- **Machine-aware safety** — Keep machine-local paths and secrets out of shared state while still recording whether required local overlays exist.
- **Drift and copy-target repair** — Compare this machine with the shared manifest, preview apply/export actions, and pull or discard changed copy targets.
- **Skill tagging and filters** — Tag skills, use tags to group similar skills, and filter by source or tag, including an **Untagged** filter.
- **Git backup and restore** — Version-control your skill library with Git for backup and multi-machine sync, then restore snapshot versions from Version History when needed.
- **Custom tools** — Add your own agents/tools with custom skills directories, or override the default path for any built-in tool.
- **Activity log & Export Logs** — Install / remove / update / sync operations are recorded locally. Use **Settings → Export Logs** to bundle recent logs and activity history into a single zip for issue reports.

## Core Concepts

<p align="center">
  <img src="assets/diagram-concept-map.png" width="640" alt="Concept map: Library, Preset, Global Workspace, Project Workspace, Agent" />
</p>

- **Skills are reusable agent capabilities** — A skill is usually a folder with a `SKILL.md` file and supporting files. Skills can be installed from Git, local folders, archives, or marketplaces.
- **Presets are reusable skill groups** — A preset is a named collection of skills. Activate a preset in any workspace to add all its skills to the selected agents; deactivate to remove them. Applying a preset is a one-time operation, not a live subscription.
- **Profiles describe desired agent state** — A profile is the portable form of "which agents should have which skills and resources". Profiles are generated from your presets and AgentPort metadata.
- **Packages describe source ownership** — A package records where a group of skills/artifacts came from, such as a plugin repo, marketplace package, or imported local source.
- **Artifacts are managed files/resources** — Skills are one type of artifact. AgentPort can also model related resources such as commands, hooks, config snippets, and other agent-specific files.
- **Machines keep local-only data local** — Machine overlays store private paths, local origins, and secret presence separately from shared manifest data.
- **Global Workspace manages per-agent global skills** — Each installed agent has its own global skills folder (e.g. `~/.claude/skills/` for Claude Code). Each agent page lists everything in that folder, even skills installed without AgentPort.
- **Project Workspaces are project-local skill sets** — A project workspace manages skills inside a specific project (e.g. `<project>/.claude/skills/`). Skills added here only apply to that project.
- **Diff tells you what changed** — Use Diff before applying or exporting environment changes, especially when moving between computers or after editing skills through an agent.

## Quick Start

1. Open **Install Skills** and import skills from local folders, Git repositories, archives, or the marketplace.
2. Open **Skills** to review your central library. Add tags, inspect docs, and decide which skills belong together.
3. Create or select a **Preset** for a reusable group, such as "default coding", "review", or "frontend".
4. Open **Global Workspace** and pick an agent such as Claude Code, Codex, OpenCode, or Gemini CLI. Apply a preset or use **+ Add Skills** to add individual skills.
5. For project-only behavior, link a **Project Workspace** and apply skills there instead of globally.
6. For multi-machine sync, open **Environment → Overview**, initialize/rebuild the AgentPort manifest, then inspect **Profiles**, **Packages**, **Artifacts**, **Machines**, and **Diff**.
7. Configure Git in **Settings**, then use **Sync to Git** from **Skills** to back up and move the central library between computers.

## Git Backup

Back up the `skills/` folder inside your current central repository to a Git repo for version history and multi-machine sync. By default this is `~/.skills-manager/skills/`.

### Quick setup

1. Create a private repository (recommended).
2. Open **Settings → Git Sync Configuration** and save your remote URL.
3. Open **Skills**.
4. Choose one:
- Existing remote: click **Start Backup** to clone from the configured remote.
- New local repo: click **Start Backup** to initialize locally, then use **Sync to Git**.
5. Use **Sync to Git** from the Skills toolbar.

`Sync to Git` automatically handles pull, commit, and push based on current repo status.
Each successful sync creates a snapshot version tag. You can open **Version History** in **Skills**, inspect the timeline, and restore any snapshot as a new commit.

### Authentication

- SSH URL (`git@github.com:...`): requires SSH key configured on your machine and added to GitHub.
- HTTPS URL (`https://github.com/...`): push usually requires a Personal Access Token (PAT).

> **Note:** The SQLite database (`skills-manager.db` inside your current central repository, `~/.skills-manager/skills-manager.db` by default) is not included in Git — it stores metadata that can be rebuilt by scanning the skill files.

## Supported Tools

Cursor · Claude Code · Codex · OpenCode · Amp · Kilo Code · Roo Code · Goose · Gemini CLI · GitHub Copilot · Windsurf · TRAE IDE · Antigravity · Clawdbot · Droid

You can also add custom tools in **Settings** and manage their skills the same way.

## In-App Help

The **Help** button in **Settings** mirrors the current product flow: the sidebar model, recommended workflows, presets, skill installation, the Library, the singleton Environment, the Global Workspace and the **+ Add Skills** sheet, Project Workspaces, Git backup, and environment-level settings. It is intended as the in-app version of this quick-start guide.

## Tech Stack

| Layer | Tech |
|-------|------|
| Frontend | React 19, TypeScript, Vite, Tailwind CSS |
| Desktop | Tauri 2 |
| Backend | Rust |
| Storage | SQLite (`rusqlite`) |
| i18n | react-i18next |

## Getting Started

### Prerequisites

- Node.js 18+
- Rust toolchain
- [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for your OS

### Development

```bash
npm install
npm run tauri:dev
```

### CLI

The repository includes an agent-friendly CLI built on the same Rust shared core used by the desktop app. Both the CLI and the desktop app go through the same SQLite database, central library, and sync engine.

```bash
# Repository / library overview
npm run cli -- repo status
npm run cli -- skills list
npm run cli -- skills show db

# Install skills (default: enter library only — does NOT sync to agents)
npm run cli -- skills install ./my-skill                       # local path
npm run cli -- skills install https://github.com/foo/bar.git   # git URL
npm run cli -- skills install vercel-labs/agent-skills@react-best-practices  # skills.sh
npm run cli -- skills install foo/bar --sync                   # add to active preset + sync to agents

# Update / check from upstream (git skills re-clone, local skills re-import source)
npm run cli -- skills update --all
npm run cli -- skills check --all

# Search the skills.sh marketplace (no API key needed)
npm run cli -- skills search react --limit 5

# Remove (--yes required; --dry-run available)
npm run cli -- skills remove <ref> --dry-run
npm run cli -- skills remove <ref> --yes

# Enable / disable skills by changing preset membership
npm run cli -- presets add-skill <preset> <ref>
npm run cli -- presets remove-skill <preset> <ref>

# Sync the active preset out to enabled agents
npm run cli -- skills sync --dry-run
npm run cli -- skills sync --tool claude_code

# Adopt skills that already exist in an agent directory (e.g. ~/.claude/skills/)
npm run cli -- skills adopt ~/.claude/skills --dry-run
npm run cli -- skills adopt ~/.claude/skills

# Tag
npm run cli -- skills tag add <ref> web frontend
npm run cli -- skills tag list

# Presets
npm run cli -- presets list
npm run cli -- presets preview Default
npm run cli -- presets apply Default
npm run cli -- presets add-skill <preset> <skill>
npm run cli -- presets remove-skill <preset> <skill>

# Export one skill to an arbitrary directory (one-shot copy, not managed)
npm run cli -- skills export db --dest ~/.claude/skills/db

# Git-backed skills repo
npm run cli -- git status
npm run cli -- git pull
npm run cli -- git commit -m "chore: update skills"
```

Available command groups:
- `repo` — inspect or change the configured base directory
- `tools` — list detected tool targets and paths
- `skills` — manage skills in the central library (`list / show / install / update / check / remove / enable / disable / sync / search / adopt / tag / export`)
- `presets` — list presets, preview / apply, add or remove skills from a preset
- `git` — operate on the git-backed `skills/` repository (`clone`, `pull`, `push`, `commit`, `versions`, `restore`)

Extra flags:
- `--skills-root <path>` — operate on a cloned/exported skills repo directly instead of the local app default. The manager's state (DB, presets, cache, logs) lives in `~/.skills-manager/external/<name>-<hash>/`, namespaced by the canonical path of the skills root, so the external checkout itself stays clean.
- `--json` — machine-readable output for scripts/agents

```bash
npm run -s cli -- --skills-root /path/to/my-skills --json skills list
```

#### Install the binary on PATH

Agents and scripts that invoke `skills-manager-cli` directly (without `npm run`) need the binary on PATH. Install it with:

```bash
npm run cli:install
# equivalent to:
# cargo install --path src-tauri --bin skills-manager-cli --locked --force
```

This drops the binary at `~/.cargo/bin/skills-manager-cli`. Re-run after pulling updates to refresh it.

#### Concurrent use with the desktop app

The CLI and desktop app share the same SQLite database. SQLite serializes writes safely, but the running app does not auto-refresh its in-memory caches when the CLI mutates state — restart or trigger a manual refresh in the app after `presets apply`, `git pull`, or other CLI write operations.

### Build

```bash
npm run tauri:build
npm run cli:build
```

## Troubleshooting

### macOS: Gatekeeper blocks the app on first launch

AgentPort is ad-hoc signed but not notarized (no paid Apple Developer ID), so macOS Gatekeeper will warn the first time you open it.

- **"App can't be opened because it is from an unidentified developer"** (releases from v1.20.0 onward) — Right-click the app in Finder and choose **Open**, then confirm in the dialog. Or open **System Settings → Privacy & Security** and click **Open Anyway** after the first failed launch.
- **"App is damaged and can't be opened"** (releases up to and including v1.19.0) — Run this in Terminal, then open the app again:

  ```bash
  xattr -cr /Applications/skills-manager.app
  ```

  Replace the path with wherever you placed the `.app` file if it's not in `/Applications`.

## License

MIT
