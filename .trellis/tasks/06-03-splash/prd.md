# 重构启动 splash 生命周期管理

## Goal

将启动 splash 的生命周期收敛到 `index.html` 内的独立控制器中，避免 React 挂载失败、启动链路异常或 CSS transition 事件丢失时，`pebble-outline` SVG 全屏遮罩永久覆盖 UI。React 只负责发出“应用已准备好”的信号，不再直接拥有 splash DOM 的销毁逻辑。

## What I already know

* 当前 splash 在 `index.html` 中先于 React 渲染，使用 `position: fixed; inset: 0; z-index: 99999` 覆盖整个视口。
* 提交 `2c35e42` 已加入 `window.__pebbleDismissSplash()` 和 6 秒自毁兜底，解决了永久遮挡问题。
* 当前 `App.tsx` 仍保留 fallback DOM 删除逻辑，控制权分散在 HTML 与 React 两处。
* 更优雅的目标是：HTML 控制器统一负责最小显示时间、淡出、超时兜底、幂等删除和事件通知；React 只调用 ready/dismiss API。

## Assumptions

* 不引入新依赖。
* 不改变 splash 的视觉样式和最小显示时长策略。
* 保留启动耗时日志，但让“splash removed”在真正删除时触发。

## Requirements

* `index.html` 暴露结构化的 `window.pebbleSplash` 控制器。
* 控制器提供幂等的 `dismiss(reason?: string)` 方法。
* 控制器继续支持最小显示时间和最大显示时间。
* 控制器在 transition 丢失时仍能用 timeout 删除 DOM。
* 控制器在真正删除后派发事件，供 React/应用层打点。
* `App.tsx` 不再直接操作 `#splash` DOM，只通知控制器应用已挂载。
* 为 `window.pebbleSplash` 增加 TypeScript 全局类型。

## Acceptance Criteria

* [x] React 正常挂载时，调用 `window.pebbleSplash.dismiss("app-mounted")` 或等价 ready 信号。
* [x] React 未挂载时，splash 最多显示约 6 秒后自动淡出并移除。
* [x] transitionend 未触发时，splash 仍会被 timeout 移除。
* [x] 重复调用 dismiss 不会重复删除或报错。
* [x] `App.tsx` 中没有直接 `document.getElementById("splash")` 删除逻辑。
* [x] TypeScript 类型检查通过。

## Definition of Done

* 相关代码修改完成。
* lint / typecheck 通过，或明确记录无法运行的原因。
* 若发现应沉淀的新约定，更新 Trellis spec；否则说明无需更新。

## Out of Scope

* 不重做 splash 视觉设计。
* 不引入启动失败错误页。
* 不调整 i18n/queryClient/AuthProvider 启动顺序。

## Technical Notes

* 主要文件：`index.html`、`src/App.tsx`。
* 可能新增类型文件：`src/vite-env.d.ts` 或项目已有全局声明文件。
* 当前打点函数为 `src/lib/startupTiming.ts` 的 `logStartupTiming`。
