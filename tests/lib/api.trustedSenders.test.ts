import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { listTrustedSenders, removeTrustedSender } from "../../src/lib/api";

function jsonResponse(value: unknown) {
  return new Response(JSON.stringify(value), {
    status: 200,
    headers: { "Content-Type": "application/json" },
  });
}

function lastRequest() {
  const [url, init] = vi.mocked(fetch).mock.calls.at(-1)!;
  return { url: new URL(String(url)), init: init as RequestInit };
}

describe("trusted senders API", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.stubGlobal("fetch", vi.fn());
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("omits accountId when listing trusted senders for all accounts", async () => {
    vi.mocked(fetch).mockResolvedValueOnce(jsonResponse([]));

    await listTrustedSenders(null);

    const request = lastRequest();
    expect(request.url.pathname).toBe("/api/trusted-senders");
    expect(request.url.searchParams.has("accountId")).toBe(false);
    expect(request.init.method).toBe("GET");
  });

  it("deletes trusted sender by account and email", async () => {
    vi.mocked(fetch).mockResolvedValueOnce(jsonResponse(null));

    await removeTrustedSender("account-1", "sender@example.com");

    const request = lastRequest();
    expect(request.url.pathname).toBe("/api/trusted-senders");
    expect(request.url.searchParams.get("accountId")).toBe("account-1");
    expect(request.url.searchParams.get("email")).toBe("sender@example.com");
    expect(request.init.method).toBe("DELETE");
  });
});
