import { describe, expect, it } from "vitest";
import {
  extractTranslateStreamDelta,
  extractTranslateStreamFullText,
  parseServerSentEvents,
} from "../../src/lib/api-client";

describe("translate stream parser", () => {
  it("parses complete SSE blocks and keeps an incomplete tail", () => {
    const parsed = parseServerSentEvents(
      'event: response.output_text.delta\ndata: {"delta":"你"}\n\ndata: {"delta":"好"}',
    );

    expect(parsed.events).toEqual([
      { event: "response.output_text.delta", data: '{"delta":"你"}' },
    ]);
    expect(parsed.rest).toBe('data: {"delta":"好"}');
  });

  it("extracts OpenAI chat completion content deltas", () => {
    const delta = extractTranslateStreamDelta({
      choices: [{ delta: { content: "你好" } }],
    });

    expect(delta).toBe("你好");
  });

  it("extracts OpenAI responses output text deltas", () => {
    const delta = extractTranslateStreamDelta({
      type: "response.output_text.delta",
      delta: "你好",
    });

    expect(delta).toBe("你好");
  });

  it("extracts full text from non-delta response events", () => {
    const text = extractTranslateStreamFullText({
      output: [{ content: [{ type: "output_text", text: "你好" }] }],
    });

    expect(text).toBe("你好");
  });
});
