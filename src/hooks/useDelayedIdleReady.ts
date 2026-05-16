import { useEffect, useState } from "react";
import { scheduleIdleWork } from "@/app/lazyViewPreload";

export function useDelayedIdleReady(delayMs: number) {
  const [ready, setReady] = useState(false);

  useEffect(() => {
    setReady(false);
    return scheduleIdleWork(() => setReady(true), window, delayMs);
  }, [delayMs]);

  return ready;
}
