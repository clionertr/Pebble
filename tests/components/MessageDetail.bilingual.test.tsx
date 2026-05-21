import { render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import MessageDetail from "../../src/components/MessageDetail";
import type { Message, TranslateResult } from "../../src/lib/api";

const mockMessage: Message = {
  id: "message-1",
  account_id: "account-1",
  remote_id: "remote-1",
  message_id_header: null,
  in_reply_to: null,
  references_header: null,
  thread_id: null,
  subject: "Bilingual message",
  snippet: "Bilingual translation test",
  from_address: "sender@example.com",
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
  body_text: "plain body",
  body_html_raw: "",
};

const mocks = vi.hoisted(() => ({
  bilingualMode: true,
  bilingualLoading: true,
  bilingualResult: {
    translated: "partial translated text",
    segments: [],
  } as TranslateResult | null,
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
  trustSender: vi.fn(),
}));

vi.mock("../../src/hooks/useMessageLoader", () => ({
  useMessageLoader: () => ({
    message: mockMessage,
    setMessage: vi.fn(),
    rendered: null,
    loading: false,
  }),
}));

vi.mock("../../src/hooks/queries", () => ({
  useAccountsQuery: () => ({
    data: [{ id: "account-1", email: "recipient@example.com" }],
  }),
}));

vi.mock("../../src/hooks/useBilingualTranslation", () => ({
  useBilingualTranslation: () => ({
    bilingualMode: mocks.bilingualMode,
    bilingualResult: mocks.bilingualResult,
    bilingualLoading: mocks.bilingualLoading,
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
  default: () => <div>privacy banner</div>,
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

describe("MessageDetail bilingual rendering", () => {
  beforeEach(() => {
    mocks.bilingualMode = true;
    mocks.bilingualLoading = true;
    mocks.bilingualResult = {
      translated: "partial translated text",
      segments: [],
    };
  });

  it("shows partial translation while a slow translation is still running", () => {
    render(<MessageDetail messageId="message-1" onBack={vi.fn()} />);

    expect(screen.getByText("Translating...")).toBeTruthy();
    expect(screen.getByText("partial translated text")).toBeTruthy();
  });
});
