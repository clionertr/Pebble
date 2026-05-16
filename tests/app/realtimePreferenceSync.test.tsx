import { renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { setNotificationsEnabled, setRealtimePreference } from "../../src/lib/api";
import { useRealtimePreferenceSync } from "../../src/app/useRealtimePreferenceSync";

const mocks = vi.hoisted(() => ({
  realtimeMode: "battery" as "realtime" | "balanced" | "battery" | "manual",
  notificationsEnabled: false,
}));

vi.mock("../../src/lib/api", () => ({
  setNotificationsEnabled: vi.fn(() => Promise.resolve()),
  setRealtimePreference: vi.fn(() => Promise.resolve()),
}));

vi.mock("../../src/stores/sync.store", () => ({
  useSyncStore: (selector: (state: {
    realtimeMode: typeof mocks.realtimeMode;
    notificationsEnabled: boolean;
  }) => unknown) =>
    selector({
      realtimeMode: mocks.realtimeMode,
      notificationsEnabled: mocks.notificationsEnabled,
    }),
}));

describe("useRealtimePreferenceSync", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.realtimeMode = "battery";
    mocks.notificationsEnabled = false;
  });

  it("syncs notification gate before applying realtime preference", async () => {
    renderHook(() => useRealtimePreferenceSync());

    await waitFor(() => expect(setRealtimePreference).toHaveBeenCalledWith("battery"));
    expect(setNotificationsEnabled).toHaveBeenCalledWith(false);
    expect(vi.mocked(setNotificationsEnabled).mock.invocationCallOrder[0])
      .toBeLessThan(vi.mocked(setRealtimePreference).mock.invocationCallOrder[0]);
  });
});
