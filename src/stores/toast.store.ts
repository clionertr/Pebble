import { create } from "zustand";

export type ToastType = "success" | "error" | "info";

export interface Toast {
  id: string;
  message: string;
  type: ToastType;
  duration?: number;
  action?: { label: string; onClick: () => void };
  leaving?: boolean;
}

interface ToastState {
  toasts: Toast[];
  addToast: (toast: Omit<Toast, "id">) => void;
  removeToast: (id: string) => void;
}

let nextId = 0;
const EXIT_MS = 200; // 与 CSS 的 --duration-base 同数量级，略大于确保过渡播完

export const useToastStore = create<ToastState>((set, get) => ({
  toasts: [],

  addToast: (toast) => {
    const id = String(++nextId);
    set((s) => ({ toasts: [...s.toasts, { ...toast, id }] }));
    const duration = toast.duration ?? (toast.type === "error" ? 5000 : 3000);
    setTimeout(() => get().removeToast(id), duration);
  },

  removeToast: (id) => {
    const target = get().toasts.find((t) => t.id === id);
    if (!target || target.leaving) return; // 已在退场，防止双触发
    set((s) => ({ toasts: s.toasts.map((t) => (t.id === id ? { ...t, leaving: true } : t)) }));
    setTimeout(() => {
      set((s) => ({ toasts: s.toasts.filter((t) => t.id !== id) }));
    }, EXIT_MS);
  },
}));
