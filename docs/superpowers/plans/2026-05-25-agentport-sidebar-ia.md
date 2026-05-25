# AgentPort Sidebar IA Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reorganize the left sidebar into the approved AgentPort product shell: Overview, Library, Environment Model, and Workspaces.

**Architecture:** Keep all route paths and page components unchanged. Refactor only the sidebar information architecture and related i18n labels, reusing the existing compact sidebar row styling, drag/drop behavior, and pinned Settings footer.

**Tech Stack:** React 19, TypeScript, React Router, react-i18next, lucide-react, Tailwind CSS, Vite, Playwright smoke verification through the bundled Node runtime.

---

## File Structure

- Modify `src/components/Sidebar.tsx`
  - Replace the flat `NAV_ITEMS` rendering with grouped static nav sections.
  - Keep existing draggable Presets, Global Workspace, Lobster Workspace, and Project Workspace behavior.
  - Move Presets and workspace groups under one visual `Workspaces` section.
  - Add optional localStorage-backed open state for `Environment Model`.
- Modify `src/i18n/en.json`
  - Change sidebar app name to AgentPort.
  - Add sidebar section labels.
  - Rename the AgentPort root nav label to Environment.
- Modify `src/i18n/zh.json`
  - Same label changes in Simplified Chinese.
- Modify `src/i18n/zh-TW.json`
  - Same label changes in Traditional Chinese.

No route files, backend commands, data models, or command palette navigation behavior should change in this implementation pass.

---

## Task 1: Add Sidebar IA Labels

**Files:**
- Modify: `src/i18n/en.json`
- Modify: `src/i18n/zh.json`
- Modify: `src/i18n/zh-TW.json`

- [ ] **Step 1: Update English sidebar labels**

In `src/i18n/en.json`, update the top app name and sidebar object to this shape while preserving unrelated keys:

```json
"app": {
  "name": "AgentPort"
},
"sidebar": {
  "overview": "Overview",
  "dashboard": "Dashboard",
  "environment": "Environment",
  "library": "Library",
  "mySkills": "Skills",
  "globalWorkspace": "Global Workspace",
  "lobsterAgents": "Lobster Agents",
  "installSkills": "Install Skills",
  "environmentModel": "Environment Model",
  "agentport": "Environment",
  "agentportPackages": "Packages",
  "agentportProfiles": "Profiles",
  "agentportArtifacts": "Artifacts",
  "agentportMachines": "Machines",
  "agentportDiff": "Diff",
  "workspaces": "Workspaces",
  "presets": "Presets",
  "newPreset": "New Preset",
  "projects": "Project Workspaces",
  "addProject": "Link Project",
  "settings": "Settings"
}
```

- [ ] **Step 2: Update Simplified Chinese sidebar labels**

In `src/i18n/zh.json`, update the same keys:

```json
"app": {
  "name": "AgentPort"
},
"sidebar": {
  "overview": "总览",
  "dashboard": "Dashboard",
  "environment": "Environment",
  "library": "资源库",
  "mySkills": "Skills",
  "globalWorkspace": "全局工作区",
  "lobsterAgents": "龙虾 Agent",
  "installSkills": "安装 Skills",
  "environmentModel": "环境模型",
  "agentport": "Environment",
  "agentportPackages": "Packages",
  "agentportProfiles": "Profiles",
  "agentportArtifacts": "Artifacts",
  "agentportMachines": "Machines",
  "agentportDiff": "Diff",
  "workspaces": "工作区",
  "presets": "Preset",
  "newPreset": "新建 Preset",
  "projects": "项目工作区",
  "addProject": "关联项目",
  "settings": "设置"
}
```

- [ ] **Step 3: Update Traditional Chinese sidebar labels**

In `src/i18n/zh-TW.json`, update the same keys:

```json
"app": {
  "name": "AgentPort"
},
"sidebar": {
  "overview": "總覽",
  "dashboard": "Dashboard",
  "environment": "Environment",
  "library": "資源庫",
  "mySkills": "Skills",
  "globalWorkspace": "全域工作區",
  "lobsterAgents": "龍蝦 Agent",
  "installSkills": "安裝 Skills",
  "environmentModel": "環境模型",
  "agentport": "Environment",
  "agentportPackages": "Packages",
  "agentportProfiles": "Profiles",
  "agentportArtifacts": "Artifacts",
  "agentportMachines": "Machines",
  "agentportDiff": "Diff",
  "workspaces": "工作區",
  "presets": "Preset",
  "newPreset": "新增 Preset",
  "projects": "專案工作區",
  "addProject": "關聯專案",
  "settings": "設定"
}
```

- [ ] **Step 4: Validate JSON syntax**

Run:

```bash
node -e "for (const f of ['src/i18n/en.json','src/i18n/zh.json','src/i18n/zh-TW.json']) { JSON.parse(require('fs').readFileSync(f, 'utf8')); console.log(f + ' ok'); }"
```

Expected output includes:

```text
src/i18n/en.json ok
src/i18n/zh.json ok
src/i18n/zh-TW.json ok
```

- [ ] **Step 5: Commit**

```bash
git add src/i18n/en.json src/i18n/zh.json src/i18n/zh-TW.json
git commit -m "feat: update AgentPort sidebar labels"
```

---

## Task 2: Refactor Static Sidebar Sections

**Files:**
- Modify: `src/components/Sidebar.tsx`

- [ ] **Step 1: Add the icon type import**

In `src/components/Sidebar.tsx`, update the React import:

```tsx
import { useState, useEffect, useRef, useMemo, type ComponentType } from "react";
```

- [ ] **Step 2: Replace the flat `NAV_ITEMS` array**

In `src/components/Sidebar.tsx`, replace the existing `NAV_ITEMS` constant with typed grouped items:

```tsx
  type StaticNavItem = {
    name: string;
    path: string;
    icon: ComponentType<{ className?: string }>;
  };

  const overviewItems: StaticNavItem[] = [
    { name: t("sidebar.dashboard"), path: "/", icon: LayoutDashboard },
    { name: t("sidebar.agentport"), path: "/agentport", icon: FileCode2 },
  ];

  const libraryItems: StaticNavItem[] = [
    { name: t("sidebar.mySkills"), path: "/my-skills", icon: Layers },
    { name: t("sidebar.installSkills"), path: "/install", icon: Download },
  ];

  const environmentModelItems: StaticNavItem[] = [
    { name: t("sidebar.agentportProfiles"), path: "/agentport/profiles", icon: SlidersHorizontal },
    { name: t("sidebar.agentportPackages"), path: "/agentport/packages", icon: Boxes },
    { name: t("sidebar.agentportArtifacts"), path: "/agentport/artifacts", icon: FileCode2 },
    { name: t("sidebar.agentportMachines"), path: "/agentport/machines", icon: HardDrive },
    { name: t("sidebar.agentportDiff"), path: "/agentport/diff", icon: GitCompareArrows },
  ];
```

- [ ] **Step 3: Add a reusable section heading helper**

Add this helper inside `Sidebar` before `renderToolGroup`:

```tsx
  const renderSectionHeading = (label: string, options?: {
    isOpen?: boolean;
    onToggle?: () => void;
  }) => {
    const collapsible = Boolean(options?.onToggle);
    return (
      <div className="mb-1.5 px-2.5 flex items-center gap-1">
        {collapsible ? (
          <button
            onClick={options?.onToggle}
            className="flex min-w-0 flex-1 items-center gap-1 text-left outline-none"
          >
            {options?.isOpen
              ? <ChevronDown className="h-3 w-3 shrink-0 text-faint" />
              : <ChevronRight className="h-3 w-3 shrink-0 text-faint" />}
            <span className="truncate text-[12px] font-semibold tracking-[0.01em] text-muted whitespace-nowrap">
              {label}
            </span>
          </button>
        ) : (
          <span className="truncate pl-4 text-[12px] font-semibold tracking-[0.01em] text-muted whitespace-nowrap">
            {label}
          </span>
        )}
      </div>
    );
  };
```

- [ ] **Step 4: Add a reusable static nav renderer**

Add this helper below `renderSectionHeading`:

```tsx
  const renderStaticNavItems = (items: StaticNavItem[]) => (
    <div className="space-y-0.5">
      {items.map((item) => {
        const Icon = item.icon;
        const isActive = location.pathname === item.path;
        return (
          <Link
            key={item.path}
            to={item.path}
            className={cn(
              "flex items-center gap-2.5 px-2.5 py-[7px] rounded-[5px] text-sm font-medium transition-colors outline-none",
              isActive
                ? "bg-surface-active text-primary"
                : "text-tertiary hover:text-secondary hover:bg-surface-hover"
            )}
          >
            <Icon className={cn("w-4 h-4 shrink-0", isActive ? "text-accent" : "text-muted")} />
            <span className="truncate">{item.name}</span>
          </Link>
        );
      })}
    </div>
  );
```

- [ ] **Step 5: Add Environment Model open state**

Add state beside the existing sidebar open-state declarations:

```tsx
  const [environmentModelOpen, setEnvironmentModelOpen] = useState(() => {
    const stored = localStorage.getItem("agentport:sidebar:environment-model-open");
    return stored === null ? true : stored === "true";
  });
```

Add this effect near the other `useEffect` hooks:

```tsx
  useEffect(() => {
    localStorage.setItem(
      "agentport:sidebar:environment-model-open",
      String(environmentModelOpen)
    );
  }, [environmentModelOpen]);
```

- [ ] **Step 6: Replace the top flat nav markup**

Replace the current top nav block:

```tsx
        {/* Nav */}
        <div className="px-2.5 space-y-0.5 shrink-0">
          {NAV_ITEMS.map((item) => {
            ...
          })}
        </div>
```

with:

```tsx
        {/* Primary navigation */}
        <div className="px-2.5 shrink-0">
          {renderSectionHeading(t("sidebar.overview"))}
          {renderStaticNavItems(overviewItems)}

          <div className="mx-0.5 mt-3.5 mb-2.5 border-t border-border-subtle" />

          {renderSectionHeading(t("sidebar.library"))}
          {renderStaticNavItems(libraryItems)}

          <div className="mx-0.5 mt-3.5 mb-2.5 border-t border-border-subtle" />

          {renderSectionHeading(t("sidebar.environmentModel"), {
            isOpen: environmentModelOpen,
            onToggle: () => setEnvironmentModelOpen((v) => !v),
          })}
          {environmentModelOpen && renderStaticNavItems(environmentModelItems)}
        </div>
```

- [ ] **Step 7: Run TypeScript build for fast feedback**

Run:

```bash
npm run build
```

Expected: TypeScript and Vite build complete successfully. The existing chunk-size warning is acceptable.

- [ ] **Step 8: Commit**

```bash
git add src/components/Sidebar.tsx
git commit -m "feat: group sidebar around AgentPort"
```

---

## Task 3: Move Dynamic Lists Under Workspaces

**Files:**
- Modify: `src/components/Sidebar.tsx`

- [ ] **Step 1: Replace the divider before the scrollable section**

The divider between primary nav and the scrollable area can remain, but the scrollable area should start with the Workspaces heading. In `src/components/Sidebar.tsx`, inside:

```tsx
        <div className="px-2.5 flex-1 overflow-y-auto scrollbar-hide min-h-0">
```

insert this before the Presets heading:

```tsx
          {renderSectionHeading(t("sidebar.workspaces"))}
```

- [ ] **Step 2: Replace the Presets heading markup**

Replace the duplicated Presets heading JSX:

```tsx
          <div className="mb-1.5 px-2.5 flex items-center gap-1">
            <button
              onClick={() => setPresetsOpen((v) => !v)}
              className="flex min-w-0 flex-1 items-center gap-1 text-left outline-none"
            >
              {presetsOpen
                ? <ChevronDown className="h-3 w-3 shrink-0 text-faint" />
                : <ChevronRight className="h-3 w-3 shrink-0 text-faint" />}
              <span className="truncate text-[12px] font-semibold tracking-[0.01em] text-muted whitespace-nowrap">
                {t("sidebar.presets")}
              </span>
            </button>
          </div>
```

with:

```tsx
          {renderSectionHeading(t("sidebar.presets"), {
            isOpen: presetsOpen,
            onToggle: () => setPresetsOpen((v) => !v),
          })}
```

- [ ] **Step 3: Keep Global Workspace and Project Workspace behavior unchanged**

Do not modify the `renderToolGroup` calls except for their new visual position under Workspaces. The coding workspace call should still use:

```tsx
          {renderToolGroup({
            category: "coding",
            headingLabel: t("sidebar.globalWorkspace"),
            allAgentsLabel: t("globalWorkspace.allAgents"),
            emptyLabel: t("globalWorkspace.noAgents"),
            basePath: "/global-workspace",
            droppableId: "global-workspace-tools",
            tools: orderedCodingTools,
            isOpen: globalWorkspaceOpen,
            onToggle: () => setGlobalWorkspaceOpen((v) => !v),
            hideWhenEmpty: false,
          })}
```

- [ ] **Step 4: Replace the Projects heading markup**

Replace the duplicated Projects heading JSX with:

```tsx
          {renderSectionHeading(t("sidebar.projects"), {
            isOpen: projectsOpen,
            onToggle: () => setProjectsOpen((v) => !v),
          })}
```

- [ ] **Step 5: Run lint and build**

Run:

```bash
npm run lint
npm run build
```

Expected: both commands exit 0. Existing Vite chunk-size warning is acceptable.

- [ ] **Step 6: Commit**

```bash
git add src/components/Sidebar.tsx
git commit -m "feat: nest workspaces in sidebar"
```

---

## Task 4: Browser Smoke Verification

**Files:**
- No source files expected unless verification finds a real bug.

- [ ] **Step 1: Start the local dev server**

Run:

```bash
npm run dev -- --host 127.0.0.1 --port 1420
```

Expected: Vite serves the app at `http://127.0.0.1:1420/`.

- [ ] **Step 2: Verify static sidebar labels in a mocked browser session**

Run a Playwright smoke check against `http://127.0.0.1:1420/` with Tauri invoke mocks. The script should assert:

```js
const bodyText = await page.locator('body').innerText();
const checks = {
  hasBrand: bodyText.includes('AgentPort'),
  hasOverview: bodyText.includes('Overview') || bodyText.includes('总览') || bodyText.includes('總覽'),
  hasLibrary: bodyText.includes('Library') || bodyText.includes('资源库') || bodyText.includes('資源庫'),
  hasEnvironmentModel: bodyText.includes('Environment Model') || bodyText.includes('环境模型') || bodyText.includes('環境模型'),
  hasWorkspaces: bodyText.includes('Workspaces') || bodyText.includes('工作区') || bodyText.includes('工作區'),
  hasEnvironmentItem: bodyText.includes('Environment'),
  hasProfiles: bodyText.includes('Profiles'),
  hasPackages: bodyText.includes('Packages'),
  hasArtifacts: bodyText.includes('Artifacts'),
  hasMachines: bodyText.includes('Machines'),
  hasDiff: bodyText.includes('Diff'),
};
if (Object.values(checks).some((value) => !value)) {
  throw new Error(JSON.stringify(checks, null, 2));
}
```

Expected: all checks are true and no page errors are recorded.

- [ ] **Step 3: Verify route active behavior**

In the same Playwright session, navigate to:

```text
/
/my-skills
/install
/agentport
/agentport/profiles
/agentport/packages
/agentport/artifacts
/agentport/machines
/agentport/diff
```

For each route, assert that the page loads without console errors and the expected page heading or route-specific body text is present.

- [ ] **Step 4: Stop the local dev server**

Find the listener:

```bash
lsof -nP -iTCP:1420 -sTCP:LISTEN
```

Stop the Vite process with:

```bash
kill <pid>
```

- [ ] **Step 5: Commit only if source files changed**

If verification required fixes, commit them:

```bash
git add src/components/Sidebar.tsx src/i18n/en.json src/i18n/zh.json src/i18n/zh-TW.json
git commit -m "fix: polish AgentPort sidebar navigation"
```

If no files changed, do not create an empty commit.

---

## Task 5: Final Verification and Push

**Files:**
- No source files expected.

- [ ] **Step 1: Run final checks**

Run:

```bash
git status --short --branch
npm run lint
npm run build
```

Expected:

- `npm run lint` exits 0.
- `npm run build` exits 0.
- `git status` has no unstaged or staged source changes.

- [ ] **Step 2: Push the branch**

Run:

```bash
git push
```

Expected: `feature/agentport-mvp` pushes successfully to `origin`.

---

## Self-Review

- Spec coverage: The plan covers AgentPort branding, Overview, Library, Environment Model, Workspaces, pinned Settings, unchanged routes, i18n, build/lint, and browser verification.
- Placeholder scan: No TODO/TBD placeholders remain.
- Type consistency: `StaticNavItem`, `renderSectionHeading`, and `renderStaticNavItems` are introduced before use. Existing `renderToolGroup` and drag/drop state remain in `Sidebar.tsx`.
- Scope check: This is a single UI IA refactor. It does not include full product rename, updater endpoint changes, release packaging, command palette restructuring, or backend changes.
