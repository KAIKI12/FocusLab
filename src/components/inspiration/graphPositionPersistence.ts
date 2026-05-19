export const GRAPH_POSITION_STORAGE_KEY = "fl-inspiration-graph-positions";

export interface PersistedGraphPosition {
  x: number;
  y: number;
}

export type PersistedGraphPositionMap = Record<string, PersistedGraphPosition>;

export function readPersistedGraphPositions(): PersistedGraphPositionMap {
  try {
    const raw = localStorage.getItem(GRAPH_POSITION_STORAGE_KEY);
    if (!raw) return {};
    const parsed = JSON.parse(raw);
    if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) return {};

    const result: PersistedGraphPositionMap = {};
    for (const [id, value] of Object.entries(parsed as Record<string, unknown>)) {
      if (!value || typeof value !== "object") continue;
      const position = value as Record<string, unknown>;
      if (
        typeof position.x !== "number"
        || Number.isNaN(position.x)
        || typeof position.y !== "number"
        || Number.isNaN(position.y)
      ) {
        continue;
      }
      result[id] = { x: position.x, y: position.y };
    }
    return result;
  } catch {
    return {};
  }
}

export function writePersistedGraphPositions(positions: PersistedGraphPositionMap) {
  localStorage.setItem(GRAPH_POSITION_STORAGE_KEY, JSON.stringify(positions));
}

export function prunePersistedGraphPositions(
  positions: PersistedGraphPositionMap,
  validIds: string[],
): PersistedGraphPositionMap {
  const validSet = new Set(validIds);
  return Object.fromEntries(
    Object.entries(positions).filter(([id]) => validSet.has(id)),
  );
}
