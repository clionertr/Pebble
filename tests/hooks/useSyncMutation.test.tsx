import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook } from "@testing-library/react";
import type { ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { startSync, triggerSync, wakeSync } from "../../src/lib/api";
import { useSyncMutation } from "../../src/hooks/mutations/useSyncMutation";

vi.mock("../../src/lib/api", () => ({
  startSync: vi.fn(() => Promise.resolve("started")),
  triggerSync: vi.fn(() => Promise.resolve()),
  wakeSync: vi.fn(() => Promise.resolve({ failures: [] })),
}));

vi.mock("../../src/stores/ui.store", () => ({
  realtimePreferenceToPollInterval: () => 3,
  useUIStore: (selector: (s: { realtimeMode: "realtime" }) => unknown) =>
    selector({ realtimeMode: "realtime" }),
}));

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: {
      mutations: { retry: false },
      queries: { retry: false },
    },
  });

  return ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
}

describe("useSyncMutation", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("requests a one-shot manual sync instead of starting a polling worker", async () => {
    const { result } = renderHook(() => useSyncMutation(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync("account-1");
    });

    expect(wakeSync).toHaveBeenCalledWith({
      accountIds: ["account-1"],
      reason: "manual",
    });
    expect(triggerSync).not.toHaveBeenCalled();
    expect(startSync).not.toHaveBeenCalled();
  });
});
