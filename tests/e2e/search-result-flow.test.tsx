import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { SearchHit } from "../../src/lib/api";
import SearchView from "../../src/features/search/SearchView";
import { useUIStore } from "../../src/stores/ui.store";

const searchMessages = vi.fn();

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, fallback?: string) => {
      const labels: Record<string, string> = {
        "inbox.searchPlaceholder": "Search mail",
        "search.title": "Search",
        "search.searchButton": "Search",
        "search.filters": "Filters",
        "search.results": "Search results",
      };
      return labels[key] ?? fallback ?? key;
    },
  }),
}));

vi.mock("@tanstack/react-query", () => ({
  useQuery: ({ queryFn, enabled }: { queryFn: () => Promise<SearchHit[]>; enabled: boolean }) => {
    if (!enabled) {
      return { data: [], isLoading: false, error: null, refetch: vi.fn() };
    }
    const data = queryFn();
    return {
      data,
      isLoading: false,
      error: null,
      refetch: () => queryFn(),
    };
  },
}));

vi.mock("@tanstack/react-virtual", () => ({
  useVirtualizer: ({ count }: { count: number }) => ({
    getTotalSize: () => count * 76,
    getVirtualItems: () =>
      Array.from({ length: count }, (_, index) => ({
        index,
        key: `row-${index}`,
        start: index * 76,
      })),
    measureElement: vi.fn(),
  }),
}));

vi.mock("../../src/lib/api", () => ({
  advancedSearch: vi.fn(),
  searchMessages: (query: string) => searchMessages(query),
}));

vi.mock("../../src/features/search/SearchFilters", () => ({
  default: () => <div>Search filters</div>,
}));

vi.mock("../../src/components/MessageDetail", () => ({
  default: ({ messageId }: { messageId: string }) => <div>Message detail {messageId}</div>,
}));

describe("Search result core flow", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    useUIStore.setState({ activeView: "search", searchQuery: "", isMobile: false });
    searchMessages.mockReturnValue([
      {
        message_id: "message-1",
        score: 1,
        subject: "Invoice total",
        snippet: "The invoice total is ready",
        from_address: "sender@example.com",
        date: 1_700_000_000,
      },
    ]);
  });

  it("searches from the toolbar and opens a result detail", async () => {
    render(<SearchView />);

    fireEvent.change(screen.getByRole("textbox", { name: "Search" }), {
      target: { value: "invoice" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Search" }));

    await waitFor(() => expect(searchMessages).toHaveBeenCalledWith("invoice"));
    fireEvent.click(screen.getByRole("option", { name: /Invoice total/ }));

    expect(screen.getByText("Message detail message-1")).toBeTruthy();
  });
});
