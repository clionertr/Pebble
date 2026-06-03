import { useMutation } from "@tanstack/react-query";
import { wakeSync } from "@/lib/api";

export function useSyncMutation() {
  return useMutation({
    mutationFn: (accountId: string) => wakeSync({ accountIds: [accountId], reason: "manual" }),
    // Data refresh is driven by mail:sync-complete and mail:new events in StatusBar
  });
}
