import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import ComposeView from "../../src/features/compose/ComposeView";

const mocks = vi.hoisted(() => ({
  mutate: vi.fn(),
  closeCompose: vi.fn(),
  setComposeDirty: vi.fn(),
  loadDraftFromStorage: vi.fn(),
  recipients: {
    to: [] as string[],
    cc: [] as string[],
    bcc: [] as string[],
  },
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (_key: string, fallback?: string) => fallback ?? _key }),
}));

vi.mock("../../src/stores/mail.store", () => ({
  useMailStore: (selector: (state: { activeAccountId: string }) => unknown) =>
    selector({ activeAccountId: "account-1" }),
}));

vi.mock("../../src/stores/compose.store", () => ({
  useComposeStore: Object.assign(
    (selector: (state: {
      composeMode: string;
      composeReplyTo: null;
      closeCompose: () => void;
      showComposeLeaveConfirm: boolean;
      confirmCloseCompose: () => void;
      cancelCloseCompose: () => void;
    }) => unknown) =>
      selector({
        composeMode: "new",
        composeReplyTo: null,
        closeCompose: mocks.closeCompose,
        showComposeLeaveConfirm: false,
        confirmCloseCompose: vi.fn(),
        cancelCloseCompose: vi.fn(),
      }),
    {
      getState: () => ({ setComposeDirty: mocks.setComposeDirty }),
    },
  ),
}));

vi.mock("../../src/hooks/queries", () => ({
  useAccountsQuery: () => ({
    data: [{ id: "account-1", email: "me@example.com", display_name: "Me" }],
    isLoading: false,
  }),
}));

vi.mock("../../src/hooks/mutations", () => ({
  useSendEmailMutation: () => ({
    isPending: false,
    mutate: mocks.mutate,
  }),
}));

vi.mock("../../src/hooks/useComposeRecipients", () => ({
  useComposeRecipients: () => ({
    fromAccountId: "account-1",
    setFromAccountId: vi.fn(),
    to: mocks.recipients.to,
    setTo: vi.fn(),
    cc: mocks.recipients.cc,
    setCc: vi.fn(),
    bcc: mocks.recipients.bcc,
    setBcc: vi.fn(),
    showCc: false,
    setShowCc: vi.fn(),
    showBcc: false,
    setShowBcc: vi.fn(),
  }),
}));

vi.mock("../../src/hooks/useComposeDraft", () => ({
  useComposeDraft: () => ({
    draftIdRef: { current: null },
    draftIdsByAccountRef: { current: {} },
  }),
  loadDraftFromStorage: mocks.loadDraftFromStorage,
  clearDraftStorage: vi.fn(),
}));

vi.mock("../../src/hooks/useComposeEditor", () => ({
  useComposeEditor: () => ({
    editor: {
      getHTML: () => "<p>Hello</p>",
      getText: () => "Hello",
      commands: { setContent: vi.fn() },
    },
    editorMode: "rich",
    rawSource: "",
    setRawSource: vi.fn(),
    richTextHtml: "<p>Hello</p>",
    htmlPreview: false,
    setHtmlPreview: vi.fn(),
    switchMode: vi.fn(),
    textareaRef: { current: null },
    quotedReplyHtml: "",
  }),
  appendReplyQuoteHtml: (bodyHtml: string) => bodyHtml,
}));

vi.mock("../../src/features/compose/ComposeToolbar", () => ({
  ModeButton: ({ label, onClick }: { label: string; onClick: () => void }) => (
    <button type="button" onClick={onClick}>
      {label}
    </button>
  ),
  EditorToolbar: () => <div />,
  MarkdownToolbar: () => <div />,
  composeStyles: {
    backBtn: {},
    fieldRow: {},
    fieldLabel: {},
    toggleBtn: {},
  },
}));

vi.mock("../../src/components/ContactAutocomplete", () => ({
  default: ({
    id,
    name,
    ariaLabelledBy,
    inputValue,
    placeholder,
    onInputValueChange,
  }: {
    id?: string;
    name?: string;
    ariaLabelledBy?: string;
    inputValue?: string;
    placeholder?: string;
    onInputValueChange?: (value: string) => void;
  }) => (
    <input
      id={id}
      name={name}
      role="combobox"
      aria-labelledby={ariaLabelledBy}
      aria-controls="mock-contact-options"
      aria-expanded="false"
      value={inputValue ?? ""}
      placeholder={placeholder}
      onChange={(event) => onInputValueChange?.(event.currentTarget.value)}
    />
  ),
}));

vi.mock("@tiptap/react", () => ({
  EditorContent: () => <div data-testid="editor" />,
}));

vi.mock("../../src/lib/templates", () => ({
  listTemplates: () => [],
  saveTemplate: vi.fn(),
  deleteTemplate: vi.fn(),
}));

vi.mock("../../src/stores/confirm.store", () => ({
  useConfirmStore: { getState: () => ({ confirm: vi.fn().mockResolvedValue(true) }) },
}));

vi.mock("../../src/stores/toast.store", () => ({
  useToastStore: { getState: () => ({ addToast: vi.fn() }) },
}));

vi.mock("../../src/lib/api", () => ({
  deleteDraft: vi.fn(),
  stageComposeAttachment: vi.fn(),
}));

describe("Compose send core flow", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.recipients.to = [];
    mocks.recipients.cc = [];
    mocks.recipients.bcc = [];
    mocks.loadDraftFromStorage.mockReturnValue(null);
  });

  it("accepts a typed recipient and submits the send mutation", async () => {
    render(<ComposeView />);

    fireEvent.change(screen.getByRole("combobox", { name: "To" }), {
      target: { value: "reader@example.com" },
    });
    fireEvent.change(screen.getByRole("textbox", { name: "Subject" }), {
      target: { value: "Status update" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Send" }));

    await waitFor(() =>
      expect(mocks.mutate).toHaveBeenCalledWith(
        expect.objectContaining({
          accountId: "account-1",
          to: ["reader@example.com"],
          subject: "Status update",
          bodyHtml: "<p>Hello</p>",
        }),
        expect.any(Object),
      ),
    );
  });
});
