import { useEffect } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { updateMessageFlags } from "@/lib/api";
import { useUIStore } from "@/stores/ui.store";

interface NotificationClickMessage {
  type?: string;
  data?: {
    kind?: string;
    messageId?: string;
    url?: string;
  };
}

export function useNotificationClickRouting() {
  const queryClient = useQueryClient();

  useEffect(() => {
    const routeNotification = (data: NotificationClickMessage["data"]) => {
      if (!data) return;
      if (data.messageId) {
        useUIStore.getState().openMessageInInbox(data.messageId);
        updateMessageFlags(data.messageId, true).catch(() => {});
        queryClient.invalidateQueries({ queryKey: ["messages"] });
        queryClient.invalidateQueries({ queryKey: ["threads"] });
        return;
      }
      useUIStore.getState().setActiveView("inbox");
    };

    const consumeUrl = () => {
      const url = new URL(window.location.href);
      const messageId = url.searchParams.get("messageId");
      const notification = url.searchParams.get("pebbleNotification");
      if (!messageId && !notification) return;

      routeNotification({ messageId: messageId ?? undefined });
      url.searchParams.delete("messageId");
      url.searchParams.delete("pebbleNotification");
      window.history.replaceState(window.history.state, "", `${url.pathname}${url.search}${url.hash}`);
    };

    const handleMessage = (event: MessageEvent<NotificationClickMessage>) => {
      if (event.data?.type !== "pebble:notification-click") return;
      routeNotification(event.data.data);
    };

    consumeUrl();
    navigator.serviceWorker?.addEventListener("message", handleMessage);
    return () => navigator.serviceWorker?.removeEventListener("message", handleMessage);
  }, [queryClient]);
}
