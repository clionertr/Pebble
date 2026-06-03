import { useState, useEffect, useRef } from "react";
import { EditorContent } from "@tiptap/react";
import {
  ArrowLeft,
  Send,
  X,
  AlertCircle,
  Type,
  FileCode2,
  Hash,
  Eye,
  EyeOff,
  Paperclip,
  ChevronDown,
  ChevronRight,
} from "lucide-react";
import { useTranslation } from "react-i18next";
import { useMailStore } from "@/stores/mail.store";
import { useComposeStore } from "@/stores/compose.store";
import { useAccountsQuery } from "@/hooks/queries";
import { useSendEmailMutation } from "@/hooks/mutations";
import ContactAutocomplete from "@/components/ContactAutocomplete";
import { listTemplates, saveTemplate } from "@/lib/templates";
import type { EmailTemplate } from "@/lib/templates";
import { useComposeRecipients } from "@/hooks/useComposeRecipients";
import { useComposeDraft, loadDraftFromStorage, clearDraftStorage } from "@/hooks/useComposeDraft";
import { deleteDraft, stageComposeAttachment } from "@/lib/api";
import { appendReplyQuoteHtml, useComposeEditor } from "@/hooks/useComposeEditor";
import { useConfirmStore } from "@/stores/confirm.store";
import { useToastStore } from "@/stores/toast.store";
import type { Account } from "@/lib/api-types";
import type { ComposeAttachment } from "./compose-draft";
import {
  ComposeAttachmentList,
  ComposeLeaveConfirmDialog,
  SaveTemplatePanel,
  TemplateMenu,
} from "./ComposePanels";
import { ModeButton, EditorToolbar, MarkdownToolbar, composeStyles } from "./ComposeToolbar";
import { isValidEmailAddress, mergePendingRecipient } from "./recipient-utils";

export default function ComposeView() {
  const composeMode = useComposeStore((s) => s.composeMode);
  const { data: accounts = [], isLoading } = useAccountsQuery();

  if (composeMode === "new" && isLoading) {
    return <div style={{ height: "100%" }} />;
  }

  return <ComposeViewInner accounts={accounts} />;
}

function ComposeViewInner({ accounts }: { accounts: Account[] }) {
  const { t } = useTranslation();
  const composeMode = useComposeStore((s) => s.composeMode);
  const composeReplyTo = useComposeStore((s) => s.composeReplyTo);
  const composePrefill = useComposeStore((s) => s.composePrefill);
  const closeCompose = useComposeStore((s) => s.closeCompose);
  const showComposeLeaveConfirm = useComposeStore((s) => s.showComposeLeaveConfirm);
  const confirmCloseCompose = useComposeStore((s) => s.confirmCloseCompose);
  const cancelCloseCompose = useComposeStore((s) => s.cancelCloseCompose);
  const activeAccountId = useMailStore((s) => s.activeAccountId);

  const isReply = composeMode === "reply" || composeMode === "reply-all";
  const restoredDraft = useRef(
    composeMode === "new" ? loadDraftFromStorage(accounts.map((account) => account.id)) : null,
  );

  // ─── Recipients ──────────────────────────────────────────────────────────────
  const {
    fromAccountId,
    setFromAccountId,
    to,
    setTo,
    cc,
    setCc,
    bcc,
    setBcc,
    showCc,
    setShowCc,
    showBcc,
    setShowBcc,
  } = useComposeRecipients({
    composeMode,
    composeReplyTo,
    accounts,
    activeAccountId,
    restoredDraft: restoredDraft.current,
    composePrefill,
  });
  const [toInputValue, setToInputValue] = useState("");
  const [ccInputValue, setCcInputValue] = useState("");
  const [bccInputValue, setBccInputValue] = useState("");

  // ─── Subject ─────────────────────────────────────────────────────────────────
  const [subject, setSubject] = useState(() => {
    if (restoredDraft.current) return restoredDraft.current.subject;
    if (composePrefill?.subject) return composePrefill.subject;
    if (!composeReplyTo) return "";
    if (isReply) return `Re: ${composeReplyTo.subject.replace(/^(Re:\s*|Fwd:\s*)+/i, "")}`;
    if (composeMode === "forward")
      return `Fwd: ${composeReplyTo.subject.replace(/^(Re:\s*|Fwd:\s*)+/i, "")}`;
    return "";
  });
  const [sendError, setSendError] = useState<string | null>(null);
  const sendMutation = useSendEmailMutation();

  // ─── Editor ──────────────────────────────────────────────────────────────────
  const {
    editor,
    editorMode,
    rawSource,
    setRawSource,
    richTextHtml,
    htmlPreview,
    setHtmlPreview,
    switchMode,
    textareaRef,
    quotedReplyHtml,
  } = useComposeEditor({
    fromAccountId,
    composeMode,
    composeReplyTo,
    isReply,
    t,
    restoredDraft: restoredDraft.current,
    prefillBody: composePrefill?.body,
  });

  // ─── Draft persistence ───────────────────────────────────────────────────────
  // Delay the dirty-snapshot until the editor has run its initial setContent
  // cycle — otherwise the snapshot captures an empty richTextHtml and then
  // flips dirty when signature/quoted-reply text populates.
  // For "new" composes with no signature the editor stays empty but is still
  // "ready" once mounted, so we gate on editor presence plus one effect tick.
  // Attachments are part of the draft snapshot and must be declared before
  // the draft persistence hook.
  const attachInputRef = useRef<HTMLInputElement>(null);
  const [attachments, setAttachments] = useState<ComposeAttachment[]>(
    () => restoredDraft.current?.attachments ?? [],
  );
  const [isDragging, setIsDragging] = useState(false);

  const [editorReady, setEditorReady] = useState(false);
  useEffect(() => {
    if (editor && !editorReady) setEditorReady(true);
  }, [editor, editorReady]);
  const { draftIdRef, draftIdsByAccountRef } = useComposeDraft({
    to,
    cc,
    bcc,
    subject,
    rawSource,
    richTextHtml,
    editorMode,
    composeMode,
    fromAccountId,
    attachments,
    editorReady,
  });

  // ─── Templates ───────────────────────────────────────────────────────────────
  const [showTemplates, setShowTemplates] = useState(false);
  const [templates, setTemplates] = useState<EmailTemplate[]>([]);
  const [showSaveTemplate, setShowSaveTemplate] = useState(false);
  const [templateName, setTemplateName] = useState("");
  const [showQuotedReply, setShowQuotedReply] = useState(false);

  async function refreshTemplates() {
    try {
      const next = await listTemplates();
      setTemplates(next);
      return next;
    } catch (err) {
      console.warn("Failed to load templates:", err);
      return [];
    }
  }

  async function stageAttachmentFiles(files: FileList | File[]) {
    const staged: ComposeAttachment[] = [];
    for (const file of Array.from(files)) {
      const bytes = Array.from(new Uint8Array(await file.arrayBuffer()));
      const path = await stageComposeAttachment(file.name, bytes);
      staged.push({ name: file.name, path, size: file.size });
    }
    setAttachments((prev) => [...prev, ...staged]);
  }

  async function handleSaveTemplate() {
    if (!templateName.trim()) return;
    const bodyContent = editorMode === "rich" && editor ? editor.getHTML() : rawSource;
    await saveTemplate({ name: templateName.trim(), subject, body: bodyContent });
    setTemplateName("");
    setShowSaveTemplate(false);
    void refreshTemplates();
  }

  useEffect(() => {
    if (!sendError) return;
    const timer = setTimeout(() => setSendError(null), 5000);
    return () => clearTimeout(timer);
  }, [sendError]);

  async function handleSend() {
    const finalTo = mergePendingRecipient(to, toInputValue).filter(Boolean);
    const finalCc = mergePendingRecipient(cc, ccInputValue).filter(Boolean);
    const finalBcc = mergePendingRecipient(bcc, bccInputValue).filter(Boolean);

    if (!fromAccountId || finalTo.length === 0) return;

    setTo(finalTo);
    setCc(finalCc);
    setBcc(finalBcc);
    if (isValidEmailAddress(toInputValue)) setToInputValue("");
    if (isValidEmailAddress(ccInputValue)) setCcInputValue("");
    if (isValidEmailAddress(bccInputValue)) setBccInputValue("");

    if (!subject.trim()) {
      const confirmed = await useConfirmStore.getState().confirm({
        title: t("compose.noSubjectTitle", "No subject"),
        message: t("compose.noSubjectConfirm", "Send without a subject?"),
        confirmLabel: t("common.send", "Send"),
      });
      if (!confirmed) return;
    }

    setSendError(null);

    let bodyHtml = "";
    let bodyText = "";

    if (editorMode === "rich" && editor) {
      bodyHtml = editor.getHTML();
      bodyText = editor.getText();
    } else if (editorMode === "html") {
      bodyHtml = rawSource;
      const tmp = document.createElement("div");
      tmp.innerHTML = rawSource;
      bodyText = tmp.textContent || tmp.innerText || "";
    } else {
      if (editor) {
        editor.commands.setContent(rawSource);
        bodyHtml = editor.getHTML();
        bodyText = rawSource;
      }
    }
    const outgoingBodyHtml = appendReplyQuoteHtml(bodyHtml, quotedReplyHtml);

    const inReplyTo =
      isReply && composeReplyTo?.message_id_header ? composeReplyTo.message_id_header : undefined;

    sendMutation.mutate(
      {
        accountId: fromAccountId,
        to: finalTo,
        cc: finalCc,
        bcc: finalBcc,
        subject,
        bodyText,
        bodyHtml: outgoingBodyHtml || undefined,
        inReplyTo: inReplyTo || undefined,
        attachmentPaths: attachments.length > 0 ? attachments.map((a) => a.path) : undefined,
      },
      {
        onSuccess: () => {
          const draftIdsToDelete = { ...draftIdsByAccountRef.current };
          if (draftIdRef.current && fromAccountId) {
            draftIdsToDelete[fromAccountId] = draftIdRef.current;
          }
          for (const [draftAccountId, draftId] of Object.entries(draftIdsToDelete)) {
            deleteDraft(draftAccountId, draftId).catch((err) => {
              console.warn("Failed to delete draft after send:", err);
              useToastStore.getState().addToast({
                type: "error",
                message: t(
                  "compose.draftCleanupFailed",
                  "Sent, but failed to remove the saved draft. You can delete it from Drafts.",
                ),
              });
            });
          }
          draftIdsByAccountRef.current = {};
          draftIdRef.current = null;
          clearDraftStorage();
          useComposeStore.getState().setComposeDirty(false);
          closeCompose();
        },
        onError: (e) => {
          const msg = e instanceof Error ? e.message : String(e);
          setSendError(msg || t("compose.sendError", "Failed to send"));
        },
      },
    );
  }

  const title =
    composeMode === "reply"
      ? t("compose.reply", "Reply")
      : composeMode === "reply-all"
        ? t("compose.replyAll", "Reply All")
        : composeMode === "forward"
          ? t("compose.forward", "Forward")
          : t("compose.newMessage", "New Message");
  const hasToRecipient = to.some(Boolean) || isValidEmailAddress(toInputValue);
  const sendDisabled = sendMutation.isPending || !fromAccountId || !hasToRecipient;

  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100%" }}>
      {/* Header */}
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          padding: "10px 20px",
          borderBottom: "1px solid var(--color-border)",
          flexShrink: 0,
        }}
      >
        <div style={{ display: "flex", alignItems: "center", gap: "12px" }}>
          <button
            onClick={closeCompose}
            style={composeStyles.backBtn}
            onMouseEnter={(e) => {
              e.currentTarget.style.backgroundColor = "var(--color-bg-hover, rgba(0,0,0,0.04))";
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.backgroundColor = "transparent";
            }}
          >
            <ArrowLeft size={16} />
            {t("compose.back", "Back")}
          </button>
          <span style={{ fontWeight: 600, fontSize: "15px", color: "var(--color-text-primary)" }}>
            {title}
          </span>
        </div>
        <button
          onClick={handleSend}
          disabled={sendDisabled}
          style={{
            display: "flex",
            alignItems: "center",
            gap: "6px",
            padding: "7px 20px",
            backgroundColor: sendMutation.isPending
              ? "var(--color-text-secondary)"
              : "var(--color-accent, #2563eb)",
            color: "#fff",
            border: "none",
            borderRadius: "6px",
            cursor: sendDisabled ? "default" : "pointer",
            opacity: hasToRecipient ? 1 : 0.5,
            fontSize: "13px",
            fontWeight: 500,
          }}
        >
          <Send size={14} />
          {sendMutation.isPending ? t("compose.sending", "Sending...") : t("compose.send", "Send")}
        </button>
      </div>

      {/* Error banner */}
      {sendError && (
        <div
          role="alert"
          aria-live="assertive"
          style={{
            display: "flex",
            alignItems: "center",
            gap: "8px",
            padding: "8px 20px",
            backgroundColor: "var(--color-error-bg, #fef2f2)",
            color: "var(--color-error, #dc2626)",
            fontSize: "13px",
            borderBottom: "1px solid var(--color-border)",
          }}
        >
          <AlertCircle size={14} />
          <span style={{ flex: 1 }}>{sendError}</span>
          <button
            onClick={() => setSendError(null)}
            aria-label={t("common.close", "Close")}
            style={{
              background: "none",
              border: "none",
              cursor: "pointer",
              color: "inherit",
              display: "flex",
            }}
          >
            <X size={14} />
          </button>
        </div>
      )}

      {/* Fields + Editor */}
      <div
        className="scroll-region compose-scroll"
        style={{
          flex: 1,
          display: "flex",
          flexDirection: "column",
          minHeight: 0,
          overflow: "auto",
        }}
      >
        <div className="compose-form-shell">
          {/* From */}
          {accounts.length > 1 && (
            <div style={composeStyles.fieldRow}>
              <label htmlFor="compose-from-account" style={composeStyles.fieldLabel}>
                {t("compose.from", "From")}
              </label>
              <select
                id="compose-from-account"
                name="from"
                value={fromAccountId}
                onChange={(e) => setFromAccountId(e.target.value)}
                style={{
                  flex: 1,
                  padding: "6px 0",
                  border: "none",
                  backgroundColor: "transparent",
                  fontSize: "13px",
                  color: "var(--color-text-primary)",
                  cursor: "pointer",
                }}
              >
                {accounts.map((acc) => (
                  <option key={acc.id} value={acc.id}>
                    {acc.display_name ? `${acc.display_name} <${acc.email}>` : acc.email}
                  </option>
                ))}
              </select>
            </div>
          )}

          {/* To */}
          <div style={composeStyles.fieldRow}>
            <label
              id="compose-to-label"
              htmlFor="compose-to-input"
              style={composeStyles.fieldLabel}
            >
              {t("compose.to", "To")}
            </label>
            <ContactAutocomplete
              id="compose-to-input"
              name="to"
              ariaLabelledBy="compose-to-label"
              value={to}
              onChange={setTo}
              accountId={fromAccountId}
              inputValue={toInputValue}
              onInputValueChange={setToInputValue}
              placeholder="recipient@example.com"
            />
            <div style={{ display: "flex", gap: "4px", padding: "0 8px", flexShrink: 0 }}>
              {!showCc && (
                <button onClick={() => setShowCc(true)} style={composeStyles.toggleBtn}>
                  {t("compose.cc", "Cc")}
                </button>
              )}
              {!showBcc && (
                <button onClick={() => setShowBcc(true)} style={composeStyles.toggleBtn}>
                  {t("compose.bcc", "Bcc")}
                </button>
              )}
            </div>
          </div>

          {showCc && (
            <div style={composeStyles.fieldRow}>
              <label
                id="compose-cc-label"
                htmlFor="compose-cc-input"
                style={composeStyles.fieldLabel}
              >
                {t("compose.cc", "Cc")}
              </label>
              <ContactAutocomplete
                id="compose-cc-input"
                name="cc"
                ariaLabelledBy="compose-cc-label"
                value={cc}
                onChange={setCc}
                accountId={fromAccountId}
                inputValue={ccInputValue}
                onInputValueChange={setCcInputValue}
                placeholder="cc@example.com"
              />
            </div>
          )}

          {showBcc && (
            <div style={composeStyles.fieldRow}>
              <label
                id="compose-bcc-label"
                htmlFor="compose-bcc-input"
                style={composeStyles.fieldLabel}
              >
                {t("compose.bcc", "Bcc")}
              </label>
              <ContactAutocomplete
                id="compose-bcc-input"
                name="bcc"
                ariaLabelledBy="compose-bcc-label"
                value={bcc}
                onChange={setBcc}
                accountId={fromAccountId}
                inputValue={bccInputValue}
                onInputValueChange={setBccInputValue}
                placeholder="bcc@example.com"
              />
            </div>
          )}

          {/* Subject */}
          <div style={composeStyles.fieldRow}>
            <label htmlFor="compose-subject" style={composeStyles.fieldLabel}>
              {t("compose.subject", "Subject")}
            </label>
            <input
              id="compose-subject"
              name="subject"
              type="text"
              value={subject}
              onChange={(e) => setSubject(e.target.value)}
              placeholder=""
              autoComplete="off"
              style={{
                flex: 1,
                padding: "8px 0",
                border: "none",
                backgroundColor: "transparent",
                fontSize: "13px",
                color: "var(--color-text-primary)",
              }}
            />
          </div>

          {/* Toolbar */}
          <div className="compose-editor-toolbar-row">
            <div className="compose-toolbar-tools">
              <div className="compose-toolbar-actions">
                <button
                  type="button"
                  onClick={() => attachInputRef.current?.click()}
                  title={t("compose.attach", "Attach file")}
                  aria-label={t("compose.attach", "Attach file")}
                  className="compose-toolbar-icon-button"
                >
                  <Paperclip size={13} />
                </button>
                <input
                  ref={attachInputRef}
                  type="file"
                  multiple
                  style={{
                    position: "absolute",
                    width: 1,
                    height: 1,
                    padding: 0,
                    margin: -1,
                    overflow: "hidden",
                    clip: "rect(0,0,0,0)",
                    border: 0,
                  }}
                  tabIndex={-1}
                  aria-hidden="true"
                  onChange={(e) => {
                    const files = e.target.files;
                    if (!files) return;
                    void stageAttachmentFiles(files).catch((err) => {
                      console.warn("Failed to stage attachment:", err);
                      setSendError(t("compose.attachmentStageError", "Failed to attach file"));
                    });
                    e.target.value = "";
                  }}
                />
                <TemplateMenu
                  show={showTemplates}
                  templates={templates}
                  editor={editor}
                  onToggle={() => setShowTemplates((value) => !value)}
                  onRefresh={refreshTemplates}
                  onStartSave={() => {
                    setShowSaveTemplate(true);
                    setShowTemplates(false);
                  }}
                  onApply={(template) => {
                    setSubject(template.subject);
                    setRawSource(template.body);
                    setShowTemplates(false);
                  }}
                  t={t}
                />
              </div>
              <div className="compose-toolbar-divider" />

              {/* Formatting toolbar */}
              <div className="compose-format-toolbar-slot">
                {editorMode === "rich" && editor && <EditorToolbar editor={editor} />}
                {editorMode === "markdown" && (
                  <MarkdownToolbar
                    textareaRef={textareaRef}
                    onInsert={setRawSource}
                    source={rawSource}
                  />
                )}
                {editorMode === "html" && (
                  <button
                    type="button"
                    onClick={() => setHtmlPreview((v) => !v)}
                    title={
                      htmlPreview
                        ? t("compose.mode.hidePreview", "Hide preview")
                        : t("compose.mode.showPreview", "Show preview")
                    }
                    className={`compose-toolbar-text-button${htmlPreview ? " is-active" : ""}`}
                  >
                    {htmlPreview ? <EyeOff size={13} /> : <Eye size={13} />}
                    {htmlPreview
                      ? t("compose.mode.hidePreview", "Hide preview")
                      : t("compose.mode.showPreview", "Show preview")}
                  </button>
                )}
              </div>
            </div>

            {/* Mode tabs */}
            <div
              className="compose-mode-tabs"
              role="group"
              aria-label={t("compose.mode.label", "Editor mode")}
            >
              <ModeButton
                icon={Type}
                label={t("compose.mode.rich", "Rich Text")}
                active={editorMode === "rich"}
                onClick={() => switchMode("rich")}
              />
              <ModeButton
                icon={Hash}
                label={t("compose.mode.markdown", "Markdown")}
                active={editorMode === "markdown"}
                onClick={() => switchMode("markdown")}
              />
              <ModeButton
                icon={FileCode2}
                label={t("compose.mode.html", "HTML")}
                active={editorMode === "html"}
                onClick={() => switchMode("html")}
              />
            </div>
          </div>

          {/* Attachment list */}
          <ComposeAttachmentList
            attachments={attachments}
            onRemove={(index) => setAttachments((prev) => prev.filter((_, j) => j !== index))}
            t={t}
          />

          {/* Save template dialog */}
          {showSaveTemplate && (
            <SaveTemplatePanel
              name={templateName}
              onNameChange={setTemplateName}
              onSave={() => void handleSaveTemplate()}
              onCancel={() => setShowSaveTemplate(false)}
              t={t}
            />
          )}

          {/* Editor area */}
          <div
            className="compose-editor-area"
            onDragOver={(e) => {
              e.preventDefault();
              setIsDragging(true);
            }}
            onDragLeave={() => setIsDragging(false)}
            onDrop={async (e) => {
              e.preventDefault();
              setIsDragging(false);
              const files = e.dataTransfer.files;
              if (!files.length) return;
              try {
                await stageAttachmentFiles(files);
              } catch (err) {
                console.warn("Failed to stage dropped attachment:", err);
                setSendError(t("compose.attachmentStageError", "Failed to attach file"));
              }
            }}
            onPaste={async (e) => {
              const items = e.clipboardData?.items;
              if (!items) return;
              const imageFiles: File[] = [];
              for (const item of Array.from(items)) {
                if (item.type.startsWith("image/")) {
                  const file = item.getAsFile();
                  if (file) imageFiles.push(file);
                }
              }
              if (!imageFiles.length) return;
              e.preventDefault();
              try {
                await stageAttachmentFiles(imageFiles);
              } catch (err) {
                console.warn("Failed to stage pasted image:", err);
                setSendError(t("compose.attachmentStageError", "Failed to attach file"));
              }
            }}
          >
            <div className="compose-editor-surface">
              {editorMode === "rich" ? (
                <EditorContent editor={editor} className="compose-editor-content" />
              ) : editorMode === "html" && htmlPreview ? (
                <div className="compose-preview-split">
                  <textarea
                    ref={textareaRef}
                    value={rawSource}
                    onChange={(e) => setRawSource(e.target.value)}
                    placeholder={t("compose.mode.htmlPlaceholder", "Write HTML source...")}
                    spellCheck={false}
                    className="compose-source-input compose-source-input--split is-code"
                  />
                  <iframe
                    sandbox="allow-same-origin"
                    srcDoc={rawSource}
                    title={t("compose.mode.preview", "Preview")}
                    className="compose-html-preview"
                  />
                </div>
              ) : (
                <textarea
                  ref={textareaRef}
                  value={rawSource}
                  onChange={(e) => setRawSource(e.target.value)}
                  placeholder={
                    editorMode === "markdown"
                      ? t("compose.mode.markdownPlaceholder", "Write in Markdown...")
                      : t("compose.mode.htmlPlaceholder", "Write HTML source...")
                  }
                  spellCheck={editorMode === "markdown"}
                  className={`compose-source-input${editorMode === "html" ? " is-code" : ""}`}
                />
              )}
            </div>
            {quotedReplyHtml && (
              <div className="compose-quoted-reply">
                <button
                  type="button"
                  className="compose-quoted-reply-toggle"
                  aria-expanded={showQuotedReply}
                  onClick={() => setShowQuotedReply((value) => !value)}
                >
                  {showQuotedReply ? <ChevronDown size={14} /> : <ChevronRight size={14} />}
                  {showQuotedReply
                    ? t("compose.hideQuotedReply", "Hide quoted message")
                    : t("compose.showQuotedReply", "Show quoted message")}
                </button>
                {showQuotedReply && (
                  <div
                    className="scroll-region compose-quoted-reply-content"
                    dangerouslySetInnerHTML={{ __html: quotedReplyHtml }}
                  />
                )}
              </div>
            )}
            {isDragging && (
              <div className="compose-drop-overlay">
                <Paperclip size={20} />
                {t("compose.dropFiles", "Drop files to attach")}
              </div>
            )}
          </div>
        </div>
      </div>

      {/* Compose leave confirmation dialog */}
      {showComposeLeaveConfirm && (
        <ComposeLeaveConfirmDialog
          onCancel={cancelCloseCompose}
          onConfirm={confirmCloseCompose}
          t={t}
        />
      )}
    </div>
  );
}
