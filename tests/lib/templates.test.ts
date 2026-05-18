import { beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  listEmailTemplates: vi.fn(),
  saveEmailTemplate: vi.fn(),
  deleteEmailTemplate: vi.fn(),
}));

vi.mock("../../src/lib/api-client", () => ({
  listEmailTemplates: mocks.listEmailTemplates,
  saveEmailTemplate: mocks.saveEmailTemplate,
  deleteEmailTemplate: mocks.deleteEmailTemplate,
}));

import { deleteTemplate, listTemplates, saveTemplate } from "../../src/lib/templates";

describe("templates secure storage", () => {
  beforeEach(() => {
    localStorage.clear();
    mocks.listEmailTemplates.mockReset();
    mocks.saveEmailTemplate.mockReset();
    mocks.deleteEmailTemplate.mockReset();
  });

  it("loads templates from backend storage and clears legacy localStorage", async () => {
    localStorage.setItem("pebble-templates", JSON.stringify([{ id: "legacy" }]));
    mocks.listEmailTemplates.mockResolvedValue([{ id: "template-1", name: "Intro", subject: "Hello", body: "Body", createdAt: 1 }]);

    const templates = await listTemplates();

    expect(mocks.listEmailTemplates).toHaveBeenCalledWith();
    expect(templates).toHaveLength(1);
    expect(localStorage.getItem("pebble-templates")).toBeNull();
  });

  it("saves and deletes templates through backend storage without writing localStorage", async () => {
    mocks.saveEmailTemplate
      .mockResolvedValueOnce({ id: "template-1", name: "Intro", subject: "Hello", body: "Body", createdAt: 1 });
    mocks.deleteEmailTemplate.mockResolvedValueOnce(undefined);

    await saveTemplate({ name: "Intro", subject: "Hello", body: "Body" });
    await deleteTemplate("template-1");

    expect(mocks.saveEmailTemplate).toHaveBeenCalledWith({ name: "Intro", subject: "Hello", body: "Body" });
    expect(mocks.deleteEmailTemplate).toHaveBeenCalledWith("template-1");
    expect(localStorage.getItem("pebble-templates")).toBeNull();
  });
});
