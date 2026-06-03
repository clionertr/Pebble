import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import AccountSetup from "../../src/components/AccountSetup";
import { startOAuthLogin } from "../../src/lib/api";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (_key: string, fallback?: string) => fallback ?? _key,
  }),
}));

vi.mock("../../src/lib/i18n", () => ({
  default: {
    t: (_key: string, fallback?: string) => fallback ?? _key,
  },
}));

vi.mock("../../src/lib/api", () => ({
  addAccount: vi.fn(),
  startOAuthLogin: vi.fn(),
  testImapConnection: vi.fn(),
  wakeSync: vi.fn(),
}));

function renderAccountSetup() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });

  return render(
    <QueryClientProvider client={queryClient}>
      <AccountSetup onClose={vi.fn()} />
    </QueryClientProvider>,
  );
}

describe("OAuth account core flow", () => {
  afterEach(() => {
    vi.clearAllMocks();
  });

  it("starts provider OAuth from the account setup flow", () => {
    renderAccountSetup();

    fireEvent.click(screen.getByRole("button", { name: "Sign in with Google" }));

    expect(startOAuthLogin).toHaveBeenCalledWith("gmail", undefined, undefined);
  });
});
