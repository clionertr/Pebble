/**
 * Deferred persistence utility — queues localStorage writes and flushes
 * them on requestIdleCallback to avoid blocking the main thread.
 */
type PersistOp = () => void;

let persistQueue: PersistOp[] = [];
let idleHandle: number | null = null;

function flushPersistQueue() {
  const ops = persistQueue;
  persistQueue = [];
  idleHandle = null;
  for (const op of ops) {
    try {
      op();
    } catch {
      // QuotaExceededError or other storage error — silently drop
    }
  }
}

function scheduleFlush() {
  if (idleHandle != null) return;
  if (typeof requestIdleCallback !== "undefined") {
    idleHandle = requestIdleCallback(flushPersistQueue, { timeout: 1000 });
  } else {
    idleHandle = window.setTimeout(flushPersistQueue, 100);
  }
}

export function deferPersist(op: PersistOp) {
  persistQueue.push(op);
  scheduleFlush();
}

// Flush any pending writes before the page unloads
if (typeof window !== "undefined") {
  window.addEventListener("beforeunload", flushPersistQueue);
}
