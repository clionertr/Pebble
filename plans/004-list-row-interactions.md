# 004 — 列表行与高频交互：hover 工具条、批量栏、归档退场、FAB

- **Status**: DONE
- **修订 A（审查方裁决，优先级最高）**：
  1. 归档/删除的 200ms 延迟是**有意的行为变更**，测试应跟随行为更新。允许最小修改 `tests/components/MessageItem.test.tsx` 与 `tests/components/MessageList.test.tsx` 两个文件：用 `vi.useFakeTimers()` + `vi.advanceTimersByTime(200)`（或 `waitFor`）适配新时序；批量工具栏改为常驻 DOM 后，断言"工具栏不存在"的用例改为断言其不可见/不可达。禁止删除测试用例、禁止放宽与本计划无关的断言、禁止改这两个文件之外的任何测试。
  2. 批量工具栏折叠态必须加 visibility 门控（防止键盘 Tab 摸到隐形按钮），在目标 2 的 CSS 基础上追加：

```css
.batch-toolbar-reveal > div {
  visibility: hidden;
  transition: visibility 0s linear var(--duration-base);
}
.batch-toolbar-reveal[data-open="true"] > div {
  visibility: visible;
  transition: none;
}
```

  （注意与已有 `.batch-toolbar-reveal > div { overflow: hidden; min-height: 0; }` 合并为一条规则块。开时立即可见，关时等 180ms 折叠完再隐藏。）
- **Commit**: c2ad347
- **Severity**: MEDIUM-HIGH（全是最高频界面）
- **Category**: Purpose & frequency / Interruptibility / Performance
- **Estimated scope**: 4 个文件（index.css、MessageItem.tsx、MessageList.tsx、ComposeFAB.tsx）

依赖：001。

## 问题

1. **行内操作按钮爆米花**（src/components/MessageItem.tsx:57, 94-108, 239-256）：hover 工具条靠 `showActions` state 条件渲染，鼠标扫过列表时整块 mount/unmount 闪现。这是全应用最高频的动效面。

```tsx
// MessageItem.tsx:57 — 现状
const [showActions, setShowActions] = useState(false);
// :94-108 — onMouseEnter/Leave/Focus/Blur 四个手写 handler
// :239 — {showActions && ( <div ...操作按钮 /> )}
```

2. **批量工具栏硬切**（src/components/MessageList.tsx:196-264）：`{batchMode && (<div ...>)}` 直接 mount，列表被猛地推下去。
3. **归档/删除瞬移**（MessageItem.tsx:262, 306）：`patchMessagesCache` 立即把行抽走，列表跳合。邮件应用最值得做的动效（Gmail/Superhuman 都有行折叠）。
4. **FAB 用 JS 手写 hover**（src/components/ComposeFAB.tsx:34-43）：mouseenter/mouseleave 改 style，触屏上会"粘住"（tap 触发 mouseenter 不触发 leave）；scale(1.08) 偏浮夸；没有按压反馈。

```tsx
// ComposeFAB.tsx:36-43 — 现状
onMouseEnter={(e) => {
  e.currentTarget.style.transform = "scale(1.08)";
  ...
```

## 目标

### 1. hover 工具条 → 纯 CSS（删 JS state）

MessageItem.tsx：

- 删除 `showActions` state（:57）与 onMouseEnter/onMouseLeave/onFocus/onBlur 四个 handler（:94-108）。
- `{showActions && (` 改为无条件渲染，操作条 div 加 `className="row-actions"`（原有内联样式保留，删掉里面的 `boxShadow` 不动——都保留，只加 class）。

index.css 新增：

```css
/* 行内操作条：常驻 DOM，opacity 浮现；触屏无 hover 则常显 */
.row-actions {
  opacity: 0;
  transform: translateX(4px);
  pointer-events: none;
  transition:
    opacity var(--duration-fast) var(--ease-out),
    transform var(--duration-fast) var(--ease-out);
}

.message-list-row:hover .row-actions,
.message-list-row:focus-within .row-actions,
.thread-list-row:hover .row-actions,
.thread-list-row:focus-within .row-actions {
  opacity: 1;
  transform: translateX(0);
  pointer-events: auto;
}

@media (hover: none) {
  .row-actions {
    opacity: 1;
    transform: none;
    pointer-events: auto;
  }
}
```

键盘可达性由 `:focus-within` 覆盖（原来 onFocus/onBlur 干的事），这是净删代码。ThreadItem 如有同款 hover 工具条（自查 src/components/ThreadItem.tsx / ThreadMessageBubble.tsx），套同一 class；没有就不加。

### 2. 批量工具栏滑入滑出（grid-rows 技巧，双向免 JS）

MessageList.tsx:196 `{batchMode && (<div style={{...}}>` 改为常驻包裹层：

```tsx
<div className="batch-toolbar-reveal" data-open={batchMode}>
  <div>
    <div style={{ /* 原工具栏的全部内联样式原样保留 */ }}>
      {/* 原工具栏内容原样保留 */}
    </div>
  </div>
</div>
```

（注意三层：reveal 外层 grid、中层 overflow 裁剪、内层原工具栏。原 `{batchMode && ...}` 条件删除，改由 CSS 控制显隐。）

index.css：

```css
/* 批量模式工具栏：grid-template-rows 0fr→1fr，天然支持双向过渡 */
.batch-toolbar-reveal {
  display: grid;
  grid-template-rows: 0fr;
  transition: grid-template-rows var(--duration-base) var(--ease-out);
}
.batch-toolbar-reveal[data-open="true"] {
  grid-template-rows: 1fr;
}
.batch-toolbar-reveal > div {
  overflow: hidden;
  min-height: 0;
}
```

### 3. 归档/删除的行退场（两阶段移除）

MessageItem.tsx：

- 加 `const [removing, setRemoving] = useState(false);`
- 行根 div（:75）加 `data-removing={removing || undefined}`。
- 归档按钮（:258 起）与垃圾邮件按钮（:302 起）的 onClick 改为两阶段：先 `setRemoving(true)`，200ms 后执行原有的全部逻辑（snapshot → patch → API →成功/失败处理原样保留）：

```tsx
onClick={(e) => {
  e.stopPropagation();
  setRemoving(true);
  setTimeout(() => {
    // ← 原 onClick 里 e.stopPropagation() 之后的全部代码原样搬进来
    // 失败分支 restoreMessagesCache 之后补一行 setRemoving(false)（行要回来）
  }, 200);
}}
```

index.css：

```css
/* 行移除：折叠+淡出，虚拟列表随 ResizeObserver 逐帧跟进 */
.message-list-row[data-removing] {
  height: 0 !important;
  padding-top: 0;
  padding-bottom: 0;
  border-bottom-width: 0;
  opacity: 0;
  transition:
    opacity 120ms var(--ease-out),
    height 160ms var(--ease-out) 40ms,
    padding 160ms var(--ease-out) 40ms,
    border-bottom-width 160ms var(--ease-out) 40ms;
}
```

（行内联 height: "76px" 需要 `!important` 压制；opacity 先走 40ms，随后高度折叠，下方行由虚拟器的 measureElement 跟随合拢。）

**回退方案（写死在这里，遇到再用）**：若实测下方行合拢肉眼可见卡顿/跳动（虚拟列表逐帧重排不平滑），放弃高度折叠，改为只 `opacity: 0` 过渡 120ms 后移除（transition 只留 opacity 一条，setTimeout 改 140ms）。在结果报告里写明用了哪个方案。

### 4. FAB 改 CSS + 按压反馈

ComposeFAB.tsx：删除 onMouseEnter/onMouseLeave 两个 handler 与内联 `transition` 行，加 `className="compose-fab"`。

index.css：

```css
/* 写信 FAB：hover 仅限精确指针，按压缩小反馈 */
.compose-fab {
  transition:
    transform var(--duration-fast) var(--ease-out),
    box-shadow var(--duration-fast) var(--ease-out);
}

@media (hover: hover) and (pointer: fine) {
  .compose-fab:hover {
    transform: scale(1.05);
    box-shadow: 0 6px 20px rgba(0, 0, 0, 0.3);
  }
}

.compose-fab:active {
  transform: scale(0.95);
}
```

### 5. 行操作按钮按压反馈（顺手，2 行）

index.css：

```css
.row-actions button:active {
  transform: scale(0.88);
}
```

（按钮已有各自 transition？没有——它们无 transition，scale 瞬时到位即可，按下松开由浏览器 :active 生命周期控制，足够脆。不加 transition，保持简单。）

## 仓库约定

- 令牌/总闸来自 001。
- 行 class：`.message-list-row` / `.thread-list-row` 已存在（index.css:227-244），hover 背景也在那里，新规则挨着放。
- MessageItem 是 `memo` 组件（MessageItem.tsx:412），删 state 只会更快。

## 步骤

1. index.css 加 `.row-actions`、`.batch-toolbar-reveal`、`[data-removing]`、`.compose-fab` 四组规则。
2. MessageItem.tsx：删 showActions、改无条件渲染、加 removing 两阶段。
3. MessageList.tsx：批量栏包 reveal 层。
4. ComposeFAB.tsx：JS hover → class。

## 边界

- 不改任何业务逻辑（缓存 patch/restore、API 调用、toast 文案原样）。
- 不改虚拟器配置（estimateSize/overscan/measureElement）。
- 星标按钮不做旋转/弹跳等花活——按压 scale 已够（高频操作）。
- 不引入新依赖。
- 与标注行号不符时停下报告。

## 验证

- **机械**：`pnpm lint`、`pnpm build:frontend`、`pnpm test` 全绿。
- **感受**：
  - 鼠标快速上下扫过邮件列表：操作条淡入淡出，无 mount 闪现；键盘 Tab 进行内按钮时操作条也出现。
  - 触屏模拟（DevTools device mode）：操作条常显。
  - 进/退批量模式：工具栏 180ms 滑入滑出，列表不猛跳。
  - hover 单封邮件点归档：行 160ms 折叠，下方行平滑上移，随后 toast 确认；断网（DevTools offline）点归档：行折叠后弹错误 toast，行回弹恢复。
  - FAB：hover 轻微放大，按下缩小；触屏 tap 不会卡在放大态。
  - 关闭动画开关：以上全部瞬时（归档仍多 200ms 延迟才移除——可接受，元素已瞬时隐形）。
- **完成标准**：以上全过，MessageItem.tsx 中 `showActions` 与 ComposeFAB.tsx 中 `onMouseEnter` 均已不存在。
