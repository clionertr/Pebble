export interface Event<T> {
  event: string;
  payload: T;
}

interface QueuedRequest {
  method: string;
  params: any;
  resolve: (value: any) => void;
  reject: (reason?: any) => void;
}

let requestQueue: QueuedRequest[] = [];
let batchTimeout: ReturnType<typeof setTimeout> | null = null;

const processBatch = async (requests: QueuedRequest[], retries = 3): Promise<void> => {
  const payload = requests.map(req => ({ method: req.method, params: req.params }));
  
  try {
    const res = await fetch('/rpc/batch', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload)
    });
    
    if (!res.ok) {
      throw new Error(`HTTP error! status: ${res.status}`);
    }
    
    const results = await res.json();
    if (!Array.isArray(results) || results.length !== requests.length) {
      throw new Error("Invalid batch response format");
    }
    
    for (let i = 0; i < requests.length; i++) {
      const data = results[i];
      if (data && typeof data === 'object' && 'error' in data) {
        requests[i].reject(new Error(data.error as string));
      } else {
        requests[i].resolve(data);
      }
    }
  } catch (error) {
    if (retries > 0) {
      console.warn(`Batch request failed, retrying... (${retries} retries left)`, error);
      await new Promise(r => setTimeout(r, 1000));
      return processBatch(requests, retries - 1);
    }
    requests.forEach(req => req.reject(error));
  }
};

export const invoke = async <T>(method: string, args: any = {}): Promise<T> => { 
  return new Promise((resolve, reject) => {
    requestQueue.push({ method, params: args, resolve, reject });
    
    if (!batchTimeout) {
      batchTimeout = setTimeout(() => {
        const queueToProcess = requestQueue;
        requestQueue = [];
        batchTimeout = null;
        processBatch(queueToProcess);
      }, 50);
    }
  });
};

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

export const getCurrentWindow = () => ({ 
  hide: async () => {}, 
  close: async () => {}, 
  minimize: async () => {}, 
  maximize: async () => {}, 
  unmaximize: async () => {}, 
  toggleMaximize: async () => {}, 
  startDragging: async () => {}, 
  isMaximized: async () => false, 
  setFocus: async () => {}, 
  show: async () => {},
  onCloseRequested: async (cb: (event: any) => void) => {
    console.log('Mock onCloseRequested', cb);
    return () => {};
  }
});

export const getVersion = async () => "0.0.0";
export const downloadDir = async () => "/tmp";
