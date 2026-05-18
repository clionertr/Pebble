import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { accountsQueryKey } from "../../src/hooks/queries/useAccountsQuery";
import {
  enableGmailRealtime,
  getGlobalProxy,
  getGmailRealtimeConfig,
  getOAuthAccountProxy,
  listAccounts,
  updateAccount,
  updateGmailRealtimeConfig,
  updateGlobalProxy,
  updateOAuthAccountProxy,
} from "../../src/lib/api";

function jsonResponse(value: unknown) {
  return new Response(JSON.stringify(value), {
    status: 200,
    headers: { "Content-Type": "application/json" },
  });
}

function fetchMock() {
  return vi.mocked(fetch);
}

function lastRequest() {
  const [url, init] = fetchMock().mock.calls.at(-1)!;
  return { url: new URL(String(url)), init: init as RequestInit };
}

describe("useAccountsQuery", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.stubGlobal("fetch", vi.fn());
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("should have correct query key", () => {
    expect(accountsQueryKey).toEqual(["accounts"]);
  });

  it("listAccounts reads accounts from the REST API", async () => {
    const mockAccounts = [
      {
        id: "a1",
        email: "test@example.com",
        display_name: "Test User",
        provider: "imap" as const,
        created_at: 1000,
        updated_at: 1000,
      },
    ];
    fetchMock().mockResolvedValueOnce(jsonResponse(mockAccounts));

    const result = await listAccounts();

    expect(result).toEqual(mockAccounts);
    const request = lastRequest();
    expect(request.url.pathname).toBe("/api/accounts");
    expect(request.init.method).toBe("GET");
    expect(request.init.credentials).toBe("same-origin");
  });

  it("getOAuthAccountProxy reuses the account proxy REST endpoint", async () => {
    fetchMock().mockResolvedValueOnce(jsonResponse({ host: "127.0.0.1", port: 7890 }));

    const result = await getOAuthAccountProxy("account-1");

    expect(result).toEqual({ host: "127.0.0.1", port: 7890 });
    expect(lastRequest().url.pathname).toBe("/api/accounts/account-1/proxy");
  });

  it("updateOAuthAccountProxy sends snake_case proxy fields", async () => {
    fetchMock().mockResolvedValueOnce(jsonResponse(null));

    await updateOAuthAccountProxy("account-1", "127.0.0.1", 7890);

    const request = lastRequest();
    expect(request.url.pathname).toBe("/api/accounts/account-1/proxy");
    expect(request.init.method).toBe("PUT");
    expect(JSON.parse(String(request.init.body))).toEqual({
      proxy_host: "127.0.0.1",
      proxy_port: 7890,
    });
  });

  it("getGlobalProxy reads the global proxy endpoint", async () => {
    fetchMock().mockResolvedValueOnce(jsonResponse({ host: "127.0.0.1", port: 7890 }));

    const result = await getGlobalProxy();

    expect(result).toEqual({ host: "127.0.0.1", port: 7890 });
    expect(lastRequest().url.pathname).toBe("/api/proxy");
  });

  it("updateGlobalProxy sends snake_case proxy fields", async () => {
    fetchMock().mockResolvedValueOnce(jsonResponse(null));

    await updateGlobalProxy("127.0.0.1", 7890);

    const request = lastRequest();
    expect(request.url.pathname).toBe("/api/proxy");
    expect(request.init.method).toBe("PUT");
    expect(JSON.parse(String(request.init.body))).toEqual({
      proxy_host: "127.0.0.1",
      proxy_port: 7890,
    });
  });

  it("updateAccount sends backend account field names", async () => {
    fetchMock().mockResolvedValueOnce(jsonResponse(null));

    await updateAccount(
      "account-1",
      "user@example.com",
      "User",
      undefined,
      undefined,
      undefined,
      undefined,
      undefined,
      undefined,
      undefined,
      undefined,
      undefined,
      "#22c55e",
    );

    const request = lastRequest();
    expect(request.url.pathname).toBe("/api/accounts/account-1");
    expect(request.init.method).toBe("PATCH");
    expect(JSON.parse(String(request.init.body))).toEqual({
      email: "user@example.com",
      display_name: "User",
      account_color: "#22c55e",
    });
  });

  it("getGmailRealtimeConfig reads the account realtime endpoint", async () => {
    fetchMock().mockResolvedValueOnce(jsonResponse({ account_id: "account-1" }));

    await getGmailRealtimeConfig("account-1");

    expect(lastRequest().url.pathname).toBe("/api/accounts/account-1/gmail-realtime");
  });

  it("enableGmailRealtime sends the fallback interval", async () => {
    fetchMock().mockResolvedValueOnce(jsonResponse({}));

    await enableGmailRealtime("account-1", 30);

    const request = lastRequest();
    expect(request.url.pathname).toBe("/api/accounts/account-1/gmail-realtime/enable");
    expect(JSON.parse(String(request.init.body))).toEqual({ fallback_interval_minutes: 30 });
  });

  it("updateGmailRealtimeConfig sends the fallback interval", async () => {
    fetchMock().mockResolvedValueOnce(jsonResponse({}));

    await updateGmailRealtimeConfig("account-1", 45);

    const request = lastRequest();
    expect(request.url.pathname).toBe("/api/accounts/account-1/gmail-realtime");
    expect(request.init.method).toBe("PUT");
    expect(JSON.parse(String(request.init.body))).toEqual({ fallback_interval_minutes: 45 });
  });
});
