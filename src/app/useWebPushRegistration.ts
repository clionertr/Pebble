import { useEffect } from "react";
import { disableCurrentDeviceNotifications, restoreCurrentDeviceNotifications } from "@/lib/web-push";
import { useSyncStore } from "@/stores/sync.store";

export function useWebPushRegistration() {
  const notificationsEnabled = useSyncStore((state) => state.notificationsEnabled);
  const realtimeMode = useSyncStore((state) => state.realtimeMode);
  const setNotificationsEnabled = useSyncStore((state) => state.setNotificationsEnabled);

  useEffect(() => {
    if (!notificationsEnabled) return;
    if (realtimeMode === "manual") {
      setNotificationsEnabled(false);
      disableCurrentDeviceNotifications().catch(() => {});
      return;
    }
    if (!("Notification" in window)) {
      setNotificationsEnabled(false);
      return;
    }
    if (Notification.permission === "denied") {
      setNotificationsEnabled(false);
      return;
    }
    if (Notification.permission !== "granted") return;

    let cancelled = false;
    restoreCurrentDeviceNotifications()
      .then((device) => {
        if (!cancelled && !device) setNotificationsEnabled(false);
      })
      .catch(() => {
        if (!cancelled) setNotificationsEnabled(false);
      });
    return () => {
      cancelled = true;
    };
  }, [notificationsEnabled, realtimeMode, setNotificationsEnabled]);
}
