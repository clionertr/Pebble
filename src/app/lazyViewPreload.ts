export type LazyViewImporter = () => Promise<unknown>;

type IdleWindow = Window & {
  requestIdleCallback?: (callback: IdleRequestCallback, options?: IdleRequestOptions) => number;
  cancelIdleCallback?: (handle: number) => void;
};

export function scheduleLazyViewPreload(
  preload: () => Promise<unknown>,
  win: Window = window,
  delayMs: number = 0
) {
  const idleWindow = win as IdleWindow;
  let handle: number;
  let isIdleHandle = false;

  const run = () => {
    void preload();
  };

  const scheduleRun = () => {
    if (idleWindow.requestIdleCallback) {
      isIdleHandle = true;
      handle = idleWindow.requestIdleCallback(run, { timeout: 2000 });
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
