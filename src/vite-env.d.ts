/// <reference types="vite/client" />

interface PebbleSplashController {
  dismiss(reason?: string): void;
  isDismissed(): boolean;
  isRemoved(): boolean;
}

interface WindowEventMap {
  "pebble:splash-removed": CustomEvent<{
    reason: string;
    elapsedMs: number;
  }>;
}

interface Window {
  pebbleSplash?: PebbleSplashController;
  __splashStart?: number;
}
