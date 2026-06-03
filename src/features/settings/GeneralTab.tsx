import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  deleteNotificationDevice,
  listNotificationDevices,
  renameNotificationDevice,
  sendTestNotification,
  type NotificationDevice,
} from "@/lib/api";
import {
  disableCurrentDeviceNotifications,
  enableCurrentDeviceNotifications,
  getStoredWebPushDeviceId,
  supportsWebPush,
} from "@/lib/web-push";
import { useUIStore } from "@/stores/ui.store";
import { useSyncStore, type RealtimePreference } from "@/stores/sync.store";

const REALTIME_OPTIONS: Array<{
  mode: RealtimePreference;
  labelKey: string;
  fallback: string;
  descriptionKey: string;
  descriptionFallback: string;
}> = [
  {
    mode: "realtime",
    labelKey: "settings.realtimeModeRealtime",
    fallback: "Realtime (recommended)",
    descriptionKey: "settings.realtimeModeRealtimeDesc",
    descriptionFallback:
      "IMAP uses IDLE push when supported. Other providers check about every 3 seconds while you are active.",
  },
  {
    mode: "balanced",
    labelKey: "settings.realtimeModeBalanced",
    fallback: "Balanced",
    descriptionKey: "settings.realtimeModeBalancedDesc",
    descriptionFallback: "Checks about every 15 seconds while you are active.",
  },
  {
    mode: "battery",
    labelKey: "settings.realtimeModeBattery",
    fallback: "Battery saver",
    descriptionKey: "settings.realtimeModeBatteryDesc",
    descriptionFallback:
      "Checks about every 60 seconds while you are active and slows down in the background.",
  },
  {
    mode: "manual",
    labelKey: "settings.realtimeModeManual",
    fallback: "Manual only",
    descriptionKey: "settings.realtimeModeManualDesc",
    descriptionFallback: "Stops background checks. Use Sync now to run a single pass.",
  },
];

export default function GeneralTab() {
  const { t } = useTranslation();
  const realtimeMode = useSyncStore((s) => s.realtimeMode);
  const setRealtimeMode = useSyncStore((s) => s.setRealtimeMode);
  const notificationsEnabled = useSyncStore((s) => s.notificationsEnabled);
  const setNotificationsEnabled = useSyncStore((s) => s.setNotificationsEnabled);
  const [notificationMessage, setNotificationMessage] = useState<string | null>(null);
  const [notificationBusy, setNotificationBusy] = useState(false);
  const [devices, setDevices] = useState<NotificationDevice[]>([]);
  const [testingDeviceId, setTestingDeviceId] = useState<string | null>(null);
  const currentDeviceId = getStoredWebPushDeviceId();

  const refreshDevices = useCallback(() => {
    listNotificationDevices()
      .then(setDevices)
      .catch(() => {});
  }, []);

  useEffect(() => {
    refreshDevices();
  }, [refreshDevices, notificationsEnabled]);

  const toggleNotifications = useCallback(async () => {
    setNotificationMessage(null);
    setNotificationBusy(true);
    try {
      if (notificationsEnabled) {
        setNotificationsEnabled(false);
        await disableCurrentDeviceNotifications();
        setNotificationMessage(
          t("settings.notificationsDisabled", "Notifications are off on this device."),
        );
        refreshDevices();
        return;
      }

      if (realtimeMode === "manual") {
        setNotificationMessage(
          t(
            "settings.notificationsManualBlocked",
            "Notifications need background mail checks. Choose Realtime, Balanced, or Battery saver first.",
          ),
        );
        return;
      }

      if (!supportsWebPush()) {
        setNotificationMessage(
          t(
            "settings.notificationsUnsupported",
            "This browser does not support Web Push notifications.",
          ),
        );
        return;
      }

      await enableCurrentDeviceNotifications();
      setNotificationsEnabled(true);
      setNotificationMessage(
        t("settings.notificationsEnabled", "Notifications are on for this device."),
      );
      refreshDevices();
    } catch (error) {
      setNotificationsEnabled(false);
      setNotificationMessage(
        error instanceof Error
          ? error.message
          : t("settings.notificationsEnableFailed", "Failed to enable notifications."),
      );
    } finally {
      setNotificationBusy(false);
    }
  }, [notificationsEnabled, realtimeMode, refreshDevices, setNotificationsEnabled, t]);

  const renameDevice = useCallback(
    async (deviceId: string, deviceName: string) => {
      const trimmed = deviceName.trim();
      if (!trimmed) return;
      try {
        const updated = await renameNotificationDevice(deviceId, trimmed);
        setDevices((items) => items.map((item) => (item.id === deviceId ? updated : item)));
      } catch {
        setNotificationMessage(
          t("settings.notificationRenameFailed", "Failed to rename notification device."),
        );
      }
    },
    [t],
  );

  const removeDevice = useCallback(
    async (deviceId: string) => {
      try {
        if (deviceId === getStoredWebPushDeviceId()) {
          setNotificationsEnabled(false);
          await disableCurrentDeviceNotifications();
        } else {
          await deleteNotificationDevice(deviceId);
        }
        setDevices((items) => items.filter((item) => item.id !== deviceId));
      } catch {
        setNotificationMessage(
          t("settings.notificationRemoveFailed", "Failed to remove notification device."),
        );
      }
    },
    [setNotificationsEnabled, t],
  );

  const testDevice = useCallback(
    async (deviceId: string) => {
      setTestingDeviceId(deviceId);
      setNotificationMessage(null);
      try {
        await sendTestNotification(deviceId);
        setNotificationMessage(t("settings.notificationTestSent", "Test notification sent."));
      } catch {
        setNotificationMessage(
          t("settings.notificationTestFailed", "Failed to send test notification."),
        );
      } finally {
        setTestingDeviceId(null);
      }
    },
    [t],
  );

  const showUnreadCount = useUIStore((s) => s.showFolderUnreadCount);
  const setShowUnreadCount = useUIStore((s) => s.setShowFolderUnreadCount);

  const toggleUnreadCount = useCallback(() => {
    setShowUnreadCount(!showUnreadCount);
  }, [showUnreadCount, setShowUnreadCount]);

  return (
    <div>
      <h3 style={{ fontSize: "14px", fontWeight: 600, marginBottom: "8px" }}>
        {t("settings.realtimeMode", "Realtime Mode")}
      </h3>
      <p
        style={{
          fontSize: "12px",
          color: "var(--color-text-secondary)",
          marginBottom: "12px",
          marginTop: 0,
        }}
      >
        {t("settings.realtimeModeDesc", "Choose how aggressively Pebble checks for new mail.")}
      </p>
      <div
        role="group"
        aria-label={t("settings.realtimeMode", "Realtime Mode")}
        style={{ display: "flex", gap: "8px", flexWrap: "wrap" }}
      >
        {REALTIME_OPTIONS.map((option) => {
          const selected = realtimeMode === option.mode;
          const label = t(option.labelKey, option.fallback);
          return (
            <button
              key={option.mode}
              type="button"
              aria-label={label}
              aria-pressed={selected}
              onClick={() => setRealtimeMode(option.mode)}
              style={{
                flex: "1 1 180px",
                minWidth: 0,
                padding: "8px 10px",
                borderRadius: "6px",
                border: selected
                  ? "2px solid var(--color-accent)"
                  : "1px solid var(--color-border)",
                backgroundColor: selected ? "var(--color-bg-hover)" : "transparent",
                cursor: "pointer",
                textAlign: "left",
                color: "var(--color-text-primary)",
              }}
            >
              <span
                style={{
                  display: "block",
                  fontSize: "13px",
                  fontWeight: selected ? 600 : 500,
                  lineHeight: 1.3,
                }}
              >
                {label}
              </span>
              <span
                style={{
                  display: "block",
                  marginTop: "4px",
                  fontSize: "12px",
                  lineHeight: 1.35,
                  color: "var(--color-text-secondary)",
                }}
              >
                {t(option.descriptionKey, option.descriptionFallback)}
              </span>
            </button>
          );
        })}
      </div>

      <h3 style={{ fontSize: "14px", fontWeight: 600, marginBottom: "16px", marginTop: "32px" }}>
        {t("settings.notifications")}
      </h3>
      <label
        style={{
          display: "flex",
          alignItems: "center",
          gap: "8px",
          cursor: "pointer",
          fontSize: "13px",
          color: "var(--color-text-primary)",
        }}
      >
        <input
          type="checkbox"
          checked={notificationsEnabled}
          disabled={notificationBusy}
          onChange={() => {
            void toggleNotifications();
          }}
        />
        <span>{t("settings.enableNotifications")}</span>
      </label>
      <p
        style={{
          fontSize: "12px",
          color: "var(--color-text-secondary)",
          marginTop: "8px",
          marginBottom: 0,
        }}
      >
        {t(
          "settings.notificationsDesc",
          "This uses encrypted browser Web Push. It works only on this device and requires HTTPS outside localhost.",
        )}
      </p>
      {notificationMessage && (
        <p
          style={{
            fontSize: "12px",
            color: "var(--color-text-secondary)",
            marginTop: "8px",
            marginBottom: 0,
          }}
        >
          {notificationMessage}
        </p>
      )}

      {devices.length > 0 && (
        <div style={{ marginTop: "16px", display: "flex", flexDirection: "column", gap: "8px" }}>
          <div style={{ fontSize: "12px", fontWeight: 600, color: "var(--color-text-primary)" }}>
            {t("settings.notificationDevices", "Notification devices")}
          </div>
          {devices.map((device) => {
            const isCurrent = currentDeviceId === device.id;
            const paused = device.status === "paused";
            return (
              <div
                key={device.id}
                style={{
                  display: "grid",
                  gridTemplateColumns: "minmax(160px, 1fr) auto auto",
                  gap: "8px",
                  alignItems: "center",
                  padding: "8px",
                  border: "1px solid var(--color-border)",
                  borderRadius: "6px",
                }}
              >
                <div>
                  <input
                    aria-label={t("settings.notificationDeviceName", "Device name")}
                    defaultValue={device.device_name}
                    onBlur={(event) => {
                      void renameDevice(device.id, event.currentTarget.value);
                    }}
                    style={{
                      width: "100%",
                      border: "1px solid var(--color-border)",
                      borderRadius: "4px",
                      padding: "4px 6px",
                      background: "var(--color-bg-primary)",
                      color: "var(--color-text-primary)",
                      fontSize: "12px",
                    }}
                  />
                  <div
                    style={{
                      marginTop: "4px",
                      fontSize: "11px",
                      color: "var(--color-text-secondary)",
                    }}
                  >
                    {paused
                      ? t(
                          "settings.notificationDevicePaused",
                          "Paused · sign in again on this device to resume",
                        )
                      : t("settings.notificationDeviceActive", "Active")}
                    {isCurrent
                      ? ` · ${t("settings.notificationDeviceCurrent", "current device")}`
                      : ""}
                  </div>
                </div>
                <button
                  type="button"
                  disabled={testingDeviceId === device.id || paused}
                  onClick={() => {
                    void testDevice(device.id);
                  }}
                  style={{
                    padding: "5px 10px",
                    fontSize: "12px",
                    cursor: paused ? "not-allowed" : "pointer",
                  }}
                >
                  {testingDeviceId === device.id
                    ? t("common.testing", "Testing...")
                    : t("settings.notificationTest", "Test")}
                </button>
                <button
                  type="button"
                  onClick={() => {
                    void removeDevice(device.id);
                  }}
                  style={{ padding: "5px 10px", fontSize: "12px", cursor: "pointer" }}
                >
                  {t("common.remove", "Remove")}
                </button>
              </div>
            );
          })}
        </div>
      )}

      <h3 style={{ fontSize: "14px", fontWeight: 600, marginBottom: "16px", marginTop: "32px" }}>
        {t("settings.folderCounts", "Folder Counts")}
      </h3>
      <label
        style={{
          display: "flex",
          alignItems: "center",
          gap: "8px",
          cursor: "pointer",
          fontSize: "13px",
          color: "var(--color-text-primary)",
        }}
      >
        <input type="checkbox" checked={showUnreadCount} onChange={toggleUnreadCount} />
        <span>{t("settings.showUnreadCount", "Show unread count badges in sidebar")}</span>
      </label>
    </div>
  );
}
