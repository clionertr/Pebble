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
  );
}

export function getStarred(accountId: string, limit?: number, offset?: number) {
  const qs = new URLSearchParams({ accountId });
  if (limit) qs.set('limit', String(limit));
  if (offset) qs.set('offset', String(offset));
  return apiGet<{ messages: unknown[]; total: number; hasMore: boolean }>(
    `${BASE}/starred?${qs}`,
  );
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
  return apiGet<{ hits: unknown[]; total: number }>(`${BASE}/search?${qs}`);
}

// ── Kanban ────────────────────────────────────────────────────────────

export function getKanban(column?: string) {
  const qs = column ? `?column=${encodeURIComponent(column)}` : '';
  return apiGet<{ cards: unknown[]; notes: Record<string, string> }>(`${BASE}/kanban${qs}`);
}

// ── Snooze ────────────────────────────────────────────────────────────

export function getSnoozed() {
  return apiGet<unknown[]>(`${BASE}/snoozed`);
}
