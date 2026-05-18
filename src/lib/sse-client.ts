// SSE client for real-time backend events via EventSource.
// RPC invoke() bridge removed — all API calls now use api-client.ts REST functions.

export interface Event<T> {
  event: string;
  payload: T;
}

class SseClient {
  private source: EventSource | null = null;
  private listeners: Map<string, Set<(event: Event<any>) => void>> = new Map();
  private reconnectAttempt = 0;
  private maxReconnectAttempts = 10;
  private baseReconnectDelayMs = 1000;
  private maxReconnectDelayMs = 30000;

  private computeReconnectDelay(): number {
    const delay = this.baseReconnectDelayMs * Math.pow(2, this.reconnectAttempt);
    return Math.min(delay, this.maxReconnectDelayMs) + Math.random() * 1000;
  }

  connect() {
    if (this.source) return;
    this.reconnectAttempt = 0;
    this.doConnect();
  }

  private doConnect() {
    if (this.source) {
      this.source.close();
      this.source = null;
    }

    this.source = new EventSource('/events');

    this.source.onopen = () => {
      this.reconnectAttempt = 0;
    };

    this.source.onerror = () => {
      if (this.source?.readyState === EventSource.CLOSED) {
        this.reconnectAttempt++;
        if (this.reconnectAttempt > this.maxReconnectAttempts) {
          console.error('[SSE] Max reconnect attempts reached, giving up');
          return;
        }
        const delay = this.computeReconnectDelay();
        console.warn(
          `[SSE] Connection closed, reconnecting in ${Math.round(delay)}ms (attempt ${this.reconnectAttempt}/${this.maxReconnectAttempts})`
        );
        setTimeout(() => this.doConnect(), delay);
      }
    };
  }

  listen<T>(event: string, handler: (e: Event<T>) => void): () => void {
    if (!this.source) this.connect();

    if (!this.listeners.has(event)) {
      this.listeners.set(event, new Set());
      this.source!.addEventListener(event, (e: MessageEvent) => {
        try {
          const payload = JSON.parse(e.data);
          const callbacks = this.listeners.get(event);
          if (callbacks) {
            callbacks.forEach(cb => cb({ event, payload }));
          }
        } catch (err) {
          console.error('Failed to parse SSE event data', err);
        }
      });
    }

    this.listeners.get(event)!.add(handler);

    return () => {
      const callbacks = this.listeners.get(event);
      if (callbacks) {
        callbacks.delete(handler);
      }
    };
  }
}

const sseClient = new SseClient();

export const listen = async <T>(
  event: string,
  handler: (event: Event<T>) => void
): Promise<() => void> => {
  return sseClient.listen(event, handler);
};
