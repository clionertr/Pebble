import { useQuery, type QueryClient } from "@tanstack/react-query";
import { getShell } from "@/lib/api";
import type { ShellData } from "@/lib/api-client";

export const shellQueryKey = ["shell"] as const;

export const gmailRealtimeQueryKey = (accountId: string) => ["gmail-realtime", accountId] as const;

export function hydrateShellQueryData(queryClient: QueryClient, shell: ShellData) {
  queryClient.setQueryData(["accounts"], shell.accounts);

  for (const [accountId, folders] of Object.entries(shell.folders)) {
    queryClient.setQueryData(["folders", accountId], folders);
  }

  for (const [accountId, counts] of Object.entries(shell.unreadCounts)) {
    queryClient.setQueryData(["folder-unread-counts", accountId], counts);
  }

  for (const [accountId, config] of Object.entries(shell.gmailRealtime)) {
    queryClient.setQueryData(gmailRealtimeQueryKey(accountId), config);
  }
}

export async function fetchShellSnapshot(queryClient: QueryClient): Promise<ShellData> {
  const shell = await queryClient.fetchQuery({
    queryKey: shellQueryKey,
    queryFn: getShell,
    staleTime: 60_000,
  });
  hydrateShellQueryData(queryClient, shell);
  return shell;
}

export function useShellQuery() {
  return useQuery({
    queryKey: shellQueryKey,
    queryFn: getShell,
    staleTime: 60_000,
  });
}
