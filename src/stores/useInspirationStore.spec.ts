import { nextTick } from "vue";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { invoke } from "@tauri-apps/api/core";

import { useInspirationStore } from "@/stores/useInspirationStore";

const STORAGE_KEY = "fl-inspirations";
const DEFERRED_RECOMMENDATION_STORAGE_KEY = "fl-inspiration-deferred-recommendations";
const DEFAULT_CREATED_AT = "2026-04-01T00:00:00.000Z";

function backendInspiration(overrides: Record<string, unknown> = {}) {
  return {
    id: "created-1",
    content: "draft idea",
    goal_id: null,
    image_path: null,
    summary: null,
    keywords: [],
    verification: "none",
    embedding_status: "pending",
    converted_task_id: null,
    converted_at: null,
    created_at: DEFAULT_CREATED_AT,
    updated_at: DEFAULT_CREATED_AT,
    ...overrides,
  };
}

function mockInvokeByCommand(handlers: Record<string, (args?: unknown) => unknown>) {
  vi.mocked(invoke).mockImplementation((cmd, args) => {
    const handler = handlers[String(cmd)];
    if (!handler) return Promise.resolve(undefined);
    try {
      return Promise.resolve(handler(args));
    } catch (error) {
      return Promise.reject(error);
    }
  });
}

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
    mockInvokeByCommand({
      list_inspirations: () => [],
      batch_embed_pending: () => 0,
      create_inspiration: () => backendInspiration(),
      extract_inspiration_keywords: () => ({ keywords: [], summary: null }),
    });

    const store = useInspirationStore();
    const item = await store.create("  draft idea  ");

    expect(item).toMatchObject({
      content: "draft idea",
      goalId: null,
      imagePath: null,
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
    mockInvokeByCommand({
      list_inspirations: () => [],
      batch_embed_pending: () => 0,
      migrate_inspirations_from_local: () => 0,
    });

    const store = useInspirationStore();
    await store.ensureLoaded();

    expect(store.items).toHaveLength(1);
    expect(store.items[0]).toMatchObject({
      id: "legacy-1",
      content: "legacy note",
      goalId: null,
      imagePath: null,
      summary: null,
      keywords: [],
      verification: "none",
      embeddingStatus: "pending",
    });
  });

  it("ensureLoaded normalizes backend image_path into imagePath", async () => {
    mockInvokeByCommand({
      list_inspirations: () => [backendInspiration({ id: "with-image", image_path: "C:\\fake\\idea.png" })],
      batch_embed_pending: () => 0,
    });

    const store = useInspirationStore();
    await store.ensureLoaded();

    expect(store.items[0]).toMatchObject({
      id: "with-image",
      imagePath: "C:\\fake\\idea.png",
    });
  });

  it("create forwards clipboard image payload to backend", async () => {
    const captured: Array<{ cmd: string; args?: unknown }> = [];
    mockInvokeByCommand({
      list_inspirations: () => [],
      batch_embed_pending: () => 0,
      create_inspiration: (args) => {
        captured.push({ cmd: "create_inspiration", args });
        return backendInspiration({ image_path: "C:\\fake\\idea.png" });
      },
      extract_inspiration_keywords: () => ({ keywords: [], summary: null }),
    });

    const store = useInspirationStore();
    const item = await store.create("draft idea", {
      image: {
        bytes: [137, 80, 78, 71],
        mimeType: "image/png",
      },
    });

    expect(captured).toHaveLength(1);
    expect(captured[0]?.args).toEqual({
      input: {
        content: "draft idea",
        goalId: null,
        imageBytes: [137, 80, 78, 71],
        imageMimeType: "image/png",
      },
    });
    expect(item?.imagePath).toBe("C:\\fake\\idea.png");
  });

  it("create does not add a local-only inspiration when persistence fails", async () => {
    mockInvokeByCommand({
      list_inspirations: () => [],
      batch_embed_pending: () => 0,
      create_inspiration: () => {
        throw new Error("database unavailable");
      },
    });

    const store = useInspirationStore();

    await expect(store.create("draft idea")).rejects.toThrow("database unavailable");
    expect(store.items).toEqual([]);
  });

  it("acceptRecommendation clears stale duplicate recommendations", async () => {
    const recommendation = {
      candidateId: "idea-2",
      candidateContent: "old idea",
      relation: "related" as const,
      reason: "same method",
      confidence: 0.9,
    };
    mockInvokeByCommand({
      list_inspirations: () => [],
      batch_embed_pending: () => 0,
      link_inspirations: () => {
        throw new Error("DUPLICATE_LINK:already linked");
      },
    });

    const store = useInspirationStore();
    store.pendingRecommendations["idea-1"] = [recommendation];

    await expect(store.acceptRecommendation("idea-1", recommendation)).resolves.toBeUndefined();
    expect(store.pendingRecommendations["idea-1"]).toEqual([]);
  });

  it("acceptRecommendation persists ai source type and reason", async () => {
    const recommendation = {
      candidateId: "idea-2",
      candidateContent: "old idea",
      relation: "contradicts" as const,
      reason: "新实验结果和旧结论冲突",
      confidence: 0.93,
    };
    const captured: Array<{ cmd: string; args?: unknown }> = [];
    mockInvokeByCommand({
      list_inspirations: () => [],
      batch_embed_pending: () => 0,
      link_inspirations: (args) => {
        captured.push({ cmd: "link_inspirations", args });
        return {
          id: "link-1",
          sourceId: "idea-1",
          targetId: "idea-2",
          relation: "contradicts",
          sourceType: "ai_accepted",
          reason: recommendation.reason,
          createdAt: DEFAULT_CREATED_AT,
        };
      },
    });

    const store = useInspirationStore();
    store.pendingRecommendations["idea-1"] = [recommendation];

    await store.acceptRecommendation("idea-1", recommendation);

    expect(captured).toHaveLength(1);
    expect(captured[0]?.args).toEqual({
      input: {
        sourceId: "idea-1",
        targetId: "idea-2",
        relation: "contradicts",
        sourceType: "ai_accepted",
        reason: recommendation.reason,
      },
    });
    expect(store.linksById["idea-1"]?.[0]).toMatchObject({
      sourceType: "ai_accepted",
      reason: recommendation.reason,
    });
    expect(store.pendingRecommendations["idea-1"]).toEqual([]);
  });

  it("ignoreRecommendation persists ignored relation and clears local pending state", async () => {
    const recommendation = {
      candidateId: "idea-2",
      candidateContent: "old idea",
      relation: "contradicts" as const,
      reason: "这条候选当前不采纳",
      confidence: 0.88,
    };
    const captured: Array<{ cmd: string; args?: unknown }> = [];
    mockInvokeByCommand({
      list_inspirations: () => [],
      batch_embed_pending: () => 0,
      ignore_inspiration_recommendation: (args) => {
        captured.push({ cmd: "ignore_inspiration_recommendation", args });
        return undefined;
      },
    });

    const store = useInspirationStore();
    store.pendingRecommendations["idea-1"] = [recommendation];
    store.deferRecommendation("idea-1", "idea-2", "contradicts");

    await store.ignoreRecommendation("idea-1", "idea-2");

    expect(captured).toHaveLength(1);
    expect(captured[0]?.args).toEqual({
      input: {
        sourceId: "idea-1",
        candidateId: "idea-2",
        relation: "contradicts",
      },
    });
    expect(store.pendingRecommendations["idea-1"]).toEqual([]);
    expect(store.deferredRecommendationKeys).toEqual([]);
  });

  it("deferRecommendation persists keys and restore helpers remove them", async () => {
    mockInvokeByCommand({
      list_inspirations: () => [],
      batch_embed_pending: () => 0,
    });

    const store = useInspirationStore();

    store.deferRecommendation("idea-1", "idea-2", "related");
    store.deferRecommendation("idea-1", "idea-3", "contradicts");
    store.deferRecommendation("idea-1", "idea-2", "related");
    await nextTick();

    expect(store.deferredRecommendationKeys).toEqual([
      "idea-1::idea-2::related",
      "idea-1::idea-3::contradicts",
    ]);
    expect(localStorage.setItem).toHaveBeenLastCalledWith(
      DEFERRED_RECOMMENDATION_STORAGE_KEY,
      JSON.stringify([
        "idea-1::idea-2::related",
        "idea-1::idea-3::contradicts",
      ]),
    );

    store.restoreDeferredRecommendation("idea-1::idea-2::related");
    await nextTick();
    expect(store.deferredRecommendationKeys).toEqual(["idea-1::idea-3::contradicts"]);

    store.restoreDeferredRecommendations(["idea-1::idea-3::contradicts"]);
    await nextTick();
    expect(store.deferredRecommendationKeys).toEqual([]);
  });

  it("loadRecommendations keeps only deferred keys that still exist in the latest result", async () => {
    localStorage.setItem(
      DEFERRED_RECOMMENDATION_STORAGE_KEY,
      JSON.stringify([
        "idea-1::idea-2::related",
        "idea-1::idea-3::contradicts",
        "idea-9::idea-8::related",
      ]),
    );
    mockInvokeByCommand({
      list_inspirations: () => [],
      batch_embed_pending: () => 0,
      suggest_related_inspirations: () => [
        {
          candidateId: "idea-2",
          candidateContent: "old idea",
          relation: "related",
          reason: "same topic",
          confidence: 0.84,
        },
      ],
    });

    const store = useInspirationStore();

    await store.loadRecommendations("idea-1");
    await nextTick();

    expect(store.deferredRecommendationKeys).toEqual([
      "idea-1::idea-2::related",
      "idea-9::idea-8::related",
    ]);
    expect(store.pendingRecommendations["idea-1"]).toHaveLength(1);
  });
});
