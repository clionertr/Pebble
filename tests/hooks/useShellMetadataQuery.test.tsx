import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { useAccountsQuery, useFoldersForAccountsQuery } from "../../src/hooks/queries";

function jsonResponse(value: unknown) {
  return new Response(JSON.stringify(value), {
    status: 200,
    headers: { "Content-Type": "application/json" },
  });
}

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
    },
  });

  return ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
}

describe("shell-backed metadata queries", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.stubGlobal("fetch", vi.fn((input: RequestInfo | URL) => {
      const pathname = new URL(String(input), window.location.origin).pathname;
      if (pathname === "/api/shell") {
        return Promise.resolve(jsonResponse({
          accounts: [
            {
              id: "account-1",
              email: "a@example.com",
              display_name: "A",
              provider: "gmail",
              created_at: 1,
              updated_at: 1,
            },
            {
              id: "account-2",
              email: "b@example.com",
              display_name: "B",
              provider: "imap",
              created_at: 2,
              updated_at: 2,
            },
          ],
          folders: {
            "account-1": [
              {
                id: "folder-1",
                account_id: "account-1",
                remote_id: "INBOX",
                name: "Inbox A",
                folder_type: "folder",
                role: "inbox",
                parent_id: null,
                color: null,
                is_system: true,
                sort_order: 1,
              },
            ],
            "account-2": [
              {
                id: "folder-2",
                account_id: "account-2",
                remote_id: "INBOX",
                name: "Inbox B",
                folder_type: "folder",
                role: "inbox",
                parent_id: null,
                color: null,
                is_system: true,
                sort_order: 2,
              },
            ],
          },
          unreadCounts: {
            "account-1": { "folder-1": 3 },
            "account-2": { "folder-2": 5 },
          },
          gmailRealtime: {
            "account-1": {
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
            },
          },
        }));
      }
      return Promise.resolve(jsonResponse([]));
    }));
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("hydrates accounts and multi-account folders from one shell request", async () => {
    const { result } = renderHook(() => ({
      accounts: useAccountsQuery(),
      folders: useFoldersForAccountsQuery(["account-1", "account-2"]),
    }), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.accounts.data?.map((account) => account.id)).toEqual([
        "account-1",
        "account-2",
      ]);
      expect(result.current.folders.data.map((folder) => folder.id)).toEqual([
        "folder-1",
        "folder-2",
      ]);
    });

    const paths = vi.mocked(fetch).mock.calls.map(([url]) =>
      new URL(String(url), window.location.origin).pathname
    );
    expect(paths).toEqual(["/api/shell"]);
  });
});
