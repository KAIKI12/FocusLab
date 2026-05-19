import { describe, expect, it, vi } from "vitest";

import {
  GRAPH_POSITION_STORAGE_KEY,
  prunePersistedGraphPositions,
  readPersistedGraphPositions,
  writePersistedGraphPositions,
} from "@/components/inspiration/graphPositionPersistence";

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

describe("graphPositionPersistence", () => {
  const localStorageMock = createStorageMock();

  it("reads only valid persisted positions", () => {
    vi.stubGlobal("localStorage", localStorageMock);
    localStorage.setItem(
      GRAPH_POSITION_STORAGE_KEY,
      JSON.stringify({
        "idea-1": { x: 120, y: 240 },
        "idea-2": { x: "bad", y: 300 },
        "idea-3": { x: 320, y: null },
      }),
    );

    expect(readPersistedGraphPositions()).toEqual({
      "idea-1": { x: 120, y: 240 },
    });
  });

  it("writes positions with the shared storage key", () => {
    vi.stubGlobal("localStorage", localStorageMock);

    writePersistedGraphPositions({
      "idea-1": { x: 180, y: 220 },
      "idea-2": { x: 320, y: 410 },
    });

    expect(localStorage.setItem).toHaveBeenLastCalledWith(
      GRAPH_POSITION_STORAGE_KEY,
      JSON.stringify({
        "idea-1": { x: 180, y: 220 },
        "idea-2": { x: 320, y: 410 },
      }),
    );
  });

  it("prunes removed node ids before persisting", () => {
    expect(
      prunePersistedGraphPositions(
        {
          "idea-1": { x: 10, y: 20 },
          "idea-2": { x: 30, y: 40 },
          "idea-3": { x: 50, y: 60 },
        },
        ["idea-1", "idea-3"],
      ),
    ).toEqual({
      "idea-1": { x: 10, y: 20 },
      "idea-3": { x: 50, y: 60 },
    });
  });
});
