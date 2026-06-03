export type LazyViewImporter = () => Promise<unknown>;
export type ScheduledIdleWork = () => void | Promise<unknown>;

type IdleWindow = Window & {
  requestIdleCallback?: (callback: IdleRequestCallback, options?: IdleRequestOptions) => number;
  cancelIdleCallback?: (handle: number) => void;
};

export function createLazyViewPreloader(importers: readonly LazyViewImporter[]) {
  let preloadPromise: Promise<PromiseSettledResult<unknown>[]> | null = null;

  return function preloadLazyViews() {
    if (!preloadPromise) {
      preloadPromise = Promise.allSettled(importers.map((importer) => importer()));
    }
    return preloadPromise;
  };
}

export function scheduleIdleWork(
  work: ScheduledIdleWork,
  win: Window = window,
  delayMs: number = 0,
) {
  const idleWindow = win as IdleWindow;
  let handle: number;
  let isIdleHandle = false;

  const run = () => {
    void work();
  };

  const scheduleRun = () => {
    if (idleWindow.requestIdleCallback) {
      isIdleHandle = true;
      handle = idleWindow.requestIdleCallback(run, { timeout: 1200 });
    } else {
      handle = win.setTimeout(run, 0);
    }
  };

  if (delayMs > 0) {
    handle = win.setTimeout(scheduleRun, delayMs);
  } else {
    scheduleRun();
  }

  return () => {
    if (isIdleHandle && idleWindow.cancelIdleCallback) {
      idleWindow.cancelIdleCallback(handle);
    } else {
      win.clearTimeout(handle);
    }
  };
}

export function scheduleLazyViewPreload(
  preload: () => Promise<unknown>,
  win: Window = window,
  delayMs: number = 0,
) {
  return scheduleIdleWork(preload, win, delayMs);
}
