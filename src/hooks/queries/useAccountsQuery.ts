import { useQuery, useQueryClient } from "@tanstack/react-query";
import { fetchShellSnapshot } from "./useShellQuery";

export const accountsQueryKey = ["accounts"] as const;

export function useAccountsQuery() {
  const queryClient = useQueryClient();
  return useQuery({
    queryKey: accountsQueryKey,
    queryFn: async () => (await fetchShellSnapshot(queryClient)).accounts,
    staleTime: 60_000,
  });
}
