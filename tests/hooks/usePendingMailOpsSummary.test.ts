import { describe, it, expect, vi, beforeEach } from "vitest";



import { invoke } from "@tauri-apps/api/core";
const mockInvoke = vi.mocked(invoke);

import {
  pendingMailOpsSummaryQueryKey,
} from "../../src/hooks/queries/usePendingMailOpsSummary";
import { getPendingMailOpsSummary } from "../../src/lib/api";

vi.mock("../../src/tauri-mock", () => ({
  invoke: vi.fn(),
}));


describe("usePendingMailOpsSummary", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should generate correct query key with accountId", () => {
    expect(pendingMailOpsSummaryQueryKey("a1")).toEqual(["pendingMailOps", "a1"]);
    expect(pendingMailOpsSummaryQueryKey(null)).toEqual(["pendingMailOps", null]);
  });

  it("getPendingMailOpsSummary should call the correct Tauri command", async () => {
    const mockSummary = {
      pending_count: 1,
      in_progress_count: 0,
      failed_count: 2,
      total_active_count: 3,
      last_error: "network unavailable",
      updated_at: 123,
    };
    mockInvoke.mockResolvedValueOnce(mockSummary);

    const result = await getPendingMailOpsSummary("a1");

    expect(result).toEqual(mockSummary);
    expect(mockInvoke).toHaveBeenCalledWith("get_pending_mail_ops_summary", { accountId: "a1" });
  });
});
