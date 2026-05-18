import { useEffect, useMemo, useRef } from "react";
import { useAccountsQuery } from "@/hooks/queries";
import { startSync, triggerSync } from "@/lib/api";
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
  const accountIds = useMemo(() => {
    const ids = accounts.map((account) => account.id).filter(Boolean);
    if (activeAccountId && !ids.includes(activeAccountId)) {
      ids.push(activeAccountId);
    }
    return ids;
  }, [accounts, activeAccountId]);

  useEffect(() => {
    if (accountIds.length === 0) return;

    const triggerAccount = (accountId: string, reason: string, ensureRunning: boolean) => {
      const trigger = () => {
        triggerSync(accountId, reason).catch(() => {});
      };

      if (ensureRunning && realtimeMode !== "manual") {
        startSync(accountId, pollInterval)
          .catch(() => {})
          .finally(trigger);
        return;
      }

      trigger();
    };

    const onFocus = () => {
      for (const accountId of accountIds) {
        triggerAccount(accountId, "window_focus", true);
      }
    };

    window.addEventListener("focus", onFocus);
    return () => {
      window.removeEventListener("focus", onFocus);
    };
  }, [accountIds, pollInterval, realtimeMode]);

  useEffect(() => {
    const previous = previousNetworkStatus.current;
    previousNetworkStatus.current = networkStatus;

    if (accountIds.length === 0 || previous !== "offline" || networkStatus !== "online") return;
    for (const accountId of accountIds) {
      if (realtimeMode === "manual") {
        triggerSync(accountId, "network_online").catch(() => {});
      } else {
        startSync(accountId, pollInterval)
          .catch(() => {})
          .finally(() => {
            triggerSync(accountId, "network_online").catch(() => {});
          });
      }
    }
  }, [accountIds, networkStatus, pollInterval, realtimeMode]);
}
