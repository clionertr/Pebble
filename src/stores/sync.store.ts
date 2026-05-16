import { create } from "zustand";
import { deferPersist } from "@/lib/deferPersist";

export type NetworkStatus = "online" | "offline";
export type RealtimeMode = "realtime" | "polling" | "manual" | "backoff" | "offline" | "auth_required" | "error";
export type RealtimePreference = "realtime" | "balanced" | "battery" | "manual";

export interface RealtimeStatus {
  account_id: string;
  mode: RealtimeMode;
  provider: string;
  last_success_at?: number | null;
  next_retry_at?: number | null;
  message?: string | null;
}

const REALTIME_PREFERENCE_KEY = "pebble-realtime-mode";
const REALTIME_PREFERENCES = new Set<RealtimePreference>(["realtime", "balanced", "battery", "manual"]);
const NOTIFICATIONS_KEY = "pebble-notifications-enabled";
const KEEP_RUNNING_BACKGROUND_KEY = "pebble-keep-running-background";

function readRealtimePreference(): RealtimePreference {
  const stored = localStorage.getItem(REALTIME_PREFERENCE_KEY);
  return REALTIME_PREFERENCES.has(stored as RealtimePreference)
    ? (stored as RealtimePreference)
    : "realtime";
}

export function readNotificationsEnabledPreference(): boolean {
  const stored = localStorage.getItem(NOTIFICATIONS_KEY);
  return stored === null ? true : stored === "true";
}

export function readKeepRunningInBackgroundPreference(): boolean {
  const stored = localStorage.getItem(KEEP_RUNNING_BACKGROUND_KEY);
  return stored === null ? true : stored === "true";
}

export function realtimePreferenceToPollInterval(mode: RealtimePreference): number {
  switch (mode) {
    case "realtime": return 3;
    case "balanced": return 15;
    case "battery": return 60;
    case "manual": return 0;
  }
}

const initialRealtimeMode = readRealtimePreference();

interface SyncState {
  syncStatus: "idle" | "syncing" | "error";
  networkStatus: NetworkStatus;
  lastMailError: string | null;
  realtimeStatusByAccount: Record<string, RealtimeStatus>;
  realtimeMode: RealtimePreference;
  pollInterval: number;
  notificationsEnabled: boolean;
  keepRunningInBackground: boolean;
  setSyncStatus: (status: "idle" | "syncing" | "error") => void;
  setNetworkStatus: (status: NetworkStatus) => void;
  setLastMailError: (error: string | null) => void;
  setRealtimeStatus: (accountId: string, status: RealtimeStatus) => void;
  setRealtimeMode: (mode: RealtimePreference) => void;
  setPollInterval: (secs: number) => void;
  setNotificationsEnabled: (enabled: boolean) => void;
  setKeepRunningInBackground: (enabled: boolean) => void;
}

export const useSyncStore = create<SyncState>((set) => ({
  syncStatus: "idle",
  networkStatus: "online",
  lastMailError: null,
  realtimeStatusByAccount: {},
  realtimeMode: initialRealtimeMode,
  pollInterval: realtimePreferenceToPollInterval(initialRealtimeMode),
  notificationsEnabled: readNotificationsEnabledPreference(),
  keepRunningInBackground: readKeepRunningInBackgroundPreference(),
  setSyncStatus: (status) => set({ syncStatus: status }),
  setNetworkStatus: (status) => set({ networkStatus: status }),
  setLastMailError: (error) => set({ lastMailError: error }),
  setRealtimeStatus: (accountId, status) =>
    set((state) => ({
      realtimeStatusByAccount: { ...state.realtimeStatusByAccount, [accountId]: status },
    })),
  setRealtimeMode: (mode) => {
    const pollInterval = realtimePreferenceToPollInterval(mode);
    deferPersist(() => localStorage.setItem(REALTIME_PREFERENCE_KEY, mode));
    deferPersist(() => localStorage.setItem("pebble-poll-interval", String(pollInterval)));
    set({ realtimeMode: mode, pollInterval });
  },
  setPollInterval: (secs) => {
    deferPersist(() => localStorage.setItem("pebble-poll-interval", String(secs)));
    set({ pollInterval: secs });
  },
  setNotificationsEnabled: (enabled) => {
    deferPersist(() => localStorage.setItem(NOTIFICATIONS_KEY, String(enabled)));
    set({ notificationsEnabled: enabled });
  },
  setKeepRunningInBackground: (enabled) => {
    deferPersist(() => localStorage.setItem(KEEP_RUNNING_BACKGROUND_KEY, String(enabled)));
    set({ keepRunningInBackground: enabled });
  },
}));
