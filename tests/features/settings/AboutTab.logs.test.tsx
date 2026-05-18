import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import AboutTab from "../../../src/features/settings/AboutTab";

const mocks = vi.hoisted(() => ({
  readAppLog: vi.fn(),
}));

vi.mock("../../../src/lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../../src/lib/api")>()),
  readAppLog: mocks.readAppLog,
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (_key: string, fallback?: string) => fallback ?? _key,
  }),
}));

describe("AboutTab diagnostics", () => {
  beforeEach(() => {
    mocks.readAppLog.mockReset();
    mocks.readAppLog.mockResolvedValue({
      path: "/var/lib/pebble/logs/pebble.log",
      content: "first line\nlatest line",
      truncated: false,
    });
  });

  it("opens the diagnostic log after five quick app icon clicks", async () => {
    render(<AboutTab />);

    const iconButton = screen.getByRole("button", { name: "Open diagnostic log" });

    for (let i = 0; i < 4; i += 1) {
      fireEvent.click(iconButton);
    }

    expect(mocks.readAppLog).not.toHaveBeenCalled();

    fireEvent.click(iconButton);

    await waitFor(() => {
      expect(mocks.readAppLog).toHaveBeenCalledWith(65536);
    });
    expect(screen.getByRole("dialog", { name: "Diagnostic log" })).toBeTruthy();
    expect(screen.getByText(/latest line/)).toBeTruthy();
    expect(screen.getByText(/pebble\.log$/)).toBeTruthy();
  });

  it("keeps the diagnostic log open when clicking the backdrop", async () => {
    render(<AboutTab />);

    const iconButton = screen.getByRole("button", { name: "Open diagnostic log" });
    for (let i = 0; i < 5; i += 1) {
      fireEvent.click(iconButton);
    }

    const dialog = await screen.findByRole("dialog", { name: "Diagnostic log" });
    fireEvent.mouseDown(dialog);
    fireEvent.click(dialog);

    expect(screen.queryByRole("dialog", { name: "Diagnostic log" })).not.toBeNull();
  });
});
