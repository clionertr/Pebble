import { useQuery } from "@tanstack/react-query";
import { listPendingMailOps } from "@/lib/api";
import { useSyncStore } from "@/stores/sync.store";

export const pendingMailOpsQueryKey = (accountId: string | null) =>
  ["pendingMailOpsList", accountId] as const;

export function usePendingMailOpsQuery(accountId: string | null, limit = 100) {
  const sseActive = useSyncStore((s) => {
    if (!accountId) return false;
    return s.realtimeStatusByAccount[accountId]?.mode === "realtime";
  });
  return useQuery({
    queryKey: pendingMailOpsQueryKey(accountId),
    queryFn: () => listPendingMailOps(accountId, limit),
    // SSE 活跃时信任推送；否则 30s 轮询作为后备（对齐 staleTime）
    refetchInterval: sseActive ? false : 30_000,
  });
}
