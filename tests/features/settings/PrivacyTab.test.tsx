import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import PrivacyTab from "../../../src/features/settings/PrivacyTab";

const mocks = vi.hoisted(() => ({
  activeAccountId: null as string | null,
  accounts: [
    {
      id: "account-1",
      email: "me@example.com",
      display_name: "Me",
      provider: "imap",
      created_at: 1,
      updated_at: 1,
    },
    {
      id: "account-2",
      email: "work@example.com",
      display_name: "Work",
      provider: "imap",
      created_at: 2,
      updated_at: 2,
    },
  ],
  listTrustedSenders: vi.fn(),
  removeTrustedSender: vi.fn(),
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (_key: string, fallback?: string) => fallback ?? _key,
  }),
}));

vi.mock("../../../src/stores/mail.store", () => ({
  useMailStore: (selector: (state: { activeAccountId: string | null }) => unknown) =>
    selector({ activeAccountId: mocks.activeAccountId }),
}));

vi.mock("../../../src/stores/toast.store", () => ({
  useToastStore: {
    getState: () => ({ addToast: vi.fn() }),
  },
}));

vi.mock("../../../src/hooks/queries/useAccountsQuery", () => ({
  useAccountsQuery: () => ({ data: mocks.accounts }),
}));

vi.mock("../../../src/lib/api", () => ({
  listTrustedSenders: mocks.listTrustedSenders,
  removeTrustedSender: mocks.removeTrustedSender,
}));

describe("PrivacyTab", () => {
  beforeEach(() => {
    mocks.activeAccountId = null;
    mocks.listTrustedSenders.mockReset();
    mocks.listTrustedSenders.mockResolvedValue([]);
    mocks.removeTrustedSender.mockReset();
  });

  it("selects relaxed as the default privacy mode when there is no stored preference", async () => {
    localStorage.removeItem("pebble-privacy-mode");

    render(<PrivacyTab />);

    await waitFor(() => {
      expect(mocks.listTrustedSenders).toHaveBeenCalledWith(null);
    });
    expect(screen.getByText("Load external images by default. Trackers are still blocked.")).toBeTruthy();
    expect(screen.getByRole("button", { name: "Relaxed" }).getAttribute("style")).toContain(
      "var(--color-accent)",
    );
  });

  it("loads all trusted senders with account emails when there is no active account", async () => {
    mocks.listTrustedSenders.mockResolvedValue([
      {
        account_id: "account-1",
        email: "personal@example.com",
        trust_type: "all",
        created_at: 1,
      },
      {
        account_id: "account-2",
        email: "work-sender@example.com",
        trust_type: "images",
        created_at: 2,
      },
    ]);

    render(<PrivacyTab />);

    expect(await screen.findByText("personal@example.com")).toBeTruthy();
    expect(screen.getByText("work-sender@example.com")).toBeTruthy();
    expect(screen.getByText("me@example.com")).toBeTruthy();
    expect(screen.getByText("work@example.com")).toBeTruthy();
    expect(mocks.listTrustedSenders).toHaveBeenCalledWith(null);
  });

  it("removes trusted sender records by active account and email", async () => {
    mocks.activeAccountId = "account-1";
    mocks.listTrustedSenders.mockResolvedValue([
      {
        account_id: "account-1",
        email: "all@example.com",
        trust_type: "all",
        created_at: 1,
      },
      {
        account_id: "account-1",
        email: "images@example.com",
        trust_type: "images",
        created_at: 2,
      },
    ]);
    mocks.removeTrustedSender.mockResolvedValue(undefined);

    render(<PrivacyTab />);

    expect(await screen.findByText("all@example.com")).toBeTruthy();
    expect(screen.getByText("images@example.com")).toBeTruthy();

    fireEvent.click(screen.getAllByTitle("Delete")[0]);

    await waitFor(() => {
      expect(mocks.removeTrustedSender).toHaveBeenCalledWith("account-1", "all@example.com");
    });
    expect(screen.queryByText("all@example.com")).toBeNull();
    expect(screen.getByText("images@example.com")).toBeTruthy();
  });

  it("removes trusted sender records by row account in all accounts mode", async () => {
    mocks.activeAccountId = null;
    mocks.listTrustedSenders.mockResolvedValue([
      {
        account_id: "account-1",
        email: "shared@example.com",
        trust_type: "all",
        created_at: 1,
      },
      {
        account_id: "account-2",
        email: "shared@example.com",
        trust_type: "images",
        created_at: 2,
      },
    ]);
    mocks.removeTrustedSender.mockResolvedValue(undefined);

    render(<PrivacyTab />);

    expect(await screen.findAllByText("shared@example.com")).toHaveLength(2);

    fireEvent.click(screen.getAllByTitle("Delete")[1]);

    await waitFor(() => {
      expect(mocks.removeTrustedSender).toHaveBeenCalledWith("account-2", "shared@example.com");
    });
    expect(screen.getAllByText("shared@example.com")).toHaveLength(1);
    expect(screen.getByText("me@example.com")).toBeTruthy();
  });
});
