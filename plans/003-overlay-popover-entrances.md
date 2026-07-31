# 003 — 弹层与抽屉：入场动效 + 移动端抽屉退场

- **Status**: DONE
- **Commit**: c2ad347
- **Severity**: MEDIUM
- **Category**: Physicality & origin / Missed opportunities
- **Estimated scope**: 6 个文件（index.css、ConfirmDialog.tsx、SelectionActionPopover.tsx、ContactAutocomplete.tsx、Layout.tsx、新增 hooks/useDelayedUnmount.ts）

依赖：001。

**红线（先读）**：CommandPalette（src/features/command-palette/CommandPalette.tsx）**禁止加任何动画**。命令面板是超高频键盘操作，当前的瞬时开合是正确设计（Raycast 同款），保持原样。

## 问题

1. ConfirmDialog（src/components/ConfirmDialog.tsx:97-127）：遮罩 + 面板均无入场——`rgba(0,0,0,0.5)` 全屏遮罩瞬间砸下，很硬：

```tsx
// src/components/ConfirmDialog.tsx:103-111 — 现状：无任何动效
style={{
  position: "fixed",
  inset: 0,
  backgroundColor: "rgba(0,0,0,0.5)",
  ...
}}
```

2. SelectionActionPopover（src/components/SelectionActionPopover.tsx:46-64 主体、97-110 子菜单）：划词工具条与"更多"子菜单直接 mount，无入场、无 transform-origin。
3. ContactAutocomplete（src/components/ContactAutocomplete.tsx）：收件人联想下拉直接 mount。
4. 移动端抽屉（src/app/Layout.tsx:159-171）：入场用 keyframe `animate-slide-in-left`，关闭时直接 unmount——抽屉瞬间蒸发：

```tsx
// src/app/Layout.tsx:159-171 — 现状
{isMobile && drawerOpen && (
  <>
    <button ... className="absolute inset-0 bg-black/20 z-40 transition-opacity fade-in" onClick={() => setDrawerOpen(false)} />
    <div className="absolute inset-y-0 left-0 z-50 shadow-2xl animate-slide-in-left">
      <Sidebar />
    </div>
  </>
)}
```

5. 视图切换一致性：InboxView/SearchView/SnoozedView/StarredView 根节点带 `.fade-in`，但 KanbanView、SettingsView、ComposeView 没有——切到这三个视图是硬切，其他是淡入，不统一。

## 目标

### 1. 通用弹层入场类（index.css 新增，放在 Animations 分节内）

```css
/* 弹层入场：从触发点方向 0.97 缩放浮现。修改 --popover-origin 控制方向 */
.popover-enter {
  --popover-origin: top left;
  transform-origin: var(--popover-origin);
  transition:
    opacity var(--duration-base) var(--ease-out),
    transform var(--duration-base) var(--ease-out);
}

@starting-style {
  .popover-enter {
    opacity: 0;
    transform: scale(0.97);
  }
}

/* 对话框：遮罩淡入 + 面板轻微上浮缩放（面板居中出现，origin 保持 center 是正确的） */
.dialog-backdrop-enter {
  transition: opacity var(--duration-base) var(--ease-out);
}
@starting-style {
  .dialog-backdrop-enter {
    opacity: 0;
  }
}

.dialog-panel-enter {
  transition:
    opacity var(--duration-slow) var(--ease-out),
    transform var(--duration-slow) var(--ease-out);
}
@starting-style {
  .dialog-panel-enter {
    opacity: 0;
    transform: translateY(8px) scale(0.96);
  }
}
```

### 2. 应用到组件（只加 className，不动现有内联样式）

- ConfirmDialog.tsx：外层遮罩 div（:98）加 `className="dialog-backdrop-enter"`；内层面板 div（:113）加 `className="dialog-panel-enter"`。退场不做（对话框确认/取消后应立刻让路，瞬时消失可接受——刻意取舍）。
- SelectionActionPopover.tsx：主体 div（:47，role="toolbar" 那个）加 `className="popover-enter"`（默认 origin top left，正确：它出现在选区下方）；子菜单 div（:98，role="menu" 那个）加 `className="popover-enter"` 并在其内联 style 里加 `"--popover-origin": "top right"`（TS 写法：`style={{ ..., ["--popover-origin" as string]: "top right" }}`，因为子菜单右对齐触发按钮）。
- ContactAutocomplete.tsx：找到联想下拉的绝对定位容器（列表根，通常 `position: "absolute"` + 列表项映射的那个 div），加 `className="popover-enter"`。若该文件结构与预期不符（没有明显的下拉容器），跳过此文件并在结果中说明，不要硬套。

### 3. 移动端抽屉退场

新文件 `src/hooks/useDelayedUnmount.ts`：

```ts
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
```

Layout.tsx 抽屉块改为（保持 Sidebar 与按钮的既有属性）：

```tsx
{isMobile && drawerMounted && (
  <>
    <button
      type="button"
      aria-label={t("common.close", "Close")}
      className="absolute inset-0 bg-black/20 z-40 drawer-backdrop"
      data-open={drawerOpen}
      onClick={() => setDrawerOpen(false)}
    />
    <div className="absolute inset-y-0 left-0 z-50 shadow-2xl drawer-panel" data-open={drawerOpen}>
      <Sidebar />
    </div>
  </>
)}
```

组件顶部：`const drawerMounted = useDelayedUnmount(drawerOpen, 240);`（240 = --duration-slow）。

index.css：**删除** `@keyframes slideInFromLeft` 与 `.animate-slide-in-left`（index.css:164-175，此类删完后全库无引用），新增：

```css
/* 移动端抽屉：transition 双向滑动（可中断，中途反向不重播） */
.drawer-panel {
  transform: translateX(-100%);
  transition: transform var(--duration-slow) var(--ease-out);
}
.drawer-panel[data-open="true"] {
  transform: translateX(0);
}

.drawer-backdrop {
  opacity: 0;
  transition: opacity var(--duration-slow) var(--ease-out);
}
.drawer-backdrop[data-open="true"] {
  opacity: 1;
}
```

注意：mount 首帧 `data-open` 已是 true 时 transition 不会播（没有起始帧）——所以入场依赖的是 mount 时 `drawerOpen` 从 false→true 的正常流程。实际流程正是如此（先 isMobile 渲染、点击汉堡才 setDrawerOpen(true)？不——mount 与 open 同帧发生）。因此抽屉面板也要 @starting-style：

```css
@starting-style {
  .drawer-panel[data-open="true"] {
    transform: translateX(-100%);
  }
  .drawer-backdrop[data-open="true"] {
    opacity: 0;
  }
}
```

同时删除原 backdrop 上的 `transition-opacity fade-in` 两个类（被 .drawer-backdrop 取代）。

### 4. 视图切换一致性

给以下视图的根容器补 `fade-in` class（与 InboxView.tsx:113 的用法一致，只加 class 不动结构）：

- src/features/kanban/KanbanView.tsx 根 div
- src/features/settings/SettingsView.tsx 根 div
- src/features/compose/ComposeView.tsx 根 div

若某视图根已有 className 字符串，追加即可；若根是 Fragment，包在最近的块级根上。

## 仓库约定

- 令牌与总闸来自 001。
- class 命名用 kebab-case 语义名（`.drawer-panel`），与现有 `.compose-*`、`.search-*` 一致。
- Tailwind 工具类与自定义 class 混用是常态（Layout.tsx:164 现状如此）。

## 步骤

1. index.css 加 popover/dialog/drawer 三组类，删 slideInFromLeft 块。
2. 新建 useDelayedUnmount.ts。
3. Layout.tsx 改抽屉块。
4. ConfirmDialog / SelectionActionPopover / ContactAutocomplete 加 class。
5. 三个视图根补 fade-in。

## 边界

- **CommandPalette 一行都不许碰。**
- 不改各弹层的定位逻辑与焦点管理（ConfirmDialog 的 focus trap、popover 的 stopPropagation 都保持）。
- 不引入新依赖。
- 桌面端 Sidebar（非抽屉形态）不动。
- 与本计划标注行号对不上时停下报告。

## 验证

- **机械**：`pnpm lint`、`pnpm build:frontend`、`pnpm test` 全绿。
- **感受**：
  - 删除邮件触发确认框：遮罩 180ms 淡入，面板从下方 8px 处 240ms 浮起；点取消瞬时关闭。
  - 邮件正文划选文字：工具条从左上角轻微放大浮现；点"更多"子菜单从右上角浮现。
  - 窗口缩到 <768px，点汉堡开抽屉再立即点遮罩关闭：抽屉滑出滑回，中途反向不闪跳（快速连点验证可中断性）。
  - 切换到看板/设置视图：与收件箱一样有淡入。
  - Cmd+K 命令面板：依旧瞬时开合，**没有**动画。
  - 关闭动画开关后：以上全部瞬时化。
- **完成标准**：以上全过，`grep -rn "animate-slide-in-left\|slideInFromLeft" src` 无结果。
