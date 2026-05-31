import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

class FakeEventSource {
  static readonly CLOSED = 2;
  static instances: FakeEventSource[] = [];

  readyState = 0;
  onopen: (() => void) | null = null;
  onerror: (() => void) | null = null;
  listeners = new Map<string, (event: MessageEvent) => void>();

  constructor(public readonly url: string) {
    FakeEventSource.instances.push(this);
  }

  addEventListener(event: string, handler: (event: MessageEvent) => void) {
    this.listeners.set(event, handler);
  }

  close() {
    this.readyState = FakeEventSource.CLOSED;
  }
}

describe("sse-client reconnect callbacks", () => {
  beforeEach(() => {
    vi.resetModules();
    vi.useFakeTimers();
    vi.spyOn(Math, "random").mockReturnValue(0);
    FakeEventSource.instances = [];
    vi.stubGlobal("EventSource", FakeEventSource);
  });

  afterEach(() => {
    vi.useRealTimers();
    vi.unstubAllGlobals();
    vi.restoreAllMocks();
  });

  it("notifies reconnect listeners only after a previously opened connection reconnects", async () => {
    const { listen, onSseReconnect } = await import("../../src/lib/sse-client");
    const reconnect = vi.fn();

    onSseReconnect(reconnect);
    await listen("mail:new", vi.fn());

    FakeEventSource.instances[0].onopen?.();
    expect(reconnect).not.toHaveBeenCalled();

    FakeEventSource.instances[0].readyState = FakeEventSource.CLOSED;
    FakeEventSource.instances[0].onerror?.();
    await vi.advanceTimersByTimeAsync(2000);

    expect(FakeEventSource.instances).toHaveLength(2);
    FakeEventSource.instances[1].onopen?.();

    expect(reconnect).toHaveBeenCalledTimes(1);
  });
});
