import { Pencil, Plug, PowerOff, RadioTower, Trash2 } from "lucide-react";
import type { TFunction } from "i18next";
import type { Account, GmailRealtimeConfig } from "@/lib/api";
import type { RealtimeStatus } from "@/stores/sync.store";
import { getAccountColor } from "@/lib/accountColors";

type AccountsListProps = {
  accounts: Account[];
  accountColorsById: Map<string, string>;
  realtimeStatusByAccount: Record<string, RealtimeStatus>;
  gmailRealtimeByAccount: Record<string, GmailRealtimeConfig>;
  gmailRealtimeActionId: string | null;
  testingId: string | null;
  onToggleGmailRealtime: (account: Account) => void;
  onTestConnection: (accountId: string) => void;
  onEdit: (account: Account) => void;
  onDelete: (target: { id: string; email: string }) => void;
  t: TFunction;
};

export function AccountsList({
  accounts,
  accountColorsById,
  realtimeStatusByAccount,
  gmailRealtimeByAccount,
  gmailRealtimeActionId,
  testingId,
  onToggleGmailRealtime,
  onTestConnection,
  onEdit,
  onDelete,
  t,
}: AccountsListProps) {
  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        gap: "1px",
        borderRadius: "8px",
        overflow: "hidden",
        border: "1px solid var(--color-border)",
      }}
    >
      {accounts.map((account, index) => {
        const realtimeStatus = realtimeStatusByAccount[account.id];
        const realtimeLabel = getAccountRealtimeStatusText(realtimeStatus, t);
        const gmailRealtimeConfig = gmailRealtimeByAccount[account.id];
        const gmailRealtimeLabel =
          account.provider === "gmail"
            ? getGmailRealtimeStatusText(
                gmailRealtimeConfig,
                gmailRealtimeActionId === account.id && !gmailRealtimeConfig?.enabled
                  ? "enabling"
                  : null,
                t,
              )
            : null;
        const accountColor = accountColorsById.get(account.id) ?? getAccountColor(account);

        return (
          <div
            key={account.id}
            style={{
              display: "flex",
              alignItems: "center",
              justifyContent: "space-between",
              padding: "14px 16px",
              backgroundColor: "var(--color-bg)",
              borderTop: index > 0 ? "1px solid var(--color-border)" : "none",
            }}
          >
            <div style={{ display: "flex", flexDirection: "column", gap: "2px" }}>
              <div style={{ display: "flex", alignItems: "center", gap: "8px", minWidth: 0 }}>
                <span
                  aria-hidden="true"
                  style={{
                    width: "8px",
                    height: "8px",
                    borderRadius: "50%",
                    backgroundColor: accountColor,
                    flexShrink: 0,
                  }}
                />
                <span style={{ fontSize: "13px", fontWeight: 500 }}>{account.display_name}</span>
              </div>
              <span style={{ fontSize: "12px", color: "var(--color-text-secondary)" }}>
                {account.email}
              </span>
              <span
                style={{
                  fontSize: "11px",
                  color: "var(--color-text-secondary)",
                  textTransform: "capitalize",
                }}
              >
                {account.provider}
              </span>
              {realtimeLabel && (
                <span
                  aria-label={realtimeLabel}
                  title={realtimeStatus?.message ?? realtimeLabel}
                  style={{ fontSize: "11px", color: "var(--color-text-secondary)" }}
                >
                  {realtimeLabel}
                </span>
              )}
              {gmailRealtimeLabel && (
                <span
                  aria-label={gmailRealtimeLabel}
                  title={gmailRealtimeConfig?.lastError ?? gmailRealtimeLabel}
                  style={{ fontSize: "11px", color: "var(--color-text-secondary)" }}
                >
                  {t("settings.gmailRealtime", "Gmail realtime")}: {gmailRealtimeLabel}
                </span>
              )}
            </div>
            <div style={{ display: "flex", gap: "4px", alignItems: "center" }}>
              {account.provider === "gmail" && (
                <GmailRealtimeButton
                  account={account}
                  config={gmailRealtimeConfig}
                  busy={gmailRealtimeActionId === account.id}
                  onToggle={onToggleGmailRealtime}
                  t={t}
                />
              )}
              <IconButton
                onClick={() => onTestConnection(account.id)}
                disabled={testingId === account.id}
                busy={testingId === account.id}
                title={t("accountSetup.testConnection", "Test Connection")}
                activeColor="var(--color-accent)"
              >
                <Plug size={15} />
              </IconButton>
              <IconButton
                onClick={() => onEdit(account)}
                title={t("settings.editAccount", "Edit account")}
                activeColor="var(--color-accent)"
              >
                <Pencil size={15} />
              </IconButton>
              <IconButton
                onClick={() => onDelete({ id: account.id, email: account.email })}
                title={t("settings.removeAccount")}
                activeColor="#ef4444"
                activeBackground="rgba(239,68,68,0.08)"
              >
                <Trash2 size={15} />
              </IconButton>
            </div>
          </div>
        );
      })}
    </div>
  );
}

type IconButtonProps = {
  children: React.ReactNode;
  title: string;
  onClick: () => void;
  disabled?: boolean;
  busy?: boolean;
  activeColor: string;
  activeBackground?: string;
};

function IconButton({
  children,
  title,
  onClick,
  disabled = false,
  busy = false,
  activeColor,
  activeBackground = "var(--color-bg-hover)",
}: IconButtonProps) {
  return (
    <button
      type="button"
      onClick={onClick}
      disabled={disabled}
      title={title}
      aria-label={title}
      style={{
        display: "flex",
        alignItems: "center",
        padding: "6px",
        borderRadius: "6px",
        border: "none",
        backgroundColor: "transparent",
        color: busy ? activeColor : "var(--color-text-secondary)",
        cursor: busy ? "wait" : "pointer",
        opacity: busy ? 0.6 : 1,
      }}
      onMouseEnter={(e) => {
        if (disabled) return;
        e.currentTarget.style.color = activeColor;
        e.currentTarget.style.backgroundColor = activeBackground;
      }}
      onMouseLeave={(e) => {
        if (disabled) return;
        e.currentTarget.style.color = "var(--color-text-secondary)";
        e.currentTarget.style.backgroundColor = "transparent";
      }}
    >
      {children}
    </button>
  );
}

function GmailRealtimeButton({
  account,
  config,
  busy,
  onToggle,
  t,
}: {
  account: Account;
  config: GmailRealtimeConfig | undefined;
  busy: boolean;
  onToggle: (account: Account) => void;
  t: TFunction;
}) {
  const disabled = busy || (!!config?.configMissing && !config.enabled);
  const label = getGmailRealtimeActionLabel(config, t);

  return (
    <button
      type="button"
      onClick={() => onToggle(account)}
      disabled={disabled}
      title={label}
      aria-label={label}
      style={{
        display: "flex",
        alignItems: "center",
        padding: "6px",
        borderRadius: "6px",
        border: "none",
        backgroundColor: "transparent",
        color: config?.enabled ? "var(--color-accent)" : "var(--color-text-secondary)",
        cursor: disabled ? "not-allowed" : "pointer",
        opacity: disabled ? 0.55 : 1,
      }}
      onMouseEnter={(e) => {
        if (disabled) return;
        e.currentTarget.style.color = config?.enabled ? "#ef4444" : "var(--color-accent)";
        e.currentTarget.style.backgroundColor = "var(--color-bg-hover)";
      }}
      onMouseLeave={(e) => {
        e.currentTarget.style.color = config?.enabled
          ? "var(--color-accent)"
          : "var(--color-text-secondary)";
        e.currentTarget.style.backgroundColor = "transparent";
      }}
    >
      {config?.enabled ? <PowerOff size={15} /> : <RadioTower size={15} />}
    </button>
  );
}

function getAccountRealtimeStatusText(status: RealtimeStatus | undefined, t: TFunction) {
  if (!status) return null;

  if (status.message) {
    return status.message;
  }

  switch (status.mode) {
    case "realtime":
      return t("status.realtimeConnected", "Realtime connected");
    case "polling":
      return t("status.realtimePolling", "Polling");
    case "manual":
      return t("status.realtimeManual", "Manual only");
    case "backoff":
      return t("status.realtimeBackoff", "Retrying");
    case "auth_required":
      return t("status.realtimeAuthRequired", "Reconnect required");
    case "offline":
      return t("status.offline", "Offline");
    case "error":
      return t("status.realtimeError", "Realtime error");
  }
}

export function getGmailRealtimeStatusText(
  config: GmailRealtimeConfig | undefined | null,
  transientStatus: "enabling" | null,
  t: TFunction,
) {
  if (transientStatus === "enabling") {
    return t("settings.gmailRealtimeEnabling", "Enabling...");
  }
  if (!config) return null;

  switch (config.status) {
    case "not_enabled":
      return t("settings.gmailRealtimeNotEnabled", "Not enabled");
    case "enabling":
      return t("settings.gmailRealtimeEnabling", "Enabling...");
    case "realtime_enabled":
      return t("settings.gmailRealtimeEnabledStatus", "Realtime enabled");
    case "renewing":
      return t("settings.gmailRealtimeRenewing", "Renewing...");
    case "realtime_error":
      return t("settings.gmailRealtimeError", "Realtime error");
    case "reconnect_required":
      return t("settings.gmailRealtimeReconnectRequired", "Reconnect required");
    case "config_missing":
      return t("settings.gmailRealtimeConfigMissing", "Config missing");
  }
}

function getGmailRealtimeActionLabel(config: GmailRealtimeConfig | undefined, t: TFunction) {
  if (config?.enabled) {
    return t("settings.disableGmailRealtime", "Disable realtime Gmail");
  }
  if (config?.configMissing) {
    return t("settings.gmailRealtimeConfigMissing", "Config missing");
  }
  return t("settings.enableGmailRealtime", "Enable realtime Gmail");
}
