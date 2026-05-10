/**
 * useInspirationStore · 灵感速记 / 图谱 store。
 * - 当前优先走 Tauri SQLite 持久化
 * - 保留 localStorage 读取兼容旧数据结构
 * - 提供目标挂载 / 手动关联 / 推荐结果缓存的前端状态容器
 */

import { defineStore } from "pinia";
import { computed, ref, watch } from "vue";
import { listen } from "@tauri-apps/api/event";

import { invokeCmd } from "@/composables/useTauriInvoke";
import { useTaskStore } from "@/stores/useTaskStore";
import type { InspirationLink, InspirationRecommendation, InspirationRecord } from "@/types";

const STORAGE_KEY = "fl-inspirations";

export type InspirationItem = InspirationRecord;

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

      const createdAt = String(record.createdAt ?? record.created_at ?? nowIso());
      const updatedAt = String(record.updatedAt ?? record.updated_at ?? createdAt);
      const convertedTaskId = record.convertedTaskId
        ? String(record.convertedTaskId)
        : record.converted_task_id
          ? String(record.converted_task_id)
          : null;
      const convertedAt = record.convertedAt
        ? String(record.convertedAt)
        : record.converted_at
          ? String(record.converted_at)
          : null;
      const goalId = record.goalId
        ? String(record.goalId)
        : record.goal_id
          ? String(record.goal_id)
          : null;
      const summary = record.summary ? String(record.summary) : null;
      // 兼容三种来源:
      // 1) 后端 SQLite 返回 (D1 修复后已为 string[])
      // 2) 旧 localStorage 数据 (历史可能写过 string[])
      // 3) 极少数旧版本 string JSON (兜底解析)
      let keywords: string[] = [];
      if (Array.isArray(record.keywords)) {
        keywords = record.keywords.map((keyword) => String(keyword)).filter(Boolean);
      } else if (typeof record.keywords === "string" && record.keywords.startsWith("[")) {
        try {
          const parsed = JSON.parse(record.keywords);
          if (Array.isArray(parsed)) keywords = parsed.map(String).filter(Boolean);
        } catch {
          /* ignore */
        }
      }
      // B6: verification 5 态 + legacy "resolved"。
      // 任何非法值回落到 "none",保持前端不会崩。
      const rawVerification = String(record.verification ?? "none");
      const verification: "none" | "needs_check" | "possibly_wrong" | "verified" | "overturned" | "resolved" =
        rawVerification === "needs_check"
          || rawVerification === "possibly_wrong"
          || rawVerification === "verified"
          || rawVerification === "overturned"
          || rawVerification === "resolved"
          ? rawVerification
          : "none";
      const embeddingStatus: "pending" | "done" | "failed" =
        record.embeddingStatus === "done" || record.embeddingStatus === "failed"
          ? record.embeddingStatus
          : record.embedding_status === "done" || record.embedding_status === "failed"
            ? record.embedding_status as "done" | "failed"
            : "pending";

      return {
        id: String(record.id ?? `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`),
        content,
        goalId,
        summary,
        keywords,
        verification,
        embeddingStatus,
        createdAt,
        updatedAt,
        convertedTaskId,
        convertedAt,
      };
    })
    .filter((item): item is InspirationItem => item !== null)
    .sort((a, b) => b.createdAt.localeCompare(a.createdAt));
}

function readLegacyStorage(): InspirationItem[] {
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
  const linksById = ref<Record<string, InspirationLink[]>>({});
  const pendingRecommendations = ref<Record<string, InspirationRecommendation[]>>({});
  const recommendationLoading = ref<Record<string, boolean>>({});
  const recommendationError = ref<Record<string, string>>({});
  /// 每条灵感最近一次"分析关联"完成的时间戳。有值 = UI 显示「重新分析」,无值 = 「分析关联」。
  const recommendationAnalyzedAt = ref<Record<string, string>>({});
  const loaded = ref(false);
  const saving = ref(false);
  let recommendationListenerInstalled = false;

  /// 安装一次性事件监听:仅监听 B3 的 `inspiration://embedding_model_changed`:model 切换后
  /// 后端清空旧向量,前端刷新 items + 自动触发批量重建。
  /// (注:历史上还监听过 `inspiration://recommendations_ready` 用于"创建即跑"的静默回填,
  ///  方案 A 之后该路径已废弃 — LLM 推荐统一改为用户主动 loadRecommendations(id) 触发。)
  async function ensureRecommendationListener() {
    if (recommendationListenerInstalled) return;
    recommendationListenerInstalled = true;
    try {
      // B3: embedding model 变化通知
      await listen<{ newModel: string }>(
        "inspiration://embedding_model_changed",
        async () => {
          // 后端已把所有 embedding_status 置 pending、清空 embeddings,刷新本地视图
          for (const item of items.value) {
            item.embeddingStatus = "pending";
          }
          // 自动触发批量重建,失败也无所谓(下次进入页面再试)
          try {
            await invokeCmd<number>("batch_embed_pending");
          } catch (e) {
            console.warn("[inspiration] auto rebuild after model change failed", e);
          }
        },
      );
    } catch (e) {
      console.warn("[inspiration] embedding 事件监听安装失败", e);
    }
  }

  function persistLegacySnapshot() {
    if (!loaded.value) return;
    localStorage.setItem(STORAGE_KEY, JSON.stringify(items.value));
  }

  async function ensureLoaded() {
    if (loaded.value) return;
    await reload();
    invokeCmd("batch_embed_pending").catch(() => {});
  }

  async function reload() {
    void ensureRecommendationListener();
    try {
      const listed = await invokeCmd<InspirationItem[]>("list_inspirations");
      if (Array.isArray(listed) && listed.length > 0) {
        items.value = normalizeItems(listed);
      } else {
        const legacy = readLegacyStorage();
        if (legacy.length > 0) {
          try {
            const migrated = await invokeCmd<number>("migrate_inspirations_from_local", { items: legacy });
            if (migrated > 0) {
              localStorage.removeItem(STORAGE_KEY);
              const fresh = await invokeCmd<InspirationItem[]>("list_inspirations");
              items.value = normalizeItems(fresh);
            } else {
              items.value = legacy;
            }
          } catch {
            items.value = legacy;
          }
        }
      }
    } catch {
      items.value = readLegacyStorage();
    }
    loaded.value = true;
  }

  const totalCount = computed(() => items.value.length);
  const convertedCount = computed(() => items.value.filter((item) => !!item.convertedTaskId).length);
  const pendingCount = computed(() => items.value.filter((item) => !item.convertedTaskId).length);
  const latestItems = computed(() => items.value.slice(0, 3));

  async function create(content: string, goalId?: string | null) {
    await ensureLoaded();
    const trimmed = content.trim();
    if (!trimmed) return null;

    try {
      const created = normalizeItems([
        await invokeCmd<InspirationItem>("create_inspiration", {
          input: { content: trimmed, goalId: goalId ?? null },
        }),
      ])[0];
      items.value.unshift(created);
      // D2: 关键词/摘要 AI 提取失败时不再 silent 吞,日志显式记录;
      // UI 层根据 keywords 是否为空判断展示。失败原因通过 console 留痕。
      invokeCmd<{ keywords: string[]; summary: string | null }>("extract_inspiration_keywords", { id: created.id })
        .then((result) => {
          const target = items.value.find((i) => i.id === created.id);
          if (target && result) {
            target.keywords = result.keywords;
            target.summary = result.summary;
          }
        })
        .catch((err) => {
          console.warn("[inspiration] AI 关键词提取失败,卡片不显示关键词:", err);
        });
      return created;
    } catch {
      const timestamp = nowIso();
      const item: InspirationItem = {
        id: `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
        content: trimmed,
        goalId: goalId ?? null,
        summary: null,
        keywords: [],
        verification: "none",
        embeddingStatus: "pending",
        createdAt: timestamp,
        updatedAt: timestamp,
        convertedTaskId: null,
        convertedAt: null,
      };
      items.value.unshift(item);
      return item;
    }
  }

  async function remove(id: string) {
    await ensureLoaded();
    try {
      await invokeCmd<void>("delete_inspiration", { id });
    } finally {
      items.value = items.value.filter((item) => item.id !== id);
      delete linksById.value[id];
      delete pendingRecommendations.value[id];
      // B1: 清理其他卡片 pending 推荐里指向已删除 id 的引用,
      // 避免 UI 显示"(已删除)"幽灵推荐。
      for (const [otherId, recos] of Object.entries(pendingRecommendations.value)) {
        const filtered = recos.filter((r) => r.candidateId !== id);
        if (filtered.length !== recos.length) {
          pendingRecommendations.value[otherId] = filtered;
        }
      }
      // 同步清理任何 link 缓存里"另一端是已删除 id"的条目
      for (const [otherId, links] of Object.entries(linksById.value)) {
        const filtered = links.filter((l) => l.sourceId !== id && l.targetId !== id);
        if (filtered.length !== links.length) {
          linksById.value[otherId] = filtered;
        }
      }
    }
  }

  async function assignGoal(id: string, goalId: string | null) {
    await ensureLoaded();
    await invokeCmd<void>("update_inspiration_goal", {
      input: { id, goalId },
    });
    const target = items.value.find((item) => item.id === id);
    if (target) {
      target.goalId = goalId;
      target.updatedAt = nowIso();
    }
  }

  /// B2: goal 归档/删除时,清空所有挂载该 goal 的灵感关联。
  /// 后端做 batch UPDATE,前端同步内存 state,避免 reload 后又看到旧 goalId。
  async function syncGoalRemoval(goalId: string) {
    if (!goalId) return 0;
    try {
      const affected = await invokeCmd<number>("clear_inspirations_for_goal", { goalId });
      // 同步前端内存
      const now = nowIso();
      for (const item of items.value) {
        if (item.goalId === goalId) {
          item.goalId = null;
          item.updatedAt = now;
        }
      }
      return affected;
    } catch (e) {
      console.warn("[inspiration] syncGoalRemoval failed", e);
      return 0;
    }
  }

  async function updateVerification(
    id: string,
    verification: "none" | "needs_check" | "possibly_wrong" | "verified" | "overturned" | "resolved",
  ) {
    await ensureLoaded();
    await invokeCmd<void>("update_inspiration_verification", {
      input: { id, verification },
    });
    const target = items.value.find((item) => item.id === id);
    if (target) {
      target.verification = verification;
      target.updatedAt = nowIso();
    }
  }

  async function loadLinks(id: string) {
    linksById.value[id] = await invokeCmd<InspirationLink[]>("list_inspiration_links", {
      inspirationId: id,
    });
    return linksById.value[id];
  }

  async function linkManually(sourceId: string, targetId: string, relation: "related" | "contradicts" = "related") {
    try {
      const created = await invokeCmd<InspirationLink>("link_inspirations", {
        input: {
          sourceId,
          targetId,
          relation,
          sourceType: "manual",
        },
      });
      if (!linksById.value[sourceId]) linksById.value[sourceId] = [];
      if (!linksById.value[targetId]) linksById.value[targetId] = [];
      linksById.value[sourceId] = [created, ...linksById.value[sourceId]];
      linksById.value[targetId] = [created, ...linksById.value[targetId]];
      return created;
    } catch (error) {
      // D4: 区分"已重复关联"和其他错误,前端可显式 toast/提示。
      const msg = String(error ?? "");
      if (msg.includes("DUPLICATE_LINK")) {
        throw new Error("这两条灵感已经关联过了");
      }
      throw error;
    }
  }

  async function unlink(sourceId: string, targetId: string) {
    await invokeCmd<void>("unlink_inspirations", {
      input: { sourceId, targetId },
    });
    const filterLink = (links: InspirationLink[] | undefined) =>
      (links ?? []).filter((link) => !(link.sourceId === sourceId && link.targetId === targetId) && !(link.sourceId === targetId && link.targetId === sourceId));
    linksById.value[sourceId] = filterLink(linksById.value[sourceId]);
    linksById.value[targetId] = filterLink(linksById.value[targetId]);
  }

  async function loadRecommendations(id: string) {
    recommendationLoading.value[id] = true;
    recommendationError.value[id] = "";
    try {
      pendingRecommendations.value[id] = await invokeCmd<InspirationRecommendation[]>("suggest_related_inspirations", {
        inspirationId: id,
      });
      // 记录"已分析"时间戳,供 UI 区分「分析关联」/「重新分析」
      recommendationAnalyzedAt.value[id] = nowIso();
      return pendingRecommendations.value[id];
    } catch (error) {
      recommendationError.value[id] = String(error ?? "AI 推荐暂不可用");
      pendingRecommendations.value[id] = [];
      throw error;
    } finally {
      recommendationLoading.value[id] = false;
    }
  }

  function clearRecommendation(id: string, candidateId: string) {
    pendingRecommendations.value[id] = (pendingRecommendations.value[id] ?? []).filter(
      (item) => item.candidateId !== candidateId,
    );
  }

  async function acceptRecommendation(id: string, recommendation: InspirationRecommendation) {
    await linkManually(id, recommendation.candidateId, recommendation.relation);
    clearRecommendation(id, recommendation.candidateId);
  }

  async function convertToTask(id: string) {
    await ensureLoaded();
    const item = items.value.find((entry) => entry.id === id);
    if (!item || item.convertedTaskId || saving.value) return null;

    saving.value = true;
    try {
      const tasks = useTaskStore();
      const created = await tasks.create({ name: item.content, quadrant: "important_not_urgent" });
      await invokeCmd<void>("mark_inspiration_converted", { id, taskId: created.id });
      const timestamp = nowIso();
      item.convertedTaskId = created.id;
      item.convertedAt = timestamp;
      item.updatedAt = timestamp;
      return created;
    } finally {
      saving.value = false;
    }
  }

  /// 重试单条灵感的 embedding 索引。成功 → status=done,失败 → 仍 failed。
  async function retryEmbedding(id: string): Promise<boolean> {
    const item = items.value.find((i) => i.id === id);
    if (!item) return false;
    // 立即把状态切到 pending,UI 显示"索引中"
    item.embeddingStatus = "pending";
    try {
      await invokeCmd<void>("retry_embed_inspiration", { id });
      item.embeddingStatus = "done";
      return true;
    } catch (e) {
      console.warn("[inspiration] retry embedding failed", e);
      item.embeddingStatus = "failed";
      return false;
    }
  }

  watch(items, persistLegacySnapshot, { deep: true });

  return {
    items,
    linksById,
    pendingRecommendations,
    recommendationLoading,
    recommendationError,
    recommendationAnalyzedAt,
    loaded,
    saving,
    totalCount,
    convertedCount,
    pendingCount,
    latestItems,
    ensureLoaded,
    reload,
    create,
    remove,
    assignGoal,
    syncGoalRemoval,
    updateVerification,
    loadLinks,
    linkManually,
    unlink,
    loadRecommendations,
    acceptRecommendation,
    convertToTask,
    retryEmbedding,
  };
});
