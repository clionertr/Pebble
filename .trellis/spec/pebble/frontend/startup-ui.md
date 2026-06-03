# Startup UI

## Scenario: 启动 splash 生命周期

### 1. Scope / Trigger

- Trigger: 修改 `index.html` 的 `#splash`、启动脚本、`src/main.tsx`、`src/App.tsx` 或启动耗时打点。
- 范围：`index.html`、`src/App.tsx`、`src/vite-env.d.ts`、`src/lib/startupTiming.ts`。

### 2. Signatures

- 全局控制器：`window.pebbleSplash?: PebbleSplashController`。
- dismiss API：`window.pebbleSplash.dismiss(reason?: string): void`。
- 查询 API：`isDismissed(): boolean`、`isRemoved(): boolean`。
- 删除完成事件：`window` 派发 `pebble:splash-removed`，`detail` 为 `{ reason: string; elapsedMs: number }`。
- 启动起点：`window.__splashStart?: number`。

### 3. Contracts

- `#splash` 在 `index.html` 中创建，就必须由 `index.html` 内的控制器负责淡出和删除。
- React 只发送“应用已就绪”信号，例如 `window.pebbleSplash?.dismiss("app-mounted")`；不得在 React 组件中直接删除 `#splash` 或 `#splash-style`。
- 控制器必须幂等：重复调用 `dismiss()`、transition 回调和 timeout 兜底不能导致重复删除或抛错。
- 控制器必须同时支持最小显示时间和最大显示时间；React 未挂载时，最大显示时间到达后仍要自动移除。
- `transitionend` 只能作为正常路径，必须保留 timeout 兜底，防止后台标签页、reduced motion 或样式异常导致事件缺失。
- 新增或变更 `window` 全局 API 时，必须同步更新 `src/vite-env.d.ts`。

### 4. Validation & Error Matrix

- React 正常挂载 -> 调用 `dismiss("app-mounted")`，等待最小显示时间后淡出并删除。
- React 启动链路抛错或 bundle 未加载 -> `maxMs` 到达后自动 `dismiss("timeout")`。
- `transitionend` 未触发 -> 删除 timeout 到达后强制删除 DOM。
- `#splash` 已不存在 -> 删除逻辑仍清理 `#splash-style` 并安全结束。
- 重复调用 `dismiss()` -> 后续调用直接返回，不改变已进入的删除流程。

### 5. Good/Base/Bad Cases

- Good: `index.html` 暴露 `window.pebbleSplash`，`App.tsx` 监听 `pebble:splash-removed` 记录打点，并只调用 `dismiss("app-mounted")`。
- Base: React 很快挂载，控制器等到最小显示时间后再淡出，避免描边动画被截断。
- Bad: 在 `App.tsx` 中重新写 `document.getElementById("splash")?.remove()`，会让启动遮罩的 owner 分裂，React 未挂载时仍可能失去清理路径。

### 6. Tests Required

- `pnpm exec tsc --noEmit`，确认全局类型可用。
- `pnpm lint`，确认 React 侧无 lint 问题。
- `pnpm build:frontend`，确认 `index.html` 启动脚本能进入生产构建。
- 触及 `startupTiming` 时运行相关 Vitest；如果新增可测试模块，补单元测试覆盖正常挂载、未挂载超时、transition 丢失和重复调用。

### 7. Wrong vs Correct

#### Wrong

```typescript
const splash = document.getElementById("splash");
splash?.remove();
document.getElementById("splash-style")?.remove();
```

#### Correct

```typescript
window.addEventListener("pebble:splash-removed", () => {
  logStartupTiming("splash removed");
}, { once: true });

window.pebbleSplash?.dismiss("app-mounted");
```
