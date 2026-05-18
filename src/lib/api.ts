import * as client from "./api-client";

// Re-export all IPC types so existing `import { Foo } from "@/lib/api"` keeps working.
export type {
  Account,
  AccountProxyMode,
  AccountProxySetting,
  AddAccountRequest,
  AdvancedSearchQuery,
  AppLogSnapshot,
  Attachment,
  BackupPreview,
  ConnectionSecurity,
  EmailAddress,
  Folder,
  GmailRealtimeConfig,
  HttpProxyConfig,
  KanbanCard,
  KanbanColumnType,
  KnownContact,
  Label,
  Message,
  MessageSummary,
  PendingMailOp,
  PendingMailOpsSummary,
  PrivacyMode,
  RenderedHtml,
  Rule,
  SearchHit,
  SnoozedMessage,
  ThreadSummary,
  TranslateConfig,
  TranslateResult,
  TrustedSender,
} from "./ipc-types";

import type {
  Account,
  AccountProxyMode,
  AccountProxySetting,
  AddAccountRequest,
  AdvancedSearchQuery,
  AppLogSnapshot,
  Attachment,
  BackupPreview,
  ConnectionSecurity,
  Folder,
  GmailRealtimeConfig,
  HttpProxyConfig,
  KanbanCard,
  KanbanColumnType,
  KnownContact,
  Label,
  Message,
  MessageSummary,
  PendingMailOp,
  PendingMailOpsSummary,
  PrivacyMode,
  RenderedHtml,
  Rule,
  SearchHit,
  SnoozedMessage,
  ThreadSummary,
  TranslateConfig,
  TranslateResult,
  TrustedSender,
} from "./ipc-types";

// ─── Account API ─────────────────────────────────────────────────────────────

export async function healthCheck(): Promise<string> {
  return client.healthCheck();
}

export async function readAppLog(maxBytes: number): Promise<AppLogSnapshot> {
  return client.readAppLog(maxBytes) as Promise<AppLogSnapshot>;
}

export interface MailDisplayTiming {
  accountId?: string | null;
  messageId: string;
  source?: string | null;
  activeFolderId?: string | null;
  backendReceivedAtMs?: number | null;
  backendSseAtMs?: number | null;
  messageReceivedAtMs?: number | null;
  frontendSseAtMs: number;
  displayedAtMs: number;
  frontendSseToDisplayMs?: number | null;
}

export async function recordMailDisplayTiming(timing: MailDisplayTiming): Promise<void> {
  return client.recordMailDisplayTiming(timing);
}

export async function getGlobalProxy(): Promise<HttpProxyConfig | null> {
  return client.getGlobalProxy() as Promise<HttpProxyConfig | null>;
}

export async function getAccountProxy(accountId: string): Promise<HttpProxyConfig | null> {
  return client.getAccountProxy(accountId) as Promise<HttpProxyConfig | null>;
}

export async function getAccountProxySetting(accountId: string): Promise<AccountProxySetting> {
  return client.getAccountProxySetting(accountId) as Promise<AccountProxySetting>;
}

export async function updateAccountProxy(
  accountId: string,
  proxyHost?: string,
  proxyPort?: number,
): Promise<void> {
  return client.updateAccountProxy(accountId, proxyHost, proxyPort);
}

export async function updateAccountProxySetting(
  accountId: string,
  mode: AccountProxyMode,
  proxyHost?: string,
  proxyPort?: number,
): Promise<void> {
  return client.updateAccountProxySetting(accountId, mode, proxyHost, proxyPort);
}

export async function updateGlobalProxy(
  proxyHost?: string,
  proxyPort?: number,
): Promise<void> {
  return client.updateGlobalProxy(proxyHost, proxyPort);
}

/** @deprecated OAuth sign-in must start through `startOAuthLogin` and `/auth/login`. */
export async function completeOAuthFlow(
  _provider: string,
  _email: string,
  _displayName: string,
  _proxyHost?: string,
  _proxyPort?: number,
): Promise<Account> {
  // OAuth flow now handled entirely by /auth/login redirect + /auth/callback.
  throw new Error("completeOAuthFlow is deprecated; use startOAuthLogin + /auth/login redirect");
}

export function startOAuthLogin(
  provider: "gmail" | "outlook",
  proxyHost?: string,
  proxyPort?: number,
) {
  const params = new URLSearchParams({ provider });
  const trimmedProxyHost = proxyHost?.trim();
  if (trimmedProxyHost) {
    params.set("proxyHost", trimmedProxyHost);
  }
  if (proxyPort !== undefined) {
    params.set("proxyPort", String(proxyPort));
  }
  window.location.assign(`/auth/login?${params.toString()}`);
}

export async function getOAuthAccountProxy(accountId: string): Promise<HttpProxyConfig | null> {
  return client.getAccountProxy(accountId) as Promise<HttpProxyConfig | null>;
}

export async function getOAuthAccountProxySetting(accountId: string): Promise<AccountProxySetting> {
  return client.getAccountProxySetting(accountId) as Promise<AccountProxySetting>;
}

export async function updateOAuthAccountProxy(
  accountId: string,
  proxyHost?: string,
  proxyPort?: number,
): Promise<void> {
  return client.updateAccountProxy(accountId, proxyHost, proxyPort);
}

export async function updateOAuthAccountProxySetting(
  accountId: string,
  mode: AccountProxyMode,
  proxyHost?: string,
  proxyPort?: number,
): Promise<void> {
  return client.updateAccountProxySetting(accountId, mode, proxyHost, proxyPort);
}

export async function addAccount(request: AddAccountRequest): Promise<Account> {
  return client.addAccount(request) as Promise<Account>;
}

export async function testAccountConnection(accountId: string): Promise<string> {
  return client.testAccountConnection(accountId) as Promise<string>;
}

export async function testImapConnection(
  imapHost: string,
  imapPort: number,
  imapSecurity: ConnectionSecurity,
  proxyHost?: string,
  proxyPort?: number,
  username?: string,
  password?: string,
): Promise<string> {
  return client.testImapConnection({
    imap_host: imapHost, imap_port: imapPort, imap_security: imapSecurity,
    proxy_host: proxyHost, proxy_port: proxyPort, username, password,
  }) as Promise<string>;
}

export async function listAccounts(): Promise<Account[]> {
  return client.listAccounts() as Promise<Account[]>;
}

export async function updateAccount(
  accountId: string,
  email: string,
  displayName: string,
  password?: string,
  imapHost?: string,
  imapPort?: number,
  smtpHost?: string,
  smtpPort?: number,
  imapSecurity?: ConnectionSecurity,
  smtpSecurity?: ConnectionSecurity,
  proxyHost?: string,
  proxyPort?: number,
  accountColor?: string,
): Promise<void> {
  return client.updateAccount(accountId, {
    email, displayName, password,
    imapHost, imapPort, smtpHost, smtpPort,
    imapSecurity, smtpSecurity,
    proxyHost, proxyPort, accountColor,
  });
}

export async function deleteAccount(accountId: string): Promise<void> {
  return client.deleteAccount(accountId);
}

export async function getGmailRealtimeConfig(accountId: string): Promise<GmailRealtimeConfig> {
  return client.getGmailRealtimeConfig(accountId) as Promise<GmailRealtimeConfig>;
}

export async function enableGmailRealtime(
  accountId: string,
  fallbackIntervalMinutes?: number,
): Promise<GmailRealtimeConfig> {
  return client.enableGmailRealtime(accountId, fallbackIntervalMinutes) as Promise<GmailRealtimeConfig>;
}

export async function disableGmailRealtime(accountId: string): Promise<GmailRealtimeConfig> {
  return client.disableGmailRealtime(accountId) as Promise<GmailRealtimeConfig>;
}

export async function updateGmailRealtimeConfig(
  accountId: string,
  fallbackIntervalMinutes: number,
): Promise<GmailRealtimeConfig> {
  return client.updateGmailRealtimeConfig(accountId, fallbackIntervalMinutes) as Promise<GmailRealtimeConfig>;
}

// ─── Folder API ──────────────────────────────────────────────────────────────

export async function listFolders(accountId: string): Promise<Folder[]> {
  return client.listFolders(accountId) as Promise<Folder[]>;
}

// ─── Message API ─────────────────────────────────────────────────────────────

export async function listMessages(
  folderId: string,
  limit: number,
  offset: number,
  folderIds?: string[],
): Promise<MessageSummary[]> {
  return client.getInbox({ folderId, limit, offset, folderIds }) as Promise<MessageSummary[]>;
}

export async function listStarredMessages(
  accountId: string,
  limit: number,
  offset: number,
): Promise<MessageSummary[]> {
  return client.getStarred(accountId, limit, offset) as Promise<MessageSummary[]>;
}

export async function getMessage(messageId: string): Promise<Message | null> {
  return client.getMessage(messageId) as Promise<Message | null>;
}

/** Batch-fetch multiple messages in a single IPC call. */
export async function getMessagesBatch(messageIds: string[]): Promise<Message[]> {
  return client.getMessagesBatch(messageIds) as Promise<Message[]>;
}

export async function getRenderedHtml(
  messageId: string,
  privacyMode: PrivacyMode,
): Promise<RenderedHtml> {
  return client.getRenderedHtml(messageId, privacyMode as string) as Promise<RenderedHtml>;
}

/** Single IPC call that returns both Message and RenderedHtml. */
export async function getMessageWithHtml(
  messageId: string,
  privacyMode: PrivacyMode,
): Promise<[Message, RenderedHtml] | null> {
  return client.getMessageWithHtml(messageId, privacyMode as string) as Promise<[Message, RenderedHtml] | null>;
}

export async function updateMessageFlags(
  messageId: string,
  isRead?: boolean,
  isStarred?: boolean,
): Promise<void> {
  return client.updateMessageFlags(messageId, isRead, isStarred);
}

// Rapid-toggle guard: archive_message is toggle-based (archive ⇄ unarchive),
// so a double-click would flip the state back. This Set coalesces concurrent
// calls per-message; it is NOT idempotency — a second click *after* the first
// resolves is intentionally allowed to unarchive.
const archivingIds = new Set<string>();

export async function archiveMessage(messageId: string): Promise<string> {
  if (archivingIds.has(messageId)) {
    return "skipped";
  }
  archivingIds.add(messageId);
  try {
    const res = await client.archiveMessage(messageId) as { targetFolder?: string };
    return res.targetFolder ?? "archived";
  } finally {
    archivingIds.delete(messageId);
  }
}

export async function deleteMessage(messageId: string): Promise<void> {
  return client.deleteMessage(messageId);
}

export async function restoreMessage(messageId: string): Promise<void> {
  return client.restoreMessage(messageId);
}

export async function moveToFolder(messageId: string, targetFolderId: string): Promise<void> {
  return client.moveToFolder(messageId, targetFolderId);
}

export async function emptyTrash(accountId: string): Promise<number> {
  return client.emptyTrash(accountId) as Promise<number>;
}

export async function getPendingMailOpsSummary(
  accountId: string | null,
): Promise<PendingMailOpsSummary> {
  return client.getPendingMailOpsSummary(accountId) as Promise<PendingMailOpsSummary>;
}

export async function listPendingMailOps(
  accountId: string | null,
  limit = 100,
): Promise<PendingMailOp[]> {
  return client.listPendingMailOps(accountId, limit) as Promise<PendingMailOp[]>;
}

export async function cancelPendingMailOp(id: string): Promise<void> {
  return client.cancelPendingMailOp(id);
}

export async function deletePendingMailOp(id: string): Promise<void> {
  return client.deletePendingMailOp(id);
}

// ─── Trusted Senders API ────────────────────────────────────────────────────

export async function listTrustedSenders(accountId: string): Promise<TrustedSender[]> {
  return client.listTrustedSenders(accountId) as Promise<TrustedSender[]>;
}

export async function removeTrustedSender(accountId: string, email: string): Promise<void> {
  return client.removeTrustedSender(accountId, email);
}

export async function trustSender(accountId: string, email: string, trustType: "images" | "all"): Promise<void> {
  return client.trustSender(accountId, email, trustType);
}

export async function isTrustedSender(accountId: string, email: string): Promise<boolean> {
  return client.isTrustedSender(accountId, email);
}

// ─── Search API ──────────────────────────────────────────────────────────────

export async function searchMessages(
  query: string,
  limit?: number,
): Promise<SearchHit[]> {
  return client.searchMessages(query, limit) as Promise<SearchHit[]>;
}

export async function advancedSearch(
  query: AdvancedSearchQuery,
  limit?: number,
): Promise<SearchHit[]> {
  return client.advancedSearch(query, limit) as Promise<SearchHit[]>;
}

// ─── Sync API ────────────────────────────────────────────────────────────────

export async function startSync(accountId: string, pollIntervalSecs?: number): Promise<string> {
  return client.startSync(accountId, pollIntervalSecs) as Promise<string>;
}

export async function triggerSync(accountId: string, reason: string): Promise<void> {
  return client.triggerSync(accountId, reason);
}

export type RealtimePreference = "realtime" | "balanced" | "battery" | "manual";

export async function setRealtimePreference(mode: RealtimePreference): Promise<void> {
  return client.setRealtimePreference(mode);
}

export async function setNotificationsEnabled(enabled: boolean): Promise<void> {
  return client.setNotificationsEnabled(enabled);
}

export async function stopSync(accountId: string): Promise<void> {
  return client.stopSync(accountId);
}

// ─── Attachment API ──────────────────────────────────────────────────────────

export async function listAttachments(messageId: string): Promise<Attachment[]> {
  return client.listAttachments(messageId) as Promise<Attachment[]>;
}

export async function getAttachmentPath(attachmentId: string): Promise<string | null> {
  return client.getAttachmentDownloadUrl(attachmentId);
}

export async function downloadAttachment(_attachmentId: string, _saveTo: string): Promise<string> {
  // Browser handles download natively via anchor click on the download URL.
  return "downloaded";
}

// ─── Kanban API ──────────────────────────────────────────────────────────────

export async function moveToKanban(messageId: string, column: KanbanColumnType, position?: number): Promise<void> {
  return client.moveToKanban(messageId, column, position);
}

export async function listKanbanCards(column?: KanbanColumnType): Promise<KanbanCard[]> {
  return client.getKanban(column) as Promise<KanbanCard[]>;
}

export async function removeFromKanban(messageId: string): Promise<void> {
  return client.removeFromKanban(messageId);
}

export async function listKanbanContextNotes(): Promise<Record<string, string>> {
  return client.listKanbanContextNotes();
}

export async function setKanbanContextNote(
  messageId: string,
  note: string,
): Promise<Record<string, string>> {
  return client.setKanbanContextNote(messageId, note);
}

export async function mergeKanbanContextNotes(
  notes: Record<string, string>,
): Promise<Record<string, string>> {
  return client.mergeKanbanContextNotes(notes);
}

// ─── Snooze API ──────────────────────────────────────────────────────────────

export async function snoozeMessage(messageId: string, until: number, returnTo: string): Promise<void> {
  return client.snoozeMessage(messageId, until, returnTo);
}

export async function unsnoozeMessage(messageId: string): Promise<void> {
  return client.unsnoozeMessage(messageId);
}

export async function listSnoozed(): Promise<SnoozedMessage[]> {
  return client.getSnoozed() as Promise<SnoozedMessage[]>;
}

// ─── Rules API ───────────────────────────────────────────────────────────────

export async function createRule(name: string, priority: number, conditions: string, actions: string): Promise<Rule> {
  return client.createRule(name, priority, conditions, actions) as Promise<Rule>;
}

export async function listRules(): Promise<Rule[]> {
  return client.listRules() as Promise<Rule[]>;
}

export async function updateRule(rule: Rule): Promise<void> {
  return client.updateRule(rule);
}

export async function deleteRule(ruleId: string): Promise<void> {
  return client.deleteRule(ruleId);
}

// ─── Compose API ─────────────────────────────────────────────────────────────

export async function sendEmail(
  accountId: string,
  to: string[],
  cc: string[],
  bcc: string[],
  subject: string,
  bodyText: string,
  bodyHtml?: string,
  inReplyTo?: string,
  attachmentPaths?: string[],
): Promise<void> {
  return client.sendEmail({
    accountId, to, cc, bcc, subject, bodyText, bodyHtml, inReplyTo, attachmentPaths,
  });
}

export async function stageComposeAttachment(filename: string, bytes: number[]): Promise<string> {
  // Convert bytes to File for FormData upload
  const blob = new Blob([new Uint8Array(bytes)]);
  const file = new File([blob], filename);
  return client.stageAttachment(file);
}

// ─── Batch Operations ───────────────────────────────────────────────────────

export async function batchArchive(messageIds: string[]): Promise<number> {
  return client.batchArchive(messageIds) as Promise<number>;
}

export async function batchDelete(messageIds: string[]): Promise<number> {
  return client.batchDelete(messageIds) as Promise<number>;
}

export async function batchMarkRead(messageIds: string[], isRead: boolean): Promise<number> {
  return client.batchMarkRead(messageIds, isRead) as Promise<number>;
}

export async function batchStar(messageIds: string[], starred: boolean): Promise<number> {
  return client.batchStar(messageIds, starred) as Promise<number>;
}

// ─── Translate API ───────────────────────────────────────────────────────────

export async function translateText(text: string, fromLang: string, toLang: string): Promise<TranslateResult> {
  return client.translateText(text, fromLang, toLang) as Promise<TranslateResult>;
}

export async function getTranslateConfig(): Promise<TranslateConfig | null> {
  return client.getTranslateConfig() as Promise<TranslateConfig | null>;
}

export async function saveTranslateConfig(providerType: string, config: string, isEnabled: boolean): Promise<void> {
  return client.saveTranslateConfig(providerType, config, isEnabled);
}

export async function testTranslateConnection(config: string): Promise<string> {
  return client.testTranslateConnection(config) as Promise<string>;
}

// ─── Thread API ──────────────────────────────────────────────────────────────

export async function listThreads(
  folderId: string,
  limit: number,
  offset: number,
  folderIds?: string[],
): Promise<ThreadSummary[]> {
  return client.listThreads(folderId, limit, offset, folderIds) as Promise<ThreadSummary[]>;
}

export async function listThreadMessages(threadId: string): Promise<Message[]> {
  return client.listThreadMessages(threadId) as Promise<Message[]>;
}

// ─── Labels API ──────────────────────────────────────────────────────────────

export async function getMessageLabels(messageId: string): Promise<Label[]> {
  return client.getMessageLabels(messageId) as Promise<Label[]>;
}

export async function getMessageLabelsBatch(messageIds: string[]): Promise<Record<string, Label[]>> {
  return client.getMessageLabelsBatch(messageIds) as Promise<Record<string, Label[]>>;
}

export async function addMessageLabel(messageId: string, labelName: string): Promise<void> {
  return client.addMessageLabel(messageId, labelName);
}

export async function removeMessageLabel(messageId: string, labelName: string): Promise<void> {
  return client.removeMessageLabel(messageId, labelName);
}

export async function listLabels(): Promise<Label[]> {
  return client.listLabels() as Promise<Label[]>;
}

// ─── Cloud Sync API ─────────────────────────────────────────────────────────

export async function testWebdavConnection(url: string, username: string, password: string): Promise<string> {
  return client.testWebdavConnection(url, username, password) as Promise<string>;
}

export async function backupToWebdav(url: string, username: string, password: string): Promise<string> {
  return client.backupToWebdav(url, username, password) as Promise<string>;
}

export async function previewWebdavBackup(url: string, username: string, password: string): Promise<BackupPreview> {
  return client.previewWebdavBackup(url, username, password) as Promise<BackupPreview>;
}

export async function restoreFromWebdav(url: string, username: string, password: string): Promise<string> {
  return client.restoreFromWebdav(url, username, password) as Promise<string>;
}

// ─── Contacts API ────────────────────────────────────────────────────────────

export async function searchContacts(
  accountId: string,
  query: string,
  limit?: number,
): Promise<KnownContact[]> {
  return client.searchContacts(accountId, query, limit) as Promise<KnownContact[]>;
}

// ─── Drafts API ──────────────────────────────────────────────────────────────

export async function saveDraft(args: {
  accountId: string;
  to: string[];
  cc: string[];
  bcc: string[];
  subject: string;
  bodyText: string;
  bodyHtml?: string;
  inReplyTo?: string;
  existingDraftId?: string;
  attachmentPaths?: string[];
}): Promise<string> {
  const result = await client.saveDraft(args);
  return result.draftId;
}

export async function deleteDraft(accountId: string, draftId: string): Promise<void> {
  return client.deleteDraft(accountId, draftId);
}

// ─── Folder Counts API ───────────────────────────────────────────────────────

export async function getFolderUnreadCounts(accountId: string): Promise<Record<string, number>> {
  const shell = await client.getShell();
  return (shell.unreadCounts as Record<string, Record<string, number>>)[accountId] || {};
}
