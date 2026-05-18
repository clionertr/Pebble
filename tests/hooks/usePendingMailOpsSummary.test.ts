import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { pendingMailOpsSummaryQueryKey } from "../../src/hooks/queries/usePendingMailOpsSummary";
import { getPendingMailOpsSummary } from "../../src/lib/api";

function jsonResponse(value: unknown) {
  return new Response(JSON.stringify(value), {
    status: 200,
    headers: { "Content-Type": "application/json" },
  });
}

describe("usePendingMailOpsSummary", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.stubGlobal("fetch", vi.fn());
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("should generate correct query key with accountId", () => {
    expect(pendingMailOpsSummaryQueryKey("a1")).toEqual(["pendingMailOps", "a1"]);
    expect(pendingMailOpsSummaryQueryKey(null)).toEqual(["pendingMailOps", null]);
  });

  it("getPendingMailOpsSummary reads the REST summary endpoint", async () => {
    const mockSummary = {
      pending_count: 1,
      in_progress_count: 0,
      failed_count: 2,
      total_active_count: 3,
      last_error: "network unavailable",
      updated_at: 123,
    };
    vi.mocked(fetch).mockResolvedValueOnce(jsonResponse(mockSummary));

    const result = await getPendingMailOpsSummary("a1");

    expect(result).toEqual(mockSummary);
    const [url] = vi.mocked(fetch).mock.calls[0];
    const requestUrl = new URL(String(url));
    expect(requestUrl.pathname).toBe("/api/pending-ops/summary");
    expect(requestUrl.searchParams.get("accountId")).toBe("a1");
  });
});
