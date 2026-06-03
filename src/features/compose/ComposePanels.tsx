import { useRef, useEffect } from "react";
import type { Editor } from "@tiptap/react";
import { BookTemplate, FileText, Trash2, X } from "lucide-react";
import type { TFunction } from "i18next";
import { deleteTemplate } from "@/lib/templates";
import type { EmailTemplate } from "@/lib/templates";
import { useConfirmStore } from "@/stores/confirm.store";
import type { ComposeAttachment } from "./compose-draft";

type AttachmentListProps = {
  attachments: ComposeAttachment[];
  onRemove: (index: number) => void;
  t: TFunction;
};

export function ComposeAttachmentList({ attachments, onRemove, t }: AttachmentListProps) {
  if (attachments.length === 0) return null;

  return (
    <div className="compose-inline-panel">
      {attachments.map((att, index) => (
        <div
          key={`${att.path}:${index}`}
          style={{
            display: "flex",
            alignItems: "center",
            gap: "4px",
            padding: "4px 8px",
            borderRadius: "4px",
            backgroundColor: "var(--color-bg-hover)",
            fontSize: "12px",
          }}
        >
          <FileText size={12} />
          <span
            style={{
              maxWidth: "150px",
              overflow: "hidden",
              textOverflow: "ellipsis",
              whiteSpace: "nowrap",
            }}
          >
            {att.name}
          </span>
          <span style={{ color: "var(--color-text-secondary)", fontSize: "11px" }}>
            {att.size < 1024 * 1024
              ? `${(att.size / 1024).toFixed(0)} KB`
              : `${(att.size / (1024 * 1024)).toFixed(1)} MB`}
          </span>
          <button
            type="button"
            onClick={() => onRemove(index)}
            aria-label={t("compose.removeAttachment", "Remove attachment {{name}}", {
              name: att.name,
            })}
            title={t("compose.removeAttachment", "Remove attachment {{name}}", {
              name: att.name,
            })}
            style={{
              border: "none",
              background: "none",
              cursor: "pointer",
              padding: "0 2px",
              color: "var(--color-text-secondary)",
            }}
          >
            <X size={12} />
          </button>
        </div>
      ))}
    </div>
  );
}

type TemplateMenuProps = {
  show: boolean;
  templates: EmailTemplate[];
  editor: Editor | null;
  onToggle: () => void;
  onRefresh: () => Promise<EmailTemplate[]>;
  onStartSave: () => void;
  onApply: (template: EmailTemplate) => void;
  t: TFunction;
};

export function TemplateMenu({
  show,
  templates,
  editor,
  onToggle,
  onRefresh,
  onStartSave,
  onApply,
  t,
}: TemplateMenuProps) {
  return (
    <div style={{ position: "relative" }}>
      <button
        type="button"
        onClick={() => {
          void onRefresh();
          onToggle();
        }}
        aria-haspopup="listbox"
        aria-expanded={show}
        aria-label={t("compose.templates", "Templates")}
        title={t("compose.templates", "Templates")}
        className={`compose-toolbar-icon-button${show ? " is-active" : ""}`}
      >
        <BookTemplate size={13} />
      </button>
      {show && (
        <div
          className="scroll-region compose-template-scroll"
          style={{
            position: "absolute",
            top: "100%",
            left: 0,
            zIndex: 100,
            backgroundColor: "var(--color-bg)",
            border: "1px solid var(--color-border)",
            borderRadius: "8px",
            boxShadow: "0 8px 24px rgba(0,0,0,0.12)",
            minWidth: "220px",
            maxHeight: "300px",
            overflowY: "auto",
          }}
        >
          <div
            style={{
              padding: "8px",
              borderBottom: "1px solid var(--color-border)",
              display: "flex",
              justifyContent: "space-between",
              alignItems: "center",
            }}
          >
            <span id="compose-templates-label" style={{ fontSize: "12px", fontWeight: 600 }}>
              {t("compose.templates", "Templates")}
            </span>
            <button
              type="button"
              onClick={onStartSave}
              style={{
                fontSize: "11px",
                border: "none",
                background: "none",
                cursor: "pointer",
                color: "var(--color-accent)",
              }}
            >
              {t("compose.saveAsTemplate", "Save current")}
            </button>
          </div>
          {templates.length === 0 ? (
            <div
              style={{
                padding: "16px",
                textAlign: "center",
                fontSize: "12px",
                color: "var(--color-text-secondary)",
              }}
            >
              {t("compose.noTemplates", "No templates saved")}
            </div>
          ) : (
            <ul
              role="listbox"
              aria-labelledby="compose-templates-label"
              style={{ listStyle: "none", margin: 0, padding: 0 }}
            >
              {templates.map((template) => {
                const applyTemplate = () => {
                  onApply(template);
                  if (editor) editor.commands.setContent(template.body);
                };
                return (
                  <li
                    key={template.id}
                    role="option"
                    aria-selected={false}
                    tabIndex={0}
                    onClick={applyTemplate}
                    onKeyDown={(e) => {
                      if (e.key === "Enter" || e.key === " ") {
                        e.preventDefault();
                        applyTemplate();
                      }
                    }}
                    style={{
                      display: "flex",
                      alignItems: "center",
                      padding: "8px",
                      borderBottom: "1px solid var(--color-border)",
                      cursor: "pointer",
                      fontSize: "12px",
                    }}
                  >
                    <div style={{ flex: 1, overflow: "hidden" }}>
                      <div style={{ fontWeight: 500 }}>{template.name}</div>
                      <div
                        style={{
                          color: "var(--color-text-secondary)",
                          whiteSpace: "nowrap",
                          overflow: "hidden",
                          textOverflow: "ellipsis",
                        }}
                      >
                        {template.subject}
                      </div>
                    </div>
                    <button
                      type="button"
                      onClick={async (e) => {
                        e.stopPropagation();
                        const confirmed = await useConfirmStore.getState().confirm({
                          title: t("compose.deleteTemplate", "Delete template"),
                          message:
                            t("compose.deleteTemplate", "Delete template") + ` "${template.name}"?`,
                          destructive: true,
                        });
                        if (confirmed) {
                          await deleteTemplate(template.id);
                          void onRefresh();
                        }
                      }}
                      aria-label={t("compose.deleteTemplate", "Delete template")}
                      title={t("compose.deleteTemplate", "Delete template")}
                      style={{
                        border: "none",
                        background: "none",
                        cursor: "pointer",
                        color: "var(--color-text-secondary)",
                        padding: "2px",
                      }}
                    >
                      <Trash2 size={12} />
                    </button>
                  </li>
                );
              })}
            </ul>
          )}
        </div>
      )}
    </div>
  );
}

type SaveTemplatePanelProps = {
  name: string;
  onNameChange: (value: string) => void;
  onSave: () => void;
  onCancel: () => void;
  t: TFunction;
};

export function SaveTemplatePanel({
  name,
  onNameChange,
  onSave,
  onCancel,
  t,
}: SaveTemplatePanelProps) {
  const inputRef = useRef<HTMLInputElement>(null);
  useEffect(() => {
    inputRef.current?.focus();
  }, []);
  return (
    <div className="compose-inline-panel">
      <input
        ref={inputRef}
        type="text"
        value={name}
        onChange={(e) => onNameChange(e.target.value)}
        placeholder={t("compose.templateName", "Template name")}
        style={{
          flex: 1,
          padding: "6px 8px",
          fontSize: "12px",
          border: "1px solid var(--color-border)",
          borderRadius: "4px",
          backgroundColor: "var(--color-bg)",
          color: "var(--color-text-primary)",
        }}
        onKeyDown={(e) => {
          if (e.key === "Enter" && name.trim()) {
            onSave();
          }
          if (e.key === "Escape") onCancel();
        }}
      />
      <button
        type="button"
        onClick={() => {
          if (!name.trim()) return;
          onSave();
        }}
        style={{
          padding: "5px 12px",
          fontSize: "12px",
          border: "none",
          borderRadius: "4px",
          backgroundColor: "var(--color-accent)",
          color: "#fff",
          cursor: "pointer",
        }}
      >
        {t("common.save")}
      </button>
      <button
        type="button"
        onClick={onCancel}
        style={{
          padding: "5px 8px",
          fontSize: "12px",
          border: "1px solid var(--color-border)",
          borderRadius: "4px",
          backgroundColor: "transparent",
          color: "var(--color-text-secondary)",
          cursor: "pointer",
        }}
      >
        {t("common.cancel")}
      </button>
    </div>
  );
}

type LeaveConfirmDialogProps = {
  onCancel: () => void;
  onConfirm: () => void;
  t: TFunction;
};

export function ComposeLeaveConfirmDialog({ onCancel, onConfirm, t }: LeaveConfirmDialogProps) {
  return (
    <div
      style={{
        position: "fixed",
        inset: 0,
        zIndex: 9999,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        backgroundColor: "rgba(0,0,0,0.4)",
      }}
      role="presentation"
    >
      <button
        type="button"
        aria-label={t("common.cancel")}
        onClick={onCancel}
        style={{
          position: "absolute",
          inset: 0,
          border: "none",
          background: "transparent",
          cursor: "default",
        }}
      />
      {/* role="dialog" + tabIndex is a WAI-ARIA interactive pattern; jsx-a11y 6.10.2 only recognizes a subset */}
      {/* eslint-disable-next-line jsx-a11y/no-noninteractive-element-interactions */}
      <div
        role="dialog"
        aria-modal="true"
        aria-labelledby="compose-leave-title"
        tabIndex={-1}
        ref={(el) => el?.focus()}
        onKeyDown={(e) => {
          if (e.key === "Escape") onCancel();
        }}
        style={{
          position: "relative",
          zIndex: 1,
          width: "380px",
          backgroundColor: "var(--color-sidebar-bg)",
          color: "var(--color-text-primary)",
          border: "1px solid var(--color-border)",
          borderRadius: "8px",
          padding: "24px",
          boxShadow: "0 20px 60px rgba(0,0,0,0.3)",
          display: "flex",
          flexDirection: "column" as const,
          gap: "16px",
        }}
      >
        <h3 id="compose-leave-title" style={{ margin: 0, fontSize: "15px", fontWeight: 600 }}>
          {t("compose.leaveTitle", "Discard draft?")}
        </h3>
        <p
          style={{
            margin: 0,
            fontSize: "13px",
            color: "var(--color-text-secondary)",
            lineHeight: 1.5,
          }}
        >
          {t("compose.leaveMessage", "You have unsaved changes. Are you sure you want to leave?")}
        </p>
        <div style={{ display: "flex", justifyContent: "flex-end", gap: "8px" }}>
          <button
            type="button"
            onClick={onCancel}
            style={{
              padding: "7px 16px",
              borderRadius: "6px",
              fontSize: "13px",
              border: "1px solid var(--color-border)",
              cursor: "pointer",
              backgroundColor: "transparent",
              color: "var(--color-text-primary)",
            }}
          >
            {t("compose.leaveCancel", "Keep editing")}
          </button>
          <button
            type="button"
            onClick={onConfirm}
            style={{
              padding: "7px 16px",
              borderRadius: "6px",
              fontSize: "13px",
              fontWeight: 600,
              border: "none",
              cursor: "pointer",
              backgroundColor: "#ef4444",
              color: "#fff",
            }}
          >
            {t("compose.leaveConfirm", "Discard")}
          </button>
        </div>
      </div>
    </div>
  );
}
