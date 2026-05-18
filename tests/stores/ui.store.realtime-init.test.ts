import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("react-i18next", () => ({
  initReactI18next: {
    type: "3rdParty",
    init: vi.fn(),
  },
}));

describe("SyncStore realtime initialization", () => {
  beforeEach(() => {
    vi.resetModules();
    localStorage.clear();
  });

  it("preserves manual mode as a zero poll interval on startup", async () => {
    localStorage.setItem("pebble-realtime-mode", "manual");
    localStorage.setItem("pebble-poll-interval", "0");

    const { useSyncStore } = await import("../../src/stores/sync.store");

    expect(useSyncStore.getState().realtimeMode).toBe("manual");
    expect(useSyncStore.getState().pollInterval).toBe(0);
  });
});
