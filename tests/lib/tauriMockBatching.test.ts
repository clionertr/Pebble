import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

function okResponse(value: unknown) {
  return {
    ok: true,
    json: vi.fn().mockResolvedValue(value),
  } as unknown as Response;
}

describe("JSON-RPC batching", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.resetModules();
    vi.stubGlobal("fetch", vi.fn().mockResolvedValue(okResponse(["ok"])));
  });

  afterEach(() => {
    vi.useRealTimers();
    vi.unstubAllGlobals();
  });

  it("sends startup-critical reads without waiting for the batch window", async () => {
    const { invoke } = await import("../../src/tauri-mock");

    const promise = invoke("list_messages", { folderId: "inbox", limit: 50, offset: 0 });

    expect(fetch).not.toHaveBeenCalled();

    await vi.advanceTimersByTimeAsync(0);

    expect(fetch).toHaveBeenCalledOnce();
    expect(JSON.parse(String(vi.mocked(fetch).mock.calls[0][1]?.body))).toEqual([
      {
        method: "list_messages",
        params: { folderId: "inbox", limit: 50, offset: 0 },
      },
    ]);
    await expect(promise).resolves.toBe("ok");
  });

  it("coalesces startup-critical reads that happen in the same event-loop turn", async () => {
    vi.stubGlobal("fetch", vi.fn().mockResolvedValue(okResponse(["accounts", "messages"])));
    const { invoke } = await import("../../src/tauri-mock");

    const accountsPromise = invoke("list_accounts");
    const messagesPromise = invoke("list_messages", { folderId: "inbox", limit: 50, offset: 0 });

    await vi.advanceTimersByTimeAsync(0);

    expect(fetch).toHaveBeenCalledOnce();
    expect(JSON.parse(String(vi.mocked(fetch).mock.calls[0][1]?.body))).toEqual([
      {
        method: "list_accounts",
        params: {},
      },
      {
        method: "list_messages",
        params: { folderId: "inbox", limit: 50, offset: 0 },
      },
    ]);
    await expect(accountsPromise).resolves.toBe("accounts");
    await expect(messagesPromise).resolves.toBe("messages");
  });

  it("promotes an existing queued batch when a startup-critical read arrives", async () => {
    vi.stubGlobal("fetch", vi.fn().mockResolvedValue(okResponse(["log", "messages"])));
    const { invoke } = await import("../../src/tauri-mock");

    const logPromise = invoke("read_app_log", { maxBytes: 4096 });
    const messagesPromise = invoke("list_messages", { folderId: "inbox", limit: 50, offset: 0 });

    await vi.advanceTimersByTimeAsync(0);

    expect(fetch).toHaveBeenCalledOnce();
    expect(JSON.parse(String(vi.mocked(fetch).mock.calls[0][1]?.body))).toEqual([
      {
        method: "read_app_log",
        params: { maxBytes: 4096 },
      },
      {
        method: "list_messages",
        params: { folderId: "inbox", limit: 50, offset: 0 },
      },
    ]);
    await expect(logPromise).resolves.toBe("log");
    await expect(messagesPromise).resolves.toBe("messages");
  });

  it("keeps non-critical commands batched behind the short queue window", async () => {
    const { invoke } = await import("../../src/tauri-mock");

    const promise = invoke("read_app_log", { maxBytes: 4096 });

    expect(fetch).not.toHaveBeenCalled();

    await vi.advanceTimersByTimeAsync(50);

    expect(fetch).toHaveBeenCalledOnce();
    expect(JSON.parse(String(vi.mocked(fetch).mock.calls[0][1]?.body))).toEqual([
      {
        method: "read_app_log",
        params: { maxBytes: 4096 },
      },
    ]);
    await expect(promise).resolves.toBe("ok");
  });
});
