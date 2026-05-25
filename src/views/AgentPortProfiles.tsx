import { useCallback, useEffect, useMemo, useState } from "react";
import {
  CheckCircle2,
  FileCode2,
  Layers,
  Loader2,
  RefreshCw,
  SlidersHorizontal,
  Target,
} from "lucide-react";
import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router-dom";
import { toast } from "sonner";
import { cn } from "../utils";
import * as api from "../lib/tauri";
import { getErrorMessage } from "../lib/error";

function uniqueSkillCount(profiles: api.AgentPortProfileSummary[]) {
  return new Set(profiles.flatMap((profile) => profile.skills)).size;
}

export function AgentPortProfiles() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const [status, setStatus] = useState<api.AgentPortProfilesStatus | null>(null);
  const [loading, setLoading] = useState(false);

  const refresh = useCallback(async () => {
    setLoading(true);
    try {
      setStatus(await api.agentportProfilesStatus());
    } catch (error) {
      toast.error(getErrorMessage(error, t("agentportProfiles.errors.refresh")));
    } finally {
      setLoading(false);
    }
  }, [t]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const profiles = status?.profiles ?? [];
  const activeProfile = profiles.find((profile) => profile.active);
  const enabledToolCount = useMemo(
    () => new Set(profiles.flatMap((profile) => profile.enabled_tools)).size,
    [profiles],
  );
  const stats = [
    {
      label: t("agentportProfiles.stats.profiles"),
      value: status?.profile_count ?? 0,
      icon: SlidersHorizontal,
      tone: "text-accent-light bg-accent-bg",
    },
    {
      label: t("agentportProfiles.stats.skills"),
      value: uniqueSkillCount(profiles),
      icon: Layers,
      tone: "text-sky-400 bg-sky-500/[0.08]",
    },
    {
      label: t("agentportProfiles.stats.tools"),
      value: enabledToolCount,
      icon: Target,
      tone: "text-violet-400 bg-violet-500/[0.08]",
    },
    {
      label: t("agentportProfiles.stats.active"),
      value: activeProfile ? 1 : 0,
      icon: CheckCircle2,
      tone: activeProfile ? "text-emerald-400 bg-emerald-500/[0.08]" : "text-amber-400 bg-amber-500/[0.08]",
    },
  ];

  return (
    <div className="app-page app-page-narrow">
      <div className="app-page-header flex flex-wrap items-start justify-between gap-3 pb-3 pr-2">
        <div>
          <h1 className="app-page-title flex items-center gap-2">
            <SlidersHorizontal className="h-4 w-4 text-accent" />
            {t("agentportProfiles.title")}
          </h1>
          <p className="app-page-subtitle">{t("agentportProfiles.subtitle")}</p>
        </div>
        <div className="flex flex-wrap gap-2">
          <button
            type="button"
            onClick={() => navigate("/agentport")}
            className="app-button-secondary"
          >
            <FileCode2 className="h-4 w-4" />
            {t("agentportProfiles.actions.environment")}
          </button>
          <button
            type="button"
            onClick={refresh}
            disabled={loading}
            className="app-button-primary"
          >
            {loading ? <Loader2 className="h-4 w-4 animate-spin" /> : <RefreshCw className="h-4 w-4" />}
            {t("agentportProfiles.actions.refresh")}
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
        <h2 className="app-section-title mb-2.5">{t("agentportProfiles.sections.profiles")}</h2>
        <div className="app-panel overflow-hidden divide-y divide-border-subtle">
          {loading && profiles.length === 0 ? (
            <div className="flex items-center gap-2 px-4 py-6 text-[13px] text-muted">
              <Loader2 className="h-4 w-4 animate-spin" />
              {t("agentportProfiles.loading")}
            </div>
          ) : profiles.length === 0 ? (
            <div className="flex items-center gap-2 px-4 py-6 text-[13px] text-muted">
              <SlidersHorizontal className="h-4 w-4" />
              {t("agentportProfiles.empty")}
            </div>
          ) : (
            profiles.map((profile) => <ProfileRow key={profile.id} profile={profile} />)
          )}
        </div>
      </section>
    </div>
  );
}

function ProfileRow({ profile }: { profile: api.AgentPortProfileSummary }) {
  const { t } = useTranslation();

  return (
    <div className="px-4 py-3">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div className="min-w-0">
          <div className="flex min-w-0 flex-wrap items-center gap-2">
            <h3 className="truncate text-[13px] font-medium text-secondary" title={profile.id}>
              {profile.name}
            </h3>
            <span className="app-badge py-0.5 text-[11px]">{profile.id}</span>
            {profile.active && (
              <span className="inline-flex h-6 items-center gap-1.5 rounded-md border border-accent-border bg-accent-bg px-2 text-[12px] font-medium text-accent-light">
                <CheckCircle2 className="h-3 w-3" />
                {t("agentportProfiles.status.active")}
              </span>
            )}
          </div>
          {profile.description && (
            <p className="mt-1 text-[12px] leading-5 text-muted">{profile.description}</p>
          )}
        </div>
      </div>

      <div className="mt-3 grid grid-cols-2 gap-2 text-[12px] text-muted md:grid-cols-4">
        <Metric label={t("agentportProfiles.metrics.skills")} value={String(profile.skill_count)} />
        <Metric label={t("agentportProfiles.metrics.tools")} value={String(profile.enabled_tool_count)} />
        <Metric label={t("agentportProfiles.metrics.toggles")} value={String(profile.tool_toggle_count)} />
        <Metric
          label={t("agentportProfiles.metrics.active")}
          value={profile.active ? t("agentportProfiles.status.yes") : t("agentportProfiles.status.no")}
        />
      </div>

      {profile.enabled_tools.length > 0 && (
        <div className="mt-3 flex flex-wrap gap-1.5">
          {profile.enabled_tools.map((tool) => (
            <span key={tool} className="app-badge py-0.5 text-[11px]">
              {tool}
            </span>
          ))}
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
