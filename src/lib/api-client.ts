// Typed HTTP API client — replaces the RPC invoke() pattern.
// Each function wraps a standard fetch() call to the /api endpoints.
// In Phase 3+, this expands to cover all read/mutation endpoints.

const BASE = '/api';

export class ApiError extends Error {
  constructor(
    public status: number,
    public body: { error: string },
  ) {
    super(body.error);
    this.name = 'ApiError';
  }
}

async function request<T>(
  method: string,
  path: string,
  body?: unknown,
): Promise<T> {
  const url = new URL(path, window.location.origin);
  const init: RequestInit = {
    method,
    headers: { 'Content-Type': 'application/json' },
    credentials: 'same-origin',
  };
  if (body !== undefined) {
    init.body = JSON.stringify(body);
  }
  const res = await fetch(url.toString(), init);
  if (!res.ok) {
    const errBody = await res.json().catch(() => ({ error: res.statusText }));
    throw new ApiError(res.status, errBody);
  }
  return res.json();
}

export function apiGet<T>(path: string): Promise<T> {
  return request<T>('GET', path);
}

export function apiPost<T>(path: string, body?: unknown): Promise<T> {
  return request<T>('POST', path, body);
}

export function apiPatch<T>(path: string, body?: unknown): Promise<T> {
  return request<T>('PATCH', path, body);
}

export function apiDelete<T>(path: string): Promise<T> {
  return request<T>('DELETE', path);
}

// ── Auth ─────────────────────────────────────────────────────────────

export interface AuthStatus { authenticated: boolean }

export function login(password: string): Promise<AuthStatus> {
  return apiPost<AuthStatus>(`${BASE}/auth/login`, { password });
}

export function logout(): Promise<AuthStatus> {
  return apiPost<AuthStatus>(`${BASE}/auth/logout`);
}

export function getAuthStatus(): Promise<AuthStatus> {
  return apiGet<AuthStatus>(`${BASE}/auth/status`);
}

// ── Health ────────────────────────────────────────────────────────────

export function healthCheck(): Promise<string> {
  return apiGet<string>(`${BASE}/health`);
}

// ── Shell (composite) ─────────────────────────────────────────────────

export interface ShellData {
  accounts: unknown[];
  folders: Record<string, unknown[]>;
  unreadCounts: Record<string, Record<string, number>>;
}

export function getShell(): Promise<ShellData> {
  return apiGet<ShellData>(`${BASE}/shell`);
}

// ── Messages (reads) ──────────────────────────────────────────────────

export interface InboxParams {
  accountId: string;
  folderId: string;
  limit?: number;
  offset?: number;
  folderIds?: string[];
}

export function getInbox(params: InboxParams) {
  const qs = new URLSearchParams({ accountId: params.accountId, folderId: params.folderId });
  if (params.limit) qs.set('limit', String(params.limit));
  if (params.offset) qs.set('offset', String(params.offset));
  if (params.folderIds?.length) qs.set('folderIds', params.folderIds.join(','));
  return apiGet<{ messages: unknown[]; total: number; hasMore: boolean }>(
    `${BASE}/inbox?${qs}`,
  ).then(r => r.messages);
}

export function getStarred(accountId: string, limit?: number, offset?: number) {
  const qs = new URLSearchParams({ accountId });
  if (limit) qs.set('limit', String(limit));
  if (offset) qs.set('offset', String(offset));
  return apiGet<{ messages: unknown[]; total: number; hasMore: boolean }>(
    `${BASE}/starred?${qs}`,
  ).then(r => r.messages);
}

export function getMessage(id: string) {
  return apiGet<unknown>(`${BASE}/messages/${encodeURIComponent(id)}`);
}

export function getMessagesBatch(messageIds: string[]) {
  return apiPost<unknown[]>(`${BASE}/messages/batch`, { messageIds });
}

// ── Threads ───────────────────────────────────────────────────────────

export function listThreadMessages(threadId: string) {
  return apiGet<unknown[]>(`${BASE}/threads/${encodeURIComponent(threadId)}/messages`);
}

// ── Search ────────────────────────────────────────────────────────────

export function searchMessages(q: string, limit?: number) {
  const qs = new URLSearchParams({ q });
  if (limit) qs.set('limit', String(limit));
  return apiGet<{ hits: unknown[]; total: number }>(`${BASE}/search?${qs}`).then(r => r.hits);
}

// ── Kanban ────────────────────────────────────────────────────────────

export function getKanban(column?: string) {
  const qs = column ? `?column=${encodeURIComponent(column)}` : '';
  return apiGet<{ cards: unknown[]; notes: Record<string, string> }>(`${BASE}/kanban${qs}`).then(r => r.cards);
}

// ── Snooze ────────────────────────────────────────────────────────────

export function getSnoozed() {
  return apiGet<unknown[]>(`${BASE}/snoozed`);
}

// ── Accounts ───────────────────────────────────────────────────────────

export function listAccounts() {
  return apiGet<unknown[]>(`${BASE}/accounts`);
}

// ── Folders ────────────────────────────────────────────────────────────

export function listFolders(accountId: string) {
  return apiGet<unknown[]>(`${BASE}/accounts/${encodeURIComponent(accountId)}/folders`);
}

// ── Messages (detail views) ────────────────────────────────────────────

export function getRenderedHtml(messageId: string, privacyMode: string) {
  const qs = new URLSearchParams({ privacyMode });
  return apiGet<unknown>(`${BASE}/messages/${encodeURIComponent(messageId)}/html?${qs}`);
}

export function getMessageWithHtml(messageId: string, privacyMode: string) {
  const qs = new URLSearchParams({ privacyMode });
  return apiGet<unknown>(`${BASE}/messages/${encodeURIComponent(messageId)}/full?${qs}`);
}

// ── Messages (mutations) ───────────────────────────────────────────────

export function updateMessageFlags(messageId: string, isRead?: boolean, isStarred?: boolean) {
  return apiPatch<void>(`${BASE}/messages/${encodeURIComponent(messageId)}/flags`, { isRead, isStarred });
}

export function archiveMessage(messageId: string) {
  return apiPost<unknown>(`${BASE}/messages/${encodeURIComponent(messageId)}/archive`);
}

export function deleteMessage(messageId: string) {
  return apiDelete<void>(`${BASE}/messages/${encodeURIComponent(messageId)}`);
}

export function restoreMessage(messageId: string) {
  return apiPost<void>(`${BASE}/messages/${encodeURIComponent(messageId)}/restore`);
}

export function moveToFolder(messageId: string, targetFolderId: string) {
  return apiPost<void>(`${BASE}/messages/${encodeURIComponent(messageId)}/move`, { targetFolderId });
}

export function emptyTrash(accountId: string) {
  return apiDelete<number>(`${BASE}/accounts/${encodeURIComponent(accountId)}/trash`);
}

// ── Batch mutations ────────────────────────────────────────────────────

export function batchArchive(messageIds: string[]) {
  return apiPost<number>(`${BASE}/messages/batch/archive`, { messageIds });
}

export function batchDelete(messageIds: string[]) {
  return apiPost<number>(`${BASE}/messages/batch/delete`, { messageIds });
}

export function batchMarkRead(messageIds: string[], isRead: boolean) {
  return apiPost<number>(`${BASE}/messages/batch/read`, { messageIds, isRead });
}

export function batchStar(messageIds: string[], starred: boolean) {
  return apiPost<number>(`${BASE}/messages/batch/star`, { messageIds, starred });
}

// ── Labels ─────────────────────────────────────────────────────────────

export function listLabels() {
  return apiGet<unknown[]>(`${BASE}/labels`);
}

export function getMessageLabels(messageId: string) {
  return apiGet<unknown[]>(`${BASE}/messages/${encodeURIComponent(messageId)}/labels`);
}

export function getMessageLabelsBatch(messageIds: string[]) {
  return apiPost<Record<string, unknown[]>>(`${BASE}/messages/batch/labels`, { messageIds });
}

export function addMessageLabel(messageId: string, labelName: string) {
  return apiPost<void>(`${BASE}/messages/${encodeURIComponent(messageId)}/labels`, { labelName });
}

export function removeMessageLabel(messageId: string, labelName: string) {
  return apiDelete<void>(`${BASE}/messages/${encodeURIComponent(messageId)}/labels/${encodeURIComponent(labelName)}`);
}

// ── Snooze mutations ───────────────────────────────────────────────────

export function snoozeMessage(messageId: string, until: number, returnTo: string) {
  return apiPost<void>(`${BASE}/snoozed`, { messageId, until, returnTo });
}

export function unsnoozeMessage(messageId: string) {
  return apiDelete<void>(`${BASE}/snoozed/${encodeURIComponent(messageId)}`);
}

// ── Advanced Search ────────────────────────────────────────────────────

export function advancedSearch(query: unknown, limit?: number) {
  return apiPost<{ hits: unknown[]; total: number }>(`${BASE}/search/advanced`, { query, limit }).then(r => r.hits);
}

// ── Pending Ops ────────────────────────────────────────────────────────

export function getPendingMailOpsSummary(accountId: string | null) {
  const qs = accountId ? `?accountId=${encodeURIComponent(accountId)}` : '';
  return apiGet<unknown>(`${BASE}/pending-ops/summary${qs}`);
}

export function listPendingMailOps(accountId: string | null, limit?: number) {
  const qs = new URLSearchParams();
  if (accountId) qs.set('accountId', accountId);
  if (limit) qs.set('limit', String(limit));
  return apiGet<unknown[]>(`${BASE}/pending-ops?${qs}`);
}

export function cancelPendingMailOp(id: string) {
  return apiPost<void>(`${BASE}/pending-ops/${encodeURIComponent(id)}/cancel`);
}

export function deletePendingMailOp(id: string) {
  return apiDelete<void>(`${BASE}/pending-ops/${encodeURIComponent(id)}`);
}

// ── Kanban mutations ───────────────────────────────────────────────────

export function moveToKanban(messageId: string, column: string, position?: number) {
  return apiPost<void>(`${BASE}/kanban/cards`, { messageId, column, position });
}

export function removeFromKanban(messageId: string) {
  return apiDelete<void>(`${BASE}/kanban/cards/${encodeURIComponent(messageId)}`);
}

export function setKanbanContextNote(messageId: string, note: string) {
  return apiPut<Record<string, string>>(`${BASE}/kanban/notes/${encodeURIComponent(messageId)}`, { note });
}

export function mergeKanbanContextNotes(notes: Record<string, string>) {
  return apiPatch<Record<string, string>>(`${BASE}/kanban/notes`, { notes });
}

export function listKanbanContextNotes() {
  return apiGet<Record<string, string>>(`${BASE}/kanban/notes`);
}

// ── Trusted Senders ────────────────────────────────────────────────────

export function listTrustedSenders(accountId: string) {
  return apiGet<unknown[]>(`${BASE}/trusted-senders?accountId=${encodeURIComponent(accountId)}`);
}

export function trustSender(accountId: string, email: string, trustType: string) {
  return apiPost<void>(`${BASE}/trusted-senders`, { accountId, email, trustType });
}

export function removeTrustedSender(accountId: string, email: string) {
  return apiDelete<void>(`${BASE}/trusted-senders?accountId=${encodeURIComponent(accountId)}&email=${encodeURIComponent(email)}`);
}

export function isTrustedSender(accountId: string, email: string) {
  return apiGet<boolean>(`${BASE}/trusted-senders/check?accountId=${encodeURIComponent(accountId)}&email=${encodeURIComponent(email)}`);
}

// ── Helpers ────────────────────────────────────────────────────────────

export function apiPut<T>(path: string, body?: unknown): Promise<T> {
  return request<T>('PUT', path, body);
}

// ── Compose / Send ────────────────────────────────────────────────────

export function sendEmail(params: {
  accountId: string;
  to: string[];
  cc: string[];
  bcc: string[];
  subject: string;
  bodyText: string;
  bodyHtml?: string;
  inReplyTo?: string;
  attachmentPaths?: string[];
  draftId?: string;
}) {
  return apiPost<void>(`${BASE}/messages/send`, params);
}

// ── Drafts ─────────────────────────────────────────────────────────────

export function saveDraft(params: {
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
}) {
  return apiPost<{ draftId: string }>(`${BASE}/drafts`, params);
}

export function deleteDraft(accountId: string, draftId: string) {
  return apiDelete<void>(`${BASE}/drafts/${encodeURIComponent(draftId)}?accountId=${encodeURIComponent(accountId)}`);
}

// ── Attachments ────────────────────────────────────────────────────────

export function listAttachments(messageId: string) {
  return apiGet<unknown[]>(`${BASE}/messages/${encodeURIComponent(messageId)}/attachments`);
}

export function getAttachmentDownloadUrl(attachmentId: string) {
  return `${BASE}/attachments/${encodeURIComponent(attachmentId)}`;
}

export async function stageAttachment(file: File): Promise<string> {
  const formData = new FormData();
  formData.append('file', file);
  const url = new URL(`${BASE}/attachments/stage`, window.location.origin);
  const res = await fetch(url.toString(), {
    method: 'POST',
    credentials: 'same-origin',
    body: formData,
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }));
    throw new ApiError(res.status, err);
  }
  const data = await res.json();
  return (data as { path: string }).path;
}

// ── Contacts ───────────────────────────────────────────────────────────

export function searchContacts(accountId: string, query: string, limit?: number) {
  const qs = new URLSearchParams({ accountId, q: query });
  if (limit) qs.set('limit', String(limit));
  return apiGet<unknown[]>(`${BASE}/contacts?${qs}`);
}

// ── Templates ──────────────────────────────────────────────────────────

export function listEmailTemplates() {
  return apiGet<unknown[]>(`${BASE}/templates`);
}

export function saveEmailTemplate(template: unknown) {
  return apiPost<unknown>(`${BASE}/templates`, template);
}

export function deleteEmailTemplate(id: string) {
  return apiDelete<void>(`${BASE}/templates/${encodeURIComponent(id)}`);
}

// ── Signatures ─────────────────────────────────────────────────────────

export function getEmailSignature(accountId: string) {
  return apiGet<{ signature: string }>(`${BASE}/accounts/${encodeURIComponent(accountId)}/signature`);
}

export function setEmailSignature(accountId: string, signature: string) {
  return apiPut<void>(`${BASE}/accounts/${encodeURIComponent(accountId)}/signature`, { signature });
}

// ── Rules ──────────────────────────────────────────────────────────────

export function listRules() {
  return apiGet<unknown[]>(`${BASE}/rules`);
}

export function createRule(name: string, priority: number, conditions: string, actions: string) {
  return apiPost<unknown>(`${BASE}/rules`, { name, priority, conditions, actions });
}

export function updateRule(rule: unknown) {
  return apiPut<void>(`${BASE}/rules/${(rule as { id: string }).id}`, rule);
}

export function deleteRule(ruleId: string) {
  return apiDelete<void>(`${BASE}/rules/${encodeURIComponent(ruleId)}`);
}

// ── Translate ──────────────────────────────────────────────────────────

export function translateText(text: string, fromLang: string, toLang: string) {
  return apiPost<unknown>(`${BASE}/translate`, { text, fromLang, toLang });
}

export function getTranslateConfig() {
  return apiGet<unknown>(`${BASE}/translate/config`);
}

export function saveTranslateConfig(providerType: string, config: string, isEnabled: boolean) {
  return apiPut<void>(`${BASE}/translate/config`, { providerType, config, isEnabled });
}

export function testTranslateConnection(config: string) {
  return apiPost<string>(`${BASE}/translate/test`, { config });
}

// ── Cloud Sync (WebDAV) ────────────────────────────────────────────────

export function testWebdavConnection(url: string, username: string, password: string) {
  return apiPost<string>(`${BASE}/cloud-sync/webdav/test`, { url, username, password });
}

export function backupToWebdav(url: string, username: string, password: string) {
  return apiPost<string>(`${BASE}/cloud-sync/webdav/backup`, { url, username, password });
}

export function previewWebdavBackup(url: string, username: string, password: string) {
  return apiPost<unknown>(`${BASE}/cloud-sync/webdav/preview`, { url, username, password });
}

export function restoreFromWebdav(url: string, username: string, password: string) {
  return apiPost<string>(`${BASE}/cloud-sync/webdav/restore`, { url, username, password });
}

// ── Diagnostics ────────────────────────────────────────────────────────

export function readAppLog(maxBytes: number) {
  const qs = new URLSearchParams({ maxBytes: String(maxBytes) });
  return apiGet<unknown>(`${BASE}/logs?${qs}`);
}

export function recordMailDisplayTiming(timing: unknown) {
  return apiPost<void>(`${BASE}/diagnostics/mail-timing`, timing);
}

// ── Proxy ──────────────────────────────────────────────────────────────

export function getGlobalProxy() {
  return apiGet<unknown>(`${BASE}/proxy`);
}

export function updateGlobalProxy(proxyHost?: string, proxyPort?: number) {
  return apiPut<void>(`${BASE}/proxy`, { proxyHost, proxyPort });
}

// ── Preferences ────────────────────────────────────────────────────────

export function setRealtimePreference(mode: string) {
  return apiPut<void>(`${BASE}/preferences/realtime`, { mode });
}

export function setNotificationsEnabled(enabled: boolean) {
  return apiPut<void>(`${BASE}/preferences/notifications`, { enabled });
}

// ── Accounts management ────────────────────────────────────────────────

export function addAccount(request: unknown) {
  return apiPost<unknown>(`${BASE}/accounts`, request);
}

export function updateAccount(accountId: string, body: Record<string, unknown>) {
  return apiPatch<void>(`${BASE}/accounts/${encodeURIComponent(accountId)}`, body);
}

export function deleteAccount(accountId: string) {
  return apiDelete<void>(`${BASE}/accounts/${encodeURIComponent(accountId)}`);
}

export function testAccountConnection(accountId: string) {
  return apiPost<string>(`${BASE}/accounts/${encodeURIComponent(accountId)}/test-connection`);
}

export function testImapConnection(request: unknown) {
  return apiPost<string>(`${BASE}/imap/test-connection`, request);
}

// ── Account proxy ──────────────────────────────────────────────────────

export function getAccountProxy(accountId: string) {
  return apiGet<unknown>(`${BASE}/accounts/${encodeURIComponent(accountId)}/proxy`);
}

export function updateAccountProxy(accountId: string, proxyHost?: string, proxyPort?: number) {
  return apiPut<void>(`${BASE}/accounts/${encodeURIComponent(accountId)}/proxy`, { proxyHost, proxyPort });
}

export function getAccountProxySetting(accountId: string) {
  return apiGet<unknown>(`${BASE}/accounts/${encodeURIComponent(accountId)}/proxy-setting`);
}

export function updateAccountProxySetting(accountId: string, mode: string, proxyHost?: string, proxyPort?: number) {
  return apiPut<void>(`${BASE}/accounts/${encodeURIComponent(accountId)}/proxy-setting`, { mode, proxyHost, proxyPort });
}

// ── Sync commands ──────────────────────────────────────────────────────

export function startSync(accountId: string, pollIntervalSecs?: number) {
  return apiPost<string>(`${BASE}/accounts/${encodeURIComponent(accountId)}/sync/start`, { pollIntervalSecs });
}

export function triggerSync(accountId: string, reason: string) {
  return apiPost<void>(`${BASE}/accounts/${encodeURIComponent(accountId)}/sync/trigger`, { reason });
}

export function stopSync(accountId: string) {
  return apiPost<void>(`${BASE}/accounts/${encodeURIComponent(accountId)}/sync/stop`);
}

// ── Gmail realtime ─────────────────────────────────────────────────────

export function getGmailRealtimeConfig(accountId: string) {
  return apiGet<unknown>(`${BASE}/accounts/${encodeURIComponent(accountId)}/gmail-realtime`);
}

export function enableGmailRealtime(accountId: string, fallbackIntervalMinutes?: number) {
  return apiPost<unknown>(`${BASE}/accounts/${encodeURIComponent(accountId)}/gmail-realtime/enable`, { fallbackIntervalMinutes });
}

export function disableGmailRealtime(accountId: string) {
  return apiPost<unknown>(`${BASE}/accounts/${encodeURIComponent(accountId)}/gmail-realtime/disable`);
}

export function updateGmailRealtimeConfig(accountId: string, fallbackIntervalMinutes: number) {
  return apiPut<unknown>(`${BASE}/accounts/${encodeURIComponent(accountId)}/gmail-realtime`, { fallbackIntervalMinutes });
}
