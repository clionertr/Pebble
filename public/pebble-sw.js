self.addEventListener("push", (event) => {
  event.waitUntil(handlePush(event));
});

self.addEventListener("notificationclick", (event) => {
  event.notification.close();
  event.waitUntil(handleNotificationClick(event.notification.data || {}));
});

async function handlePush(event) {
  const payload = readPayload(event);
  if (!payload) return;

  const windows = await clients.matchAll({ type: "window", includeUncontrolled: true });
  const hasVisibleClient = windows.some((client) => client.visibilityState === "visible" || client.focused);
  if (hasVisibleClient && !payload.allowForeground) return;

  await self.registration.showNotification(payload.title || "Pebble", {
    body: payload.body || "",
    tag: payload.tag || "pebble-mail",
    data: payload,
    timestamp: payload.timestamp ? payload.timestamp * 1000 : Date.now(),
  });
}

async function handleNotificationClick(payload) {
  const url = notificationUrl(payload);
  const windows = await clients.matchAll({ type: "window", includeUncontrolled: true });

  for (const client of windows) {
    if (!sameOrigin(client.url)) continue;
    client.postMessage({ type: "pebble:notification-click", data: payload });
    await client.focus();
    return;
  }

  await clients.openWindow(url);
}

function readPayload(event) {
  if (!event.data) return null;
  try {
    return event.data.json();
  } catch {
    return { title: "Pebble", body: event.data.text(), url: "/", tag: "pebble-mail" };
  }
}

function notificationUrl(payload) {
  const url = new URL(payload.url || "/", self.location.origin);
  url.searchParams.set("pebbleNotification", payload.kind || "mail");
  if (payload.messageId) url.searchParams.set("messageId", payload.messageId);
  return url.toString();
}

function sameOrigin(url) {
  try {
    return new URL(url).origin === self.location.origin;
  } catch {
    return false;
  }
}
