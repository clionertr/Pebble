# 001 — 动效基建：令牌 + 全局动画开关（用户显式要求）

- **Status**: DONE
- **Commit**: c2ad347
- **Severity**: HIGH
- **Category**: Cohesion & tokens / Accessibility / 用户需求（动画可开关）
- **Estimated scope**: 4 个文件（index.css、theme.store.ts、Layout.tsx、AppearanceTab.tsx）+ 2 个 locale JSON

## 问题

1. 全库没有动效令牌。`src/styles/index.css` 里 `0.14s ease` 手写了 15+ 次，缓动曲线各处不一，后续所有动效计划（002-005）没有统一的值可引用。
2. 用户明确要求"动画可开关"，目前没有任何开关。
3. 现有 reduced-motion 处理是一刀切（`src/styles/index.css:293-302`）：

```css
/* src/styles/index.css:293-302 — 现状 */
@media (prefers-reduced-motion: reduce) {
  *,
  *::before,
  *::after {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
  }
}
```

它把 `.spinner`（`animation: spin 0.8s linear infinite`，index.css:222-225）也冻结了——加载指示器停转看起来像程序卡死。

4. 死代码：`.fade-in-item` 及其 10 条 nth-child 延迟规则（index.css:246-279）在整个 src/ 中零引用（已用 grep 验证），纯噪音。

## 目标

### 1. 动效令牌（加在 index.css 的 `:root` 块内，即现有颜色变量之后）

```css
  /* 动效令牌：所有动画/过渡必须引用这些值，不许手写曲线和时长 */
  --ease-out: cubic-bezier(0.23, 1, 0.32, 1);
  --ease-in-out: cubic-bezier(0.77, 0, 0.175, 1);
  --duration-fast: 140ms; /* hover / 颜色反馈 */
  --duration-base: 180ms; /* 弹层、下拉、toast 入场 */
  --duration-slow: 240ms; /* 抽屉、对话框 */
```

注意：令牌放在 `:root` 里颜色变量后面即可，`[data-theme="dark"]` 块不需要重复（动效与主题无关）。

### 2. 动画开关（三态：跟随系统 / 开 / 关）

`src/stores/theme.store.ts` 完全镜像现有 theme 的模式（`Theme`/`resolveTheme`/`applyThemeToDom`，见该文件 6-18 行）：

```ts
export type MotionPref = "system" | "on" | "off";

export function resolveMotion(pref: MotionPref): "on" | "off" {
  if (pref === "system") {
    return window.matchMedia("(prefers-reduced-motion: reduce)").matches ? "off" : "on";
  }
  return pref;
}

export function applyMotionToDom(pref: MotionPref) {
  document.documentElement.setAttribute("data-motion", resolveMotion(pref));
}
```

State 接口与 store 增加（照抄 theme 字段的写法，含 `deferPersist`）：

```ts
// ThemeState 接口新增：
motion: MotionPref;
setMotion: (motion: MotionPref) => void;

// create() 内新增：
motion: (localStorage.getItem("pebble-motion") as MotionPref) || "system",
setMotion: (motion) => {
  deferPersist(() => localStorage.setItem("pebble-motion", motion));
  applyMotionToDom(motion);
  set({ motion });
},
```

### 3. Layout.tsx 挂载时应用 + 监听系统偏好

`src/app/Layout.tsx:144-152` 现有 theme effect 是这样：

```tsx
useEffect(() => {
  applyThemeToDom(theme);
  if (theme === "system") {
    const mql = window.matchMedia("(prefers-color-scheme: dark)");
    const listener = () => applyThemeToDom("system");
    mql.addEventListener("change", listener);
    return () => mql.removeEventListener("change", listener);
  }
}, [theme]);
```

在它下面加一个完全同构的 motion effect（import 处补 `applyMotionToDom`）：

```tsx
const motion = useThemeStore((s) => s.motion);

useEffect(() => {
  applyMotionToDom(motion);
  if (motion === "system") {
    const mql = window.matchMedia("(prefers-reduced-motion: reduce)");
    const listener = () => applyMotionToDom("system");
    mql.addEventListener("change", listener);
    return () => mql.removeEventListener("change", listener);
  }
}, [motion]);
```

### 4. CSS 端：属性驱动的关闭规则（替换 index.css:293-302 整块）

```css
/* 动画开关（WCAG 2.1 SC 2.3.3）：
   - JS 未加载 / motion=system 时，跟随系统 prefers-reduced-motion（CSS 兜底首帧）
   - 用户显式选 off 时无条件关闭
   - .spinner 例外：加载指示是状态信息，冻结看起来像卡死 */
@media (prefers-reduced-motion: reduce) {
  html:not([data-motion="on"]) *,
  html:not([data-motion="on"]) *::before,
  html:not([data-motion="on"]) *::after {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
  }
  html:not([data-motion="on"]) .spinner {
    animation-duration: 0.8s !important;
    animation-iteration-count: infinite !important;
  }
}

html[data-motion="off"] *,
html[data-motion="off"] *::before,
html[data-motion="off"] *::after {
  animation-duration: 0.01ms !important;
  animation-iteration-count: 1 !important;
  transition-duration: 0.01ms !important;
}

html[data-motion="off"] .spinner {
  animation-duration: 0.8s !important;
  animation-iteration-count: infinite !important;
}
```

这套规则是后续 002-005 所有新动效的总闸：新动效只要走 CSS animation/transition 就自动被关掉，无需逐处判断。

### 5. 设置界面（AppearanceTab.tsx）

`src/features/settings/AppearanceTab.tsx` 里已有 `THEMES` 卡片选择组（4-8 行定义数组，27-52 行渲染）。完全照这个模式，在主题与语言两个 section 之间加一个"动画效果" section：

```tsx
const MOTIONS: { id: MotionPref; labelKey: string; descKey: string }[] = [
  { id: "system", labelKey: "settings.motionSystem", descKey: "settings.motionSystemDesc" },
  { id: "on", labelKey: "settings.motionOn", descKey: "settings.motionOnDesc" },
  { id: "off", labelKey: "settings.motionOff", descKey: "settings.motionOffDesc" },
];
```

渲染块直接复制 THEMES 的按钮 JSX（含 `h3` 标题 `t("settings.motion")`，`marginTop: "32px"` 与语言 section 一致），把 `theme`/`setTheme` 换成 `motion`/`setMotion`。

### 6. 文案（两个 locale 都要加，加在 settings 命名空间内）

`src/locales/en.json`:

```json
"motion": "Animations",
"motionSystem": "System",
"motionSystemDesc": "Follow system reduced-motion preference",
"motionOn": "On",
"motionOnDesc": "Always play interface animations",
"motionOff": "Off",
"motionOffDesc": "Turn off interface animations"
```

`src/locales/zh.json`:

```json
"motion": "动画效果",
"motionSystem": "跟随系统",
"motionSystemDesc": "跟随系统的减弱动态效果设置",
"motionOn": "开启",
"motionOnDesc": "始终播放界面动画",
"motionOff": "关闭",
"motionOffDesc": "关闭界面动画"
```

### 7. 清理与统一

- 删除 index.css:246-279 的 `.fade-in-item` 整块（含注释 `/* Staggered fade-in for list items */`）。
- index.css 内所有 `0.14s ease` 替换为 `var(--duration-fast) ease`（值等价，纯机械替换；只动 index.css，不碰组件内联样式的 0.12s/0.15s）。

## 仓库约定

- 状态管理：zustand，偏好类状态放 `src/stores/theme.store.ts`，持久化用 `deferPersist(() => localStorage.setItem(...))`，key 前缀 `pebble-`。范例：theme.store.ts:27-40。
- DOM 属性驱动主题/偏好：`document.documentElement.setAttribute("data-*", ...)`，CSS 用属性选择器响应。范例：theme.store.ts:16-18 + index.css:22。
- i18n：`useTranslation()` + `t("settings.xxx")`，en.json/zh.json 同步加 key。

## 步骤

1. index.css `:root` 加动效令牌（目标 1）。
2. index.css 替换 reduced-motion 块（目标 4），删除 `.fade-in-item` 块，替换 `0.14s ease` → `var(--duration-fast) ease`（目标 7）。
3. theme.store.ts 加 `MotionPref`/`resolveMotion`/`applyMotionToDom` 与 store 字段（目标 2）。
4. Layout.tsx 加 motion effect（目标 3）。
5. AppearanceTab.tsx 加动画 section（目标 5）。
6. 两个 locale 加文案（目标 6）。

## 边界

- 不许改 `Theme` 相关既有逻辑。
- 不许引入新依赖。
- 不许把令牌应用到组件内联样式（那是 002-005 的事）。
- 如果 index.css 行号与本计划标注对不上（漂移），停下报告，不要凭感觉改。

## 验证

- **机械**：`pnpm lint`、`pnpm build:frontend`、`pnpm test` 全绿。
- **感受**：
  - 设置 → 外观：出现"动画效果"三卡片，切换立即生效且刷新后保持。
  - 选"关闭"后：切视图、开对话框、toast 全部瞬时呈现（无动画），但加载 spinner 仍在转。
  - DevTools Rendering 面板模拟 `prefers-reduced-motion: reduce`：motion=跟随系统时动画关闭，motion=开启时动画仍在。
  - `document.documentElement.getAttribute("data-motion")` 与所选一致。
- **完成标准**：上述全部通过，且 `grep -c "0.14s ease" src/styles/index.css` 为 0、`grep -rn "fade-in-item" src` 无结果。
