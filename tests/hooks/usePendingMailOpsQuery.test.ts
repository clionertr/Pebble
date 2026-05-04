import { describe, it, expect, vi, beforeEach } from "vitest";



import { invoke } from "@tauri-apps/api/core";
const mockInvoke = vi.mocked(invoke);

import {
  pendingMailOpsQueryKey,
} from "../../src/hooks/queries/usePendingMailOpsQuery";
import { listPendingMailOps } from "../../src/lib/api";

vi.mock("../../src/tauri-mock", () => ({
  invoke: vi.fn(),
}));


describe("usePendingMailOpsQuery", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should generate correct query key with accountId", () => {
    expect(pendingMailOpsQueryKey("a1")).toEqual(["pendingMailOpsList", "a1"]);
    expect(pendingMailOpsQueryKey(null)).toEqual(["pendingMailOpsList", null]);
  });

  it("listPendingMailOps should call the correct Tauri command", async () => {
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
    mockInvoke.mockResolvedValueOnce(mockOps);

    const result = await listPendingMailOps("a1");

    expect(result).toEqual(mockOps);
    expect(mockInvoke).toHaveBeenCalledWith("list_pending_mail_ops", {
      accountId: "a1",
      limit: 100,
    });
  });
});
