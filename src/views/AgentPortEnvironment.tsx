import { useCallback, useEffect, useMemo, useState } from "react";
import {
  AlertTriangle,
  Bot,
  CheckCircle2,
  FileCode2,
  FileWarning,
  Layers,
  Loader2,
  Play,
  RefreshCw,
  UploadCloud,
} from "lucide-react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { cn } from "../utils";
import * as api from "../lib/tauri";
import { getErrorMessage } from "../lib/error";

type BusyAction = "refresh" | "init" | "planExport" | "export" | "planApply" | "apply" | null;

function compactPath(path: string) {
  return path
    .replace(/\/Users\/[^/]+/, "~")
    .replace(/\/home\/[^/]+/, "~")
    .replace(/^[A-Za-z]:\\Users\\[^\\]+/, "~");
}

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

function reportIssueCount(report: api.AgentPortEnvResourceActionReport | null) {
  if (!report) return 0;
  return report.items.filter((item) => item.error).length;
}

export function AgentPortEnvironment() {
  const { t } = useTranslation();
  const [status, setStatus] = useState<api.AgentPortEnvStatus | null>(null);
  const [busy, setBusy] = useState<BusyAction>(null);
  const [lastResourceReport, setLastResourceReport] = useState<api.AgentPortEnvResourceActionReport | null>(null);
  const [lastApplyReport, setLastApplyReport] = useState<api.AgentPortEnvApplyReport | null>(null);
  const [lastWriteReport, setLastWriteReport] = useState<api.AgentPortEnvWriteReport | null>(null);

  const refreshStatus = useCallback(async () => {
    setBusy((current) => current ?? "refresh");
    try {
      setStatus(await api.agentportEnvStatus());
    } catch (error) {
      toast.error(getErrorMessage(error, t("agentport.errors.refresh")));
    } finally {
      setBusy((current) => current === "refresh" ? null : current);
    }
  }, [t]);

  useEffect(() => {
    void refreshStatus();
  }, [refreshStatus]);

  const doctor = status?.doctor;
  const diff = status?.diff ?? null;
  const diffIssues = diffIssueCount(diff);
  const warningCount = doctor?.warnings.length ?? 0;
  const health = useMemo(() => {
    if (!doctor) return { label: t("agentport.loading"), tone: "neutral" as const };
    if (!doctor.manifest_exists) return { label: t("agentport.needsInit"), tone: "warning" as const };
    if (warningCount > 0 || diffIssues > 0 || status?.diff_error) {
      return { label: t("agentport.needsReview"), tone: "warning" as const };
    }
    return { label: t("agentport.ready"), tone: "ok" as const };
  }, [diffIssues, doctor, status?.diff_error, t, warningCount]);

  const stats = [
    {
      label: t("agentport.stats.skills"),
      value: doctor?.skill_count ?? 0,
      icon: Layers,
      tone: "text-accent-light bg-accent-bg",
    },
    {
      label: t("agentport.stats.packages"),
      value: doctor?.package_count ?? 0,
      icon: FileCode2,
      tone: "text-sky-400 bg-sky-500/[0.08]",
    },
    {
      label: t("agentport.stats.artifacts"),
      value: doctor?.artifact_count ?? 0,
      icon: UploadCloud,
      tone: "text-violet-400 bg-violet-500/[0.08]",
    },
    {
      label: t("agentport.stats.agents"),
      value: doctor?.installed_tool_count ?? 0,
      icon: Bot,
      tone: "text-amber-400 bg-amber-500/[0.08]",
    },
  ];

  const handleInit = async () => {
    setBusy("init");
    try {
      const report = await api.agentportEnvInit(true);
      setLastWriteReport(report);
      setLastResourceReport(null);
      setLastApplyReport(null);
      toast.success(t("agentport.toasts.initialized"));
      await refreshStatus();
    } catch (error) {
      toast.error(getErrorMessage(error, t("agentport.errors.init")));
    } finally {
      setBusy(null);
    }
  };

  const handleExportResources = async (dryRun: boolean) => {
    setBusy(dryRun ? "planExport" : "export");
    try {
      const report = await api.agentportEnvExportResources(true, dryRun);
      setLastResourceReport(report);
      setLastApplyReport(null);
      setLastWriteReport(null);
      toast.success(dryRun ? t("agentport.toasts.exportPlanned") : t("agentport.toasts.exported"));
      if (!dryRun) await refreshStatus();
    } catch (error) {
      toast.error(getErrorMessage(error, t("agentport.errors.export")));
    } finally {
      setBusy(null);
    }
  };

  const handleApplyProfile = async (dryRun: boolean) => {
    setBusy(dryRun ? "planApply" : "apply");
    try {
      const report = await api.agentportEnvApply(null, dryRun);
      setLastApplyReport(report);
      setLastResourceReport(null);
      setLastWriteReport(null);
      if (report.missing_skills.length > 0) {
        toast.error(t("agentport.toasts.applyBlocked", { count: report.missing_skills.length }));
      } else {
        toast.success(dryRun ? t("agentport.toasts.applyPlanned") : t("agentport.toasts.applied"));
      }
      if (!dryRun) await refreshStatus();
    } catch (error) {
      toast.error(getErrorMessage(error, t("agentport.errors.apply")));
    } finally {
      setBusy(null);
    }
  };

  const resourceReport = lastResourceReport ?? (
    lastApplyReport
      ? { ok: lastApplyReport.ok, dry_run: lastApplyReport.dry_run, items: lastApplyReport.resources }
      : null
  );
  const resourceReportIssues = reportIssueCount(resourceReport);

  return (
    <div className="app-page app-page-narrow">
      <div className="app-page-header flex flex-col gap-3 pb-3 pr-2">
        <div className="flex flex-wrap items-start justify-between gap-3">
          <div>
            <h1 className="app-page-title flex items-center gap-2">
              <FileCode2 className="h-4 w-4 text-accent" />
              {t("agentport.title")}
            </h1>
            <p className="app-page-subtitle">{t("agentport.subtitle")}</p>
          </div>
          <div
            className={cn(
              "inline-flex items-center gap-2 rounded-lg border px-3 py-2 text-[13px] font-medium",
              health.tone === "ok"
                ? "border-accent-border bg-accent-bg text-accent-light"
                : health.tone === "warning"
                  ? "border-amber-500/30 bg-amber-500/[0.08] text-amber-400"
                  : "border-border-subtle bg-surface text-muted",
            )}
          >
            {health.tone === "ok" ? <CheckCircle2 className="h-4 w-4" /> : <AlertTriangle className="h-4 w-4" />}
            {health.label}
          </div>
        </div>

        <div className="flex flex-wrap gap-2">
          <button
            type="button"
            onClick={refreshStatus}
            disabled={busy !== null}
            className="app-button-secondary"
          >
            {busy === "refresh" ? <Loader2 className="h-4 w-4 animate-spin" /> : <RefreshCw className="h-4 w-4" />}
            {t("agentport.actions.refresh")}
          </button>
          <button
            type="button"
            onClick={handleInit}
            disabled={busy !== null}
            className="app-button-primary"
          >
            {busy === "init" ? <Loader2 className="h-4 w-4 animate-spin" /> : <FileCode2 className="h-4 w-4" />}
            {doctor?.manifest_exists ? t("agentport.actions.rebuild") : t("agentport.actions.init")}
          </button>
          <button
            type="button"
            onClick={() => handleExportResources(true)}
            disabled={busy !== null}
            className="app-button-secondary"
          >
            {busy === "planExport" ? <Loader2 className="h-4 w-4 animate-spin" /> : <FileWarning className="h-4 w-4" />}
            {t("agentport.actions.planExport")}
          </button>
          <button
            type="button"
            onClick={() => handleExportResources(false)}
            disabled={busy !== null}
            className="app-button-secondary"
          >
            {busy === "export" ? <Loader2 className="h-4 w-4 animate-spin" /> : <UploadCloud className="h-4 w-4" />}
            {t("agentport.actions.export")}
          </button>
          <button
            type="button"
            onClick={() => handleApplyProfile(true)}
            disabled={busy !== null || !doctor?.manifest_exists}
            className="app-button-secondary"
          >
            {busy === "planApply" ? <Loader2 className="h-4 w-4 animate-spin" /> : <FileWarning className="h-4 w-4" />}
            {t("agentport.actions.planApply")}
          </button>
          <button
            type="button"
            onClick={() => handleApplyProfile(false)}
            disabled={busy !== null || !doctor?.manifest_exists}
            className="app-button-secondary"
          >
            {busy === "apply" ? <Loader2 className="h-4 w-4 animate-spin" /> : <Play className="h-4 w-4" />}
            {t("agentport.actions.apply")}
          </button>
        </div>
      </div>

      <div className="grid grid-cols-2 gap-3.5 lg:grid-cols-4">
        {stats.map((stat) => {
          const Icon = stat.icon;
          return (
            <div key={stat.label} className="app-panel flex items-center justify-between px-4 py-4">
              <div>
                <p className="app-section-title mb-1">{stat.label}</p>
                <p className="text-xl font-semibold leading-none text-primary tabular-nums">{stat.value}</p>
              </div>
              <div className={cn("rounded-md border border-border-subtle p-2", stat.tone)}>
                <Icon className="h-4 w-4" />
              </div>
            </div>
          );
        })}
      </div>

      <section>
        <h2 className="app-section-title mb-2.5">{t("agentport.sections.repository")}</h2>
        <div className="app-panel overflow-hidden divide-y divide-border-subtle">
          <PathRow label={t("agentport.fields.manifest")} path={doctor?.manifest_path} ok={doctor?.manifest_exists} />
          <PathRow label={t("agentport.fields.lock")} path={doctor?.lock_path} ok={doctor?.lock_exists} />
          <StatusRow
            label={t("agentport.fields.profile")}
            value={diff?.profile ? `${diff.profile.name} (${diff.profile.id})` : t("agentport.fields.profileFallback")}
            ok={diff ? !diff.missing_profile : undefined}
          />
          <StatusRow
            label={t("agentport.fields.drift")}
            value={diff ? t("agentport.diffSummary", { count: diffIssues }) : t("agentport.notAvailable")}
            ok={Boolean(diff?.ok)}
          />
          {lastWriteReport && (
            <StatusRow
              label={t("agentport.fields.lastWrite")}
              value={t("agentport.writeSummary", {
                skills: lastWriteReport.skill_count,
                artifacts: lastWriteReport.artifact_count,
              })}
              ok
            />
          )}
        </div>
      </section>

      {(warningCount > 0 || status?.diff_error || diffIssues > 0) && (
        <section>
          <h2 className="app-section-title mb-2.5">{t("agentport.sections.review")}</h2>
          <div className="app-panel overflow-hidden divide-y divide-border-subtle">
            {status?.diff_error && (
              <IssueRow title={t("agentport.fields.diffError")} detail={status.diff_error} />
            )}
            {doctor?.warnings.map((warning) => (
              <IssueRow key={warning} title={t("agentport.fields.warning")} detail={warning} />
            ))}
            {diff && <DiffRows diff={diff} />}
          </div>
        </section>
      )}

      {(lastResourceReport || lastApplyReport) && (
        <section>
          <div className="mb-2.5 flex items-center justify-between gap-3">
            <h2 className="app-section-title">{lastApplyReport ? t("agentport.sections.applyReport") : t("agentport.sections.resourceReport")}</h2>
            <span className={cn(
              "app-badge",
              resourceReportIssues > 0 || lastApplyReport?.missing_skills.length
                ? "border-amber-500/30 bg-amber-500/[0.08] text-amber-400"
                : "border-accent-border bg-accent-bg text-accent-light",
            )}>
              {resourceReport?.dry_run ? t("agentport.report.planned") : t("agentport.report.applied")}
            </span>
          </div>
          <div className="app-panel overflow-hidden divide-y divide-border-subtle">
            {lastApplyReport && (
              <StatusRow
                label={t("agentport.report.profile")}
                value={`${lastApplyReport.profile_name} (${lastApplyReport.targets.length})`}
                ok={lastApplyReport.missing_skills.length === 0}
              />
            )}
            {lastApplyReport?.missing_skills.map((skill) => (
              <IssueRow key={skill} title={t("agentport.report.missingSkill")} detail={skill} />
            ))}
            {resourceReport && resourceReport.items.length > 0 ? (
              resourceReport.items.slice(0, 12).map((item) => (
                <ResourceRow key={`${item.id}:${item.target_path}`} item={item} />
              ))
            ) : (
              <StatusRow label={t("agentport.report.resources")} value={t("agentport.report.noItems")} ok />
            )}
            {resourceReport && resourceReport.items.length > 12 && (
              <StatusRow
                label={t("agentport.report.more")}
                value={t("agentport.report.moreCount", { count: resourceReport.items.length - 12 })}
                ok
              />
            )}
          </div>
        </section>
      )}
    </div>
  );
}

function PathRow({ label, path, ok }: { label: string; path?: string; ok?: boolean }) {
  return (
    <StatusRow
      label={label}
      value={path ? compactPath(path) : ""}
      ok={ok}
    />
  );
}

function StatusRow({ label, value, ok }: { label: string; value: string; ok?: boolean }) {
  return (
    <div className="flex min-h-[44px] items-center justify-between gap-4 px-4 py-2.5">
      <div className="min-w-0">
        <p className="text-[13px] font-medium text-secondary">{label}</p>
        <p className="mt-0.5 truncate text-[12px] text-muted" title={value}>{value}</p>
      </div>
      <span className={cn(
        "inline-flex h-6 min-w-[68px] shrink-0 items-center justify-center rounded-md border px-2 text-[12px] font-medium",
        ok
          ? "border-accent-border bg-accent-bg text-accent-light"
          : "border-border-subtle bg-surface-hover text-muted",
      )}>
        {ok ? "OK" : "--"}
      </span>
    </div>
  );
}

function IssueRow({ title, detail }: { title: string; detail: string }) {
  return (
    <div className="flex items-start gap-3 px-4 py-3">
      <AlertTriangle className="mt-0.5 h-4 w-4 shrink-0 text-amber-400" />
      <div className="min-w-0">
        <p className="text-[13px] font-medium text-secondary">{title}</p>
        <p className="mt-0.5 break-words text-[12px] leading-5 text-muted">{detail}</p>
      </div>
    </div>
  );
}

function DiffRows({ diff }: { diff: api.AgentPortEnvDiffReport }) {
  const { t } = useTranslation();
  const groups = [
    [t("agentport.diff.missingSkills"), diff.missing_skills],
    [t("agentport.diff.unmanagedSkills"), diff.unmanaged_skills],
    [t("agentport.diff.missingArtifacts"), diff.missing_artifacts],
    [t("agentport.diff.unmanagedArtifacts"), diff.unmanaged_artifacts],
  ] as const;

  return (
    <>
      {groups.flatMap(([title, items]) =>
        items.slice(0, 6).map((item) => <IssueRow key={`${title}:${item}`} title={title} detail={item} />)
      )}
      {diff.changed_artifacts.slice(0, 6).map((item) => (
        <IssueRow
          key={item.id}
          title={t("agentport.diff.changedArtifacts")}
          detail={`${item.id} · ${item.kind}`}
        />
      ))}
      {diff.missing_profile && (
        <IssueRow title={t("agentport.diff.missingProfile")} detail={diff.manifest_path} />
      )}
    </>
  );
}

function ResourceRow({ item }: { item: api.AgentPortEnvResourceActionItem }) {
  return (
    <div className="flex items-center justify-between gap-4 px-4 py-3">
      <div className="min-w-0">
        <p className="truncate text-[13px] font-medium text-secondary" title={item.id}>
          {item.id}
        </p>
        <p className="mt-0.5 truncate text-[12px] text-muted" title={item.target_path}>
          {item.tool} · {item.kind} · {compactPath(item.target_path)}
        </p>
        {item.error && <p className="mt-1 break-words text-[12px] text-amber-400">{item.error}</p>}
      </div>
      <span className={cn(
        "inline-flex h-6 min-w-[84px] shrink-0 items-center justify-center rounded-md border px-2 text-[12px] font-medium",
        item.error
          ? "border-amber-500/30 bg-amber-500/[0.08] text-amber-400"
          : "border-accent-border bg-accent-bg text-accent-light",
      )}>
        {item.status}
      </span>
    </div>
  );
}
