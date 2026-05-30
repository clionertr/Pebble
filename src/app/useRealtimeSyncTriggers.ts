import { useEffect, useMemo, useRef } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { useAccountsQuery } from "@/hooks/queries";
import { shellQueryKey } from "@/hooks/queries/useShellQuery";
import { wakeSync } from "@/lib/api";
import { useMailStore } from "@/stores/mail.store";
import { useSyncStore } from "@/stores/sync.store";

interface SyncAccount {
  id: string;
}

const EMPTY_ACCOUNTS: SyncAccount[] = [];

export function useRealtimeSyncTriggers() {
  const activeAccountId = useMailStore((s) => s.activeAccountId);
  const networkStatus = useSyncStore((s) => s.networkStatus);
  const pollInterval = useSyncStore((s) => s.pollInterval);
  const realtimeMode = useSyncStore((s) => s.realtimeMode);
  const { data: accounts = EMPTY_ACCOUNTS } = useAccountsQuery();
  const previousNetworkStatus = useRef(networkStatus);
  const queryClient = useQueryClient();
  const accountIds = useMemo(() => {
    const ids = accounts.map((account) => account.id).filter(Boolean);
    if (activeAccountId && !ids.includes(activeAccountId)) {
      ids.push(activeAccountId);
    }
    return ids;
  }, [accounts, activeAccountId]);

  useEffect(() => {
    if (accountIds.length === 0) return;
    if (realtimeMode === "manual") return;

    const wakeAccounts = (reason: string, ensureRunning: boolean) => {
      wakeSync({
        accountIds,
        reason,
        ensureRunning,
        pollIntervalSecs: ensureRunning ? pollInterval : undefined,
      }).catch(() => {});
    };

    const onFocus = () => {
      wakeAccounts("window_focus", true);
    };
    const onBlur = () => {
      wakeAccounts("window_blur", false);
    };

    window.addEventListener("focus", onFocus);
    window.addEventListener("blur", onBlur);
    return () => {
      window.removeEventListener("focus", onFocus);
      window.removeEventListener("blur", onBlur);
    };
  }, [accountIds, pollInterval, realtimeMode]);

  useEffect(() => {
    const previous = previousNetworkStatus.current;
    previousNetworkStatus.current = networkStatus;

    if (
      accountIds.length === 0
      || previous !== "offline"
      || networkStatus !== "online"
      || realtimeMode === "manual"
    ) return;

    wakeSync({
      accountIds,
      reason: "network_online",
      ensureRunning: true,
      pollIntervalSecs: pollInterval,
    }).finally(() => {
      queryClient.invalidateQueries({ queryKey: shellQueryKey });
      queryClient.invalidateQueries({ queryKey: ["messages"] });
      queryClient.invalidateQueries({ queryKey: ["threads"] });
    }).catch(() => {});
  }, [accountIds, networkStatus, pollInterval, queryClient, realtimeMode]);
}
