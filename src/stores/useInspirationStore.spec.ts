import { beforeEach, describe, expect, it, vi } from "vitest";

import { invoke } from "@tauri-apps/api/core";

import { useInspirationStore } from "@/stores/useInspirationStore";

const STORAGE_KEY = "fl-inspirations";

function createStorageMock() {
  let store: Record<string, string> = {};
  return {
    getItem: vi.fn((key: string) => store[key] ?? null),
    setItem: vi.fn((key: string, value: string) => {
      store[key] = value;
    }),
    removeItem: vi.fn((key: string) => {
      delete store[key];
    }),
    clear: vi.fn(() => {
      store = {};
    }),
  };
}

describe("useInspirationStore", () => {
  const localStorageMock = createStorageMock();

  beforeEach(() => {
    vi.stubGlobal("localStorage", localStorageMock);
    localStorage.clear();
    vi.mocked(invoke).mockReset();
  });

  it("create initializes inspiration graph defaults", async () => {
    vi.mocked(invoke).mockResolvedValueOnce([]);
    vi.mocked(invoke).mockResolvedValueOnce({
      id: "created-1",
      content: "draft idea",
      goal_id: null,
      summary: null,
      keywords: [],
      verification: "none",
      embedding_status: "pending",
      converted_task_id: null,
      converted_at: null,
      created_at: "2026-04-01T00:00:00.000Z",
      updated_at: "2026-04-01T00:00:00.000Z",
    });

    const store = useInspirationStore();
    const item = await store.create("  draft idea  ");

    expect(item).toMatchObject({
      content: "draft idea",
      goalId: null,
      summary: null,
      keywords: [],
      verification: "none",
      embeddingStatus: "pending",
      convertedTaskId: null,
      convertedAt: null,
    });
    expect(store.linksById).toEqual({});
    expect(store.pendingRecommendations).toEqual({});
  });

  it("ensureLoaded backfills legacy storage items with graph defaults", async () => {
    localStorage.setItem(
      STORAGE_KEY,
      JSON.stringify([
        {
          id: "legacy-1",
          content: "legacy note",
          createdAt: "2026-04-01T00:00:00.000Z",
          updatedAt: "2026-04-01T00:00:00.000Z",
          convertedTaskId: null,
          convertedAt: null,
        },
      ]),
    );
    vi.mocked(invoke).mockResolvedValueOnce([]);

    const store = useInspirationStore();
    await store.ensureLoaded();

    expect(store.items).toHaveLength(1);
    expect(store.items[0]).toMatchObject({
      id: "legacy-1",
      content: "legacy note",
      goalId: null,
      summary: null,
      keywords: [],
      verification: "none",
      embeddingStatus: "pending",
    });
  });
});


