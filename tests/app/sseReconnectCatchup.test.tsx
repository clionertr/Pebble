import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { wakeSync } from "../../src/lib/api";

const mocks = vi.hoisted(() => ({
  reconnectHandler: null as null | (() => void),
  pollInterval: 15,
  realtimeMode: "balanced" as "realtime" | "balanced" | "battery" | "manual",
}));

vi.mock("../../src/lib/sse-client", () => ({
  onSseReconnect: vi.fn((handler: () => void) => {
    mocks.reconnectHandler = handler;
    return vi.fn();
  }),
}));

vi.mock("../../src/lib/api", () => ({
  wakeSync: vi.fn(() => Promise.resolve({ failures: [] })),
}));

vi.mock("../../src/stores/sync.store", () => ({
  useSyncStore: (selector: (state: {
    pollInterval: number;
    realtimeMode: "realtime" | "balanced" | "battery" | "manual";
  }) => unknown) =>
    selector({
      pollInterval: mocks.pollInterval,
      realtimeMode: mocks.realtimeMode,
    }),
}));

import { useSseReconnectCatchup } from "../../src/app/useSseReconnectCatchup";

function createWrapper(queryClient: QueryClient) {
  return ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
}

describe("useSseReconnectCatchup", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.reconnectHandler = null;
    mocks.pollInterval = 15;
    mocks.realtimeMode = "balanced";
  });

  it("does not catch up on initial subscription, but wakes and invalidates after SSE reconnect", async () => {
    const queryClient = new QueryClient();
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

    renderHook(() => useSseReconnectCatchup(), { wrapper: createWrapper(queryClient) });

    expect(wakeSync).not.toHaveBeenCalled();
    expect(invalidateSpy).not.toHaveBeenCalled();
    expect(mocks.reconnectHandler).toBeTruthy();

    act(() => {
      mocks.reconnectHandler?.();
    });

    await waitFor(() =>
      expect(wakeSync).toHaveBeenCalledWith({
        reason: "network_online",
        ensureRunning: true,
        pollIntervalSecs: 15,
      }),
    );
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["shell"] });
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["messages"] });
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["threads"] });
  });

  it("invalidates cached mail data but does not wake sync in manual mode", () => {
    mocks.realtimeMode = "manual";
    mocks.pollInterval = 0;
    const queryClient = new QueryClient();
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

    renderHook(() => useSseReconnectCatchup(), { wrapper: createWrapper(queryClient) });

    act(() => {
      mocks.reconnectHandler?.();
    });

    expect(wakeSync).not.toHaveBeenCalled();
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["shell"] });
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["messages"] });
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["threads"] });
  });
});
