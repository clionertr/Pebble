# 002 — Toast 改为可中断的进/退场过渡

- **Status**: DONE
- **Commit**: c2ad347
- **Severity**: HIGH
- **Category**: Interruptibility / Physicality
- **Estimated scope**: 3 个文件（toast.store.ts、ToastContainer.tsx、index.css）

依赖：001（需要 `--duration-base` / `--ease-out` 令牌与 data-motion 总闸）。

## 问题

Toast 只有 keyframe 入场、没有退场——消失是瞬间蒸发，且队列里上方的 toast 会硬跳补位：

```tsx
// src/components/ToastContainer.tsx:60 — 现状（内联样式对象里）
animation: "toast-in 0.2s ease-out",
```

```css
/* src/styles/index.css:282-291 — 现状 */
@keyframes toast-in {
  from {
    opacity: 0;
    transform: translateY(12px) scale(0.95);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}
```

```ts
// src/stores/toast.store.ts — 现状：两处直接 filter，移除瞬间完成
addToast: (toast) => {
  const id = String(++nextId);
  set((s) => ({ toasts: [...s.toasts, { ...toast, id }] }));
  const duration = toast.duration ?? (toast.type === "error" ? 5000 : 3000);
  setTimeout(() => {
    set((s) => ({ toasts: s.toasts.filter((t) => t.id !== id) }));
  }, duration);
},
removeToast: (id) => {
  set((s) => ({ toasts: s.toasts.filter((t) => t.id !== id) }));
},
```

Keyframe 是不可中断的（重触发会从零重播）；且退场完全缺失，属于"现实世界里没有东西会凭空消失"的物理性缺陷。

## 目标

### 1. store：两阶段移除（`src/stores/toast.store.ts` 整体替换为）

```ts
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
```

### 2. ToastContainer.tsx：动效相关样式移入 CSS 类

toast 卡片 div（ToastContainer.tsx:43-63）改为：

- 加 `className="toast-card"`，加 `data-leaving={toast.leaving ? "true" : undefined}`。
- 从内联 style 对象里删除这几行（移入 CSS 类）：`animation: "toast-in 0.2s ease-out"`、`padding: "12px 16px"`。其余内联样式（颜色、边框、布局）原样保留。

### 3. index.css：删除 `@keyframes toast-in` 块（282-291 行），原位替换为

```css
/* Toast：transition 进/退场（可中断），退场同时折叠高度让队列平滑合拢 */
.toast-card {
  max-height: 160px;
  padding: 12px 16px;
  transition:
    opacity var(--duration-base) var(--ease-out),
    transform var(--duration-base) var(--ease-out),
    max-height var(--duration-base) var(--ease-out),
    padding var(--duration-base) var(--ease-out),
    margin-bottom var(--duration-base) var(--ease-out);
}

@starting-style {
  .toast-card {
    opacity: 0;
    transform: translateY(10px) scale(0.97);
  }
}

.toast-card[data-leaving="true"] {
  overflow: hidden;
  opacity: 0;
  transform: translateY(6px) scale(0.97);
  max-height: 0;
  padding-top: 0;
  padding-bottom: 0;
  margin-bottom: -8px; /* 抵消父容器 gap: 8px，兄弟才能滑动合拢而不是跳一格 */
}
```

说明：max-height/padding 属于布局属性，一般禁止参与动画；此处元素小（≤160px）、频率低（occasional）、且是队列合拢体验的最短实现，属于刻意取舍——保留此注释。

## 仓库约定

- 动效令牌来自 001：`--duration-base: 180ms`、`--ease-out: cubic-bezier(0.23, 1, 0.32, 1)`，在 index.css `:root`。
- 组件样式模式：布局/颜色内联、动效进 index.css 类（范例：`.compose-editor-surface` index.css:481-498）。
- data-motion="off" 时这些 transition 自动被 001 的总闸压到 0.01ms，无需在组件里判断。

## 步骤

1. 替换 toast.store.ts（目标 1）。
2. ToastContainer.tsx 加 className/data-leaving，删两行内联样式（目标 2）。
3. index.css 删 toast-in keyframes，加 `.toast-card` 三段规则（目标 3）。

## 边界

- 不改 toast 的视觉设计（颜色、圆角、阴影、图标）。
- 不改 ToastContainer 的定位容器（fixed bottom/right 那层）。
- 不引入新依赖，不写 JS 动画。
- @starting-style 目标浏览器已支持（Vite 目标 es2020+，Chromium/Safari 17.5+/Firefox 129+）；不需要写 data-mounted 兜底。若发现构建目标不支持，停下报告。

## 验证

- **机械**：`pnpm lint`、`pnpm build:frontend`、`pnpm test` 全绿。
- **感受**：
  - 触发一个成功 toast（如归档一封邮件）：从下方 10px 处淡入上浮，无突跳。
  - 连发 3 个 toast，点中间那个的 ×：它淡出收缩，上下两个平滑合拢，不闪跳。
  - 自动消失（等 3 秒）走同样的退场。
  - DevTools Animations 面板调到 10% 速度确认退场是"位移+收高"复合，不是瞬移。
  - 设置里关掉动画：toast 瞬时出现/消失（但仍会出现/消失，功能不损）。
- **完成标准**：以上全过，且 `grep -n "toast-in" src -r` 无结果。
