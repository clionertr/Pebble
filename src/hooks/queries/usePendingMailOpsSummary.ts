import { useQuery } from "@tanstack/react-query";
import { getPendingMailOpsSummary } from "@/lib/api";
import { useSyncStore } from "@/stores/sync.store";

export const pendingMailOpsSummaryQueryKey = (accountId: string | null) =>
  ["pendingMailOps", accountId] as const;

export function usePendingMailOpsSummary(accountId: string | null, enabled = true) {
  const sseActive = useSyncStore((s) => {
    if (!accountId) return false;
    return s.realtimeStatusByAccount[accountId]?.mode === "realtime";
  });
  return useQuery({
    queryKey: pendingMailOpsSummaryQueryKey(accountId),
    queryFn: () => getPendingMailOpsSummary(accountId),
    enabled,
    // SSE 活跃时信任推送；否则 30s 轮询作为后备（对齐 staleTime）
    refetchInterval: sseActive ? false : 30_000,
  });
}
