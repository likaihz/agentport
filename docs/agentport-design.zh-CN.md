# AgentPort 设计文档

状态：草案  
日期：2026-05-24  
来源项目：skills-manager  
推荐新名称：AgentPort

## 1. 背景

当前项目已经可以管理 AI coding agent 的 skills，并将 skills 同步到 Claude Code、Codex、OpenCode 等工具目录。这个能力很有价值，但真实使用场景已经超出了“同步 SKILL.md 文件”本身。

用户可能同时使用多个 coding agent，也可能在多台电脑之间迁移工作环境。为了获得更好的 agent 使用体验，用户还会安装第三方 skills、plugins、agents、commands、hooks、MCP servers、rules 和配置包，例如 Superpowers、ECC、oh-my-openagent 等。每次换 agent 或换电脑时，重复安装和配置这些内容的成本很高。

AgentPort 的目标是把这些分散的 agent 环境资产，组织成一套可声明、可同步、可审计、可恢复的 portable agent environment。

## 2. 产品定位

AgentPort 是一个 AI coding agent 环境管理器。

它不仅管理 skills，还管理：

- skills
- plugins
- agents
- commands
- hooks
- MCP servers
- rules
- prompts
- agent-specific settings
- global workspace config
- project workspace config
- 本地原创 skills
- profile / preset

一句话定位：

> AgentPort syncs and bootstraps your AI coding agent environment anywhere.

建议命名：

- 产品名：AgentPort
- CLI：`agentport`
- 配置目录：`~/.agentport`
- 主配置：`agentport.yaml`
- 锁文件：`agentport.lock`

当前项目可以在文档和迁移逻辑中描述为：

> AgentPort, formerly skills-manager.

## 3. 设计目标

1. 支持多 agent：Claude Code、Codex、OpenCode、Cursor、Gemini CLI 等。
2. 支持多机器：一台电脑配置好后，可以快速迁移到另一台电脑。
3. 支持多资产类型：skill、plugin、agent、command、hook、MCP、config 等。
4. 支持插件携带 skills：插件安装后产生的 skill 需要有明确来源和生命周期。
5. 支持本地原创 skills：本地 skill 内容默认可以进入 Git 备份和同步。
6. 支持 agent 修改 skill 后回流：coding agent 编辑 skill 后，可以同步回中心仓库。
7. 支持声明式 profile：用户可以声明“我的 agent 环境应该长什么样”。
8. 支持 dry-run 和 diff：任何会修改本机文件的操作都应该可以先预览。
9. 不同步 secrets：API key、token、本机绝对路径等只存在 machine-local overlay。
10. 保持当前项目优势：复用现有 central repo、CLI、tool adapters、sync engine、metadata、Git backup。

## 4. 非目标

AgentPort 不应该直接变成一个通用 dotfiles manager。

它可以管理 agent 相关配置，但不负责完整接管 shell、editor、OS、SSH、GPG 等通用开发环境配置。对于这些内容，AgentPort 可以通过外部命令或文档提示集成，但不要把边界扩大到不可控。

AgentPort 也不应该自动执行不可信安装脚本。第三方 package 可以声明 install steps，但执行前必须展示 plan，并要求用户确认。

## 5. 当前项目基础

当前 skills-manager 已经具备几个关键基础：

- 中心 skills 仓库：`~/.skills-manager/skills`
- 多工具适配器：`src-tauri/src/core/tool_adapters.rs`
- 同步引擎：`src-tauri/src/core/sync_engine.rs`
- skill metadata：`src-tauri/src/core/sync_metadata.rs`
- SQLite store：`src-tauri/src/core/skill_store.rs`
- preset/scenario：`src-tauri/src/core/scenario_service.rs`
- Git backup：`src-tauri/src/core/git_backup.rs`
- CLI：`src-tauri/src/bin/skills-manager-cli.rs`
- 本地 skill 导入和 reimport：`src-tauri/src/core/installer.rs`、`src-tauri/src/commands/skills.rs`

这些能力可以演进为 AgentPort 的底座，而不是重写。

## 6. 核心概念

### 6.1 Environment Repo

Environment Repo 是用户 agent 环境的 Git-backed source of truth。

建议目录：

```text
~/.agentport/
  repo/
    agentport.yaml
    agentport.lock
    profiles/
    packages/
    artifacts/
    skills/
    metadata/
  cache/
  logs/
  backups/
  machine/
    machine.local.yaml
    secrets.local.yaml
```

Git 应该默认管理：

- `agentport.yaml`
- `agentport.lock`
- `profiles/`
- `packages/`
- `artifacts/`
- `skills/`
- `metadata/`

Git 不应该管理：

- secrets
- machine-local absolute paths
- install cache
- logs
- temporary clone directories
- machine-specific overrides

### 6.2 Profile

Profile 表示一套期望的 agent 环境。

示例：

```yaml
version: 1

profiles:
  personal-default:
    description: Personal coding agent setup
    tools:
      claude_code:
        packages:
          - github:obra/superpowers
          - github:affaan-m/ECC
      codex:
        packages:
          - github:obra/superpowers
      opencode:
        packages:
          - github:code-yeongyu/oh-my-openagent
    artifacts:
      include:
        - skill:local/review-checklist
      exclude:
        - hook:superpowers/auto-commit
    mcp:
      include:
        - context7
        - exa
```

Profile 不是“当前机器状态”，而是“期望状态”。实际机器状态通过 scan 生成，和 profile 做 diff。

### 6.3 Package

Package 是一个上游来源，通常来自 GitHub repo、本地目录、压缩包或未来的 marketplace。

一个 package 可以提供多个 artifact。

示例：

```yaml
id: github:obra/superpowers
source:
  type: git
  repo: https://github.com/obra/superpowers.git
  revision: abc123
artifacts:
  - id: skill:superpowers/plan
    kind: skill
    path: skills/plan
  - id: command:superpowers/plan
    kind: command
    path: commands/plan.md
  - id: hook:superpowers/session-start
    kind: hook
    path: hooks/session-start.json
```

### 6.4 Artifact

Artifact 是 AgentPort 可以安装、同步、更新、卸载的最小管理单元。

Artifact 类型包括：

- `skill`
- `plugin`
- `agent`
- `command`
- `hook`
- `mcp_server`
- `config_patch`
- `rule`
- `prompt`
- `script`

Skill 不再是唯一顶层实体，而是 Artifact 的一种。

### 6.5 Target

Target 表示 artifact 被部署到哪里。

示例：

```yaml
targets:
  - tool: codex
    scope: global
    kind: skill
    path: ~/.codex/skills/review-checklist
    deploy: symlink
  - tool: claude_code
    scope: project
    kind: command
    path: .claude/commands/plan.md
    deploy: copy
```

Deploy mode 可以包括：

- `symlink`
- `copy`
- `merge_json`
- `merge_toml`
- `patch_text`
- `command`

配置文件类 artifact 应该优先使用结构化 merge，而不是字符串拼接。

## 7. 来源模型

现有项目里的 `source_type` 主要表达 `git`、`local`、`skillssh`。AgentPort 需要把来源升级成 provenance model。

每个 artifact 应该至少记录三类信息：

```text
acquired_from: 文件或配置从哪里获得
owned_by: 谁负责更新和卸载它
deployed_to: 它被部署到了哪些 agent 目标
```

示例：

```yaml
artifacts:
  - id: skill:superpowers/plan
    kind: skill
    acquired_from:
      type: package_artifact
      package: github:obra/superpowers
      path: skills/plan
      revision: abc123
    owned_by:
      type: package
      id: github:obra/superpowers
    deployed_to:
      - tool: claude_code
        scope: global
      - tool: codex
        scope: global
```

### 7.1 记录型来源

通过 AgentPort 安装的内容，来源应在安装时精确记录。

数据来自：

- 用户安装命令
- Git remote URL
- Git revision
- branch/tag
- subpath
- content hash
- package adapter
- package manifest
- deploy receipt

### 7.2 推断型来源

对于已经手工安装过的内容，AgentPort 只能通过 scan 推断。

推断可信度：

```text
exact      找到了 AgentPort receipt
matched    内容 hash 或路径匹配已知 package
probable   路径结构像某个 package，但没有强证据
unknown    无法判断，只能标记为 local/adopted
```

AgentPort 不应该把推断来源伪装成确定来源。来源记录需要包含 `confidence`。

## 8. 插件携带 skills 的处理

插件或 package 可能会安装 skills。这个场景必须作为一等模型支持。

例如：

```text
Package(superpowers)
  -> Skill(plan)
  -> Skill(review)
  -> Command(plan)
  -> Hook(session-start)
```

这些 skills 的来源不是普通 `git skill`，而是 `package_artifact`。

建议 metadata：

```json
{
  "artifact_id": "skill:superpowers/plan",
  "kind": "skill",
  "source": {
    "type": "package_artifact",
    "package_id": "github:obra/superpowers",
    "repo": "https://github.com/obra/superpowers.git",
    "revision": "abc123",
    "subpath": "skills/plan",
    "confidence": "exact"
  },
  "owner": {
    "type": "package",
    "id": "github:obra/superpowers"
  }
}
```

生命周期规则：

- 更新 package 时，更新它 owned 的 artifacts。
- 卸载 package 时，提示删除它带来的 artifacts。
- 用户可以把某个 artifact 从 package ownership 中 detach，转为独立管理。
- 如果用户修改了 package artifact，标记为 `dirty/customized`，更新时不直接覆盖。
- 如果多个 package 提供同名 skill，用 namespace 区分，例如 `superpowers/plan` 和 `ecc/plan`。
- 如果用户直接安装的 skill 与 package skill 冲突，用户直接安装版本优先，除非 profile 显式指定 package 版本。

## 9. 本地 skills 的处理

本地 skill 需要成为一等资产，而不是临时导入物。

建议三种模式：

```text
local_vendored      一次性导入，之后以 AgentPort repo 为准
local_linked        仍然关联本地开发目录，可 sync-in/reimport
local_created       直接在 AgentPort 中创建，天然由 Git 管理
```

### 9.1 local_vendored

默认模式。

用户从本地路径导入 skill 后，AgentPort 把内容复制到 Environment Repo 的 `skills/` 下，并参与 Git 同步。

共享 metadata 中不保存原始绝对路径。

```yaml
source:
  type: local
  mode: vendored
  original_path_policy: private
backup:
  type: git
  included: true
```

优点：

- 换电脑后 skill 内容完整恢复。
- 不依赖旧电脑上的原始路径。
- 不泄露本地目录结构。

限制：

- 新机器无法自动从旧机器的原始开发目录继续更新。

### 9.2 local_linked

适合用户把某个本地目录作为 skill 开发源。

共享 repo 只保存稳定 origin id，本机路径保存在 machine-local overlay。

```yaml
# Git 同步
source:
  type: local
  mode: linked
  origin_id: personal-skills/review-checklist

# 本机私有，不进 Git
origins:
  personal-skills/review-checklist:
    path: /Users/likai/Projects/skills/review-checklist
```

### 9.3 local_created

由 AgentPort 直接创建在中心 repo。

```bash
agentport skills create review-checklist
```

这种 skill 的 source of truth 天然是 Environment Repo，最适合由 coding agent 持续编辑。

## 10. 本地 skill 更新回流

Coding agent 可能会修改 skill。需要根据修改位置定义回流规则。

### 10.1 修改中心仓库副本

路径示例：

```text
~/.agentport/repo/skills/review-checklist
```

这是推荐方式。

中心仓库是 source of truth，修改后直接进入 Git diff。用户可以运行：

```bash
agentport git commit -m "update review skill"
agentport git push
```

### 10.2 修改原始本地开发目录

路径示例：

```text
~/Projects/skills/review-checklist
```

适用于 `local_linked`。

需要 sync-in：

```bash
agentport skills sync-in review-checklist
```

或者启用 watch：

```bash
agentport skills watch
```

Watch 应该 debounce，并在导入前做 hash 检查。

### 10.3 修改 agent 目录里的部署副本

路径示例：

```text
~/.codex/skills/review-checklist
~/.claude/skills/review-checklist
```

如果 deploy mode 是 `symlink`，修改等同于修改中心仓库。

如果 deploy mode 是 `copy`，修改只是 target drift。AgentPort 不应该静默接受，也不应该下次同步时悄悄覆盖。需要显式命令：

```bash
agentport skills pull-target review-checklist --tool codex
agentport skills discard-target review-checklist --tool codex
```

### 10.4 冲突检测

记录 hash：

```yaml
local_sync:
  origin_id: personal-skills/review-checklist
  last_origin_hash: h1
  last_central_hash: h1
  last_target_hashes:
    codex: h1
    claude_code: h1
```

判断规则：

```text
只有 origin 变了       -> 可以 sync-in
只有 central 变了      -> 保留 central，提示是否写回 origin
只有 copy target 变了  -> 标记 target drift
origin 和 central 都变了 -> conflict，不自动覆盖
central 和 target 都变了 -> conflict，不自动覆盖
```

## 11. Agent Adapter

当前 `ToolAdapter` 主要知道 skills 目录。AgentPort 需要升级为多资源 adapter。

建议结构：

```rust
pub struct AgentAdapter {
    pub key: String,
    pub display_name: String,
    pub resource_roots: Vec<ResourceRoot>,
}

pub struct ResourceRoot {
    pub kind: ResourceKind,
    pub scope: ResourceScope,
    pub path_template: String,
    pub deploy: DeployMode,
    pub scan: ScanMode,
}
```

示例：

```yaml
agents:
  codex:
    resources:
      - kind: skill
        scope: global
        path: ~/.codex/skills
        deploy: symlink
      - kind: config
        scope: global
        path: ~/.codex/config.toml
        deploy: merge_toml
      - kind: skill
        scope: project
        path: .codex/skills
        deploy: symlink

  claude_code:
    resources:
      - kind: skill
        scope: global
        path: ~/.claude/skills
        deploy: symlink
      - kind: command
        scope: global
        path: ~/.claude/commands
        deploy: copy
      - kind: hook
        scope: global
        path: ~/.claude/hooks
        deploy: merge_json

  opencode:
    resources:
      - kind: skill
        scope: global
        path: ~/.config/opencode/skills
        deploy: symlink
      - kind: skill
        scope: project
        path: .opencode/skills
        deploy: symlink
```

## 12. CLI 设计

CLI 是新机器初始化的关键路径，优先级高于 UI。

建议命令：

```bash
agentport env scan
agentport env diff personal-default
agentport env apply personal-default
agentport env doctor
agentport env adopt --profile personal-default
agentport env export-bootstrap-skill
```

Package：

```bash
agentport packages install github:obra/superpowers
agentport packages update github:obra/superpowers
agentport packages remove github:obra/superpowers
agentport packages list
```

Skills：

```bash
agentport skills create review-checklist
agentport skills import /path/to/skill --mode vendored
agentport skills link /path/to/skill --origin review-checklist-dev
agentport skills sync-in review-checklist
agentport skills pull-target review-checklist --tool codex
agentport skills discard-target review-checklist --tool codex
agentport skills watch
```

Git：

```bash
agentport git init
agentport git clone <url>
agentport git status
agentport git commit -m "update agent environment"
agentport git pull
agentport git push
```

所有会修改文件的命令都应该支持：

```bash
--dry-run
--json
--yes
```

## 13. Bootstrap Skill

AgentPort 应该可以生成一个“开发环境初始化 Skill”，让新的 coding agent 自主完成初始化。

这个 Skill 不应该包含复杂安装逻辑，而应该引导 agent 调用 CLI。

职责：

1. 识别 OS、shell、已安装 agent。
2. 检查 `agentport` CLI 是否存在。
3. 如果不存在，引导用户安装。
4. clone 或 pull Environment Repo。
5. 执行 `agentport env diff <profile>`。
6. 展示 plan，等待用户确认。
7. 执行 `agentport env apply <profile>`。
8. 执行 `agentport env doctor`。

示例生成命令：

```bash
agentport env export-bootstrap-skill --profile personal-default --dest ./bootstrap-agentport
```

## 14. UI 设计

UI 可以在 CLI 稳定后扩展。

新增页面：

- Environment
- Profiles
- Packages
- Artifacts
- Machines
- Drift / Diff

关键能力：

- 展示当前机器与 profile 的差异。
- 展示每个 package 提供了哪些 artifacts。
- 展示 skill 来源：direct git、package artifact、local vendored、local linked、adopted unknown。
- 展示 target 状态：synced、missing、drifted、conflict、dirty。
- 一键 apply profile。
- 一键 sync-in local linked skill。
- 一键 pull/discard copy target drift。
- 管理 machine-local overlay。
- 显示 secrets 缺失但不显示 secret 值。

## 15. 安全和隐私

AgentPort 必须默认保护用户本地信息。

规则：

- 不把 API key、token 写入 Git。
- 不把本机绝对路径写入共享 metadata。
- 不自动执行未知脚本。
- 不自动覆盖 dirty/customized artifact。
- 对 config merge 做备份。
- 对删除操作展示 plan。
- 对 package install steps 做来源展示。
- 对推断来源标记 confidence。

敏感数据放入：

```text
~/.agentport/machine/secrets.local.yaml
~/.agentport/machine/machine.local.yaml
```

这些文件必须默认进入 `.gitignore`。

## 16. 版本和锁文件

`agentport.yaml` 表示用户期望状态，`agentport.lock` 表示精确解析结果。

`agentport.yaml` 示例：

```yaml
version: 1
active_profile: personal-default

packages:
  github:obra/superpowers:
    source:
      type: git
      repo: https://github.com/obra/superpowers.git
      ref: main

profiles:
  personal-default:
    packages:
      - github:obra/superpowers
    tools:
      codex:
        enabled: true
      claude_code:
        enabled: true
```

`agentport.lock` 示例：

```yaml
version: 1
packages:
  github:obra/superpowers:
    resolved_revision: abc123
    artifacts:
      - id: skill:superpowers/plan
        kind: skill
        content_hash: sha256:...
        path: skills/plan
      - id: command:superpowers/plan
        kind: command
        content_hash: sha256:...
        path: commands/plan.md
```

## 17. 与现有代码的映射

建议渐进演进：

| 当前模块 | AgentPort 演进方向 |
| --- | --- |
| `tool_adapters.rs` | `agent_adapters.rs`，支持多 resource roots |
| `sync_engine.rs` | generalized deploy engine，支持 symlink/copy/merge |
| `sync_metadata.rs` | environment metadata，记录 artifacts/provenance/targets |
| `scenario_service.rs` | profile service |
| `skill_store.rs` | 增加 packages、artifacts、targets、origins |
| `git_backup.rs` | environment repo Git sync |
| `skills-manager-cli.rs` | `agentport` CLI |
| `installer.rs` | package/artifact installer |

MVP 可以先保留现有表结构，在 metadata 中扩展 provenance；等模型稳定后再做数据库迁移。

## 18. 实施路线

### Phase 1：Manifest 和 CLI MVP

目标：先打通新机器恢复闭环。

- 新增 `agentport.yaml`
- 新增 `agentport.lock`
- 新增 `env scan`
- 新增 `env diff`
- 新增 `env apply`
- 本地 skill 默认 vendored 并进入 Git
- 保留现有 skills sync 逻辑

### Phase 2：Package 和 provenance

目标：支持插件携带 skills。

- 新增 package model
- 新增 artifact model
- 新增 package artifact source
- 新增 ownership/detach
- 新增 dirty/customized 检测
- 内置少量 package adapter 样板

### Phase 3：本地 skill 回流

目标：支持 agent 修改 skill 后回到中心仓库。

- `skills create`
- `skills link`
- `skills sync-in`
- `skills pull-target`
- `skills discard-target`
- `skills watch`
- hash-based conflict detection

### Phase 4：多资源配置同步

目标：从 skills 扩展到完整 agent 环境。

- commands
- hooks
- MCP config
- rules
- TOML/JSON merge
- config backup and restore
- machine-local overlay

### Phase 5：UI

目标：把 CLI 能力产品化。

- Environment 页面
- Package 页面
- Profile 页面
- Machine status
- Diff viewer
- Drift/conflict resolution

## 19. 最小可行闭环

建议第一个可用版本只做这条路径：

```bash
agentport env init --from-current-machine
agentport git init
agentport git commit -m "initial agent environment"
agentport git push

# 新机器
agentport git clone <repo>
agentport env diff personal-default
agentport env apply personal-default
agentport env doctor
```

这个闭环完成后，AgentPort 已经可以解决核心问题：在新 agent 或新电脑上快速恢复可用环境。

## 20. License 和再发布

当前项目声明为 MIT License。MIT 允许修改、分发、再授权和商业使用。

发布 AgentPort 时需要保留原项目 license notice 和 copyright：

```text
Copyright (c) 2026 Tianliang Zhang
```

如果 AgentPort 内置或再分发第三方 package 内容，需要分别检查这些第三方项目自己的 license。AgentPort 可以管理第三方 package 的安装引用，但不应默认把第三方内容 vendored 进发行包，除非 license 明确允许。

## 21. 关键设计结论

1. AgentPort 的核心不是“同步更多目录”，而是“声明并恢复完整 agent 环境”。
2. Skill 应该从顶层实体降级为 Artifact 的一种。
3. Plugin/package 带来的 skill 必须保留 package provenance。
4. 本地 skill 内容默认进入 Git-backed Environment Repo。
5. 本地绝对路径只放 machine-local overlay，不进入共享仓库。
6. 中心仓库应作为本地原创 skill 的推荐 source of truth。
7. Copy-mode target 的修改不能静默回流，必须显式 pull-target。
8. CLI 是 bootstrap 的核心，UI 是后续产品化入口。
9. Bootstrap Skill 应该引导 agent 使用 CLI，而不是把复杂同步逻辑写进 Skill。
10. 所有推断来源都必须带 confidence，不能假装确定。
