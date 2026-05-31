import { useEffect } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { shellQueryKey } from "@/hooks/queries/useShellQuery";
import { wakeSync } from "@/lib/api";
import { onSseReconnect } from "@/lib/sse-client";
import { useSyncStore } from "@/stores/sync.store";

export function useSseReconnectCatchup() {
  const queryClient = useQueryClient();
  const pollInterval = useSyncStore((s) => s.pollInterval);
  const realtimeMode = useSyncStore((s) => s.realtimeMode);

  useEffect(() => {
    return onSseReconnect(() => {
      queryClient.invalidateQueries({ queryKey: shellQueryKey });
      queryClient.invalidateQueries({ queryKey: ["messages"] });
      queryClient.invalidateQueries({ queryKey: ["threads"] });

      if (realtimeMode === "manual") return;

      wakeSync({
        reason: "network_online",
        ensureRunning: true,
        pollIntervalSecs: pollInterval,
      }).catch(() => {});
    });
  }, [pollInterval, queryClient, realtimeMode]);
}
