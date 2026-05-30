import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import AccountSetup from "../../src/components/AccountSetup";
import { startOAuthLogin } from "../../src/lib/api";

vi.mock("../../src/lib/i18n", () => ({
  default: {
    t: (_key: string, fallback?: string) => fallback ?? _key,
  },
}));

vi.mock("react-i18next", () => ({
  initReactI18next: {
    type: "3rdParty",
    init: vi.fn(),
  },
  useTranslation: () => ({
    t: (_key: string, fallback?: string) => fallback ?? _key,
  }),
}));

vi.mock("../../src/lib/api", () => ({
  addAccount: vi.fn(),
  startOAuthLogin: vi.fn(),
  testImapConnection: vi.fn(),
  wakeSync: vi.fn(),
}));

describe("AccountSetup OAuth", () => {
  afterEach(() => {
    vi.clearAllMocks();
  });

  it("starts OAuth sign-in through the HTTP auth endpoint", () => {
    const queryClient = new QueryClient({
      defaultOptions: {
        queries: { retry: false },
      },
    });
    const onClose = vi.fn();

    render(
      <QueryClientProvider client={queryClient}>
        <AccountSetup onClose={onClose} />
      </QueryClientProvider>,
    );

    fireEvent.click(screen.getByRole("button", { name: "Sign in with Google" }));

    expect(startOAuthLogin).toHaveBeenCalledWith("gmail", undefined, undefined);
    expect(onClose).not.toHaveBeenCalled();
  });

  it("passes proxy settings to OAuth sign-in", () => {
    const queryClient = new QueryClient({
      defaultOptions: {
        queries: { retry: false },
      },
    });

    render(
      <QueryClientProvider client={queryClient}>
        <AccountSetup onClose={vi.fn()} />
      </QueryClientProvider>,
    );

    fireEvent.change(screen.getByLabelText("SOCKS5 Proxy"), {
      target: { value: "127.0.0.1" },
    });
    fireEvent.change(screen.getByLabelText("Port"), {
      target: { value: "7890" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Sign in with Google" }));

    expect(startOAuthLogin).toHaveBeenCalledWith("gmail", "127.0.0.1", 7890);
  });

  it("keeps the add-account dialog open when clicking the backdrop", () => {
    const queryClient = new QueryClient({
      defaultOptions: {
        queries: { retry: false },
      },
    });
    const onClose = vi.fn();

    render(
      <QueryClientProvider client={queryClient}>
        <AccountSetup onClose={onClose} />
      </QueryClientProvider>,
    );

    const dialog = screen.getByRole("dialog", { name: "Add Email Account" });
    fireEvent.mouseDown(dialog);
    fireEvent.click(dialog);

    expect(onClose).not.toHaveBeenCalled();
    expect(screen.getByRole("dialog", { name: "Add Email Account" })).toBeTruthy();
  });
});
