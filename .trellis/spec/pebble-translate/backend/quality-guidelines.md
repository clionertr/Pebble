# 翻译质量指南

> `pebble-translate` 的提供商解析、错误处理和前端渐进展示契约。

---

## Scenario: 翻译提供商响应解析与渐进显示

### 1. Scope / Trigger
- Trigger: 翻译功能横跨前端设置页、`/api/translate`、`server/src/rpc/translate.rs` 和 `crates/pebble-translate`。提供商返回格式不稳定时，最容易出现“接口有返回但 UI 不显示”或“成功返回空翻译”的假成功。
- 范围：`crates/pebble-translate/src/*.rs`、`server/src/rpc/translate.rs`、`src/hooks/useBilingualTranslation.ts`、`src/components/MessageDetail.tsx`、`src/features/translate/TranslatePopover.tsx`。

### 2. Signatures
- 主翻译接口：`POST /api/translate`，body `{ "text": string, "fromLang": string, "toLang": string }`。
- LLM 流式翻译接口：`POST /api/translate/stream`，body 同主翻译接口，response 为 `text/event-stream`。
- 成功响应：`TranslateResult { translated: string, segments: BilingualSegment[] }`。
- 配置保存：`PUT /api/translate/config`，body `{ providerType: string, config: string, isEnabled: boolean }`。
- LLM OpenAI-compatible modes：`completions` 调 `/v1/chat/completions`，`responses` 调 `/v1/responses`；流式调用必须设置 `stream: true`。

### 3. Contracts
- `translated` 不能为空。提供商成功 HTTP 响应中缺少翻译字段或字段为空时，必须返回 `PebbleError::Translate`，不得静默返回 `translated: ""`。
- DeepLX 的 `fromLang=auto` 必须保持小写 `auto`；其他语言码可转大写。DeepL 官方 API 遇到 `auto` 源语言时应省略 `source_lang`，让服务端自动检测。
- DeepLX `401 Unauthorized` 表示远端 endpoint token 无效或过期，不是 Pebble 登录态失效；错误文案要指出 token/endpoint 问题。
- LLM Responses API 非流式需要优先识别顶层 `output_text`，并兼容 `output[].content[].text`。
- LLM 流式翻译必须走后端中转：`frontend -> /api/translate/stream -> LLM SSE`。前端不能直接请求第三方 LLM API，也不能暴露 API key。
- 前端流式解析需要兼容 Chat Completions 的 `choices[].delta.content` 和 Responses 的 `response.output_text.delta` / `delta`。
- 前端双语模式只要有 `bilingualResult`，即使 `bilingualLoading=true` 也必须显示已到达的部分翻译；loading 只能作为“仍在继续”的提示，不能覆盖结果。
- 非 LLM 提供商没有 token 流，`/api/translate/stream` 应返回“仅 LLM 支持流式”的错误，前端再回退到普通 `/api/translate`。

### 4. Validation & Error Matrix
- DeepLX HTTP 401 -> `DeepLX unauthorized (401): endpoint token is invalid or expired...`。
- DeepLX JSON `code >= 400` -> `DeepLX error <code>: <message>`。
- DeepLX 缺 `data` 或 `data` 为空 -> 翻译错误。
- DeepL 缺 `translations[0].text` 或为空 -> 翻译错误。
- Generic API 的 `result_path` 指向缺失/非字符串/空字符串 -> 翻译错误。
- LLM 缺 `choices[0].message.content`、`output_text`、`output[].content[].text` 或解析后为空 -> 翻译错误。
- 非 LLM 调 `/api/translate/stream` -> `Streaming translation is only supported for LLM providers`，前端应回退普通翻译。
- 保存配置时 `providerType` 与 `config.type` 不一致 -> 翻译配置错误，不写入存储。

### 5. Good/Base/Bad Cases
- Good: LLM Responses 流式返回 `response.output_text.delta` 事件，后端即时转发 SSE，前端收到 `delta` 后立即更新正文。
- Base: HTML 邮件分块翻译时，每个 chunk 成功后更新 `bilingualResult`，正文区域立即显示部分译文，同时保留“Translating...”提示。
- Base: DeepLX/DeepL/Generic API 没有流式 token，前端先尝试流式失败后回退普通翻译，最终显示完整译文。
- Bad: 后端 `unwrap_or("")` 吞掉缺字段错误，前端得到空 `translated`，用户只看到空白或误以为翻译失败。
- Bad: UI 先判断 `bilingualLoading` 并直接返回 loading 组件，导致已经写入状态的部分结果不可见。
- Bad: 前端拿到完整翻译后再用定时器伪造打字机效果，这不是用户要求的后端/LLM 真流式。

### 6. Tests Required
- Rust 单元测试：DeepLX 401 错误文案包含 token/expired 提示。
- Rust 单元测试：DeepLX `auto` 不转成 `AUTO`；DeepL `auto` 源语言不发送 `source_lang`。
- Rust 单元测试：LLM Responses 顶层 `output_text` 和嵌套 `output[].content[].text` 都能解析。
- Rust 单元测试：LLM 流式 request body 包含 `stream: true`。
- 前端单元测试：SSE parser 能解析 Chat Completions 和 Responses 的 delta。
- 前端回归测试：`bilingualLoading=true` 且 `bilingualResult` 已存在时，`MessageDetail` 同时显示 loading 提示和部分译文。
- 项目检查：`cargo test -p pebble-translate`、`cargo test -p pebble`、`pnpm exec tsc --noEmit`、`pnpm test`。

### 7. Wrong vs Correct

#### Wrong
```rust
let translated = json.get("data").and_then(|d| d.as_str()).unwrap_or("").to_string();
```

```tsx
{bilingualMode && bilingualLoading ? <Spinner /> : <TranslatedBody />}
```

#### Correct
```rust
let translated = json
    .get("data")
    .and_then(|data| data.as_str())
    .ok_or_else(|| PebbleError::Translate("DeepLX response missing data field".to_string()))?;
```

```tsx
{bilingualMode && bilingualResult ? <TranslatedBody /> : bilingualLoading ? <Spinner /> : null}
```
