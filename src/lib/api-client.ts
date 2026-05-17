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

// ── Health ────────────────────────────────────────────────────────────

export function healthCheck(): Promise<string> {
  return apiGet<string>(`${BASE}/health`);
}
