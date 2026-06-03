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
      } catch {
        // 偏好同步失败不阻塞前端状态，后续请求会重新覆盖服务端值。
      }

      if (cancelled) return;

      setRealtimePreference(realtimeMode).catch(() => {
        // 同上：实时模式偏好是尽力同步。
      });
    }

    syncRealtimePreference();

    return () => {
      cancelled = true;
    };
  }, [realtimeMode, notificationsEnabled]);
}
