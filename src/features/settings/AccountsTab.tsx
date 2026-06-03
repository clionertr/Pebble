import { useEffect, useMemo, useState } from "react";
import { Plus, Mail } from "lucide-react";
import ConfirmDialog from "@/components/ConfirmDialog";
import { useTranslation } from "react-i18next";
import { useQueryClient } from "@tanstack/react-query";
import {
  deleteAccount,
  disableGmailRealtime,
  enableGmailRealtime,
  setRealtimePreference,
  testAccountConnection,
} from "@/lib/api";
import type { Account, GmailRealtimeConfig } from "@/lib/api";
import { useAccountsQuery, accountsQueryKey, shellQueryKey, useShellQuery } from "@/hooks/queries";
import { useMailStore } from "@/stores/mail.store";
import { useSyncStore } from "@/stores/sync.store";
import { useToastStore } from "@/stores/toast.store";
import AccountSetup from "@/components/AccountSetup";
import { extractErrorMessage } from "@/lib/extractErrorMessage";
import { assignAccountColors, getAccountColor } from "@/lib/accountColors";
import { AccountsList } from "./AccountsList";
import EditAccountModal from "./EditAccountModal";

export default function AccountsTab() {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const { data: accounts = [] } = useAccountsQuery();
  const { data: shell } = useShellQuery();
  const accountColorsById = useMemo(() => assignAccountColors(accounts), [accounts]);
  const realtimeStatusByAccount = useSyncStore((state) => state.realtimeStatusByAccount);
  const realtimeMode = useSyncStore((state) => state.realtimeMode);
  const [showSetup, setShowSetup] = useState(false);
  const [editingAccount, setEditingAccount] = useState<Account | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<{ id: string; email: string } | null>(null);
  const [testingId, setTestingId] = useState<string | null>(null);
  const [testResult, setTestResult] = useState<{ id: string; ok: boolean; message: string } | null>(
    null,
  );
  const [gmailRealtimeByAccount, setGmailRealtimeByAccount] = useState<
    Record<string, GmailRealtimeConfig>
  >({});
  const [gmailRealtimeActionId, setGmailRealtimeActionId] = useState<string | null>(null);
  const gmailAccountIds = useMemo(
    () => accounts.filter((account) => account.provider === "gmail").map((account) => account.id),
    [accounts],
  );
  const gmailAccountIdsKey = gmailAccountIds.join("|");

  useEffect(() => {
    const accountIds = gmailAccountIdsKey ? gmailAccountIdsKey.split("|") : [];
    if (accountIds.length === 0) {
      setGmailRealtimeByAccount({});
      return;
    }
    if (!shell) return;

    setGmailRealtimeByAccount(() => {
      const next: Record<string, GmailRealtimeConfig> = {};
      for (const accountId of accountIds) {
        const config = shell.gmailRealtime[accountId];
        if (config) next[accountId] = config;
      }
      return next;
    });
  }, [gmailAccountIdsKey, shell]);

  async function doTestConnection(accountId: string) {
    setTestingId(accountId);
    setTestResult(null);
    try {
      const report = await testAccountConnection(accountId);
      setTestResult({ id: accountId, ok: true, message: report });
    } catch (err) {
      const msg = extractErrorMessage(err);
      setTestResult({ id: accountId, ok: false, message: msg });
    } finally {
      setTestingId(null);
    }
  }

  async function doDelete(accountId: string) {
    try {
      await deleteAccount(accountId);
      if (useMailStore.getState().activeAccountId === accountId) {
        useMailStore.getState().setActiveAccountId(null);
      }
      await queryClient.invalidateQueries({ queryKey: shellQueryKey });
      await queryClient.invalidateQueries({ queryKey: accountsQueryKey });
      useToastStore.getState().addToast({
        message: t("settings.deleteAccountSuccess", "Account removed"),
        type: "success",
      });
    } catch (err) {
      const msg = extractErrorMessage(err);
      useToastStore.getState().addToast({
        message: t("settings.deleteAccountFailed", "Failed to remove account: {{error}}", {
          error: msg,
        }),
        type: "error",
      });
    }
  }

  async function doToggleGmailRealtime(account: Account) {
    const current = gmailRealtimeByAccount[account.id];
    setGmailRealtimeActionId(account.id);
    try {
      const next = current?.enabled
        ? await disableGmailRealtime(account.id)
        : await enableGmailRealtime(account.id, current?.fallbackIntervalMinutes ?? 15);
      setGmailRealtimeByAccount((prev) => ({ ...prev, [account.id]: next }));
      if (current?.enabled) {
        await setRealtimePreference(realtimeMode);
      }
      useToastStore.getState().addToast({
        message: current?.enabled
          ? t("settings.gmailRealtimeDisabled", "Gmail realtime disabled")
          : t("settings.gmailRealtimeEnabled", "Gmail realtime enabled"),
        type: "success",
      });
    } catch (err) {
      const msg = extractErrorMessage(err);
      useToastStore.getState().addToast({
        message: t(
          "settings.gmailRealtimeActionFailed",
          "Gmail realtime update failed: {{error}}",
          { error: msg },
        ),
        type: "error",
      });
    } finally {
      setGmailRealtimeActionId(null);
    }
  }

  return (
    <div>
      {/* Section header */}
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          marginBottom: "20px",
        }}
      >
        <h2
          style={{
            margin: 0,
            fontSize: "16px",
            fontWeight: 600,
            color: "var(--color-text-primary)",
          }}
        >
          {t("settings.emailAccounts")}
        </h2>
        <button
          onClick={() => setShowSetup(true)}
          style={{
            display: "flex",
            alignItems: "center",
            gap: "6px",
            padding: "7px 14px",
            borderRadius: "6px",
            border: "none",
            backgroundColor: "var(--color-accent)",
            color: "#fff",
            fontSize: "13px",
            fontWeight: 600,
            cursor: "pointer",
          }}
        >
          <Plus size={14} />
          {t("settings.addAccount")}
        </button>
      </div>

      {/* Empty state */}
      {accounts.length === 0 ? (
        <div
          style={{
            display: "flex",
            flexDirection: "column",
            alignItems: "center",
            gap: "12px",
            padding: "48px 0",
            color: "var(--color-text-secondary)",
          }}
        >
          <Mail size={40} strokeWidth={1.5} />
          <p style={{ margin: 0, fontSize: "14px" }}>{t("settings.noAccounts")}</p>
          <button
            onClick={() => setShowSetup(true)}
            style={{
              marginTop: "4px",
              padding: "8px 18px",
              borderRadius: "6px",
              border: "1px solid var(--color-border)",
              backgroundColor: "transparent",
              color: "var(--color-text-primary)",
              fontSize: "13px",
              cursor: "pointer",
            }}
          >
            {t("settings.addFirstAccount")}
          </button>
        </div>
      ) : (
        <AccountsList
          accounts={accounts}
          accountColorsById={accountColorsById}
          realtimeStatusByAccount={realtimeStatusByAccount}
          gmailRealtimeByAccount={gmailRealtimeByAccount}
          gmailRealtimeActionId={gmailRealtimeActionId}
          testingId={testingId}
          onToggleGmailRealtime={doToggleGmailRealtime}
          onTestConnection={doTestConnection}
          onEdit={setEditingAccount}
          onDelete={setDeleteTarget}
          t={t}
        />
      )}

      {/* Test result */}
      {testResult && (
        <div
          style={{
            marginTop: "10px",
            padding: "10px 12px",
            borderRadius: "6px",
            backgroundColor: testResult.ok ? "rgba(34,197,94,0.1)" : "rgba(239,68,68,0.1)",
            border: `1px solid ${testResult.ok ? "rgba(34,197,94,0.3)" : "rgba(239,68,68,0.3)"}`,
            color: testResult.ok ? "#22c55e" : "#ef4444",
            fontSize: "12px",
            whiteSpace: "pre-wrap",
            fontFamily: "monospace",
            lineHeight: 1.5,
          }}
        >
          {testResult.message}
        </div>
      )}

      {/* Delete confirmation */}
      {deleteTarget && (
        <ConfirmDialog
          title={t("settings.removeAccount", "Remove Account")}
          message={t("settings.confirmDeleteAccount", { email: deleteTarget.email })}
          destructive
          onCancel={() => setDeleteTarget(null)}
          onConfirm={() => {
            doDelete(deleteTarget.id);
            setDeleteTarget(null);
          }}
        />
      )}

      {/* AccountSetup modal */}
      {showSetup && <AccountSetup onClose={() => setShowSetup(false)} />}

      {/* Edit account modal */}
      {editingAccount && (
        <EditAccountModal
          account={editingAccount}
          initialColor={accountColorsById.get(editingAccount.id) ?? getAccountColor(editingAccount)}
          onClose={() => setEditingAccount(null)}
          onSaved={async () => {
            setEditingAccount(null);
            await queryClient.invalidateQueries({ queryKey: shellQueryKey });
            await queryClient.invalidateQueries({ queryKey: accountsQueryKey });
          }}
          initialGmailRealtimeConfig={gmailRealtimeByAccount[editingAccount.id]}
          onGmailRealtimeSaved={(config) => {
            setGmailRealtimeByAccount((prev) => ({ ...prev, [config.accountId]: config }));
          }}
        />
      )}
    </div>
  );
}
