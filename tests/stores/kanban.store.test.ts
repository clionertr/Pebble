import { vi, describe, it, expect, beforeEach } from "vitest";

const mocks = vi.hoisted(() => ({
  listKanbanCards: vi.fn(),
  listKanbanContextNotes: vi.fn(),
  mergeKanbanContextNotes: vi.fn(),
  moveToKanban: vi.fn(),
  removeFromKanban: vi.fn(),
  setKanbanContextNote: vi.fn(),
}));

vi.mock("../../src/lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../src/lib/api")>()),
  listKanbanCards: mocks.listKanbanCards,
  listKanbanContextNotes: mocks.listKanbanContextNotes,
  mergeKanbanContextNotes: mocks.mergeKanbanContextNotes,
  moveToKanban: mocks.moveToKanban,
  removeFromKanban: mocks.removeFromKanban,
  setKanbanContextNote: mocks.setKanbanContextNote,
}));

import { useKanbanStore } from "../../src/stores/kanban.store";

describe("KanbanStore", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    localStorage.clear();
    useKanbanStore.setState({ cards: [], cardIdSet: new Set(), contextNotes: {}, loading: false });
  });

  it("fetchCards loads cards from backend", async () => {
    const mockCards = [
      { message_id: "m1", column: "todo", position: 0, created_at: 1000, updated_at: 1000 },
      { message_id: "m2", column: "done", position: 1, created_at: 1000, updated_at: 1000 },
    ];
    mocks.listKanbanCards.mockResolvedValue(mockCards);
    mocks.listKanbanContextNotes.mockResolvedValue({});

    await useKanbanStore.getState().fetchCards();

    expect(mocks.listKanbanCards).toHaveBeenCalledWith();
    expect(mocks.listKanbanContextNotes).toHaveBeenCalledWith();
    expect(useKanbanStore.getState().cards).toHaveLength(2);
    expect(useKanbanStore.getState().loading).toBe(false);
  });

  it("moveCard performs optimistic update", async () => {
    useKanbanStore.setState({
      cards: [
        { message_id: "m1", column: "todo", position: 0, created_at: 1000, updated_at: 1000 },
      ],
    });
    mocks.moveToKanban.mockResolvedValueOnce(undefined);

    await useKanbanStore.getState().moveCard("m1", "done", 0);

    expect(useKanbanStore.getState().cards[0].column).toBe("done");
    expect(mocks.moveToKanban).toHaveBeenCalledWith("m1", "done", 0);
  });

  it("moveCard rolls back on error", async () => {
    useKanbanStore.setState({
      cards: [
        { message_id: "m1", column: "todo", position: 0, created_at: 1000, updated_at: 1000 },
      ],
    });
    mocks.moveToKanban.mockRejectedValueOnce(new Error("fail"));

    await useKanbanStore.getState().moveCard("m1", "done", 0);

    expect(useKanbanStore.getState().cards[0].column).toBe("todo");
  });

  it("removeCard removes optimistically", async () => {
    useKanbanStore.setState({
      cards: [
        { message_id: "m1", column: "todo", position: 0, created_at: 1000, updated_at: 1000 },
      ],
    });
    mocks.removeFromKanban.mockResolvedValueOnce(undefined);

    await useKanbanStore.getState().removeCard("m1");

    expect(useKanbanStore.getState().cards).toHaveLength(0);
  });

  it("removeCard rolls back on error", async () => {
    useKanbanStore.setState({
      cards: [
        { message_id: "m1", column: "todo", position: 0, created_at: 1000, updated_at: 1000 },
      ],
    });
    mocks.removeFromKanban.mockRejectedValueOnce(new Error("fail"));

    await useKanbanStore.getState().removeCard("m1");

    expect(useKanbanStore.getState().cards).toHaveLength(1);
  });

  it("stores context notes through backend storage only", async () => {
    mocks.setKanbanContextNote.mockResolvedValueOnce({ m1: "follow up on the selected paragraph" });

    await useKanbanStore.getState().setContextNote("m1", "follow up on the selected paragraph");

    expect(useKanbanStore.getState().contextNotes.m1).toBe("follow up on the selected paragraph");
    expect(mocks.setKanbanContextNote).toHaveBeenCalledWith("m1", "follow up on the selected paragraph");
    expect(localStorage.getItem("pebble-kanban-context-notes")).toBeNull();
  });
});
