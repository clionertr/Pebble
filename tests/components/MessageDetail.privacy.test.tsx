import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import MessageDetail from "../../src/components/MessageDetail";
import type { Message, PrivacyMode } from "../../src/lib/api";

const mocks = vi.hoisted(() => ({
  trustSender: vi.fn(),
  useMessageLoader: vi.fn(),
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
  trustSender: mocks.trustSender,
}));

vi.mock("../../src/hooks/useMessageLoader", () => ({
  useMessageLoader: mocks.useMessageLoader,
}));

vi.mock("../../src/hooks/queries", () => ({
  useAccountsQuery: () => ({
    data: [{ id: "account-1", email: "recipient@example.com" }],
  }),
}));

vi.mock("../../src/hooks/useBilingualTranslation", () => ({
  useBilingualTranslation: () => ({
    bilingualMode: false,
    bilingualResult: null,
    bilingualLoading: false,
    handleBilingualToggle: vi.fn(),
    resetBilingual: vi.fn(),
  }),
}));

vi.mock("../../src/components/MessageActionToolbar", () => ({
  default: () => <div>message actions</div>,
}));

vi.mock("../../src/components/AttachmentList", () => ({
  default: () => <div>attachments</div>,
}));

vi.mock("../../src/components/PrivacyBanner", () => ({
  default: ({ onLoadImages, onTrustSender }: {
    onLoadImages: () => void;
    onTrustSender: (trustType: "images" | "all") => void;
  }) => (
    <div>
      <button onClick={onLoadImages}>load images</button>
      <button onClick={() => onTrustSender("all")}>trust sender</button>
    </div>
  ),
}));

vi.mock("../../src/features/inbox/SnoozePopover", () => ({
  default: () => <div>snooze</div>,
}));

vi.mock("../../src/features/translate/TranslatePopover", () => ({
  default: () => <div>translate popover</div>,
}));

vi.mock("../../src/components/ShadowDomEmail", () => ({
  ShadowDomEmail: ({ html }: { html: string }) => <div>{html}</div>,
}));

function makeMessage(id: string): Message {
  return {
    id,
    account_id: "account-1",
    remote_id: `remote-${id}`,
    message_id_header: null,
    in_reply_to: null,
    references_header: null,
    thread_id: null,
    subject: `Subject ${id}`,
    snippet: "Snippet",
    from_address: `${id}@example.com`,
    from_name: "Sender",
    to_list: [],
    cc_list: [],
    bcc_list: [],
    has_attachments: false,
    is_read: true,
    is_starred: false,
    is_draft: false,
    date: 1_700_000_000,
    remote_version: null,
    is_deleted: false,
    deleted_at: null,
    created_at: 1_700_000_000,
    updated_at: 1_700_000_000,
    body_text: "Body",
    body_html_raw: "<p>Body</p>",
  };
}

function lastPrivacyMode(): PrivacyMode {
  const call = mocks.useMessageLoader.mock.calls.at(-1);
  if (!call) throw new Error("useMessageLoader was not called");
  return call[1] as PrivacyMode;
}

describe("MessageDetail privacy mode", () => {
  beforeEach(() => {
    localStorage.setItem("pebble-privacy-mode", "strict");
    mocks.trustSender.mockReset();
    mocks.trustSender.mockResolvedValue(undefined);
    mocks.useMessageLoader.mockReset();
    mocks.useMessageLoader.mockImplementation((messageId: string) => ({
      message: makeMessage(messageId),
      setMessage: vi.fn(),
      rendered: { html: "<p>Body</p>", trackers_blocked: [], images_blocked: 1 },
      loading: false,
      error: null,
    }));
  });

  it("does not carry one-message image loading to the next message", () => {
    const { rerender } = render(<MessageDetail messageId="message-a" onBack={vi.fn()} />);

    expect(lastPrivacyMode()).toBe("Strict");

    fireEvent.click(screen.getByRole("button", { name: "load images" }));

    expect(lastPrivacyMode()).toBe("LoadOnce");

    rerender(<MessageDetail messageId="message-b" onBack={vi.fn()} />);

    expect(lastPrivacyMode()).toBe("Strict");
  });

  it("does not carry one-message sender trust to the next message", () => {
    const { rerender } = render(<MessageDetail messageId="message-a" onBack={vi.fn()} />);

    fireEvent.click(screen.getByRole("button", { name: "trust sender" }));

    expect(lastPrivacyMode()).toEqual({ TrustSender: "message-a@example.com" });

    rerender(<MessageDetail messageId="message-b" onBack={vi.fn()} />);

    expect(lastPrivacyMode()).toBe("Strict");
  });
});
