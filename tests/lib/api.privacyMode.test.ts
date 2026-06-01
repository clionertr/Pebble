import { beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  getRenderedHtml: vi.fn(),
  getMessageWithHtml: vi.fn(),
}));

vi.mock("../../src/lib/api-client", () => ({
  getRenderedHtml: mocks.getRenderedHtml,
  getMessageWithHtml: mocks.getMessageWithHtml,
}));

import { getMessageWithHtml, getRenderedHtml } from "../../src/lib/api";

describe("privacy mode API parameters", () => {
  beforeEach(() => {
    mocks.getRenderedHtml.mockReset();
    mocks.getMessageWithHtml.mockReset();
  });

  it("serializes load-once mode for the HTTP API", async () => {
    mocks.getRenderedHtml.mockResolvedValue({
      html: "",
      trackers_blocked: [],
      images_blocked: 0,
    });

    await getRenderedHtml("message-1", "LoadOnce");

    expect(mocks.getRenderedHtml).toHaveBeenCalledWith("message-1", "load_once");
  });

  it("serializes strict and off modes for full message rendering", async () => {
    mocks.getMessageWithHtml.mockResolvedValue(null);

    await getMessageWithHtml("message-1", "Strict");
    await getMessageWithHtml("message-1", "Off");

    expect(mocks.getMessageWithHtml).toHaveBeenNthCalledWith(1, "message-1", "strict", undefined);
    expect(mocks.getMessageWithHtml).toHaveBeenNthCalledWith(2, "message-1", "off", undefined);
  });

  it("serializes trusted sender mode without stringifying the object", async () => {
    mocks.getRenderedHtml.mockResolvedValue({
      html: "",
      trackers_blocked: [],
      images_blocked: 0,
    });

    await getRenderedHtml("message-1", { TrustSender: "sender@example.com" });

    expect(mocks.getRenderedHtml).toHaveBeenCalledWith("message-1", "trust:sender@example.com");
  });
});
