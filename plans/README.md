# Pebble 动效与排版改进计划

审计基线 commit：`c2ad347`。审计方法：improve-animations skill（Emil Kowalski 标准）全量人工核对，逐条 file:line 验证过。

## 计划清单

| # | 标题 | 严重度 | 状态 |
| --- | --- | --- | --- |
| [001](001-motion-foundation-and-toggle.md) | 动效基建：令牌 + 全局动画开关（用户显式要求） | HIGH | DONE |
| [002](002-toast-enter-exit.md) | Toast 可中断进/退场 | HIGH | DONE |
| [003](003-overlay-popover-entrances.md) | 弹层与抽屉入场 + 抽屉退场 | MEDIUM | DONE |
| [004](004-list-row-interactions.md) | 列表行交互：hover 工具条 / 批量栏 / 归档折叠 / FAB | MEDIUM-HIGH | DONE |
| [005](005-typography-polish.md) | 排版层级与颜色令牌 | MEDIUM | DONE |

## 执行顺序与依赖

**001 必须最先**（定义令牌 + data-motion 总闸，002-005 全部引用）。002 / 003 / 004 相互独立，可任意顺序。**005 最后**（与 001/004 共享文件，避免冲突；且依赖 001 在 AppearanceTab 加的 MOTIONS 组）。

推荐：001 → 002 → 003 → 004 → 005，每个计划完成后跑一次 `pnpm lint && pnpm build:frontend && pnpm test` 再进下一个。

## 全局红线（每个计划都适用）

1. **CommandPalette 禁止加动画**——高频键盘操作瞬时开合是正确设计。
2. 看板 dnd-kit 拖拽动效（KanbanCard 的 useSortable transform/transition）已正确，不许碰。
3. 一切新动效必须走 CSS animation/transition（自动受 001 的 data-motion 总闸控制），禁止 JS 驱动动画。
4. 不引入任何新依赖。
5. 邮件正文（ShadowDomEmail）内部不许注入样式。

## 审计中确认"已经正确、不要动"的部分

- 命令面板无动画（对）。
- 看板拖拽（dnd-kit FLIP）。
- `.skeleton` shimmer 与 `.spinner`。
- 详情页正文 14px/1.7 行高。
- 模态框从中心出现（transform-origin: center 对模态是正确的）。
