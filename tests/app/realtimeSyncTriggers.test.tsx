import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useRealtimeSyncTriggers } from "../../src/app/useRealtimeSyncTriggers";
import { wakeSync } from "../../src/lib/api";

const mocks = vi.hoisted(() => ({
  activeAccountId: "account-1" as string | null,
  accounts: [{ id: "account-1" }, { id: "account-2" }],
  networkStatus: "online" as "online" | "offline",
  pollInterval: 3,
  realtimeMode: "realtime" as "realtime" | "balanced" | "battery" | "manual",
}));

vi.mock("../../src/lib/api", () => ({
  wakeSync: vi.fn(() => Promise.resolve({ failures: [] })),
}));

vi.mock("../../src/stores/mail.store", () => ({
  useMailStore: (selector: (s: { activeAccountId: string | null }) => unknown) =>
    selector({ activeAccountId: mocks.activeAccountId }),
}));

vi.mock("../../src/stores/sync.store", () => ({
  useSyncStore: (selector: (s: {
    networkStatus: "online" | "offline";
    pollInterval: number;
    realtimeMode: "realtime" | "balanced" | "battery" | "manual";
  }) => unknown) =>
    selector({
      networkStatus: mocks.networkStatus,
      pollInterval: mocks.pollInterval,
      realtimeMode: mocks.realtimeMode,
    }),
}));

vi.mock("../../src/hooks/queries", () => ({
  useAccountsQuery: () => ({ data: mocks.accounts }),
}));

function createWrapper(queryClient = new QueryClient()) {
  return ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
}

describe("useRealtimeSyncTriggers", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.activeAccountId = "account-1";
    mocks.accounts = [{ id: "account-1" }, { id: "account-2" }];
    mocks.networkStatus = "online";
    mocks.pollInterval = 3;
    mocks.realtimeMode = "realtime";
  });

  it("wakes every account with ensure-running when the window regains focus", async () => {
    renderHook(() => useRealtimeSyncTriggers(), { wrapper: createWrapper() });

    act(() => {
      window.dispatchEvent(new Event("focus"));
    });

    await waitFor(() => expect(wakeSync).toHaveBeenCalledWith({
      accountIds: ["account-1", "account-2"],
      reason: "window_focus",
      ensureRunning: true,
      pollIntervalSecs: 3,
    }));
  });

  it("notifies the backend once for every account when the window loses focus", () => {
    renderHook(() => useRealtimeSyncTriggers(), { wrapper: createWrapper() });

    act(() => {
      window.dispatchEvent(new Event("blur"));
    });

    expect(wakeSync).toHaveBeenCalledWith({
      accountIds: ["account-1", "account-2"],
      reason: "window_blur",
      ensureRunning: false,
      pollIntervalSecs: undefined,
    });
  });

  it("does not trigger network recovery sync on initial online mount", () => {
    renderHook(() => useRealtimeSyncTriggers(), { wrapper: createWrapper() });

    expect(wakeSync).not.toHaveBeenCalled();
  });

  it("wakes every account with ensure-running when the app transitions from offline to online", async () => {
    const queryClient = new QueryClient();
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");
    mocks.networkStatus = "offline";
    const { rerender } = renderHook(() => useRealtimeSyncTriggers(), {
      wrapper: createWrapper(queryClient),
    });

    mocks.networkStatus = "online";
    rerender();

    await waitFor(() => expect(wakeSync).toHaveBeenCalledWith({
      accountIds: ["account-1", "account-2"],
      reason: "network_online",
      ensureRunning: true,
      pollIntervalSecs: 3,
    }));
    await waitFor(() => {
      expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["shell"] });
      expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["messages"] });
      expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["threads"] });
    });
  });

  it("does not wake sync from passive focus events in manual mode", () => {
    mocks.realtimeMode = "manual";
    mocks.pollInterval = 0;
    renderHook(() => useRealtimeSyncTriggers(), { wrapper: createWrapper() });

    act(() => {
      window.dispatchEvent(new Event("focus"));
    });

    expect(wakeSync).not.toHaveBeenCalled();
  });
});
