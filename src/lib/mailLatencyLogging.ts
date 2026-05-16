import type { MessageSummary } from "@/lib/api";
import { recordMailDisplayTiming, type MailDisplayTiming } from "@/lib/api";

interface MailNewLatencyPayload {
  source?: string | null;
  backend_received_at_ms?: number | null;
  backend_sse_at_ms?: number | null;
  message_received_at_ms?: number | null;
  history_id?: string | null;
}

export interface MailNewLatencyEvent {
  account_id?: string | null;
  message_id?: string | null;
  latency?: MailNewLatencyPayload | null;
}

interface PendingDisplayTiming extends MailDisplayTiming {
  historyId?: string | null;
  frontendSsePerfMs: number;
}

const pendingDisplayTimings = new Map<string, PendingDisplayTiming>();
const reportedMessageIds = new Set<string>();

function serverReportingEnabled() {
  return localStorage.getItem("pebble.mailLatencyDebug") !== "0";
}

function browserDebugEnabled() {
  return localStorage.getItem("pebble.mailLatencyDebug") === "1";
}

function compactTiming(timing: PendingDisplayTiming) {
  return {
    accountId: timing.accountId,
    messageId: timing.messageId,
    source: timing.source,
    activeFolderId: timing.activeFolderId,
    backendReceivedAtMs: timing.backendReceivedAtMs,
    backendSseAtMs: timing.backendSseAtMs,
    messageReceivedAtMs: timing.messageReceivedAtMs,
    frontendSseAtMs: timing.frontendSseAtMs,
    displayedAtMs: timing.displayedAtMs,
    frontendSseToDisplayMs: timing.frontendSseToDisplayMs,
    historyId: timing.historyId,
    clientClockOffsetVsBackendSseMs:
      timing.backendSseAtMs == null ? null : timing.frontendSseAtMs - timing.backendSseAtMs,
  };
}

export function rememberMailNewLatencyEvent(payload: MailNewLatencyEvent) {
  if (!serverReportingEnabled() || !payload.message_id) return;

  const frontendSseAtMs = Date.now();
  const frontendSsePerfMs = performance.now();
  const timing: PendingDisplayTiming = {
    accountId: payload.account_id ?? null,
    messageId: payload.message_id,
    source: payload.latency?.source ?? null,
    backendReceivedAtMs: payload.latency?.backend_received_at_ms ?? null,
    backendSseAtMs: payload.latency?.backend_sse_at_ms ?? null,
    messageReceivedAtMs: payload.latency?.message_received_at_ms ?? null,
    frontendSseAtMs,
    displayedAtMs: frontendSseAtMs,
    frontendSseToDisplayMs: null,
    frontendSsePerfMs,
    historyId: payload.latency?.history_id ?? null,
  };
  pendingDisplayTimings.set(payload.message_id, timing);
  if (browserDebugEnabled()) {
    console.debug("[mail-latency] frontend_sse_received", compactTiming(timing));
  }
}

export function markDisplayedMessagesForMailLatencyLogging(
  messages: MessageSummary[],
  activeFolderId?: string | null,
) {
  if (!serverReportingEnabled() || pendingDisplayTimings.size === 0 || messages.length === 0) return;

  const visibleIds = new Set(messages.map((message) => message.id));
  for (const [messageId, timing] of pendingDisplayTimings) {
    if (!visibleIds.has(messageId) || reportedMessageIds.has(messageId)) continue;
    reportedMessageIds.add(messageId);
    pendingDisplayTimings.delete(messageId);

    requestAnimationFrame(() => {
      const displayedPerfMs = performance.now();
      const displayedTiming: PendingDisplayTiming = {
        ...timing,
        activeFolderId: activeFolderId ?? null,
        displayedAtMs: Date.now(),
        frontendSseToDisplayMs: Math.round(displayedPerfMs - timing.frontendSsePerfMs),
      };
      if (browserDebugEnabled()) {
        console.debug("[mail-latency] frontend_message_displayed", compactTiming(displayedTiming));
      }
      recordMailDisplayTiming(displayedTiming).catch((error) => {
        if (browserDebugEnabled()) {
          console.debug("[mail-latency] failed to record display timing", error);
        }
      });
    });
  }
}
