import { useCallback, useEffect, useMemo, useState } from "react";
import { useNavigate } from "react-router-dom";
import {
  AlertTriangle,
  Bot,
  CheckCircle2,
  Download,
  FileCode2,
  GitCompareArrows,
  Layers,
  Loader2,
  Plus,
  RefreshCw,
  type LucideIcon,
} from "lucide-react";
import { useTranslation } from "react-i18next";
import { useApp } from "../context/AppContext";
import * as api from "../lib/tauri";
import { cn } from "../utils";

type Tone = "ok" | "warning" | "neutral";

function diffIssueCount(diff: api.AgentPortEnvDiffReport | null | undefined) {
  if (!diff) return 0;
  return (
    diff.missing_skills.length +
    diff.unmanaged_skills.length +
    diff.missing_artifacts.length +
    diff.unmanaged_artifacts.length +
    diff.changed_artifacts.length +
    (diff.missing_profile ? 1 : 0)
  );
}

function toneClasses(tone: Tone) {
  if (tone === "ok") return "border-accent-border bg-accent-bg text-accent-light";
  if (tone === "warning") return "border-amber-500/30 bg-amber-500/[0.08] text-amber-400";
  return "border-border-subtle bg-surface-hover text-muted";
}

function SummaryCard({
  title,
  value,
  detail,
  tone,
  icon: Icon,
  onClick,
}: {
  title: string;
  value: string;
  detail: string;
  tone: Tone;
  icon: LucideIcon;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className="app-panel group flex min-h-[112px] items-start justify-between gap-3 px-4 py-4 text-left transition-colors hover:border-border"
    >
      <span className="min-w-0">
        <span className="app-section-title mb-2 block">{title}</span>
        <span className="block truncate text-xl font-semibold leading-none text-primary">{value}</span>
        <span className="mt-2 block text-[13px] leading-5 text-muted">{detail}</span>
      </span>
      <span className={cn("rounded-md border p-2 transition-colors", toneClasses(tone))}>
        <Icon className="h-4 w-4" />
      </span>
    </button>
  );
}

function AttentionRow({
  title,
  detail,
  tone,
  onClick,
}: {
  title: string;
  detail: string;
  tone: Tone;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className="flex w-full items-center justify-between gap-3 px-3.5 py-3 text-left transition-colors hover:bg-surface-hover"
    >
      <span className="min-w-0">
        <span className="block truncate text-[13px] font-medium text-secondary">{title}</span>
        <span className="mt-0.5 block truncate text-[13px] text-muted">{detail}</span>
      </span>
      <span className={cn("shrink-0 rounded-md border p-1.5", toneClasses(tone))}>
        {tone === "ok" ? <CheckCircle2 className="h-3.5 w-3.5" /> : <AlertTriangle className="h-3.5 w-3.5" />}
      </span>
    </button>
  );
}

export function Dashboard() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const { tools, projects, managedSkills, openSkillDetailById } = useApp();
  const [envStatus, setEnvStatus] = useState<api.AgentPortEnvStatus | null>(null);
  const [envLoading, setEnvLoading] = useState(true);

  const refreshEnvironment = useCallback(async () => {
    setEnvLoading(true);
    try {
      setEnvStatus(await api.agentportEnvStatus());
    } catch {
      setEnvStatus(null);
    } finally {
      setEnvLoading(false);
    }
  }, []);

  useEffect(() => {
    void refreshEnvironment();
  }, [refreshEnvironment]);

  const enabledAgents = useMemo(
    () => tools.filter((tool) => tool.installed && tool.enabled),
    [tools],
  );

  const totalSkills = managedSkills.length;
  const syncedSkills = useMemo(
    () => managedSkills.filter((skill) => skill.targets.length > 0).length,
    [managedSkills],
  );
  const updateCount = useMemo(
    () => managedSkills.filter((skill) => skill.update_status === "update_available").length,
    [managedSkills],
  );
  const untaggedCount = useMemo(
    () => managedSkills.filter((skill) => skill.tags.length === 0).length,
    [managedSkills],
  );
  const recentSkills = useMemo(
    () => [...managedSkills].sort((a, b) => b.updated_at - a.updated_at).slice(0, 5),
    [managedSkills],
  );
  const projectIssueCount = useMemo(
    () => projects.reduce((acc, project) => (
      acc +
      project.sync_health.project_newer +
      project.sync_health.center_newer +
      project.sync_health.diverged
    ), 0),
    [projects],
  );
  const firstProjectWithIssues = projects.find((project) => (
    project.sync_health.project_newer +
    project.sync_health.center_newer +
    project.sync_health.diverged
  ) > 0);

  const doctor = envStatus?.doctor;
  const diffIssues = diffIssueCount(envStatus?.diff);
  const envWarningCount = doctor?.warnings.length ?? 0;
  const environmentTone: Tone = envLoading
    ? "neutral"
    : !doctor?.manifest_exists || diffIssues > 0 || envWarningCount > 0 || envStatus?.diff_error
      ? "warning"
      : "ok";
  const environmentLabel = envLoading
    ? t("dashboard.environment.loading")
    : !doctor?.manifest_exists
      ? t("dashboard.environment.needsInit")
      : environmentTone === "warning"
        ? t("dashboard.environment.needsReview")
        : t("dashboard.environment.ready");

  const attentionItems = useMemo(() => {
    const items: { title: string; detail: string; tone: Tone; onClick: () => void }[] = [];
    if (!envLoading && !doctor?.manifest_exists) {
      items.push({
        title: t("dashboard.attention.environmentMissing"),
        detail: t("dashboard.attention.environmentMissingDetail"),
        tone: "warning",
        onClick: () => navigate("/agentport"),
      });
    } else if (diffIssues > 0 || envWarningCount > 0 || envStatus?.diff_error) {
      items.push({
        title: t("dashboard.attention.environmentReview"),
        detail: t("dashboard.attention.environmentReviewDetail", { count: diffIssues + envWarningCount }),
        tone: "warning",
        onClick: () => navigate("/agentport/diff"),
      });
    }
    if (updateCount > 0) {
      items.push({
        title: t("dashboard.attention.skillUpdates"),
        detail: t("dashboard.attention.skillUpdatesDetail", { count: updateCount }),
        tone: "warning",
        onClick: () => navigate("/my-skills"),
      });
    }
    if (projectIssueCount > 0 && firstProjectWithIssues) {
      items.push({
        title: t("dashboard.attention.projectDrift"),
        detail: t("dashboard.attention.projectDriftDetail", { count: projectIssueCount }),
        tone: "warning",
        onClick: () => navigate(`/project/${firstProjectWithIssues.id}`),
      });
    }
    if (enabledAgents.length === 0) {
      items.push({
        title: t("dashboard.attention.noAgents"),
        detail: t("dashboard.attention.noAgentsDetail"),
        tone: "warning",
        onClick: () => navigate("/settings"),
      });
    }
    if (totalSkills === 0) {
      items.push({
        title: t("dashboard.attention.noSkills"),
        detail: t("dashboard.attention.noSkillsDetail"),
        tone: "neutral",
        onClick: () => navigate("/install"),
      });
    }
    if (items.length === 0) {
      items.push({
        title: t("dashboard.attention.clear"),
        detail: t("dashboard.attention.clearDetail"),
        tone: "ok",
        onClick: () => navigate("/agentport"),
      });
    }
    return items.slice(0, 5);
  }, [
    diffIssues,
    doctor?.manifest_exists,
    enabledAgents.length,
    envLoading,
    envStatus?.diff_error,
    envWarningCount,
    firstProjectWithIssues,
    navigate,
    projectIssueCount,
    t,
    totalSkills,
    updateCount,
  ]);

  return (
    <div className="app-page app-page-narrow">
      <div className="app-page-header flex flex-col gap-3 pb-3 pr-2">
        <div className="flex flex-wrap items-start justify-between gap-3">
          <div>
            <h1 className="app-page-title">{t("dashboard.title")}</h1>
            <p className="app-page-subtitle text-tertiary">
              {t("dashboard.summary", {
                environment: environmentLabel,
                skills: totalSkills,
                agents: enabledAgents.length,
                projects: projects.length,
              })}
            </p>
          </div>
          <button
            type="button"
            onClick={refreshEnvironment}
            className="app-button-secondary"
            disabled={envLoading}
          >
            {envLoading ? <Loader2 className="h-4 w-4 animate-spin" /> : <RefreshCw className="h-4 w-4" />}
            {t("dashboard.refresh")}
          </button>
        </div>
      </div>

      <div className="grid gap-3.5 lg:grid-cols-3">
        <SummaryCard
          title={t("dashboard.cards.environment")}
          value={environmentLabel}
          detail={t("dashboard.cards.environmentDetail", {
            profiles: doctor?.profile_count ?? 0,
            packages: doctor?.package_count ?? 0,
            artifacts: doctor?.artifact_count ?? 0,
            issues: diffIssues + envWarningCount,
          })}
          tone={environmentTone}
          icon={FileCode2}
          onClick={() => navigate("/agentport")}
        />
        <SummaryCard
          title={t("dashboard.cards.library")}
          value={t("dashboard.cards.skillCount", { count: totalSkills })}
          detail={t("dashboard.cards.libraryDetail", {
            synced: syncedSkills,
            updates: updateCount,
            untagged: untaggedCount,
          })}
          tone={updateCount > 0 || untaggedCount > 0 ? "warning" : "ok"}
          icon={Layers}
          onClick={() => navigate("/my-skills")}
        />
        <SummaryCard
          title={t("dashboard.cards.workspaces")}
          value={t("dashboard.cards.agentProjectCount", {
            agents: enabledAgents.length,
            projects: projects.length,
          })}
          detail={t("dashboard.cards.workspaceDetail", { issues: projectIssueCount })}
          tone={projectIssueCount > 0 || enabledAgents.length === 0 ? "warning" : "ok"}
          icon={Bot}
          onClick={() => navigate("/global-workspace")}
        />
      </div>

      <section>
        <h2 className="app-section-title mb-2.5">{t("dashboard.attention.title")}</h2>
        <div className="app-panel overflow-hidden divide-y divide-border-subtle">
          {attentionItems.map((item) => (
            <AttentionRow
              key={item.title}
              title={item.title}
              detail={item.detail}
              tone={item.tone}
              onClick={item.onClick}
            />
          ))}
        </div>
      </section>

      <section>
        <h2 className="app-section-title mb-2.5">{t("dashboard.quickActions")}</h2>
        <div className="grid gap-2 sm:grid-cols-2 lg:grid-cols-4">
          <button type="button" onClick={() => navigate("/install")} className="app-button-primary justify-center">
            <Plus className="h-4 w-4" />
            {t("dashboard.actions.install")}
          </button>
          <button type="button" onClick={() => navigate("/install?tab=local")} className="app-button-secondary justify-center">
            <Download className="h-4 w-4" />
            {t("dashboard.actions.scan")}
          </button>
          <button type="button" onClick={() => navigate("/agentport")} className="app-button-secondary justify-center">
            <FileCode2 className="h-4 w-4" />
            {t("dashboard.actions.environment")}
          </button>
          <button type="button" onClick={() => navigate("/agentport/diff")} className="app-button-secondary justify-center">
            <GitCompareArrows className="h-4 w-4" />
            {t("dashboard.actions.diff")}
          </button>
        </div>
      </section>

      {recentSkills.length > 0 && (
        <section>
          <h2 className="app-section-title mb-2.5">{t("dashboard.recentSkills")}</h2>
          <div className="app-panel overflow-hidden divide-y divide-border-subtle">
            {recentSkills.map((skill) => (
              <button
                key={skill.id}
                type="button"
                onClick={() => {
                  openSkillDetailById(skill.id);
                  navigate("/my-skills");
                }}
                className="flex w-full items-center justify-between gap-3 px-3.5 py-2.5 text-left transition-colors hover:bg-surface-hover"
              >
                <span className="flex min-w-0 items-center gap-2.5">
                  <span className="flex h-6 w-6 shrink-0 items-center justify-center rounded-[4px] bg-accent-bg text-[13px] font-semibold text-accent-light">
                    {skill.name.charAt(0).toUpperCase()}
                  </span>
                  <span className="min-w-0">
                    <span className="flex items-center gap-1.5 text-[13px] font-medium text-secondary">
                      <span className="truncate">{skill.name}</span>
                      <span className="shrink-0 rounded border border-border bg-surface-hover px-1.5 py-px text-[9px] font-normal text-muted">
                        {skill.source_type}
                      </span>
                    </span>
                    <span className="mt-px block truncate text-[13px] text-muted">
                      {skill.targets.length > 0
                        ? t("dashboard.skillSynced", { tools: skill.targets.map((target) => target.tool).join(", ") })
                        : t("dashboard.skillNotSynced")}
                    </span>
                  </span>
                </span>
              </button>
            ))}
          </div>
        </section>
      )}
    </div>
  );
}
