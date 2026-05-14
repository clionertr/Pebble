import { describe, it, expect, vi, beforeEach } from "vitest";



import { invoke } from "../../src/tauri-mock";
const mockInvoke = vi.mocked(invoke);

// Import after mocking
import { accountsQueryKey } from "../../src/hooks/queries/useAccountsQuery";
import {
  enableGmailRealtime,
  getGlobalProxy,
  getGmailRealtimeConfig,
  getOAuthAccountProxy,
  listAccounts,
  updateGmailRealtimeConfig,
  updateGlobalProxy,
  updateOAuthAccountProxy,
  updateAccount,
} from "../../src/lib/api";

vi.mock("../../src/tauri-mock", () => ({
  invoke: vi.fn(),
}));


describe("useAccountsQuery", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should have correct query key", () => {
    expect(accountsQueryKey).toEqual(["accounts"]);
  });

  it("listAccounts should call the correct Tauri command", async () => {
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
    mockInvoke.mockResolvedValueOnce(mockAccounts);

    const result = await listAccounts();

    expect(result).toEqual(mockAccounts);
    expect(mockInvoke).toHaveBeenCalledWith("list_accounts");
  });

  it("getOAuthAccountProxy should call the correct Tauri command", async () => {
    mockInvoke.mockResolvedValueOnce({ host: "127.0.0.1", port: 7890 });

    const result = await getOAuthAccountProxy("account-1");

    expect(result).toEqual({ host: "127.0.0.1", port: 7890 });
    expect(mockInvoke).toHaveBeenCalledWith("get_oauth_account_proxy", {
      accountId: "account-1",
    });
  });

  it("updateOAuthAccountProxy should call the correct Tauri command", async () => {
    mockInvoke.mockResolvedValueOnce(undefined);

    await updateOAuthAccountProxy("account-1", "127.0.0.1", 7890);

    expect(mockInvoke).toHaveBeenCalledWith("update_oauth_account_proxy", {
      accountId: "account-1",
      proxyHost: "127.0.0.1",
      proxyPort: 7890,
    });
  });

  it("getGlobalProxy should call the correct Tauri command", async () => {
    mockInvoke.mockResolvedValueOnce({ host: "127.0.0.1", port: 7890 });

    const result = await getGlobalProxy();

    expect(result).toEqual({ host: "127.0.0.1", port: 7890 });
    expect(mockInvoke).toHaveBeenCalledWith("get_global_proxy");
  });

  it("updateGlobalProxy should call the correct Tauri command", async () => {
    mockInvoke.mockResolvedValueOnce(undefined);

    await updateGlobalProxy("127.0.0.1", 7890);

    expect(mockInvoke).toHaveBeenCalledWith("update_global_proxy", {
      proxyHost: "127.0.0.1",
      proxyPort: 7890,
    });
  });

  it("updateAccount should include accountColor when saving an account color", async () => {
    mockInvoke.mockResolvedValueOnce(undefined);

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

    expect(mockInvoke).toHaveBeenCalledWith("update_account", {
      accountId: "account-1",
      email: "user@example.com",
      displayName: "User",
      password: undefined,
      imapHost: undefined,
      imapPort: undefined,
      smtpHost: undefined,
      smtpPort: undefined,
      imapSecurity: undefined,
      smtpSecurity: undefined,
      proxyHost: undefined,
      proxyPort: undefined,
      accountColor: "#22c55e",
    });
  });

  it("getGmailRealtimeConfig should call the correct Tauri command", async () => {
    mockInvoke.mockResolvedValueOnce({
      accountId: "account-1",
      enabled: false,
      status: "not_enabled",
      configMissing: false,
      topicName: null,
      expirationMs: null,
      lastWatchHistoryId: null,
      lastWatchAt: null,
      lastError: null,
      fallbackIntervalMinutes: 15,
    });

    await getGmailRealtimeConfig("account-1");

    expect(mockInvoke).toHaveBeenCalledWith("get_gmail_realtime_config", {
      accountId: "account-1",
    });
  });

  it("enableGmailRealtime should pass the fallback interval", async () => {
    mockInvoke.mockResolvedValueOnce({});

    await enableGmailRealtime("account-1", 30);

    expect(mockInvoke).toHaveBeenCalledWith("enable_gmail_realtime", {
      accountId: "account-1",
      fallbackIntervalMinutes: 30,
    });
  });

  it("updateGmailRealtimeConfig should pass the fallback interval", async () => {
    mockInvoke.mockResolvedValueOnce({});

    await updateGmailRealtimeConfig("account-1", 45);

    expect(mockInvoke).toHaveBeenCalledWith("update_gmail_realtime_config", {
      accountId: "account-1",
      fallbackIntervalMinutes: 45,
    });
  });
});
