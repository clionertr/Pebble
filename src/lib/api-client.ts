// 类型化 HTTP API 客户端：前端统一通过 REST 访问后端。
// Each function wraps a standard fetch() call to the /api endpoints.
// In Phase 3+, this expands to cover all read/mutation endpoints.

import type { Account, Folder, GmailRealtimeConfig } from "./api-types";

const BASE = "/api";

export class ApiError extends Error {
  constructor(
    public status: number,
    public body: { error: string },
  ) {
    super(body.error);
    this.name = "ApiError";
  }
}

async function apiErrorFromResponse(res: Response): Promise<ApiError> {
  const errBody = await res.json().catch(() => ({ error: res.statusText }));
  return new ApiError(res.status, errBody);
}

async function request<T>(
  method: string,
  path: string,
  body?: unknown,
  signal?: AbortSignal,
): Promise<T> {
  const url = new URL(path, window.location.origin);
  const init: RequestInit = {
    method,
    headers: { "Content-Type": "application/json" },
    credentials: "same-origin",
  };
  if (body !== undefined) {
    init.body = JSON.stringify(body);
  }
  if (signal) {
    init.signal = signal;
  }
  const res = await fetch(url.toString(), init);
  if (!res.ok) {
    throw await apiErrorFromResponse(res);
  }
  return res.json();
}

export interface TranslateStreamOptions {
  onDelta?: (translated: string) => void;
  signal?: AbortSignal;
}

export interface ServerSentEventBlock {
  event: string;
  data: string;
}

export function parseServerSentEventBlock(block: string): ServerSentEventBlock | null {
  const data: string[] = [];
  let event = "message";

  for (const line of block.split(/\r?\n/)) {
    if (line.startsWith("event:")) {
      event = line.slice(6).trim();
    } else if (line.startsWith("data:")) {
      data.push(line.slice(5).trimStart());
    }
  }

  if (data.length === 0) return null;
  return { event, data: data.join("\n") };
}

export function parseServerSentEvents(buffer: string): {
  events: ServerSentEventBlock[];
  rest: string;
} {
  const blocks = buffer.split(/\r?\n\r?\n/);
  const rest = blocks.pop() ?? "";
  const events = blocks
    .map(parseServerSentEventBlock)
    .filter((event): event is ServerSentEventBlock => event !== null);
  return { events, rest };
}

function objectValue(value: unknown): Record<string, unknown> | null {
  if (!value || typeof value !== "object" || Array.isArray(value)) return null;
  return value as Record<string, unknown>;
}

function contentText(value: unknown): string {
  if (typeof value === "string") return value;
  if (!Array.isArray(value)) return "";

  return value
    .map((item) => {
      const obj = objectValue(item);
      if (!obj) return "";
      if (typeof obj.text === "string") return obj.text;
      if (typeof obj.content === "string") return obj.content;
      return "";
    })
    .join("");
}

export function extractTranslateStreamDelta(value: unknown): string {
  const root = objectValue(value);
  if (!root) return "";

  if (typeof root.delta === "string") return root.delta;
  if (typeof root.text === "string") return root.text;

  const choices = Array.isArray(root.choices) ? root.choices : [];
  for (const choiceValue of choices) {
    const choice = objectValue(choiceValue);
    const delta = objectValue(choice?.delta);
    const text = contentText(delta?.content);
    if (text) return text;
  }

  return "";
}

export function extractTranslateStreamFullText(value: unknown): string {
  const root = objectValue(value);
  if (!root) return "";

  if (typeof root.output_text === "string") return root.output_text;

  const choices = Array.isArray(root.choices) ? root.choices : [];
  for (const choiceValue of choices) {
    const choice = objectValue(choiceValue);
    const message = objectValue(choice?.message);
    const text = contentText(message?.content);
    if (text) return text;
  }

  const output = Array.isArray(root.output) ? root.output : [];
  return output.map((item) => contentText(objectValue(item)?.content)).join("");
}

function extractTranslateStreamError(value: unknown): string {
  const root = objectValue(value);
  const error = objectValue(root?.error);
  if (typeof error?.message === "string") return error.message;
  if (typeof root?.error === "string") return root.error;
  return "";
}

function applyTranslateStreamEvent(
  event: ServerSentEventBlock,
  currentText: string,
): { translated: string; done: boolean } {
  const trimmedData = event.data.trim();
  if (!trimmedData) {
    return { translated: currentText, done: false };
  }
  if (trimmedData === "[DONE]") {
    return { translated: currentText, done: true };
  }

  let parsed: unknown;
  try {
    parsed = JSON.parse(trimmedData);
  } catch {
    return { translated: currentText, done: false };
  }

  const error = extractTranslateStreamError(parsed);
  if (error) throw new Error(error);

  const delta = extractTranslateStreamDelta(parsed);
  if (delta) {
    return { translated: currentText + delta, done: false };
  }

  const fullText = currentText ? "" : extractTranslateStreamFullText(parsed);
  if (fullText) {
    return { translated: fullText, done: false };
  }

  return { translated: currentText, done: false };
}

export async function readTranslateStream(res: Response, options: TranslateStreamOptions = {}) {
  if (!res.body) throw new Error("Translate stream response has no body");

  const reader = res.body.getReader();
  const decoder = new TextDecoder();
  let buffer = "";
  let translated = "";
  let done = false;

  while (!done) {
    const chunk = await reader.read();
    if (chunk.done) break;

    buffer += decoder.decode(chunk.value, { stream: true });
    const parsed = parseServerSentEvents(buffer);
    buffer = parsed.rest;

    for (const event of parsed.events) {
      const result = applyTranslateStreamEvent(event, translated);
      done = result.done;
      if (result.translated !== translated) {
        translated = result.translated;
        options.onDelta?.(translated);
      }
      if (done) break;
    }
  }

  buffer += decoder.decode();
  const trailing = parseServerSentEventBlock(buffer.trim());
  if (!done && trailing) {
    const result = applyTranslateStreamEvent(trailing, translated);
    if (result.translated !== translated) {
      translated = result.translated;
      options.onDelta?.(translated);
    }
  }

  return { translated, segments: [] };
}

export function apiGet<T>(path: string, signal?: AbortSignal): Promise<T> {
  return request<T>("GET", path, undefined, signal);
}

export function apiPost<T>(path: string, body?: unknown): Promise<T> {
  return request<T>("POST", path, body);
}

export function apiPatch<T>(path: string, body?: unknown): Promise<T> {
  return request<T>("PATCH", path, body);
}

export function apiDelete<T>(path: string): Promise<T> {
  return request<T>("DELETE", path);
}

// ── Auth ─────────────────────────────────────────────────────────────

export interface AuthStatus {
  authenticated: boolean;
}

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
  accounts: Account[];
  folders: Record<string, Folder[]>;
  unreadCounts: Record<string, Record<string, number>>;
  gmailRealtime: Record<string, GmailRealtimeConfig>;
}

export function getShell(): Promise<ShellData> {
  return apiGet<ShellData>(`${BASE}/shell`);
}

// ── Messages (reads) ──────────────────────────────────────────────────

export interface InboxParams {
  accountId?: string;
  folderId: string;
  limit?: number;
  offset?: number;
  folderIds?: string[];
}

export function getInbox(params: InboxParams) {
  const qs = new URLSearchParams();
  if (params.accountId) qs.set("accountId", params.accountId);
  qs.set("folderId", params.folderId);
  if (params.limit) qs.set("limit", String(params.limit));
  if (params.offset) qs.set("offset", String(params.offset));
  if (params.folderIds?.length) qs.set("folderIds", params.folderIds.join(","));
  return apiGet<{ messages: unknown[]; total: number; hasMore: boolean }>(
    `${BASE}/inbox?${qs}`,
  ).then((r) => r.messages);
}

export function getStarred(accountId: string, limit?: number, offset?: number) {
  const qs = new URLSearchParams({ accountId });
  if (limit) qs.set("limit", String(limit));
  if (offset) qs.set("offset", String(offset));
  return apiGet<{ messages: unknown[]; total: number; hasMore: boolean }>(
    `${BASE}/starred?${qs}`,
  ).then((r) => r.messages);
}

export function getMessage(id: string) {
  return apiGet<unknown>(`${BASE}/messages/${encodeURIComponent(id)}`);
}

export function getMessagesBatch(messageIds: string[]) {
  return apiPost<unknown[]>(`${BASE}/messages/batch`, { messageIds });
}

// ── Threads ───────────────────────────────────────────────────────────

export function listThreads(
  folderId: string,
  limit?: number,
  offset?: number,
  folderIds?: string[],
) {
  const qs = new URLSearchParams({ folderId });
  if (limit) qs.set("limit", String(limit));
  if (offset) qs.set("offset", String(offset));
  if (folderIds?.length) qs.set("folderIds", folderIds.join(","));
  return apiGet<unknown[]>(`${BASE}/threads?${qs}`);
}

export function listThreadMessages(threadId: string) {
  return apiGet<unknown[]>(`${BASE}/threads/${encodeURIComponent(threadId)}/messages`);
}

// ── Search ────────────────────────────────────────────────────────────

export function searchMessages(q: string, limit?: number) {
  const qs = new URLSearchParams({ q });
  if (limit) qs.set("limit", String(limit));
  return apiGet<{ hits: unknown[]; total: number }>(`${BASE}/search?${qs}`).then((r) => r.hits);
}

// ── Kanban ────────────────────────────────────────────────────────────

export function getKanban(column?: string) {
  const qs = column ? `?column=${encodeURIComponent(column)}` : "";
  return apiGet<{ cards: unknown[]; notes: Record<string, string> }>(`${BASE}/kanban${qs}`).then(
    (r) => r.cards,
  );
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

export function getMessageWithHtml(messageId: string, privacyMode: string, signal?: AbortSignal) {
  const qs = new URLSearchParams({ privacyMode });
  return apiGet<unknown>(`${BASE}/messages/${encodeURIComponent(messageId)}/full?${qs}`, signal);
}

// ── Messages (mutations) ───────────────────────────────────────────────

export function updateMessageFlags(messageId: string, isRead?: boolean, isStarred?: boolean) {
  return apiPatch<void>(`${BASE}/messages/${encodeURIComponent(messageId)}/flags`, {
    isRead,
    isStarred,
  });
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
  return apiPost<void>(`${BASE}/messages/${encodeURIComponent(messageId)}/move`, {
    targetFolderId,
  });
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
  return apiDelete<void>(
    `${BASE}/messages/${encodeURIComponent(messageId)}/labels/${encodeURIComponent(labelName)}`,
  );
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
  return apiPost<{ hits: unknown[]; total: number }>(`${BASE}/search/advanced`, {
    query,
    limit,
  }).then((r) => r.hits);
}

// ── Pending Ops ────────────────────────────────────────────────────────

export function getPendingMailOpsSummary(accountId: string | null) {
  const qs = accountId ? `?accountId=${encodeURIComponent(accountId)}` : "";
  return apiGet<unknown>(`${BASE}/pending-ops/summary${qs}`);
}

export function listPendingMailOps(accountId: string | null, limit?: number) {
  const qs = new URLSearchParams();
  if (accountId) qs.set("accountId", accountId);
  if (limit) qs.set("limit", String(limit));
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
  return apiPut<Record<string, string>>(`${BASE}/kanban/notes/${encodeURIComponent(messageId)}`, {
    note,
  });
}

export function mergeKanbanContextNotes(notes: Record<string, string>) {
  return apiPatch<Record<string, string>>(`${BASE}/kanban/notes`, notes);
}

export function listKanbanContextNotes() {
  return apiGet<Record<string, string>>(`${BASE}/kanban/notes`);
}

// ── Trusted Senders ────────────────────────────────────────────────────

function trustedSendersPath(accountId: string | null) {
  const qs = new URLSearchParams();
  if (accountId) qs.set("accountId", accountId);
  const query = qs.toString();
  return `${BASE}/trusted-senders${query ? `?${query}` : ""}`;
}

export function listTrustedSenders(accountId: string | null) {
  return apiGet<unknown[]>(trustedSendersPath(accountId));
}

export function trustSender(accountId: string, email: string, trustType: string) {
  return apiPost<void>(`${BASE}/trusted-senders`, { accountId, email, trustType });
}

export function removeTrustedSender(accountId: string, email: string) {
  return apiDelete<void>(
    `${BASE}/trusted-senders?accountId=${encodeURIComponent(accountId)}&email=${encodeURIComponent(email)}`,
  );
}

export function isTrustedSender(accountId: string, email: string) {
  return apiGet<boolean>(
    `${BASE}/trusted-senders/check?accountId=${encodeURIComponent(accountId)}&email=${encodeURIComponent(email)}`,
  );
}

// ── Helpers ────────────────────────────────────────────────────────────

export function apiPut<T>(path: string, body?: unknown): Promise<T> {
  return request<T>("PUT", path, body);
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
  return apiPost<string>(`${BASE}/drafts`, params);
}

export function deleteDraft(accountId: string, draftId: string) {
  return apiDelete<void>(
    `${BASE}/drafts/${encodeURIComponent(draftId)}?accountId=${encodeURIComponent(accountId)}`,
  );
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
  formData.append("file", file);
  const url = new URL(`${BASE}/attachments/stage`, window.location.origin);
  const res = await fetch(url.toString(), {
    method: "POST",
    credentials: "same-origin",
    body: formData,
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }));
    throw new ApiError(res.status, err);
  }
  const data = await res.json();
  const staged = data as { attachments?: Array<{ path?: string }> };
  const path = staged.attachments?.[0]?.path;
  if (!path) {
    throw new ApiError(500, { error: "Attachment upload response did not include a path" });
  }
  return path;
}

// ── Contacts ───────────────────────────────────────────────────────────

export function searchContacts(accountId: string, query: string, limit?: number) {
  const qs = new URLSearchParams({ accountId, q: query });
  if (limit) qs.set("limit", String(limit));
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
  return apiGet<string>(`${BASE}/accounts/${encodeURIComponent(accountId)}/signature`);
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

export async function translateTextStream(
  text: string,
  fromLang: string,
  toLang: string,
  options: TranslateStreamOptions = {},
) {
  const url = new URL(`${BASE}/translate/stream`, window.location.origin);
  const res = await fetch(url.toString(), {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    credentials: "same-origin",
    signal: options.signal,
    body: JSON.stringify({ text, fromLang, toLang }),
  });

  if (!res.ok) {
    throw await apiErrorFromResponse(res);
  }

  return readTranslateStream(res, options);
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
  return apiPut<void>(`${BASE}/proxy`, { proxy_host: proxyHost, proxy_port: proxyPort });
}

// ── Preferences ────────────────────────────────────────────────────────

export function setRealtimePreference(mode: string) {
  return apiPut<void>(`${BASE}/preferences/realtime`, { mode });
}

export function setNotificationsEnabled(enabled: boolean) {
  return apiPut<void>(`${BASE}/preferences/notifications`, { enabled });
}

// ── Browser Push Notifications ────────────────────────────────────────

export interface BrowserPushKeys {
  p256dh: string;
  auth: string;
}

export interface BrowserPushSubscriptionPayload {
  endpoint: string;
  keys: BrowserPushKeys;
}

export type NotificationDeviceStatus = "active" | "paused";

export interface NotificationDevice {
  id: string;
  endpoint: string;
  p256dh: string;
  auth: string;
  device_name: string;
  user_agent?: string | null;
  status: NotificationDeviceStatus;
  session_id?: string | null;
  session_expires_at?: number | null;
  last_active_at: number;
  summary_sent_at?: number | null;
  created_at: number;
  updated_at: number;
}

export function getWebPushPublicKey() {
  return apiGet<{ public_key: string }>(`${BASE}/notifications/vapid-public-key`);
}

export function listNotificationDevices() {
  return apiGet<{ devices: NotificationDevice[] }>(`${BASE}/notifications/devices`);
}

export function upsertWebPushSubscription(params: {
  deviceId: string;
  deviceName?: string;
  subscription: BrowserPushSubscriptionPayload;
}) {
  return apiPost<{ device: NotificationDevice }>(`${BASE}/notifications/subscriptions`, {
    device_id: params.deviceId,
    device_name: params.deviceName,
    subscription: params.subscription,
  });
}

export function deleteWebPushSubscription(deviceId: string) {
  return apiDelete<void>(`${BASE}/notifications/subscriptions/${encodeURIComponent(deviceId)}`);
}

export function renameNotificationDevice(deviceId: string, deviceName: string) {
  return apiPatch<NotificationDevice>(
    `${BASE}/notifications/devices/${encodeURIComponent(deviceId)}`,
    {
      device_name: deviceName,
    },
  );
}

export function deleteNotificationDevice(deviceId: string) {
  return apiDelete<void>(`${BASE}/notifications/devices/${encodeURIComponent(deviceId)}`);
}

export function sendTestNotification(deviceId: string) {
  return apiPost<void>(`${BASE}/notifications/test`, { device_id: deviceId });
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
  return apiPut<void>(`${BASE}/accounts/${encodeURIComponent(accountId)}/proxy`, {
    proxy_host: proxyHost,
    proxy_port: proxyPort,
  });
}

export function getAccountProxySetting(accountId: string) {
  return apiGet<unknown>(`${BASE}/accounts/${encodeURIComponent(accountId)}/proxy-setting`);
}

export function updateAccountProxySetting(
  accountId: string,
  mode: string,
  proxyHost?: string,
  proxyPort?: number,
) {
  return apiPut<void>(`${BASE}/accounts/${encodeURIComponent(accountId)}/proxy-setting`, {
    mode,
    proxy_host: proxyHost,
    proxy_port: proxyPort,
  });
}

// ── Sync commands ──────────────────────────────────────────────────────

export function startSync(accountId: string, pollIntervalSecs?: number) {
  return apiPost<string>(`${BASE}/accounts/${encodeURIComponent(accountId)}/sync/start`, {
    poll_interval_secs: pollIntervalSecs,
  });
}

export function triggerSync(accountId: string, reason: string) {
  return apiPost<void>(`${BASE}/accounts/${encodeURIComponent(accountId)}/sync/trigger`, {
    reason,
  });
}

export interface WakeSyncRequest {
  accountIds?: string[];
  reason: string;
  ensureRunning?: boolean;
  pollIntervalSecs?: number;
}

export function wakeSync(request: WakeSyncRequest) {
  return apiPost<unknown>(`${BASE}/sync/wake`, {
    account_ids: request.accountIds,
    reason: request.reason,
    ensure_running: request.ensureRunning,
    poll_interval_secs: request.pollIntervalSecs,
  });
}

export function stopSync(accountId: string) {
  return apiPost<void>(`${BASE}/accounts/${encodeURIComponent(accountId)}/sync/stop`);
}

// ── Gmail realtime ─────────────────────────────────────────────────────

export function getGmailRealtimeConfig(accountId: string) {
  return apiGet<unknown>(`${BASE}/accounts/${encodeURIComponent(accountId)}/gmail-realtime`);
}

export function enableGmailRealtime(accountId: string, fallbackIntervalMinutes?: number) {
  return apiPost<unknown>(
    `${BASE}/accounts/${encodeURIComponent(accountId)}/gmail-realtime/enable`,
    { fallback_interval_minutes: fallbackIntervalMinutes },
  );
}

export function disableGmailRealtime(accountId: string) {
  return apiPost<unknown>(
    `${BASE}/accounts/${encodeURIComponent(accountId)}/gmail-realtime/disable`,
  );
}

export function updateGmailRealtimeConfig(accountId: string, fallbackIntervalMinutes: number) {
  return apiPut<unknown>(`${BASE}/accounts/${encodeURIComponent(accountId)}/gmail-realtime`, {
    fallback_interval_minutes: fallbackIntervalMinutes,
  });
}
