import {
  deleteWebPushSubscription,
  getWebPushPublicKey,
  upsertWebPushSubscription,
  type BrowserPushSubscriptionPayload,
  type NotificationDevice,
} from "@/lib/api";

const DEVICE_ID_KEY = "pebble-web-push-device-id";
const SERVICE_WORKER_URL = "/pebble-sw.js";

export function supportsWebPush(): boolean {
  return "serviceWorker" in navigator && "PushManager" in window && "Notification" in window;
}

export function getStoredWebPushDeviceId(): string | null {
  return localStorage.getItem(DEVICE_ID_KEY);
}

export function getCurrentWebPushDeviceId(): string {
  const existing = getStoredWebPushDeviceId();
  if (existing) return existing;

  const generated =
    typeof crypto.randomUUID === "function"
      ? crypto.randomUUID()
      : `device-${Date.now()}-${Math.random().toString(36).slice(2)}`;
  localStorage.setItem(DEVICE_ID_KEY, generated);
  return generated;
}

export async function enableCurrentDeviceNotifications(): Promise<NotificationDevice> {
  if (!supportsWebPush()) {
    throw new Error("This browser does not support Web Push notifications.");
  }

  const permission = await Notification.requestPermission();
  if (permission !== "granted") {
    throw new Error("Notification permission was not granted.");
  }

  return subscribeCurrentDevice();
}

export async function restoreCurrentDeviceNotifications(): Promise<NotificationDevice | null> {
  if (!supportsWebPush() || Notification.permission !== "granted") return null;
  return subscribeCurrentDevice();
}

export async function disableCurrentDeviceNotifications(): Promise<void> {
  const deviceId = getStoredWebPushDeviceId();
  if (!deviceId) return;

  await unsubscribeBrowserPush().catch(() => {});
  await deleteWebPushSubscription(deviceId).catch(() => {});
}

async function subscribeCurrentDevice(): Promise<NotificationDevice> {
  const registration = await navigator.serviceWorker.register(SERVICE_WORKER_URL, { scope: "/" });
  const publicKey = await getWebPushPublicKey();
  const applicationServerKey = urlBase64ToUint8Array(publicKey);
  const existing = await registration.pushManager.getSubscription();
  const subscription =
    existing ??
    (await registration.pushManager.subscribe({
      userVisibleOnly: true,
      applicationServerKey,
    }));

  return upsertWebPushSubscription({
    deviceId: getCurrentWebPushDeviceId(),
    subscription: serializeSubscription(subscription),
  });
}

async function unsubscribeBrowserPush(): Promise<void> {
  if (!("serviceWorker" in navigator)) return;
  const registration =
    (await navigator.serviceWorker.getRegistration(SERVICE_WORKER_URL)) ??
    (await navigator.serviceWorker.getRegistration());
  const subscription = await registration?.pushManager.getSubscription();
  await subscription?.unsubscribe();
}

function serializeSubscription(subscription: PushSubscription): BrowserPushSubscriptionPayload {
  const json = subscription.toJSON() as {
    endpoint?: string;
    keys?: { p256dh?: string; auth?: string };
  };
  const endpoint = json.endpoint;
  const p256dh = json.keys?.p256dh;
  const auth = json.keys?.auth;
  if (!endpoint || !p256dh || !auth) {
    throw new Error("Browser returned an incomplete PushSubscription.");
  }
  return { endpoint, keys: { p256dh, auth } };
}

export function urlBase64ToUint8Array(value: string): ArrayBuffer {
  const padding = "=".repeat((4 - (value.length % 4)) % 4);
  const base64 = `${value}${padding}`.replace(/-/g, "+").replace(/_/g, "/");
  const raw = window.atob(base64);
  const output = new Uint8Array(raw.length);
  for (let i = 0; i < raw.length; i += 1) {
    output[i] = raw.charCodeAt(i);
  }
  return output.buffer as ArrayBuffer;
}
