import { beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  getEmailSignature: vi.fn(),
  setEmailSignature: vi.fn(),
}));

vi.mock("../../src/lib/api-client", () => ({
  getEmailSignature: mocks.getEmailSignature,
  setEmailSignature: mocks.setEmailSignature,
}));

import { getSignature, setSignature } from "../../src/lib/signatures";

describe("signatures secure storage", () => {
  beforeEach(() => {
    localStorage.clear();
    mocks.getEmailSignature.mockReset();
    mocks.setEmailSignature.mockReset();
  });

  it("loads signatures from backend storage and clears legacy localStorage", async () => {
    localStorage.setItem("pebble-signatures", JSON.stringify({ "account-1": "legacy" }));
    mocks.getEmailSignature.mockResolvedValue("secure signature");

    const signature = await getSignature("account-1");

    expect(mocks.getEmailSignature).toHaveBeenCalledWith("account-1");
    expect(signature).toBe("secure signature");
    expect(localStorage.getItem("pebble-signatures")).toBeNull();
  });

  it("saves signatures through backend storage without writing localStorage", async () => {
    mocks.setEmailSignature.mockResolvedValue(undefined);

    await setSignature("account-1", "Regards");

    expect(mocks.setEmailSignature).toHaveBeenCalledWith("account-1", "Regards");
    expect(localStorage.getItem("pebble-signatures")).toBeNull();
  });
});
