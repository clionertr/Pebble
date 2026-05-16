import { useMemo } from "react";
import { useQueries, useQuery } from "@tanstack/react-query";
import { getFolderUnreadCounts } from "@/lib/api";
import { useUIStore } from "@/stores/ui.store";
import { useDelayedIdleReady } from "@/hooks/useDelayedIdleReady";

export function useFolderUnreadCounts(accountId: string | null) {
  const enabled = useUIStore((s) => s.showFolderUnreadCount);
  const ready = useDelayedIdleReady(3000);
  return useQuery({
    queryKey: ["folder-unread-counts", accountId],
    queryFn: () => getFolderUnreadCounts(accountId!),
    enabled: ready && enabled && !!accountId,
    staleTime: 30_000,
    refetchInterval: 30_000,
  });
}

export function useFolderUnreadCountsForAccounts(accountIds: string[]) {
  const enabled = useUIStore((s) => s.showFolderUnreadCount);
  const ready = useDelayedIdleReady(3000);
  const queries = useQueries({
    queries: accountIds.map((accountId) => ({
      queryKey: ["folder-unread-counts", accountId],
      queryFn: () => getFolderUnreadCounts(accountId),
      enabled: ready && enabled && !!accountId,
      staleTime: 30_000,
      refetchInterval: 30_000,
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
