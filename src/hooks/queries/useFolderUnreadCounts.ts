import { useMemo } from "react";
import { useQueries, useQuery, useQueryClient } from "@tanstack/react-query";
import { useUIStore } from "@/stores/ui.store";
import { useSyncStore } from "@/stores/sync.store";
import { useDelayedIdleReady } from "@/hooks/useDelayedIdleReady";
import { fetchShellSnapshot } from "./useShellQuery";

function useIsSseActive(accountId: string | null) {
  const status = useSyncStore((s) =>
    accountId ? s.realtimeStatusByAccount[accountId] : undefined
  );
  return status?.mode === "realtime";
}

export function useFolderUnreadCounts(accountId: string | null) {
  const enabled = useUIStore((s) => s.showFolderUnreadCount);
  const ready = useDelayedIdleReady(3000);
  const sseActive = useIsSseActive(accountId);
  const queryClient = useQueryClient();
  return useQuery({
    queryKey: ["folder-unread-counts", accountId],
    queryFn: async () => (await fetchShellSnapshot(queryClient)).unreadCounts[accountId!] ?? {},
    enabled: ready && enabled && !!accountId,
    staleTime: 30_000,
    // SSE 活跃时信任推送，不轮询；否则 30s 轮询作为后备
    refetchInterval: sseActive ? false : 30_000,
  });
}

export function useFolderUnreadCountsForAccounts(accountIds: string[]) {
  const enabled = useUIStore((s) => s.showFolderUnreadCount);
  const ready = useDelayedIdleReady(3000);
  const queryClient = useQueryClient();
  // 检查是否有任何一个账户的 SSE 不活跃（需要轮询后备）
  const sseAllActive = useSyncStore((s) =>
    accountIds.length > 0 &&
    accountIds.every((id) => s.realtimeStatusByAccount[id]?.mode === "realtime")
  );
  const queries = useQueries({
    queries: accountIds.map((accountId) => ({
      queryKey: ["folder-unread-counts", accountId],
      queryFn: async () => (await fetchShellSnapshot(queryClient)).unreadCounts[accountId] ?? {},
      enabled: ready && enabled && !!accountId,
      staleTime: 30_000,
      refetchInterval: sseAllActive ? false : 30_000,
    })),
  });

  const data = useMemo(
    () => Object.assign({}, ...queries.map((query) => query.data ?? {})),
    [queries],
  );

  return {
    data,
    isLoading: queries.some((query) => query.isLoading),
  };
}
