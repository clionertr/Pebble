import { useQuery } from "@tanstack/react-query";
import { getPendingMailOpsSummary } from "@/lib/api";

export const pendingMailOpsSummaryQueryKey = (accountId: string | null) =>
  ["pendingMailOps", accountId] as const;

export function usePendingMailOpsSummary(accountId: string | null, enabled = true) {
  return useQuery({
    queryKey: pendingMailOpsSummaryQueryKey(accountId),
    queryFn: () => getPendingMailOpsSummary(accountId),
    enabled,
    refetchInterval: 15_000,
  });
}
