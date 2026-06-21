import { fireEvent, render, waitFor } from "@testing-library/react";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { ShadowDomEmail } from "@/components/ShadowDomEmail";

const mocks = vi.hoisted(() => ({
  open: vi.fn(),
}));

const shadowDomEmailStyles = readFileSync(
  join(process.cwd(), "public/shadow-dom-email.css"),
  "utf8",
);

describe("ShadowDomEmail", () => {
  beforeEach(() => {
    mocks.open.mockReset();
    vi.stubGlobal("open", mocks.open);
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("uses app theme variables instead of hardcoded light text styles", async () => {
    document.documentElement.setAttribute("data-theme", "dark");

    const { container } = render(<ShadowDomEmail html="<p>Hello</p>" />);
    const host = container.firstChild as HTMLDivElement | null;

    await waitFor(() => {
      expect(host?.shadowRoot).not.toBeNull();
    });

    const stylesheet = host!.shadowRoot!.querySelector<HTMLLinkElement>(
      'link[rel="stylesheet"]',
    );

    expect(stylesheet?.getAttribute("href")).toBe("/shadow-dom-email.css");
    expect(shadowDomEmailStyles).toContain("var(--color-text-primary)");
    expect(shadowDomEmailStyles).toContain("var(--color-accent)");
    expect(shadowDomEmailStyles).not.toContain("color: #1a1a1a");
  });

  it("themes horizontal overflow inside email content", async () => {
    const { container } = render(<ShadowDomEmail html="<pre>long code line</pre>" />);
    const host = container.firstChild as HTMLDivElement | null;

    await waitFor(() => {
      expect(host?.shadowRoot).not.toBeNull();
    });

    expect(shadowDomEmailStyles).toContain("scrollbar-width: thin");
    expect(shadowDomEmailStyles).toContain("::-webkit-scrollbar-thumb");
  });

  it("keeps light-authored email html readable in dark theme", async () => {
    document.documentElement.setAttribute("data-theme", "dark");

    const { container } = render(
      <ShadowDomEmail html={'<div style="color: #000000">Dark inline text</div>'} />,
    );
    const host = container.firstChild as HTMLDivElement | null;

    await waitFor(() => {
      expect(host?.shadowRoot).not.toBeNull();
    });

    expect(host!.shadowRoot!.innerHTML).toContain('class="pebble-email-content"');
    expect(shadowDomEmailStyles).toContain(':host-context([data-theme="dark"]) .pebble-email-content');
    expect(shadowDomEmailStyles).toContain("color-scheme: light");
    expect(shadowDomEmailStyles).toContain("background: #fff");
    expect(shadowDomEmailStyles).toContain("color: #202124");
  });

  it("prevents full-height email wrappers from painting a gray viewport canvas", async () => {
    const html = `
      <table height="100%" style="height: 100%; background: #f1f1f1">
        <tbody><tr><td>Cloudflare content</td></tr></tbody>
      </table>
    `;

    const { container } = render(<ShadowDomEmail html={html} />);
    const host = container.firstChild as HTMLDivElement | null;

    await waitFor(() => {
      expect(host?.shadowRoot?.querySelector(".pebble-email-content")).not.toBeNull();
    });

    const shadowMarkup = host!.shadowRoot!.innerHTML;
    expect(shadowDomEmailStyles).toContain('.pebble-email-content > table[height="100%"]');
    expect(shadowMarkup).toContain('style="height: 100%; background: #f1f1f1"');
    expect(shadowDomEmailStyles).toContain("height: auto !important");
    expect(shadowDomEmailStyles).toContain("min-height: 0 !important");
  });

  it("opens http and https links in a new browser tab", async () => {
    const { container } = render(
      <ShadowDomEmail html={'<a href="http://pebble.byebug.cn/">Pebble</a>'} />,
    );
    const host = container.firstChild as HTMLDivElement | null;

    await waitFor(() => {
      expect(host?.shadowRoot?.querySelector("a")).not.toBeNull();
    });

    fireEvent.click(host!.shadowRoot!.querySelector("a")!);

    expect(mocks.open).toHaveBeenCalledWith("http://pebble.byebug.cn/", "_blank", "noopener,noreferrer");
  });

});
