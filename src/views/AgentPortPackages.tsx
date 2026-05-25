import { useCallback, useEffect, useMemo, useState } from "react";
import {
  AlertTriangle,
  Boxes,
  CheckCircle2,
  FileCode2,
  GitBranch,
  Layers,
  Loader2,
  RefreshCw,
} from "lucide-react";
import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router-dom";
import { toast } from "sonner";
import { cn } from "../utils";
import * as api from "../lib/tauri";
import { getErrorMessage } from "../lib/error";

function compactSource(source?: string | null) {
  if (!source) return "";
  return source
    .replace(/^https:\/\/github.com\//, "github:")
    .replace(/\.git$/, "")
    .replace(/\/Users\/[^/]+/, "~")
    .replace(/\/home\/[^/]+/, "~");
}

function countUpdates(packages: api.AgentPortPackageSummary[]) {
  return packages.reduce((sum, pkg) => sum + pkg.update_available_count, 0);
}

export function AgentPortPackages() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const [status, setStatus] = useState<api.AgentPortPackagesStatus | null>(null);
  const [loading, setLoading] = useState(false);

  const refresh = useCallback(async () => {
    setLoading(true);
    try {
      setStatus(await api.agentportPackagesStatus());
    } catch (error) {
      toast.error(getErrorMessage(error, t("agentportPackages.errors.refresh")));
    } finally {
      setLoading(false);
    }
  }, [t]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const packages = useMemo(() => status?.packages ?? [], [status]);
  const artifactCount = useMemo(
    () => packages.reduce((sum, pkg) => sum + pkg.artifact_count, 0),
    [packages],
  );
  const skillCount = useMemo(
    () => packages.reduce((sum, pkg) => sum + pkg.skill_count, 0),
    [packages],
  );
  const updateCount = countUpdates(packages);

  const stats = [
    {
      label: t("agentportPackages.stats.packages"),
      value: status?.package_count ?? 0,
      icon: Boxes,
      tone: "text-accent-light bg-accent-bg",
    },
    {
      label: t("agentportPackages.stats.skills"),
      value: skillCount,
      icon: Layers,
      tone: "text-sky-400 bg-sky-500/[0.08]",
    },
    {
      label: t("agentportPackages.stats.artifacts"),
      value: artifactCount,
      icon: FileCode2,
      tone: "text-violet-400 bg-violet-500/[0.08]",
    },
    {
      label: t("agentportPackages.stats.updates"),
      value: updateCount,
      icon: AlertTriangle,
      tone: updateCount > 0 ? "text-amber-400 bg-amber-500/[0.08]" : "text-emerald-400 bg-emerald-500/[0.08]",
    },
  ];

  return (
    <div className="app-page app-page-narrow">
      <div className="app-page-header flex flex-wrap items-start justify-between gap-3 pb-3 pr-2">
        <div>
          <h1 className="app-page-title flex items-center gap-2">
            <Boxes className="h-4 w-4 text-accent" />
            {t("agentportPackages.title")}
          </h1>
          <p className="app-page-subtitle">{t("agentportPackages.subtitle")}</p>
        </div>
        <div className="flex flex-wrap gap-2">
          <button
            type="button"
            onClick={() => navigate("/agentport")}
            className="app-button-secondary"
          >
            <FileCode2 className="h-4 w-4" />
            {t("agentportPackages.actions.environment")}
          </button>
          <button
            type="button"
            onClick={refresh}
            disabled={loading}
            className="app-button-primary"
          >
            {loading ? <Loader2 className="h-4 w-4 animate-spin" /> : <RefreshCw className="h-4 w-4" />}
            {t("agentportPackages.actions.refresh")}
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

      {Boolean(status?.unpackaged_skill_count) && (
        <div className="app-panel flex items-start gap-3 border-amber-500/30 bg-amber-500/[0.08] px-4 py-3">
          <AlertTriangle className="mt-0.5 h-4 w-4 shrink-0 text-amber-400" />
          <div>
            <p className="text-[13px] font-medium text-secondary">
              {t("agentportPackages.unmanaged.title")}
            </p>
            <p className="mt-0.5 text-[12px] leading-5 text-muted">
              {t("agentportPackages.unmanaged.detail", { count: status?.unpackaged_skill_count ?? 0 })}
            </p>
          </div>
        </div>
      )}

      <section>
        <h2 className="app-section-title mb-2.5">{t("agentportPackages.sections.packages")}</h2>
        <div className="app-panel overflow-hidden divide-y divide-border-subtle">
          {loading && packages.length === 0 ? (
            <div className="flex items-center gap-2 px-4 py-6 text-[13px] text-muted">
              <Loader2 className="h-4 w-4 animate-spin" />
              {t("agentportPackages.loading")}
            </div>
          ) : packages.length === 0 ? (
            <div className="flex items-center gap-2 px-4 py-6 text-[13px] text-muted">
              <Boxes className="h-4 w-4" />
              {t("agentportPackages.empty")}
            </div>
          ) : (
            packages.map((pkg) => <PackageRow key={pkg.id} pkg={pkg} />)
          )}
        </div>
      </section>
    </div>
  );
}

function PackageRow({ pkg }: { pkg: api.AgentPortPackageSummary }) {
  const { t } = useTranslation();
  const source = compactSource(pkg.source.resolved_reference ?? pkg.source.reference);
  const revision = pkg.source.revision ?? "";
  const remoteRevision = pkg.source.remote_revision ?? "";
  const hasUpdate = pkg.update_available_count > 0 || (remoteRevision && revision && remoteRevision !== revision);

  return (
    <div className="px-4 py-3">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div className="min-w-0">
          <div className="flex min-w-0 flex-wrap items-center gap-2">
            <h3 className="truncate text-[13px] font-medium text-secondary" title={pkg.id}>
              {pkg.id}
            </h3>
            <span className="app-badge py-0.5 text-[11px]">{pkg.source.type}</span>
            <span className="app-badge py-0.5 text-[11px]">{pkg.source.confidence}</span>
          </div>
          {source && (
            <p className="mt-1 truncate text-[12px] text-muted" title={source}>
              {source}
            </p>
          )}
        </div>
        <span className={cn(
          "inline-flex h-6 min-w-[92px] items-center justify-center gap-1.5 rounded-md border px-2 text-[12px] font-medium",
          hasUpdate
            ? "border-amber-500/30 bg-amber-500/[0.08] text-amber-400"
            : "border-accent-border bg-accent-bg text-accent-light",
        )}>
          {hasUpdate ? <AlertTriangle className="h-3 w-3" /> : <CheckCircle2 className="h-3 w-3" />}
          {hasUpdate ? t("agentportPackages.status.update") : t("agentportPackages.status.current")}
        </span>
      </div>

      <div className="mt-3 grid grid-cols-2 gap-2 text-[12px] text-muted md:grid-cols-4">
        <Metric label={t("agentportPackages.metrics.skills")} value={String(pkg.skill_count)} />
        <Metric label={t("agentportPackages.metrics.artifacts")} value={String(pkg.artifact_count)} />
        <Metric label={t("agentportPackages.metrics.updates")} value={String(pkg.update_available_count)} />
        <Metric label={t("agentportPackages.metrics.branch")} value={pkg.source.branch ?? "--"} />
      </div>

      {(revision || remoteRevision) && (
        <div className="mt-3 flex flex-wrap gap-2 text-[12px] text-muted">
          <span className="inline-flex items-center gap-1">
            <GitBranch className="h-3 w-3" />
            {revision || "--"}
          </span>
          {remoteRevision && remoteRevision !== revision && (
            <span className="text-amber-400">{remoteRevision}</span>
          )}
        </div>
      )}

      {pkg.managed_skill_names.length > 0 && (
        <div className="mt-3 flex flex-wrap gap-1.5">
          {pkg.managed_skill_names.slice(0, 8).map((name) => (
            <span key={name} className="app-badge py-0.5 text-[11px]">
              {name}
            </span>
          ))}
          {pkg.managed_skill_names.length > 8 && (
            <span className="app-badge py-0.5 text-[11px]">
              {t("agentportPackages.moreSkills", { count: pkg.managed_skill_names.length - 8 })}
            </span>
          )}
        </div>
      )}
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
