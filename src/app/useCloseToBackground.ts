import { useEffect } from "react";
import { getCurrentWindow } from "../tauri-mock";
import { useSyncStore } from "@/stores/sync.store";

export function useCloseToBackground() {
  useEffect(() => {
    const appWindow = getCurrentWindow();
    let unlisten: (() => void) | undefined;
    let disposed = false;

    appWindow
      .onCloseRequested((event) => {
        if (!useSyncStore.getState().keepRunningInBackground) {
          return;
        }

        event.preventDefault();
        void appWindow
          .hide()
          .catch((err) => console.warn("Failed to hide window on close", err));
      })
      .then((fn) => {
        if (disposed) {
          fn();
          return;
        }
        unlisten = fn;
      })
      .catch((err) => console.warn("Failed to register close-to-background handler", err));

    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);
}
