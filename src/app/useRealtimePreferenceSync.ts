import { useEffect } from "react";
import { setNotificationsEnabled, setRealtimePreference } from "@/lib/api";
import { useSyncStore } from "@/stores/sync.store";

export function useRealtimePreferenceSync() {
  const realtimeMode = useSyncStore((state) => state.realtimeMode);
  const notificationsEnabled = useSyncStore((state) => state.notificationsEnabled);

  useEffect(() => {
    let cancelled = false;

    async function syncRealtimePreference() {
      try {
        await setNotificationsEnabled(notificationsEnabled);
      } catch {}

      if (cancelled) return;

      setRealtimePreference(realtimeMode).catch(() => {});
    }

    syncRealtimePreference();

    return () => {
      cancelled = true;
    };
  }, [realtimeMode, notificationsEnabled]);
}
