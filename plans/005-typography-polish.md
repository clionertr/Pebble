# 005 — 排版层级与颜色令牌打磨

- **Status**: DONE
- **修订 A（审查方裁决）**：`#f59e0b` 只替换**星标语义**的出现处（渲染 Star 图标 fill/color 的地方：MessageItem、ThreadItem、StarredView 等）。以下两处**保持原样不动**：`src/lib/accountColors.ts:4`（"Amber" 是账户标记色盘的一员，不是星标）、`src/components/PrivacyBanner.tsx:40`（Shield 警示色，语义不同）。完成标准相应改为：`grep -rn "f59e0b" src` 恰好剩这 2 处。
- **Commit**: c2ad347
- **Severity**: MEDIUM
- **Category**: 排版 / Cohesion & tokens
- **Estimated scope**: 7 个文件（index.css、MessageItem.tsx、ThreadItem.tsx、MessageDetail.tsx、Sidebar.tsx、ConfirmDialog.tsx、ToastContainer.tsx、AppearanceTab.tsx）

依赖：001（令牌位置）。建议最后执行（001-004 之后），避免同文件冲突。

## 问题

1. **已读行层级是平的**（src/components/MessageItem.tsx）：已读邮件的发件人（13px 主色 normal）、主题（12.5px 主色 normal）、摘要（12px 次色 normal）几乎无差别，扫视时抓不到重点；12.5px 是刻度外的怪值。

```tsx
// MessageItem.tsx:58 — 现状
const fontWeight = message.is_read ? "normal" : "600";
// :157 fontSize: "13px"（发件人） :197 fontSize: "12.5px"（主题） :208 fontSize: "12px"（摘要）
```

ThreadItem.tsx 同款问题（:28, :72, :118, :129）。

2. **日期数字不等宽**：列表右侧时间戳（MessageItem.tsx:184-192、ThreadItem.tsx:110）逐行宽度不一，列不齐。
3. **详情页主题偏小**（src/components/MessageDetail.tsx:289-292）：`fontSize: "15px", fontWeight: "600"` 作为详情页大标题存在感不足，与列表行 13px 拉不开。
4. **颜色硬编码散落**：
   - 星标黄 `#f59e0b` 在 src/ 出现 6 处（MessageItem.tsx ×4、其余 grep `#f59e0b` 可得）。
   - 危险红两套并存：`#ef4444`（ConfirmDialog.tsx:174、Layout.tsx OfflineBanner:340-343）与 `#c0392b`（ToastContainer.tsx:13 error accent）——同一个"危险"语义两种红。
5. **设置卡片选中态抖 1px**（src/features/settings/AppearanceTab.tsx:36-38）：选中 `2px solid` / 未选中 `1px solid`，切换时卡片内容位移 1px。

```tsx
// AppearanceTab.tsx:36-38 — 现状
border:
  theme === th.id ? "2px solid var(--color-accent)" : "1px solid var(--color-border)",
```

6. **侧栏分组标签无字距**（src/components/Sidebar.tsx:222-224、450-451）：11px/600 的小标签没有 letter-spacing，小字号加粗后发闷。

## 目标

### 1. 颜色令牌（index.css `:root`，颜色变量区内追加；dark 块不用重复——两个值在深浅色下都可用）

```css
  --color-star: #f59e0b;
  --color-danger: #ef4444;
```

替换：

- 全部 6 处 `#f59e0b` → `var(--color-star)`（字符串模板处写 `"var(--color-star)"`）。
- ConfirmDialog.tsx:174 `"#ef4444"` → `"var(--color-danger)"`。
- ToastContainer.tsx:13 `error: "#c0392b"` → `error: "var(--color-danger)"`（刻意统一，视觉会变亮一档，属预期）。
- Layout.tsx OfflineBanner 内 `color: "#ef4444"` → `"var(--color-danger)"`（`rgba(239,68,68,…)` 背景/边框两行保持原样，别动）。
- ViewErrorBoundary 内 `color: "#ef4444"`（Layout.tsx:272）同样替换。

### 2. 列表行层级（MessageItem.tsx）

- `:58` 改为 `const fontWeight = message.is_read ? "normal" : "600";` 保持不变，但发件人 span（:152-163 那个外层 flex span）内联样式加 `fontWeight: message.is_read ? 500 : 600`——已读行发件人升到 medium，与正文拉开一档。
- 主题 div（:195-205）：`fontSize: "12.5px"` → `"13px"`。
- 摘要 div（:206-215）保持 12px 次色不变。
- 日期 span（:184-192）加 `fontVariantNumeric: "tabular-nums"`。

### 3. ThreadItem.tsx 同款

- 发件人区（:72 附近的 flex span）加 `fontWeight: hasUnread ? 600 : 500`（变量名以文件内为准，:28 是 `hasUnread`）。
- `:118` `fontSize: "12.5px"` → `"13px"`。
- 日期（:110）加 `fontVariantNumeric: "tabular-nums"`。

### 4. 详情页主题（MessageDetail.tsx:289-292）

```tsx
fontSize: "17px",
fontWeight: "600",
letterSpacing: "-0.01em",
lineHeight: 1.35,
```

（15px→17px；大字号收字距是排版惯例；lineHeight 防两行主题贴死。）

### 5. 侧栏分组标签（Sidebar.tsx:222-224 与 450-451 两处 11px/600）

各加一行 `letterSpacing: "0.04em"`。

### 6. 设置卡片去抖（AppearanceTab.tsx，THEMES / LANGUAGES / 001 加的 MOTIONS 三组按钮统一改）

```tsx
border: "1px solid " + (selected ? "var(--color-accent)" : "var(--color-border)"),
boxShadow: selected ? "inset 0 0 0 1px var(--color-accent)" : "none",
```

（selected 指各组自己的判断：`theme === th.id` / `language === l.id` / `motion === m.id`。边框恒 1px + 内描边补 1px，视觉等价 2px 而零位移。）

## 仓库约定

- 颜色一律 CSS 变量（index.css:3-20 是现有清单），组件内联样式引用 `var(--...)`。
- 字号刻度实际使用：11 / 12 / 13 / 14 / 15 / 17，不再引入 x.5 值。

## 步骤

1. index.css 加两个颜色令牌。
2. 全库替换 `#f59e0b`（grep 确认 6 处清零）。
3. 危险红替换（ConfirmDialog、ToastContainer、Layout ×2）。
4. MessageItem / ThreadItem 层级 + tabular-nums。
5. MessageDetail 主题字号。
6. Sidebar 标签字距。
7. AppearanceTab 卡片去抖。

## 边界

- 不改颜色值本身（--color-star 就是原来的 #f59e0b）。危险红统一到 #ef4444 是本计划唯一的视觉色变化。
- 不改 ShadowDomEmail / 邮件正文渲染（邮件 HTML 自带排版，不许注入样式）。
- 不动 compose 编辑器的字号（tiptap 区域已是 14px/1.7，正确）。
- 不引入字体文件或新依赖。

## 验证

- **机械**：`pnpm lint`、`pnpm build:frontend`、`pnpm test` 全绿；`grep -rn "#f59e0b" src` 无结果；`grep -rn "12.5px" src` 无结果。
- **感受**：
  - 收件箱扫一眼已读邮件：发件人（medium）→ 主题（regular 主色）→ 摘要（灰）三层清晰。
  - 列表右缘时间戳竖向对齐（数字等宽）。
  - 打开一封邮件：主题明显是页面主标题。
  - 设置页连续切换主题卡片：卡片边框高亮切换，内容纹丝不动（无 1px 抖动）。
  - 深色模式下复查星标、危险红、选中卡片均正常。
- **完成标准**：上述全过。
