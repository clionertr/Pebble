import { useMemo } from "react";
import { useQueries, useQuery, useQueryClient } from "@tanstack/react-query";
import { fetchShellSnapshot } from "./useShellQuery";

export const foldersQueryKey = (accountId: string) => ["folders", accountId] as const;

export function useFoldersQuery(accountId: string | null) {
  const queryClient = useQueryClient();
  return useQuery({
    queryKey: foldersQueryKey(accountId ?? ""),
    queryFn: async () => (await fetchShellSnapshot(queryClient)).folders[accountId!] ?? [],
    enabled: !!accountId,
    select: (folders) => [...folders].sort((a, b) => a.sort_order - b.sort_order),
    staleTime: 60_000,
  });
}

export function useFoldersForAccountsQuery(accountIds: string[]) {
  const queryClient = useQueryClient();
  const queries = useQueries({
    queries: accountIds.map((accountId) => ({
      queryKey: foldersQueryKey(accountId),
      queryFn: async () => (await fetchShellSnapshot(queryClient)).folders[accountId] ?? [],
      enabled: !!accountId,
      staleTime: 60_000,
    })),
  });

  const data = useMemo(
    () =>
      queries
        .flatMap((query) => query.data ?? [])
        .sort((a, b) => a.sort_order - b.sort_order || a.name.localeCompare(b.name)),
    [queries],
  );

  return {
    data,
    isFetched: accountIds.length === 0 || queries.every((query) => query.isFetched),
    isLoading: queries.some((query) => query.isLoading),
    isError: queries.some((query) => query.isError),
  };
}
