/**
 * useInspirationStore · 灵感速记 store。
 * - 负责本地灵感卡片状态
 * - 使用 localStorage 持久化
 * - 支持一键转任务
 */

import { defineStore } from "pinia";
import { computed, ref, watch } from "vue";

import { useTaskStore } from "@/stores/useTaskStore";

const STORAGE_KEY = "fl-inspirations";

export interface InspirationItem {
  id: string;
  content: string;
  createdAt: string;
  updatedAt: string;
  convertedTaskId: string | null;
  convertedAt: string | null;
}

function nowIso() {
  return new Date().toISOString();
}

function normalizeItems(raw: unknown): InspirationItem[] {
  if (!Array.isArray(raw)) return [];

  return raw
    .map((item): InspirationItem | null => {
      if (!item || typeof item !== "object") return null;
      const record = item as Record<string, unknown>;
      const content = String(record.content ?? "").trim();
      if (!content) return null;

      const createdAt = String(record.createdAt ?? nowIso());
      const updatedAt = String(record.updatedAt ?? createdAt);
      const convertedTaskId = record.convertedTaskId ? String(record.convertedTaskId) : null;
      const convertedAt = record.convertedAt ? String(record.convertedAt) : null;

      return {
        id: String(record.id ?? `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`),
        content,
        createdAt,
        updatedAt,
        convertedTaskId,
        convertedAt,
      };
    })
    .filter((item): item is InspirationItem => item !== null)
    .sort((a, b) => b.createdAt.localeCompare(a.createdAt));
}

function readStorage(): InspirationItem[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return [];
    return normalizeItems(JSON.parse(raw));
  } catch {
    return [];
  }
}

export const useInspirationStore = defineStore("inspiration", () => {
  const items = ref<InspirationItem[]>([]);
  const loaded = ref(false);
  const saving = ref(false);

  function ensureLoaded() {
    if (loaded.value) return;
    items.value = readStorage();
    loaded.value = true;
  }

  function persist() {
    if (!loaded.value) return;
    localStorage.setItem(STORAGE_KEY, JSON.stringify(items.value));
  }

  const totalCount = computed(() => items.value.length);
  const convertedCount = computed(() => items.value.filter((item) => !!item.convertedTaskId).length);
  const pendingCount = computed(() => items.value.filter((item) => !item.convertedTaskId).length);
  const latestItems = computed(() => items.value.slice(0, 6));

  function create(content: string) {
    ensureLoaded();
    const trimmed = content.trim();
    if (!trimmed) return null;

    const timestamp = nowIso();
    const item: InspirationItem = {
      id: `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
      content: trimmed,
      createdAt: timestamp,
      updatedAt: timestamp,
      convertedTaskId: null,
      convertedAt: null,
    };

    items.value.unshift(item);
    return item;
  }

  function remove(id: string) {
    ensureLoaded();
    items.value = items.value.filter((item) => item.id !== id);
  }

  async function convertToTask(id: string) {
    ensureLoaded();
    const item = items.value.find((entry) => entry.id === id);
    if (!item || item.convertedTaskId || saving.value) return null;

    saving.value = true;
    try {
      const tasks = useTaskStore();
      const created = await tasks.create({ name: item.content, quadrant: "important_not_urgent" });
      const timestamp = nowIso();
      item.convertedTaskId = created.id;
      item.convertedAt = timestamp;
      item.updatedAt = timestamp;
      return created;
    } finally {
      saving.value = false;
    }
  }

  watch(items, persist, { deep: true });

  return {
    items,
    loaded,
    saving,
    totalCount,
    convertedCount,
    pendingCount,
    latestItems,
    ensureLoaded,
    create,
    remove,
    convertToTask,
  };
});
