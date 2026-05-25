import { useCallback, useEffect, useMemo, useState } from "react";
import {
  AlertTriangle,
  Boxes,
  CheckCircle2,
  Download,
  FileCode2,
  GitCompareArrows,
  Layers,
  Loader2,
  RefreshCw,
  Route,
  RotateCcw,
} from "lucide-react";
import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router-dom";
import { toast } from "sonner";
import { cn } from "../utils";
import * as api from "../lib/tauri";
import { getErrorMessage } from "../lib/error";

function compactPath(path?: string | null) {
  if (!path) return "";
  return path
    .replace(/\/Users\/[^/]+/, "~")
    .replace(/\/home\/[^/]+/, "~");
}

function statusTone(status: string) {
  switch (status) {
    case "synced":
      return "border-accent-border bg-accent-bg text-accent-light";
    case "missing":
    case "drifted":
    case "target_newer":
    case "central_newer":
    case "conflict":
      return "border-amber-500/30 bg-amber-500/[0.08] text-amber-400";
    case "unmanaged":
      return "border-sky-500/30 bg-sky-500/[0.08] text-sky-400";
    default:
      return "border-border-subtle bg-surface-hover text-muted";
  }
}

function shortHash(hash?: string | null) {
  if (!hash) return "--";
  return hash.length > 12 ? hash.slice(0, 12) : hash;
}

export function AgentPortArtifacts() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const [status, setStatus] = useState<api.AgentPortArtifactsStatus | null>(null);
  const [targetStatus, setTargetStatus] = useState<api.AgentPortTargetDriftStatus | null>(null);
  const [loading, setLoading] = useState(false);
  const [busyTarget, setBusyTarget] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    setLoading(true);
    try {
      const [artifactStatus, driftStatus] = await Promise.all([
        api.agentportArtifactsStatus(),
        api.agentportTargetDrifts(),
      ]);
      setStatus(artifactStatus);
      setTargetStatus(driftStatus);
    } catch (error) {
      toast.error(getErrorMessage(error, t("agentportArtifacts.errors.refresh")));
    } finally {
      setLoading(false);
    }
  }, [t]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const artifacts = useMemo(() => status?.artifacts ?? [], [status]);
  const unmanagedCount = useMemo(
    () => artifacts.filter((artifact) => artifact.status === "unmanaged").length,
    [artifacts],
  );
  const targetDrifts = useMemo(() => targetStatus?.targets ?? [], [targetStatus]);

  const handleTargetAction = async (
    action: "pull" | "discard",
    target: api.AgentPortTargetDrift,
  ) => {
    const key = `${action}:${target.skill_id}:${target.tool}`;
    setBusyTarget(key);
    try {
      if (action === "pull") {
        await api.agentportPullTarget(target.skill_id, target.tool);
        toast.success(t("agentportArtifacts.toasts.pulledTarget"));
      } else {
        await api.agentportDiscardTarget(target.skill_id, target.tool);
        toast.success(t("agentportArtifacts.toasts.discardedTarget"));
      }
      await refresh();
    } catch (error) {
      toast.error(getErrorMessage(error, t(`agentportArtifacts.errors.${action}Target`)));
    } finally {
      setBusyTarget(null);
    }
  };

  const stats = [
    {
      label: t("agentportArtifacts.stats.artifacts"),
      value: status?.artifact_count ?? 0,
      icon: FileCode2,
      tone: "text-accent-light bg-accent-bg",
    },
    {
      label: t("agentportArtifacts.stats.skills"),
      value: status?.skill_count ?? 0,
      icon: Layers,
      tone: "text-sky-400 bg-sky-500/[0.08]",
    },
    {
      label: t("agentportArtifacts.stats.packageOwned"),
      value: status?.package_owned_count ?? 0,
      icon: Boxes,
      tone: "text-violet-400 bg-violet-500/[0.08]",
    },
    {
      label: t("agentportArtifacts.stats.drift"),
      value: status?.drift_count ?? 0,
      icon: AlertTriangle,
      tone: (status?.drift_count ?? 0) > 0 ? "text-amber-400 bg-amber-500/[0.08]" : "text-emerald-400 bg-emerald-500/[0.08]",
    },
  ];

  return (
    <div className="app-page app-page-narrow">
      <div className="app-page-header flex flex-wrap items-start justify-between gap-3 pb-3 pr-2">
        <div>
          <h1 className="app-page-title flex items-center gap-2">
            <FileCode2 className="h-4 w-4 text-accent" />
            {t("agentportArtifacts.title")}
          </h1>
          <p className="app-page-subtitle">{t("agentportArtifacts.subtitle")}</p>
        </div>
        <div className="flex flex-wrap gap-2">
          <button
            type="button"
            onClick={() => navigate("/agentport/diff")}
            className="app-button-secondary"
          >
            <GitCompareArrows className="h-4 w-4" />
            {t("agentportArtifacts.actions.diff")}
          </button>
          <button
            type="button"
            onClick={refresh}
            disabled={loading}
            className="app-button-primary"
          >
            {loading ? <Loader2 className="h-4 w-4 animate-spin" /> : <RefreshCw className="h-4 w-4" />}
            {t("agentportArtifacts.actions.refresh")}
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

      {Boolean(unmanagedCount) && (
        <div className="app-panel flex items-start gap-3 border-sky-500/30 bg-sky-500/[0.08] px-4 py-3">
          <Route className="mt-0.5 h-4 w-4 shrink-0 text-sky-400" />
          <div>
            <p className="text-[13px] font-medium text-secondary">
              {t("agentportArtifacts.unmanaged.title")}
            </p>
            <p className="mt-0.5 text-[12px] leading-5 text-muted">
              {t("agentportArtifacts.unmanaged.detail", { count: unmanagedCount })}
            </p>
          </div>
        </div>
      )}

      <section>
        <h2 className="app-section-title mb-2.5">{t("agentportArtifacts.sections.artifacts")}</h2>
        <div className="app-panel overflow-hidden divide-y divide-border-subtle">
          {loading && artifacts.length === 0 ? (
            <div className="flex items-center gap-2 px-4 py-6 text-[13px] text-muted">
              <Loader2 className="h-4 w-4 animate-spin" />
              {t("agentportArtifacts.loading")}
            </div>
          ) : artifacts.length === 0 ? (
            <div className="flex items-center gap-2 px-4 py-6 text-[13px] text-muted">
              <FileCode2 className="h-4 w-4" />
              {t("agentportArtifacts.empty")}
            </div>
          ) : (
            artifacts.map((artifact) => <ArtifactRow key={artifact.id} artifact={artifact} />)
          )}
        </div>
      </section>

      <section>
        <h2 className="app-section-title mb-2.5">{t("agentportArtifacts.sections.targetDrift")}</h2>
        <div className="app-panel overflow-hidden divide-y divide-border-subtle">
          {loading && !targetStatus ? (
            <div className="flex items-center gap-2 px-4 py-6 text-[13px] text-muted">
              <Loader2 className="h-4 w-4 animate-spin" />
              {t("agentportArtifacts.loadingTargets")}
            </div>
          ) : targetDrifts.length === 0 ? (
            <div className="flex items-center gap-2 px-4 py-6 text-[13px] text-muted">
              <CheckCircle2 className="h-4 w-4" />
              {t("agentportArtifacts.emptyTargets")}
            </div>
          ) : (
            targetDrifts.map((target) => (
              <TargetDriftRow
                key={`${target.skill_id}:${target.tool}`}
                target={target}
                busyTarget={busyTarget}
                onAction={handleTargetAction}
              />
            ))
          )}
        </div>
      </section>
    </div>
  );
}

function ArtifactRow({ artifact }: { artifact: api.AgentPortArtifactSummary }) {
  const { t } = useTranslation();
  const packageLabel = artifact.package_id ?? artifact.owner_id;
  const path = compactPath(artifact.path);

  return (
    <div className="px-4 py-3">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div className="min-w-0">
          <div className="flex min-w-0 flex-wrap items-center gap-2">
            <h3 className="truncate text-[13px] font-medium text-secondary" title={artifact.id}>
              {artifact.name}
            </h3>
            <span className="app-badge py-0.5 text-[11px]">{artifact.kind}</span>
            <span className="app-badge py-0.5 text-[11px]">{artifact.source_type}</span>
            <span className="app-badge py-0.5 text-[11px]">{artifact.source_confidence}</span>
          </div>
          <p className="mt-1 truncate text-[12px] text-muted" title={artifact.id}>
            {artifact.id}
          </p>
        </div>
        <span className={cn(
          "inline-flex h-6 min-w-[86px] items-center justify-center gap-1.5 rounded-md border px-2 text-[12px] font-medium",
          statusTone(artifact.status),
        )}>
          {artifact.status === "synced" ? <CheckCircle2 className="h-3 w-3" /> : <AlertTriangle className="h-3 w-3" />}
          {t(`agentportArtifacts.status.${artifact.status}`, { defaultValue: artifact.status })}
        </span>
      </div>

      <div className="mt-3 grid grid-cols-1 gap-2 text-[12px] text-muted md:grid-cols-3">
        <Metric label={t("agentportArtifacts.metrics.path")} value={path || "--"} />
        <Metric label={t("agentportArtifacts.metrics.owner")} value={packageLabel ?? "--"} />
        <Metric label={t("agentportArtifacts.metrics.targets")} value={String(artifact.target_count)} />
      </div>

      {artifact.targets.length > 0 && (
        <div className="mt-3 flex flex-wrap gap-1.5">
          {artifact.targets.slice(0, 8).map((target) => (
            <span key={`${target.tool}:${target.path}`} className="app-badge py-0.5 text-[11px]" title={compactPath(target.path)}>
              {target.tool} · {target.mode}
            </span>
          ))}
          {artifact.targets.length > 8 && (
            <span className="app-badge py-0.5 text-[11px]">
              {t("agentportArtifacts.moreTargets", { count: artifact.targets.length - 8 })}
            </span>
          )}
        </div>
      )}

      {(artifact.expected_hash || artifact.current_hash) && (
        <p className="mt-2 break-words font-mono text-[11px] text-faint">
          {artifact.expected_hash ?? "--"} -&gt; {artifact.current_hash ?? "--"}
        </p>
      )}
    </div>
  );
}

function TargetDriftRow({
  target,
  busyTarget,
  onAction,
}: {
  target: api.AgentPortTargetDrift;
  busyTarget: string | null;
  onAction: (action: "pull" | "discard", target: api.AgentPortTargetDrift) => void;
}) {
  const { t } = useTranslation();
  const pullKey = `pull:${target.skill_id}:${target.tool}`;
  const discardKey = `discard:${target.skill_id}:${target.tool}`;

  return (
    <div className="px-4 py-3">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div className="min-w-0">
          <div className="flex min-w-0 flex-wrap items-center gap-2">
            <h3 className="truncate text-[13px] font-medium text-secondary" title={target.skill_id}>
              {target.skill_name}
            </h3>
            <span className="app-badge py-0.5 text-[11px]">{target.tool}</span>
            <span className="app-badge py-0.5 text-[11px]">{target.mode}</span>
          </div>
          <p className="mt-1 truncate text-[12px] text-muted" title={target.target_path}>
            {compactPath(target.target_path)}
          </p>
        </div>
        <span className={cn(
          "inline-flex h-6 min-w-[100px] items-center justify-center gap-1.5 rounded-md border px-2 text-[12px] font-medium",
          statusTone(target.status),
        )}>
          <AlertTriangle className="h-3 w-3" />
          {t(`agentportArtifacts.targetStatus.${target.status}`, { defaultValue: target.status })}
        </span>
      </div>

      <div className="mt-3 grid grid-cols-1 gap-2 text-[12px] text-muted md:grid-cols-3">
        <Metric label={t("agentportArtifacts.metrics.centralHash")} value={shortHash(target.central_hash)} />
        <Metric label={t("agentportArtifacts.metrics.targetHash")} value={shortHash(target.target_hash)} />
        <Metric label={t("agentportArtifacts.metrics.lastSyncedHash")} value={shortHash(target.last_synced_hash)} />
      </div>

      <div className="mt-3 flex flex-wrap gap-2">
        <button
          type="button"
          onClick={() => onAction("pull", target)}
          disabled={!target.can_pull || Boolean(busyTarget)}
          className="app-button-secondary"
        >
          {busyTarget === pullKey ? <Loader2 className="h-4 w-4 animate-spin" /> : <Download className="h-4 w-4" />}
          {t("agentportArtifacts.actions.pullTarget")}
        </button>
        <button
          type="button"
          onClick={() => onAction("discard", target)}
          disabled={!target.can_discard || Boolean(busyTarget)}
          className="app-button-secondary"
        >
          {busyTarget === discardKey ? <Loader2 className="h-4 w-4 animate-spin" /> : <RotateCcw className="h-4 w-4" />}
          {t("agentportArtifacts.actions.discardTarget")}
        </button>
      </div>
    </div>
  );
}

function Metric({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-md border border-border-subtle bg-bg-secondary px-2.5 py-2">
      <p className="text-[11px] uppercase tracking-[0.06em] text-faint">{label}</p>
      <p className="mt-1 truncate font-medium text-secondary" title={value}>{value}</p>
    </div>
  );
}
