# AgentPort Sidebar Information Architecture Design

## Context

AgentPort added several new pages to the existing Skills Manager sidebar:

- Environment: `/agentport`
- Profiles: `/agentport/profiles`
- Packages: `/agentport/packages`
- Artifacts: `/agentport/artifacts`
- Machines: `/agentport/machines`
- Diff: `/agentport/diff`

Those entries are currently placed directly beside Dashboard, Skills, and Install Skills. This makes the first sidebar group feel like a flat page list instead of a clear product model. The navigation should make AgentPort feel like the main product shell, while preserving the existing skills library and workspace workflows.

## Decision

Use AgentPort as the primary sidebar brand and organize the navigation by product object model:

```text
AgentPort

Overview
  Dashboard
  Environment

Library
  Skills
  Install Skills

Environment Model
  Profiles
  Packages
  Artifacts
  Machines
  Diff

Workspaces
  Presets
    <preset list>
  Global Workspace
    All Agents
    <agent list>
  Project Workspace
    Add Project
    <project list>

Settings
```

## Rationale

The sidebar should answer four different questions in order:

1. What is the current health and desired state of my portable agent environment?
2. What central assets does AgentPort manage?
3. What declarative model explains how the environment is reconstructed?
4. Where are those assets applied locally?

This keeps Skills, Packages, Artifacts, Profiles, Machines, and Workspaces from competing as peers. Skills become one managed asset type inside AgentPort rather than the whole application identity.

## Section Semantics

### Overview

Overview contains entry points for status and environment-level actions.

- Dashboard shows high-level status and quick actions.
- Environment is the AgentPort control surface for manifest status, init, export, apply, and doctor flows.

### Library

Library contains center-repo asset management.

- Skills maps to the current `/my-skills` library page.
- Install Skills maps to the current `/install` acquisition flow.

### Environment Model

Environment Model contains the AgentPort declarative model.

- Profiles describe expected environment states.
- Packages describe package/plugin provenance.
- Artifacts describe managed assets and targets.
- Machines describe machine-local overlays and missing secret status.
- Diff describes drift between expected and actual state.

This section should be visible by default because it introduces AgentPort's new mental model. It may be collapsible and should remember local open state if implemented.

### Workspaces

Workspaces contains deployment/application views.

- Presets remain a reorderable, collapsible list.
- Global Workspace keeps the existing All Agents and per-agent entries.
- Project Workspace keeps existing linked project entries and Add Project.

Presets should move under Workspaces because they are applied to agents and projects; they are not environment-model definitions in the AgentPort manifest sense.

### Settings

Settings remains pinned at the bottom of the sidebar.

## Naming

The sidebar product name should be `AgentPort`.

For transitional product messaging, use `AgentPort, formerly Skills Manager` in About/help/release copy rather than placing both names in the sidebar. The sidebar should not make users choose between two product identities.

Suggested label mapping:

| Current label | New label |
| --- | --- |
| Skills Manager | AgentPort |
| 技能库 / My Skills | Skills |
| AgentPort | Environment |
| Packages | Packages |
| Profiles | Profiles |
| Artifacts | Artifacts |
| Machines | Machines |
| Diff | Diff |

## Implementation Boundaries

This design only changes sidebar information architecture and labels.

Do not change route paths in the first implementation pass:

- Keep `/my-skills`
- Keep `/install`
- Keep `/agentport`
- Keep `/agentport/*`
- Keep workspace and project routes

Do not change page behavior, data models, command palette behavior, or AgentPort backend commands in this pass.

## Interaction Notes

- Section headings should be visually lighter than navigable items.
- Reuse the current compact row height and 5px item radius.
- Preserve current drag-and-drop behavior for presets, agents, and projects.
- Preserve current count pills and sync health indicators.
- Keep Settings pinned below the scrollable area.
- Use local storage for new collapsible section state if a section can be collapsed.
- Active matching for AgentPort nested routes should still highlight the exact nested page, not only Environment.

## Acceptance Criteria

- The first sidebar area no longer contains a flat list of Dashboard, Skills, Install, and all AgentPort subpages.
- Product identity in the sidebar reads AgentPort.
- Dashboard and Environment are grouped under Overview.
- Skills and Install Skills are grouped under Library.
- Profiles, Packages, Artifacts, Machines, and Diff are grouped under Environment Model.
- Presets, Global Workspace, and Project Workspace are grouped under Workspaces.
- Existing routes continue to work without redirects.
- Existing preset, agent, and project drag ordering still works.
- Sidebar text remains readable without wrapping at the existing 220px width.
- Chinese, Traditional Chinese, and English i18n entries are updated consistently.

## Verification Plan

- Run `npm run lint`.
- Run `npm run build`.
- Use browser verification on desktop width to confirm:
  - AgentPort brand is visible.
  - All four main sections are visible.
  - Existing nested lists still render.
  - Active state works for `/`, `/my-skills`, `/install`, `/agentport`, and each `/agentport/*` route.
- Check a narrow app window around the current minimum width to ensure labels do not overlap or wrap awkwardly.
