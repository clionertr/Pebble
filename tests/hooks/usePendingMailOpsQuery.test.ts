import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { pendingMailOpsQueryKey } from "../../src/hooks/queries/usePendingMailOpsQuery";
import { listPendingMailOps } from "../../src/lib/api";

function jsonResponse(value: unknown) {
  return new Response(JSON.stringify(value), {
    status: 200,
    headers: { "Content-Type": "application/json" },
  });
}

describe("usePendingMailOpsQuery", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.stubGlobal("fetch", vi.fn());
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("should generate correct query key with accountId", () => {
    expect(pendingMailOpsQueryKey("a1")).toEqual(["pendingMailOpsList", "a1"]);
    expect(pendingMailOpsQueryKey(null)).toEqual(["pendingMailOpsList", null]);
  });

  it("listPendingMailOps reads pending operations from the REST API", async () => {
    const mockOps = [
      {
        id: "op-1",
        account_id: "a1",
        message_id: "m1",
        op_type: "archive",
        status: "failed",
        attempts: 2,
        last_error: "network unavailable",
        created_at: 123,
        updated_at: 456,
      },
    ];
    vi.mocked(fetch).mockResolvedValueOnce(jsonResponse(mockOps));

    const result = await listPendingMailOps("a1");

    expect(result).toEqual(mockOps);
    const [url, init] = vi.mocked(fetch).mock.calls[0];
    const requestUrl = new URL(String(url));
    expect(requestUrl.pathname).toBe("/api/pending-ops");
    expect(requestUrl.searchParams.get("accountId")).toBe("a1");
    expect(requestUrl.searchParams.get("limit")).toBe("100");
    expect((init as RequestInit).method).toBe("GET");
  });
});
