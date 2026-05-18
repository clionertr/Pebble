import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { foldersQueryKey } from "../../src/hooks/queries/useFoldersQuery";
import { listFolders } from "../../src/lib/api";

function jsonResponse(value: unknown) {
  return new Response(JSON.stringify(value), {
    status: 200,
    headers: { "Content-Type": "application/json" },
  });
}

describe("useFoldersQuery", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.stubGlobal("fetch", vi.fn());
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("should generate correct query key with accountId", () => {
    expect(foldersQueryKey("a1")).toEqual(["folders", "a1"]);
  });

  it("listFolders reads account folders from the REST API", async () => {
    const mockFolders = [
      {
        id: "f1",
        account_id: "a1",
        remote_id: "r1",
        name: "Inbox",
        folder_type: "folder" as const,
        role: "inbox" as const,
        parent_id: null,
        color: null,
        is_system: true,
        sort_order: 1,
      },
    ];
    vi.mocked(fetch).mockResolvedValueOnce(jsonResponse(mockFolders));

    const result = await listFolders("a1");

    expect(result).toEqual(mockFolders);
    const [url, init] = vi.mocked(fetch).mock.calls[0];
    expect(new URL(String(url)).pathname).toBe("/api/accounts/a1/folders");
    expect((init as RequestInit).method).toBe("GET");
  });
});
