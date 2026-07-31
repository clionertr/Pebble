import { useEffect, useState } from "react";

/** open 变 false 后，延迟 delayMs 再返回 false，给退场动画留时间 */
export function useDelayedUnmount(open: boolean, delayMs: number): boolean {
  const [mounted, setMounted] = useState(open);
  useEffect(() => {
    if (open) {
      setMounted(true);
      return;
    }
    const timer = setTimeout(() => setMounted(false), delayMs);
    return () => clearTimeout(timer);
  }, [open, delayMs]);
  return mounted;
}
