import { useCallback, useEffect, useMemo, useState } from "react";
import {
  AlertTriangle,
  CheckCircle2,
  DatabaseBackup,
  FileCode2,
  HardDrive,
  KeyRound,
  Loader2,
  RefreshCw,
  Route,
  ShieldCheck,
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

export function AgentPortMachines() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const [status, setStatus] = useState<api.AgentPortMachineStatus | null>(null);
  const [loading, setLoading] = useState(false);

  const refresh = useCallback(async () => {
    setLoading(true);
    try {
      setStatus(await api.agentportMachineStatus());
    } catch (error) {
      toast.error(getErrorMessage(error, t("agentportMachines.errors.refresh")));
    } finally {
      setLoading(false);
    }
  }, [t]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const warnings = useMemo(() => status?.warnings ?? [], [status]);
  const stats = [
    {
      label: t("agentportMachines.stats.origins"),
      value: status?.origin_count ?? 0,
      icon: Route,
      tone: "text-sky-400 bg-sky-500/[0.08]",
    },
    {
      label: t("agentportMachines.stats.secrets"),
      value: status?.secret_count ?? 0,
      icon: KeyRound,
      tone: "text-violet-400 bg-violet-500/[0.08]",
    },
    {
      label: t("agentportMachines.stats.missingSecrets"),
      value: status?.missing_secret_count ?? 0,
      icon: AlertTriangle,
      tone: (status?.missing_secret_count ?? 0) > 0 ? "text-amber-400 bg-amber-500/[0.08]" : "text-emerald-400 bg-emerald-500/[0.08]",
    },
    {
      label: t("agentportMachines.stats.backups"),
      value: status?.backup_count ?? 0,
      icon: DatabaseBackup,
      tone: "text-accent-light bg-accent-bg",
    },
  ];

  return (
    <div className="app-page app-page-narrow">
      <div className="app-page-header flex flex-wrap items-start justify-between gap-3 pb-3 pr-2">
        <div>
          <h1 className="app-page-title flex items-center gap-2">
            <HardDrive className="h-4 w-4 text-accent" />
            {t("agentportMachines.title")}
          </h1>
          <p className="app-page-subtitle">{t("agentportMachines.subtitle")}</p>
        </div>
        <div className="flex flex-wrap gap-2">
          <button
            type="button"
            onClick={() => navigate("/agentport")}
            className="app-button-secondary"
          >
            <FileCode2 className="h-4 w-4" />
            {t("agentportMachines.actions.environment")}
          </button>
          <button
            type="button"
            onClick={refresh}
            disabled={loading}
            className="app-button-primary"
          >
            {loading ? <Loader2 className="h-4 w-4 animate-spin" /> : <RefreshCw className="h-4 w-4" />}
            {t("agentportMachines.actions.refresh")}
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
        <h2 className="app-section-title mb-2.5">{t("agentportMachines.sections.localState")}</h2>
        <div className="app-panel overflow-hidden divide-y divide-border-subtle">
          <PathRow label={t("agentportMachines.fields.machineDir")} path={status?.machine_dir} ok={status?.machine_dir_exists} />
          <PathRow label={t("agentportMachines.fields.machineLocal")} path={status?.machine_local_path} ok={status?.machine_local_exists} />
          <PathRow label={t("agentportMachines.fields.secretsLocal")} path={status?.secrets_local_path} ok={status?.secrets_local_exists} />
          <PathRow label={t("agentportMachines.fields.backups")} path={status?.backups_dir} ok={status?.backups_dir_exists} />
          <StatusRow
            label={t("agentportMachines.fields.gitignore")}
            value={status?.gitignore_protected ? t("agentportMachines.status.protected") : t("agentportMachines.status.needsProtection")}
            ok={status?.gitignore_protected}
          />
        </div>
      </section>

      <section>
        <h2 className="app-section-title mb-2.5">{t("agentportMachines.sections.origins")}</h2>
        <div className="app-panel overflow-hidden divide-y divide-border-subtle">
          {loading && !status ? (
            <LoadingRow label={t("agentportMachines.loading")} />
          ) : status?.origins.length ? (
            status.origins.map((origin) => (
              <StatusRow
                key={origin.id}
                label={origin.id}
                value={compactPath(origin.path)}
                meta={origin.updated_at}
                ok
              />
            ))
          ) : (
            <StatusRow label={t("agentportMachines.empty.origins")} value={t("agentportMachines.empty.none")} />
          )}
        </div>
      </section>

      <section>
        <h2 className="app-section-title mb-2.5">{t("agentportMachines.sections.secrets")}</h2>
        <div className="app-panel overflow-hidden divide-y divide-border-subtle">
          {status?.missing_secret_ids.map((id) => (
            <StatusRow key={`missing:${id}`} label={id} value={t("agentportMachines.status.missing")} ok={false} />
          ))}
          {status?.secret_ids.map((id) => (
            <StatusRow key={id} label={id} value={t("agentportMachines.status.present")} ok />
          ))}
          {!status?.secret_ids.length && !status?.missing_secret_ids.length && (
            <StatusRow label={t("agentportMachines.empty.secrets")} value={t("agentportMachines.empty.none")} />
          )}
        </div>
      </section>

      {warnings.length > 0 && (
        <section>
          <h2 className="app-section-title mb-2.5">{t("agentportMachines.sections.warnings")}</h2>
          <div className="app-panel overflow-hidden divide-y divide-border-subtle">
            {warnings.map((warning) => (
              <StatusRow key={warning} label={t("agentportMachines.fields.warning")} value={warning} ok={false} />
            ))}
          </div>
        </section>
      )}
    </div>
  );
}

function LoadingRow({ label }: { label: string }) {
  return (
    <div className="flex items-center gap-2 px-4 py-4 text-[13px] text-muted">
      <Loader2 className="h-4 w-4 animate-spin" />
      {label}
    </div>
  );
}

function PathRow({ label, path, ok }: { label: string; path?: string | null; ok?: boolean }) {
  return (
    <StatusRow label={label} value={compactPath(path) || "--"} ok={ok} />
  );
}

function StatusRow({
  label,
  value,
  meta,
  ok,
}: {
  label: string;
  value: string;
  meta?: string;
  ok?: boolean;
}) {
  return (
    <div className="flex min-h-[48px] items-start justify-between gap-4 px-4 py-3">
      <div className="min-w-0">
        <p className="text-[13px] font-medium text-secondary">{label}</p>
        <p className="mt-0.5 break-words text-[12px] leading-5 text-muted">{value}</p>
        {meta && <p className="mt-1 text-[11px] text-faint">{meta}</p>}
      </div>
      <span className={cn(
        "inline-flex h-6 min-w-[68px] shrink-0 items-center justify-center gap-1 rounded-md border px-2 text-[12px] font-medium",
        ok
          ? "border-accent-border bg-accent-bg text-accent-light"
          : ok === false
            ? "border-amber-500/30 bg-amber-500/[0.08] text-amber-400"
            : "border-border-subtle bg-surface-hover text-muted",
      )}>
        {ok ? <CheckCircle2 className="h-3 w-3" /> : ok === false ? <AlertTriangle className="h-3 w-3" /> : <ShieldCheck className="h-3 w-3" />}
        {ok ? "OK" : "--"}
      </span>
    </div>
  );
}
