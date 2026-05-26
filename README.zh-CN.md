<p align="center">
  <img src="assets/icon.png" width="80" />
</p>

<h1 align="center">AgentPort</h1>

<p align="center">
  面向 AI coding agent、Skills、插件和工作区的可迁移环境管理器。
</p>

<p align="center">
  <a href="./README.md">English</a>
</p>

<p align="center">
  <img src="assets/demo/library.png" width="800" alt="AgentPort 技能库" />
</p>

<p align="center"><strong>安装 Skills</strong></p>
<p align="center"><img src="assets/demo/install-skills.png" width="800" alt="安装 Skills" /></p>

<p align="center"><strong>全局工作区</strong></p>
<p align="center"><img src="assets/demo/global-workspace.png" width="800" alt="全局工作区" /></p>

<p align="center"><strong>Agent 工作区</strong></p>
<p align="center"><img src="assets/demo/agent-workspace.png" width="800" alt="Agent 工作区" /></p>

<p align="center"><strong>项目工作区</strong></p>
<p align="center"><img src="assets/demo/project-workspace.png" width="800" alt="项目工作区" /></p>

<p align="center"><strong>设置</strong></p>
<p align="center"><img src="assets/demo/settings.png" width="800" alt="设置" /></p>

## AgentPort 管什么

AgentPort 按三层来组织，左侧导航也遵循这个模型。

1. **资源库** — 可复用的 Skills 都放在中央库里。你可以从 Git、本地目录、压缩包或市场安装 Skills，然后打标签、预览、更新，并用 Git 备份。
2. **Environment** — AgentPort 会把你的 agent 环境变成一份可迁移声明：Profile、Package 来源、Artifact、机器本地 overlay、漂移报告。Environment 对当前中央库来说是一个单例对象，不是多个环境的列表。
3. **工作区** — 工作区是 Skills 真正被应用到 agent 的地方。全局工作区会改 agent 的全局 Skills 目录；项目工作区只改某个项目；Preset 是可复用的 Skills 分组，可以应用到二者。

## 侧边栏导航地图

| 侧边栏区域 | 页面 | 含义 |
|------------|------|------|
| **首页** | **首页** | 产品级控制台。汇总资源库、单例 Environment 和工作区状态，并把需要处理的下一步放到前面。 |
| **资源库** | **Skills**、**安装 Skills** | 管理中央 Skills 库。安装、打标签、删除、更新、查看文档，都在这里完成。 |
| **Environment** | **概览**、**Profiles**、**Packages**、**Artifacts**、**Machines**、**Diff** | 查看 AgentPort 准备在多台机器之间同步的单例环境。概览页初始化或重建 `agentport.yaml` / `agentport.lock`；Profiles 描述期望的 agent 配置；Packages 记录插件/来源归属；Artifacts 是被纳管的文件或资源；Machines 保存本机私有 overlay 和 secret 存在状态；Diff 显示本机和 manifest 的差异。 |
| **工作区** | **Preset**、**全局工作区**、**项目工作区** | 把 Skills 应用到真实路径。Preset 是技能组；全局工作区面向每个 agent 的全局 Skills 目录；项目工作区面向某个项目里的 Skills 目录。 |
| **设置** | **设置** | 配置 Agent 路径、同步模式、Git 远程、代理、主题、语言、更新、自定义工具和日志。 |

如果你只想在一台电脑上管理 Skills，主要使用 **资源库** 和 **工作区** 就够了。若你想在多台电脑、多个工具（如 Claude Code、Codex、OpenCode、Gemini CLI）之间迁移同一套 agent 环境，就把 **Environment** 当作同步依据。

## 功能

- **统一技能库** — 从 Git 仓库、本地目录、`.zip` / `.skill` 文件或 [skills.sh](https://skills.sh) 市场安装技能，统一进入中央库，默认路径为 `~/.skills-manager`，也可在 **设置** 中修改。
- **AgentPort 环境同步** — 为 Skills、插件/Package、被纳管 Artifact、Profile 和机器本地 overlay 生成可迁移环境元数据，并支持预览/应用。
- **市场与 AI 搜索** — 浏览市场热门 Skills、关键词搜索，或配置 SkillsMP API Key 后使用 AI 搜索。
- **Preset（预设）** — 将技能分组为命名 Preset。在任意工作区点击 Preset 标签，即可为当前 Agent 范围激活或停用其全部技能。
- **全局工作区** — 每个 Agent 都有自己的页面，列出其全局目录里的所有 Skills，包括不是通过 AgentPort 安装的内容，始终反映 Agent 实际看到的状态。
- **项目工作区** — 查看并管理任意项目的本地 Skills 目录，支持与中央库双向同步，适合项目专属能力。
- **多工具同步** — 支持用软链接或复制方式把 Skills 同步到已支持工具。Skill 卡片上的 Agent 图标可一键安装或移除该 Skill。
- **Package 与来源追踪** — 记录 Skills 和 Artifacts 的来源，包括插件拥有的 package、市场安装内容，以及 vendored 到中央库的本地 Skill。
- **机器本地安全边界** — 私有路径、secret 和本机专属 overlay 不进入共享 manifest，但会记录是否存在，方便迁移后补齐。
- **漂移与 copy target 修复** — 对比本机状态和共享 manifest，预览 apply/export 操作，并对被 agent 修改过的 copy target 执行回流或丢弃。
- **技能标签** — 为 Skills 添加标签，用于归类和筛选；**未标签** 过滤项可快速定位漏打标签的 Skills。
- **Git 备份与恢复** — 用 Git 管理技能库，支持版本控制、多机同步和快照恢复。
- **自定义工具** — 添加自定义 Agent/工具并指定 Skills 目录，也可覆盖内置工具默认路径。
- **活动日志 & 导出日志** — 应用会记录本地安装/移除/更新/同步操作。在 **设置 → 导出日志** 可打包最近日志和活动记录，方便提交 Issue。

## 核心概念

- **Skill 是可复用的 agent 能力** — 一个 Skill 通常是包含 `SKILL.md` 和辅助文件的目录，可以来自 Git、本地目录、压缩包或市场。
- **Preset 是可复用的 Skills 分组** — Preset 是一组命名的 Skills 集合。在任意工作区激活 Preset，即可将其所有 Skills 添加到选定 Agent；停用则反向移除。应用 Preset 是一次性操作，不是实时订阅。
- **Profile 描述期望的 agent 状态** — Profile 表达“哪些 agent 应该拥有哪些 Skills 和资源”，由 Preset 和 AgentPort 元数据生成。
- **Package 描述来源归属** — Package 记录一组 Skills/Artifacts 来自哪里，例如插件仓库、市场 package，或导入的本地来源。
- **Artifact 是被纳管的文件或资源** — Skill 是一种 Artifact。AgentPort 也可以建模 commands、hooks、配置片段等 agent 相关资源。
- **Machine 保存本机私有数据** — Machine overlay 保存私有路径、本地来源、secret 存在状态，不把敏感信息放进共享 manifest。
- **全局工作区管理每个 Agent 的全局 Skills** — 每个已安装 Agent 都有自己的全局 Skills 目录（如 Claude Code 对应 `~/.claude/skills/`）。每个 Agent 页面会列出该目录里的所有内容，包括不是通过 AgentPort 安装的 Skills。
- **项目工作区是项目专属 Skills 集合** — 项目工作区管理某个项目里的本地 Skills（如 `<project>/.claude/skills/`），只对该项目生效。
- **Diff 告诉你哪里变了** — 在迁移电脑、应用 Profile、或 agent 直接修改 copy target 后，先看 Diff 再决定 apply/export。

## 快速上手

1. 打开 **安装 Skills**，从本地目录、Git 仓库、压缩包或市场导入 Skills。
2. 打开 **Skills** 查看中央库，给 Skills 打标签、查看文档，并决定哪些 Skills 应该放在一起。
3. 创建或选择一个 **Preset**，比如 default coding、review、frontend。
4. 打开 **全局工作区**，选择 Claude Code、Codex、OpenCode、Gemini CLI 等 Agent，应用 Preset 或用 **+ 添加 Skills** 单独添加。
5. 如果某些能力只应该在一个项目里生效，关联 **项目工作区**，把 Skills 应用到该项目而不是全局。
6. 如果要多机同步，打开 **Environment → 概览** 初始化或重建 AgentPort manifest，再检查 **Profiles**、**Packages**、**Artifacts**、**Machines** 和 **Diff**。
7. 在 **设置** 中配置 Git，然后回到 **Skills** 执行 **同步到 Git**，把中央库备份并迁移到其他电脑。

## Git 备份

将 `~/.skills-manager/skills/` 备份到 Git 仓库，用于版本管理和多机同步。

### 快速配置

1. 创建一个私有仓库（推荐）。
2. 打开 **设置 → Git 同步配置**，保存远程仓库地址。
3. 打开 **Skills** 页面。
4. 二选一：
- 已有远程仓库：点击 **开始备份**，按已配置地址克隆。
- 首次本地初始化：点击 **开始备份** 初始化本地仓库，再使用 **同步到 Git**。
5. 在 Skills 顶部工具栏点击 **同步到 Git**。

`同步到 Git` 会根据仓库状态自动处理拉取/提交/推送。
每次同步成功会自动创建一个快照版本标签。你可以在 **Skills** 中打开 **版本历史**，并将任意快照恢复为一条新的提交。

### 认证说明

- SSH 地址（`git@github.com:...`）：需要先在本机配置 SSH Key，并将公钥添加到 GitHub。
- HTTPS 地址（`https://github.com/...`）：推送通常需要 Personal Access Token（PAT）。

> **注意：** SQLite 数据库（`~/.skills-manager/skills-manager.db`）不纳入 Git 管理，它存储的元数据可通过扫描技能文件重建。

## 支持的工具

Cursor · Claude Code · Codex · OpenCode · Amp · Kilo Code · Roo Code · Goose · Gemini CLI · GitHub Copilot · Windsurf · TRAE IDE · Antigravity · Clawdbot · Droid

你也可以在**设置**中添加自定义工具，以相同方式管理其 Skills。

## 应用内帮助

设置页中的 **帮助** 按钮会展示与上面一致的产品流程：侧边栏模型、推荐工作流、Preset、安装 Skills、技能库、单例 Environment、全局工作区与 **+ 添加 Skills** 弹层、项目工作区、Git 备份，以及环境设置。它可以理解为 README 的应用内版本。

## 技术栈

| 层 | 技术 |
|----|------|
| 前端 | React 19、TypeScript、Vite、Tailwind CSS |
| 桌面 | Tauri 2 |
| 后端 | Rust |
| 存储 | SQLite（`rusqlite`） |
| 国际化 | react-i18next |

## 快速开始

### 前置依赖

- Node.js 18+
- Rust 工具链
- 当前系统的 [Tauri 依赖](https://v2.tauri.app/start/prerequisites/)

### 开发

```bash
npm install
npm run tauri:dev
```

### CLI

仓库包含一个面向 agent 的 CLI，它与桌面应用复用同一套 Rust shared core。桌面应用和 CLI 会经过同一个 SQLite 数据库、中央技能库和同步引擎。

```bash
# 查看仓库 / 技能库状态
npm run cli -- repo status
npm run cli -- skills list
npm run cli -- skills show db

# 安装 Skills（默认只进入中央库，不会同步到 Agent）
npm run cli -- skills install ./my-skill                       # 本地路径
npm run cli -- skills install https://github.com/foo/bar.git   # Git URL
npm run cli -- skills install vercel-labs/agent-skills@react-best-practices  # skills.sh
npm run cli -- skills install foo/bar --sync                   # 加入当前 Preset 并同步到 Agent

# 从上游更新 / 检查更新
npm run cli -- skills update --all
npm run cli -- skills check --all

# 搜索 skills.sh 市场（无需 API Key）
npm run cli -- skills search react --limit 5

# 删除（需要 --yes；支持 --dry-run）
npm run cli -- skills remove <ref> --dry-run
npm run cli -- skills remove <ref> --yes

# 通过修改 Preset 成员来启用 / 停用 Skills
npm run cli -- presets add-skill <preset> <ref>
npm run cli -- presets remove-skill <preset> <ref>

# 将当前 Preset 同步到启用的 Agents
npm run cli -- skills sync --dry-run
npm run cli -- skills sync --tool claude_code

# 纳管已经存在于 Agent 目录中的 Skills（如 ~/.claude/skills/）
npm run cli -- skills adopt ~/.claude/skills --dry-run
npm run cli -- skills adopt ~/.claude/skills

# 标签
npm run cli -- skills tag add <ref> web frontend
npm run cli -- skills tag list

# Presets
npm run cli -- presets list
npm run cli -- presets preview Default
npm run cli -- presets apply Default
npm run cli -- presets add-skill <preset> <skill>
npm run cli -- presets remove-skill <preset> <skill>

# 一次性导出单个 Skill 到任意目录（非托管）
npm run cli -- skills export db --dest ~/.claude/skills/db

# Git 管理的 skills 仓库
npm run cli -- git status
npm run cli -- git pull
npm run cli -- git commit -m "chore: update skills"
```

可用命令分组：
- `repo`：查看或修改当前 base directory
- `tools`：列出已检测到的工具目标与路径
- `skills`：管理中央技能库中的 Skills（`list / show / install / update / check / remove / enable / disable / sync / search / adopt / tag / export`）
- `presets`：列出 Preset，预览 / 应用 Preset，向 Preset 添加或移除 Skills
- `git`：操作 git 管理的 `skills/` 仓库（`clone`、`pull`、`push`、`commit`、`versions`、`restore`）

额外参数：
- `--skills-root <path>`：直接针对某个已 clone / 已导出的 skills repo 操作，而不是本机 app 默认目录。manager 的状态（DB、Presets、cache、logs）会落在 `~/.skills-manager/external/<name>-<hash>/`，按 skills root 的规范化路径分目录隔离，外部仓库本身保持干净。
- `--json`：给脚本 / agent 使用的机器可读输出

```bash
npm run -s cli -- --skills-root /path/to/my-skills --json skills list
```

#### 把 CLI 二进制安装到 PATH

如果 agent / 脚本直接调用 `skills-manager-cli`（而不是 `npm run`），需要先把二进制放到 PATH 上：

```bash
npm run cli:install
# 等价于：
# cargo install --path src-tauri --bin skills-manager-cli --locked --force
```

二进制会装到 `~/.cargo/bin/skills-manager-cli`。代码更新后再跑一次即可刷新。

#### 与桌面应用并发使用

CLI 和桌面应用共享同一个 SQLite 数据库。SQLite 会串行化写入，所以数据是安全的，但运行中的应用不会自动刷新它的内存缓存 —— 在 CLI 跑完 `presets apply`、`git pull` 等会改状态的命令后，重启应用或手动刷新一次。

### 构建

```bash
npm run tauri:build
npm run cli:build
```

## 常见问题

### macOS 首次启动被 Gatekeeper 拦截

AgentPort 使用 ad-hoc 签名，未做 Apple 公证（没有付费的 Apple Developer ID），所以首次打开会被 macOS Gatekeeper 提示。

- **"无法打开，因为无法验证开发者"**（v1.20.0 及之后版本）—— 在访达里右键点击应用，选择 **打开**，再在弹窗里确认即可。也可以打开 **系统设置 → 隐私与安全性**，第一次启动失败后会出现 **仍要打开** 按钮。
- **"应用已损坏，无法打开"**（v1.19.0 及之前版本）—— 在终端执行下面这条命令后重新打开应用即可：

  ```bash
  xattr -cr /Applications/skills-manager.app
  ```

  如果 `.app` 不在 `/Applications`，请替换为实际路径。

## License

MIT
