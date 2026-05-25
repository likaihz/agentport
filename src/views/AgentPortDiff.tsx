import { useCallback, useEffect, useMemo, useState } from "react";
import {
  AlertTriangle,
  CheckCircle2,
  FileCode2,
  GitCompareArrows,
  Layers,
  Loader2,
  RefreshCw,
  UploadCloud,
} from "lucide-react";
import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router-dom";
import { toast } from "sonner";
import { cn } from "../utils";
import * as api from "../lib/tauri";
import { getErrorMessage } from "../lib/error";

function shortHash(hash?: string | null) {
  if (!hash) return "--";
  return hash.length > 16 ? hash.slice(0, 16) : hash;
}

export function AgentPortDiff() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const [status, setStatus] = useState<api.AgentPortEnvStatus | null>(null);
  const [loading, setLoading] = useState(false);

  const refresh = useCallback(async () => {
    setLoading(true);
    try {
      setStatus(await api.agentportEnvStatus());
    } catch (error) {
      toast.error(getErrorMessage(error, t("agentportDiff.errors.refresh")));
    } finally {
      setLoading(false);
    }
  }, [t]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const diff = status?.diff ?? null;
  const issueCount = useMemo(() => {
    if (!diff) return 0;
    return (
      diff.missing_skills.length +
      diff.unmanaged_skills.length +
      diff.missing_artifacts.length +
      diff.unmanaged_artifacts.length +
      diff.changed_artifacts.length +
      (diff.missing_profile ? 1 : 0)
    );
  }, [diff]);

  const stats = [
    {
      label: t("agentportDiff.stats.missingSkills"),
      value: diff?.missing_skills.length ?? 0,
      icon: Layers,
      tone: "text-amber-400 bg-amber-500/[0.08]",
    },
    {
      label: t("agentportDiff.stats.unmanagedSkills"),
      value: diff?.unmanaged_skills.length ?? 0,
      icon: Layers,
      tone: "text-sky-400 bg-sky-500/[0.08]",
    },
    {
      label: t("agentportDiff.stats.missingArtifacts"),
      value: diff?.missing_artifacts.length ?? 0,
      icon: UploadCloud,
      tone: "text-violet-400 bg-violet-500/[0.08]",
    },
    {
      label: t("agentportDiff.stats.changedArtifacts"),
      value: diff?.changed_artifacts.length ?? 0,
      icon: GitCompareArrows,
      tone: "text-rose-400 bg-rose-500/[0.08]",
    },
  ];

  return (
    <div className="app-page app-page-narrow">
      <div className="app-page-header flex flex-wrap items-start justify-between gap-3 pb-3 pr-2">
        <div>
          <h1 className="app-page-title flex items-center gap-2">
            <GitCompareArrows className="h-4 w-4 text-accent" />
            {t("agentportDiff.title")}
          </h1>
          <p className="app-page-subtitle">{t("agentportDiff.subtitle")}</p>
        </div>
        <div className="flex flex-wrap gap-2">
          <button
            type="button"
            onClick={() => navigate("/agentport")}
            className="app-button-secondary"
          >
            <FileCode2 className="h-4 w-4" />
            {t("agentportDiff.actions.environment")}
          </button>
          <button
            type="button"
            onClick={refresh}
            disabled={loading}
            className="app-button-primary"
          >
            {loading ? <Loader2 className="h-4 w-4 animate-spin" /> : <RefreshCw className="h-4 w-4" />}
            {t("agentportDiff.actions.refresh")}
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
        <div className="mb-2.5 flex items-center justify-between gap-3">
          <h2 className="app-section-title">{t("agentportDiff.sections.summary")}</h2>
          <span className={cn(
            "app-badge",
            diff?.ok ? "border-accent-border bg-accent-bg text-accent-light" : "border-amber-500/30 bg-amber-500/[0.08] text-amber-400",
          )}>
            {diff?.ok ? <CheckCircle2 className="h-3 w-3" /> : <AlertTriangle className="h-3 w-3" />}
            {diff ? t("agentportDiff.issueCount", { count: issueCount }) : t("agentportDiff.noManifest")}
          </span>
        </div>
        <div className="app-panel overflow-hidden divide-y divide-border-subtle">
          {status?.diff_error && (
            <DiffRow title={t("agentportDiff.groups.error")} detail={status.diff_error} tone="warning" />
          )}
          {!diff && !status?.diff_error && (
            <DiffRow title={t("agentportDiff.groups.unavailable")} detail={t("agentportDiff.noManifest")} />
          )}
          {diff && (
            <>
              <StringList title={t("agentportDiff.groups.missingSkills")} items={diff.missing_skills} />
              <StringList title={t("agentportDiff.groups.unmanagedSkills")} items={diff.unmanaged_skills} />
              <StringList title={t("agentportDiff.groups.missingArtifacts")} items={diff.missing_artifacts} />
              <StringList title={t("agentportDiff.groups.unmanagedArtifacts")} items={diff.unmanaged_artifacts} />
              {diff.changed_artifacts.length === 0 ? (
                <DiffRow title={t("agentportDiff.groups.changedArtifacts")} detail={t("agentportDiff.emptyGroup")} ok />
              ) : (
                diff.changed_artifacts.map((artifact) => (
                  <DiffRow
                    key={artifact.id}
                    title={t("agentportDiff.groups.changedArtifacts")}
                    detail={`${artifact.id} · ${artifact.kind}`}
                    meta={`${shortHash(artifact.expected_hash)} -> ${shortHash(artifact.current_hash)}`}
                    tone="warning"
                  />
                ))
              )}
              {diff.missing_profile && (
                <DiffRow title={t("agentportDiff.groups.missingProfile")} detail={diff.manifest_path} tone="warning" />
              )}
            </>
          )}
        </div>
      </section>
    </div>
  );
}

function StringList({ title, items }: { title: string; items: string[] }) {
  const { t } = useTranslation();
  if (items.length === 0) {
    return <DiffRow title={title} detail={t("agentportDiff.emptyGroup")} ok />;
  }
  return (
    <>
      {items.map((item) => (
        <DiffRow key={`${title}:${item}`} title={title} detail={item} tone="warning" />
      ))}
    </>
  );
}

function DiffRow({
  title,
  detail,
  meta,
  ok,
  tone,
}: {
  title: string;
  detail: string;
  meta?: string;
  ok?: boolean;
  tone?: "warning";
}) {
  return (
    <div className="flex min-h-[48px] items-start justify-between gap-4 px-4 py-3">
      <div className="min-w-0">
        <p className="text-[13px] font-medium text-secondary">{title}</p>
        <p className="mt-0.5 break-words text-[12px] leading-5 text-muted">{detail}</p>
        {meta && <p className="mt-1 font-mono text-[11px] text-faint">{meta}</p>}
      </div>
      <span className={cn(
        "inline-flex h-6 min-w-[68px] shrink-0 items-center justify-center rounded-md border px-2 text-[12px] font-medium",
        ok
          ? "border-accent-border bg-accent-bg text-accent-light"
          : tone === "warning"
            ? "border-amber-500/30 bg-amber-500/[0.08] text-amber-400"
            : "border-border-subtle bg-surface-hover text-muted",
      )}>
        {ok ? "OK" : "--"}
      </span>
    </div>
  );
}
