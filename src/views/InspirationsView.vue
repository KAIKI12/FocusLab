<script setup lang="ts">
/**
 * InspirationsView · 灵感速记完整页 — 独立路由 /inspirations。
 * 顶部输入区 + 工具栏(搜索/筛选) + 3 列卡片网格(按时间分组)。
 */

import {
  ArrowRight,
  Check,
  CheckSquare,
  Copy,
  Lightbulb,
  MessageSquare,
  Plus,
  RefreshCw,
  Search,
  Sparkles,
  Square,
  Trash2,
  X,
} from "lucide-vue-next";
import { computed, onMounted, ref } from "vue";

import InspirationCorrectionPanel from "@/components/inspiration/InspirationCorrectionPanel.vue";
import InspirationGoalPicker from "@/components/inspiration/InspirationGoalPicker.vue";
import InspirationGraphFullscreenModal from "@/components/inspiration/InspirationGraphFullscreenModal.vue";
import InspirationGraphView from "@/components/inspiration/InspirationGraphView.vue";
import InspirationLinkModal from "@/components/inspiration/InspirationLinkModal.vue";
import { useGoalStore } from "@/stores/useGoalStore";
import { useInspirationStore, type InspirationItem } from "@/stores/useInspirationStore";
import { useAIStore, type QuickNoteCandidate } from "@/stores/useAIStore";
import { useChatStore } from "@/stores/useChatStore";
import { useUIStore } from "@/stores/useUIStore";
import type { InspirationRecommendation } from "@/types";

const inspiration = useInspirationStore();
const goals = useGoalStore();
const ai = useAIStore();
const chat = useChatStore();
const ui = useUIStore();

const draft = ref("");
const textareaEl = ref<HTMLTextAreaElement | null>(null);
const justSavedId = ref<string | null>(null);
const query = ref("");
const filterStatus = ref<"all" | "pending" | "converted" | "needs_check">("all");
const filterGoalId = ref<string | null>(null);
const linkingItemId = ref<string | null>(null);
const assigningItemId = ref<string | null>(null);
const linkQuery = ref("");
const linkRelation = ref<"related" | "contradicts">("related");
// AI 推荐目标归属(每条灵感独立)
const goalSuggestionLoading = ref<Record<string, boolean>>({});
const goalSuggestion = ref<Record<string, { goalId: string | null; reason: string }>>({});

// AI 纠偏分析(key: `${itemId}::${candidateId}`)
type CorrectionResult = { summary: string; oldJudgment: string; newEvidence: string; suggestion: string };
const correctionLoading = ref<Record<string, boolean>>({});
const correctionResult = ref<Record<string, CorrectionResult | null>>({});
const copiedId = ref<string | null>(null);
const copiedDraftIdx = ref<number | null>(null);
const focusedItemId = ref<string | null>(null);
const focusedItem = computed(() => inspiration.items.find((item) => item.id === focusedItemId.value) ?? null);
const focusedLinks = computed(() => (focusedItemId.value ? inspiration.linksById[focusedItemId.value] ?? [] : []));
const focusedRecommendations = computed<InspirationRecommendation[]>(() =>
  focusedItemId.value ? inspiration.pendingRecommendations[focusedItemId.value] ?? [] : [],
);
// AI 分析触发态(loading / 已分析过 / 错误)
const focusedAnalyzing = computed(() =>
  focusedItemId.value ? !!inspiration.recommendationLoading[focusedItemId.value] : false,
);
const focusedAnalyzed = computed(() =>
  focusedItemId.value ? !!inspiration.recommendationAnalyzedAt[focusedItemId.value] : false,
);
const focusedAnalyzeError = computed(() =>
  focusedItemId.value ? inspiration.recommendationError[focusedItemId.value] ?? "" : "",
);
// 全屏图谱 Modal 开关
const fullscreenGraphOpen = ref(false);

// 灵感多选
const selectMode = ref(false);
const selectedIds = ref<Set<string>>(new Set());

function toggleSelect(id: string) {
  if (selectedIds.value.has(id)) selectedIds.value.delete(id);
  else selectedIds.value.add(id);
}

function toggleSelectAll() {
  if (selectedIds.value.size === filteredItems.value.length) {
    selectedIds.value.clear();
  } else {
    selectedIds.value = new Set(filteredItems.value.map((i) => i.id));
  }
}

function exitSelectMode() {
  selectMode.value = false;
  selectedIds.value.clear();
}

async function onBatchAiExplore() {
  const items = inspiration.items.filter((i) => selectedIds.value.has(i.id));
  if (!items.length) return;
  await chat.createFromInspirations(items.map((i) => ({ id: i.id, content: i.content })));
  ui.showChat = true;
  exitSelectMode();
}

// AI 三候选 inline（使用 store session，路由切换后恢复）
const aiDrafts = ref<QuickNoteCandidate[]>([]);
const aiDraftsLoading = ref(false);
const aiDraftsError = ref("");
const showAiDrafts = ref(false);

// 从 store session 恢复上次的梳理状态（含原文）
function restoreAiSession() {
  const s = ai.optimizeSession;
  if (!s || !s.show) return;
  draft.value = s.draftText;
  aiDraftsLoading.value = s.loading;
  aiDraftsError.value = s.error ? `AI 梳理失败: ${s.error}` : "";
  aiDrafts.value = s.candidates;
  showAiDrafts.value = true;
}

onMounted(() => {
  inspiration.ensureLoaded();
  void goals.loadGoals();
  restoreAiSession();
});

function onCopy(id: string, text: string) {
  navigator.clipboard.writeText(text);
  copiedId.value = id;
  setTimeout(() => { if (copiedId.value === id) copiedId.value = null; }, 1500);
}

function onCopyDraft(idx: number, text: string) {
  navigator.clipboard.writeText(text);
  copiedDraftIdx.value = idx;
  setTimeout(() => { if (copiedDraftIdx.value === idx) copiedDraftIdx.value = null; }, 1500);
}

async function onSave() {
  const content = draft.value.trim();
  if (!content) return;
  const item = await inspiration.create(content);
  draft.value = "";
  if (item) {
    justSavedId.value = item.id;
    setTimeout(() => {
      justSavedId.value = null;
    }, 2000);
    // 注:创建灵感不再自动跑 AI 推荐(方案 A)
    // 用户聚焦后点击「✨ 分析关联」按钮显式触发,避免静默消耗 API 配额。
  }
  textareaEl.value?.focus();
}

function onKeydown(e: KeyboardEvent) {
  if ((e.metaKey || e.ctrlKey) && e.key === "Enter") {
    e.preventDefault();
    onSave();
  }
}

async function onConvert(id: string) {
  await inspiration.convertToTask(id);
}

async function onAiAssist() {
  const text = draft.value.trim();
  if (!text) return;
  aiDraftsLoading.value = true;
  aiDraftsError.value = "";
  showAiDrafts.value = true;
  try {
    aiDrafts.value = await ai.optimizeQuickNote(text);
  } catch (e) {
    aiDraftsError.value = `AI 梳理失败: ${e}`;
    aiDrafts.value = [];
  } finally {
    aiDraftsLoading.value = false;
  }
}

async function onSaveDraft(candidate: QuickNoteCandidate) {
  const item = await inspiration.create(candidate.text);
  if (item) {
    justSavedId.value = item.id;
    setTimeout(() => { justSavedId.value = null; }, 2000);
    // 同 onSave: 不再自动触发 AI 推荐,改为用户主动点击「分析关联」
  }
  showAiDrafts.value = false;
  draft.value = "";
  ai.optimizeSession = null;
}

async function onChatFromDraft(candidate: QuickNoteCandidate) {
  // 先保存灵感记录
  const item = await inspiration.create(candidate.text);
  if (item) {
    justSavedId.value = item.id;
    setTimeout(() => { justSavedId.value = null; }, 2000);
    void inspiration.loadRecommendations(item.id);
  }
  // 创建 AI 对话并跳转
  await chat.createFromInspiration(item?.id ?? "", draft.value.trim(), candidate.text);
  ui.showChat = true;
  showAiDrafts.value = false;
  draft.value = "";
  ai.optimizeSession = null;
}

// ---------- 时间分组 ----------
type Bucket = "today" | "yesterday" | "thisWeek" | "earlier";

function startOfDay(d: Date): number {
  return new Date(d.getFullYear(), d.getMonth(), d.getDate()).getTime();
}

function bucketOf(iso: string): Bucket {
  const created = new Date(iso);
  const today0 = startOfDay(new Date());
  const yesterday0 = today0 - 86_400_000;
  const weekStart = today0 - 6 * 86_400_000;
  const t = created.getTime();
  if (t >= today0) return "today";
  if (t >= yesterday0) return "yesterday";
  if (t >= weekStart) return "thisWeek";
  return "earlier";
}

const BUCKET_LABEL: Record<Bucket, string> = {
  today: "今天",
  yesterday: "昨天",
  thisWeek: "本周",
  earlier: "更早",
};
const BUCKET_ORDER: Bucket[] = ["today", "yesterday", "thisWeek", "earlier"];

// ---------- 过滤后的列表 ----------
const filteredItems = computed<InspirationItem[]>(() => {
  const q = query.value.trim().toLowerCase();
  return inspiration.items.filter((item) => {
    if (filterStatus.value === "pending" && item.convertedTaskId) return false;
    if (filterStatus.value === "converted" && !item.convertedTaskId) return false;
    if (filterStatus.value === "needs_check" && item.verification !== "needs_check") return false;
    if (filterGoalId.value && item.goalId !== filterGoalId.value) return false;
    if (q && !item.content.toLowerCase().includes(q)) return false;
    return true;
  });
});

const groupedItems = computed(() => {
  const groups: Record<Bucket, InspirationItem[]> = {
    today: [],
    yesterday: [],
    thisWeek: [],
    earlier: [],
  };
  for (const item of filteredItems.value) {
    groups[bucketOf(item.createdAt)].push(item);
  }
  return groups;
});


const goalNameById = computed<Record<string, string>>(() =>
  Object.fromEntries(goals.goals.map((g) => [g.id, g.name])),
);

function linkCountFor(id: string) {
  return (inspiration.linksById[id] ?? []).length;
}

function onSelectIdea(id: string) {
  focusedItemId.value = id;
  inspiration.loadLinks(id);
}

const linkCandidates = computed(() => {
  const base = inspiration.items.filter((item) => item.id !== linkingItemId.value);
  const q = linkQuery.value.trim().toLowerCase();
  if (!q) return base.slice(0, 6);
  return base.filter((item) => item.content.toLowerCase().includes(q)).slice(0, 6);
});


// ---------- 时间格式化 + 辅助 computed ----------

const hasAnyResult = computed(() => filteredItems.value.length > 0);

function fmtTime(iso: string) {
  try {
    const d = new Date(iso);
    const now = new Date();
    const diffMs = now.getTime() - d.getTime();
    const diffMin = Math.floor(diffMs / 60_000);
    if (diffMin < 1) return "刚刚";
    if (diffMin < 60) return `${diffMin} 分钟前`;
    const diffH = Math.floor(diffMin / 60);
    if (diffH < 24) return `${diffH} 小时前`;
    return d.toLocaleDateString("zh-CN", { month: "numeric", day: "numeric" });
  } catch {
    return "";
  }
}

async function onAssignGoal(itemId: string, goalId: string | null) {
  await inspiration.assignGoal(itemId, goalId);
  assigningItemId.value = null;
}

// D2: 统一 AI/契约错误显式提示通道。
// 4 秒自动消失,避免堆叠;同 message 多次触发会复位倒计时。
const aiToast = ref<{ kind: "error" | "info"; text: string } | null>(null);
let aiToastTimer: ReturnType<typeof setTimeout> | null = null;
function showAiToast(text: string, kind: "error" | "info" = "error") {
  if (aiToastTimer) clearTimeout(aiToastTimer);
  aiToast.value = { kind, text };
  aiToastTimer = setTimeout(() => {
    aiToast.value = null;
  }, 4000);
}

async function onSuggestGoal(itemId: string) {
  const item = inspiration.items.find((i) => i.id === itemId);
  if (!item || !goals.goals.length) return;
  goalSuggestionLoading.value[itemId] = true;
  try {
    const res = await ai.suggestGoalForInspiration(
      item.content,
      goals.goals.map((g) => ({ id: g.id, name: g.name })),
    );
    goalSuggestion.value[itemId] = res;
  } catch (e) {
    showAiToast(`AI 推荐归属失败: ${e instanceof Error ? e.message : String(e)}`);
  } finally {
    goalSuggestionLoading.value[itemId] = false;
  }
}

async function onAcceptSuggestedGoal(itemId: string) {
  const sug = goalSuggestion.value[itemId];
  if (!sug || !sug.goalId) return;
  await inspiration.assignGoal(itemId, sug.goalId);
  assigningItemId.value = null;
  delete goalSuggestion.value[itemId];
}

async function onAnalyzeCorrection(itemId: string, candidateId: string, candidateContent: string) {
  const item = inspiration.items.find((i) => i.id === itemId);
  if (!item) return;
  const key = `${itemId}::${candidateId}`;
  correctionLoading.value[key] = true;
  try {
    // 旧灵感 = 被新灵感(item)质疑/纠正的那条;新灵感 = 当前 item
    const res = await ai.analyzeCorrection(candidateContent, item.content);
    correctionResult.value[key] = res;
  } catch (e) {
    showAiToast(`AI 纠偏分析失败: ${e instanceof Error ? e.message : String(e)}`);
  } finally {
    correctionLoading.value[key] = false;
  }
}

function correctionKey(itemId: string, candidateId: string): string {
  return `${itemId}::${candidateId}`;
}

async function onLink(sourceId: string, targetId: string, relation: "related" | "contradicts" = "related") {
  try {
    await inspiration.linkManually(sourceId, targetId, relation);
    linkingItemId.value = null;
    linkQuery.value = "";
    linkRelation.value = "related";
    await inspiration.loadLinks(sourceId);
  } catch (e) {
    // D4: 重复关联等错误显式提示,而不是"点了没反应"。
    showAiToast(e instanceof Error ? e.message : String(e), "info");
  }
}

async function onUnlink(sourceId: string, targetId: string) {
  await inspiration.unlink(sourceId, targetId);
  await inspiration.loadLinks(sourceId);
}

function onToggleLinkPanel(itemId: string) {
  if (linkingItemId.value === itemId) {
    linkingItemId.value = null;
    return;
  }
  linkingItemId.value = itemId;
  // 打开面板时拉取最新关联,确保"已关联"区块准确
  void inspiration.loadLinks(itemId);
}

/// 根据 id 找灵感原文(用于在已关联列表展示)
function contentForId(id: string): string {
  return inspiration.items.find((i) => i.id === id)?.content ?? "(已删除)";
}

function onIgnoreRecommendation(itemId: string, candidateId: string) {
  const current = inspiration.pendingRecommendations[itemId] ?? [];
  inspiration.pendingRecommendations[itemId] = current.filter((item) => item.candidateId !== candidateId);
}

/**
 * 来自图谱(小图或全屏)上 AI 建议线的"接受"按钮。
 * 与右侧栏卡片上的"接受"等价,差异是失败时显式 toast。
 */
async function onAcceptRecoFromGraph(sourceId: string, reco: InspirationRecommendation) {
  try {
    await inspiration.acceptRecommendation(sourceId, reco);
  } catch (e) {
    showAiToast(`接受 AI 推荐失败: ${e instanceof Error ? e.message : String(e)}`, "error");
  }
}

/**
 * 打开全屏图谱:同时异步预加载所有 items 的 link 数据,
 * Modal 内部即时反应到 reactive linksById,先打开再慢慢补完。
 */
function onExpandGraph() {
  fullscreenGraphOpen.value = true;
  for (const item of inspiration.items) {
    if (!inspiration.linksById[item.id]) {
      inspiration.loadLinks(item.id).catch(() => {
        /* 单条失败静默 — 整体 UX 不阻塞 */
      });
    }
  }
}

/**
 * 头部「✨ 分析关联 / 🔄 重新分析」触发 — 同时服务于 mini 图与全屏 Modal。
 * 走 store.loadRecommendations(id) → 后端 suggest_related_inspirations 命令。
 * 失败显式 toast,不静默。
 */
async function onAnalyzeFocused() {
  if (!focusedItemId.value) return;
  if (focusedAnalyzing.value) return;
  try {
    await inspiration.loadRecommendations(focusedItemId.value);
  } catch (e) {
    showAiToast(`AI 分析失败: ${e instanceof Error ? e.message : String(e)}`, "error");
  }
}

/**
 * 全屏图谱里用户拖拽 handle 手动连线 → 落库到 SQLite link 表。
 * 默认 relation=related;同节点自连/重复连线由 store.linkManually 内部处理。
 * linksById 双向写入,卡片连接数自动同步。
 */
async function onCreateLink(
  sourceId: string,
  targetId: string,
  relation: "related" | "contradicts" = "related",
) {
  if (!sourceId || !targetId || sourceId === targetId) return;
  try {
    await inspiration.linkManually(sourceId, targetId, relation);
  } catch (e) {
    showAiToast(`连线失败: ${e instanceof Error ? e.message : String(e)}`, "error");
  }
}

/**
 * 判定一条灵感"已分析过且确实没有任何关联"
 * 用于在卡片右下角显示「当前无关联」灰字。
 * 必须满足全部:已分析(有时间戳) + 不在 loading + 无 error + 推荐空 + links 已加载且为空。
 */
function isAnalyzedNoLink(id: string): boolean {
  if (!inspiration.recommendationAnalyzedAt[id]) return false;
  if (inspiration.recommendationLoading[id]) return false;
  if (inspiration.recommendationError[id]) return false;
  if ((inspiration.pendingRecommendations[id] ?? []).length > 0) return false;
  const links = inspiration.linksById[id];
  if (links === undefined) return false; // links 尚未加载,先不下结论
  if (links.length > 0) return false;
  return true;
}

async function onCreateFollowup(parentId: string) {
  const parent = inspiration.items.find((i) => i.id === parentId);
  if (!parent) return;
  // D2: AI 起草失败时,显式提示并降级到本地模板,而不是 store 内部 silent fallback。
  // 用户能感知"AI 没成功",并仍能继续工作。
  let draftText: string;
  try {
    draftText = await ai.draftFollowupExperiment(parent.content);
  } catch (e) {
    showAiToast(`AI 起草失败,已用模板代替: ${e instanceof Error ? e.message : String(e)}`, "info");
    draftText = `[后续实验] 验证: ${parent.content.slice(0, 60)}`;
  }
  const item = await inspiration.create(draftText);
  if (item) {
    await inspiration.linkManually(item.id, parentId, "related");
    justSavedId.value = item.id;
    setTimeout(() => { justSavedId.value = null; }, 2000);
  }
}

// ---------- 右侧栏统计 ----------
const needsCheckCount = computed(() => inspiration.items.filter((i) => i.verification === "needs_check").length);
const linkedCount = computed(() => Object.values(inspiration.linksById).filter((arr) => arr.length > 0).length);
const embeddingDoneCount = computed(() => inspiration.items.filter((i) => i.embeddingStatus === "done").length);
const embeddingPendingTotal = computed(() => inspiration.items.length);

const goalDistribution = computed(() => {
  const map: Record<string, { id: string; name: string; count: number }> = {};
  let ungrouped = 0;
  for (const item of inspiration.items) {
    if (item.goalId) {
      const goal = goals.goals.find((g) => g.id === item.goalId);
      const name = goal?.name ?? "未知目标";
      if (!map[item.goalId]) map[item.goalId] = { id: item.goalId, name, count: 0 };
      map[item.goalId].count++;
    } else {
      ungrouped++;
    }
  }
  const entries = Object.values(map).sort((a, b) => b.count - a.count);
  if (ungrouped > 0) entries.push({ id: "", name: "未挂目标", count: ungrouped });
  return entries;
});

const allPendingRecommendations = computed(() => {
  const result: { itemId: string; reco: InspirationRecommendation }[] = [];
  for (const [itemId, recos] of Object.entries(inspiration.pendingRecommendations)) {
    for (const reco of recos) {
      result.push({ itemId, reco });
    }
  }
  return result.slice(0, 5);
});
</script>

<template>
  <section class="fl-inspirations">
    <!-- 标题 -->
    <header class="fl-ip-head">
      <div class="fl-ip-head-left">
        <h1>
          <span class="fl-ip-mark"><Sparkles :size="20" /></span>
          科研灵感工作台
        </h1>
        <p class="fl-ip-sub">
          把零散想法沉淀成可回看的研究路径。AI 只负责提示线索，连接和判断始终由你确认。
        </p>
      </div>
      <div class="fl-ip-head-right">
        <div
          class="fl-ip-ai-pill"
          title="AI 推荐基于你自己记录的灵感卡片。&#10;开启 embedding 后,卡片内容会发送给你配置的 AI 提供商生成向量;&#10;关闭 embedding 即停止此类调用。&#10;数据始终存于本地 SQLite,不参与第三方训练。"
        >
          <span class="fl-ip-pulse"></span>
          AI 推荐运行中
          <span class="fl-ip-pill-info">ⓘ</span>
        </div>
      </div>
    </header>

    <!-- 输入区 -->
    <div class="fl-ip-input-wrap">
      <div class="fl-ip-compose-head">
        <span class="fl-ip-compose-title">捕获一个研究线索</span>
        <span class="fl-ip-compose-shortcut">⌘ + Enter 保存</span>
      </div>
      <textarea
        ref="textareaEl"
        v-model="draft"
        class="fl-ip-textarea"
        placeholder="例如: 今天读到多次随机划分 + 集成评估,也许能解释我实验结果波动很大的问题……"
        rows="3"
        maxlength="500"
        spellcheck="false"
        @keydown="onKeydown"
      />
      <div class="fl-ip-input-foot">
        <div class="fl-ip-hints">
          <span class="fl-ip-hint-tag">低负担记录</span>
          <span class="fl-ip-hint-tag">语义索引</span>
          <span class="fl-ip-hint-tag">只提示高置信关系</span>
          <span
            class="fl-ip-hint-tag fl-ip-hint-privacy"
            title="点击 AI 梳理 / AI 推荐归属 / AI 纠偏分析时,卡片内容会发送给你配置的 AI 提供商。&#10;不调用 AI 时,内容只存于本地 SQLite。&#10;可在设置面板关闭 AI 推荐。"
          >🔒 AI 数据流向</span>
        </div>
        <div class="fl-ip-input-actions">
          <button
            class="fl-ip-btn fl-ip-btn-secondary"
            type="button"
            :disabled="!draft.trim()"
            @click="onAiAssist"
          >
            <Sparkles :size="14" />
            AI 梳理
          </button>
          <button
            class="fl-ip-btn fl-ip-btn-primary"
            type="button"
            :disabled="!draft.trim()"
            @click="onSave"
          >
            <Plus :size="14" />
            直接保存原文
          </button>
        </div>
      </div>
    </div>

    <!-- AI 三候选 inline -->
    <div v-if="showAiDrafts" class="fl-ip-drafts">
      <div v-if="aiDraftsLoading" class="fl-ip-drafts-loading">
        <Sparkles :size="15" /> 正在生成 3 个梳理版本…
      </div>
      <div v-else-if="aiDraftsError" class="fl-ip-drafts-error">{{ aiDraftsError }}</div>
      <template v-else>
        <article v-for="(c, idx) in aiDrafts" :key="idx" class="fl-ip-draft-card" :class="{ 'is-primary': idx === 0 }">
          <div class="fl-ip-draft-top">
            <span class="fl-ip-draft-kind">{{ c.styleName || c.style }}</span>
            <span class="fl-ip-draft-tone">{{ idx === 0 ? '保留原意' : idx === 1 ? '提炼问题' : '延展方向' }}</span>
          </div>
          <p class="fl-ip-draft-text">{{ c.text }}</p>
          <div class="fl-ip-draft-foot">
            <button class="fl-ip-draft-btn fl-ip-draft-btn-copy" type="button" title="复制" @click.stop="onCopyDraft(idx, c.text)">
              <template v-if="copiedDraftIdx === idx"><Check :size="12" /> 已复制</template>
              <template v-else><Copy :size="12" /> 复制</template>
            </button>
            <button class="fl-ip-draft-btn fl-ip-draft-btn-primary" type="button" @click="onSaveDraft(c)">保存这个版本</button>
            <button v-if="c.style === 'question'" class="fl-ip-draft-btn fl-ip-draft-btn-chat" type="button" @click="onChatFromDraft(c)"><MessageSquare :size="12" /> 与 AI 讨论</button>
          </div>
        </article>
      </template>
    </div>

    <!-- 双栏工作区 -->
    <div class="fl-ip-workspace">
      <!-- 左主区 -->
      <main class="fl-ip-main">
        <!-- 工具栏:搜索 + 筛选 -->
        <div class="fl-ip-toolbar">
      <div class="fl-ip-search">
        <Search :size="14" />
        <input
          v-model="query"
          type="search"
          placeholder="搜索灵感…"
          spellcheck="false"
        />
      </div>
      <div class="fl-ip-filters" role="tablist">
        <button class="fl-ip-pill" :class="{ 'is-active': filterStatus === 'all' }" type="button" @click="filterStatus = 'all'">全部</button>
        <button class="fl-ip-pill" :class="{ 'is-active': filterStatus === 'pending' }" type="button" @click="filterStatus = 'pending'">未转任务</button>
        <button class="fl-ip-pill" :class="{ 'is-active': filterStatus === 'converted' }" type="button" @click="filterStatus = 'converted'">已转任务</button>
        <button class="fl-ip-pill" :class="{ 'is-active': filterStatus === 'needs_check' }" type="button" @click="filterStatus = 'needs_check'">待复查</button>
        <button
          v-for="g in goalDistribution.filter(x => x.id)"
          :key="g.id"
          class="fl-ip-pill"
          :class="{ 'is-active': filterGoalId === g.id }"
          type="button"
          @click="filterGoalId = filterGoalId === g.id ? null : g.id"
        >{{ g.name }}</button>
        <button class="fl-ip-pill fl-ip-pill-select" :class="{ 'is-active': selectMode }" type="button" @click="selectMode ? exitSelectMode() : (selectMode = true)"><CheckSquare :size="12" /> 多选</button>
      </div>
    </div>

    <!-- 卡片墙 (按时间分组) -->
    <div v-if="hasAnyResult" class="fl-ip-groups">
      <div
        v-for="bucket in BUCKET_ORDER"
        v-show="groupedItems[bucket].length"
        :key="bucket"
        class="fl-ip-group"
      >
        <div class="fl-ip-group-label">{{ BUCKET_LABEL[bucket] }}</div>
        <div class="fl-ip-timeline">
          <article
            v-for="item in groupedItems[bucket]"
            :key="item.id"
            class="fl-ip-idea"
            :class="{
              'is-converted': !!item.convertedTaskId,
              'is-new': justSavedId === item.id,
              'is-warn': item.verification === 'needs_check',
              'is-resolved': item.verification === 'resolved',
              'is-focused': focusedItemId === item.id,
              'is-selected': selectMode && selectedIds.has(item.id),
            }"
            @click="selectMode ? toggleSelect(item.id) : onSelectIdea(item.id)"
          >
            <div class="fl-ip-idea-head">
              <button v-if="selectMode" class="fl-ip-check" type="button" @click.stop="toggleSelect(item.id)">
                <CheckSquare v-if="selectedIds.has(item.id)" :size="16" class="fl-ip-check-on" />
                <Square v-else :size="16" />
              </button>
              <p class="fl-ip-idea-text">{{ item.content }}</p>
              <div class="fl-ip-idea-tools">
                <button class="fl-ip-tool" type="button" title="复制" @click.stop="onCopy(item.id, item.content)"><Check v-if="copiedId === item.id" :size="14" /><Copy v-else :size="14" /></button>
                <button class="fl-ip-tool" :class="{ 'is-active': assigningItemId === item.id }" type="button" title="挂目标" @click.stop="assigningItemId = assigningItemId === item.id ? null : item.id"><Search :size="14" /></button>
                <button class="fl-ip-tool" :class="{ 'is-active': linkingItemId === item.id }" type="button" title="手动关联" @click.stop="onToggleLinkPanel(item.id)"><ArrowRight :size="14" /></button>
                <button class="fl-ip-tool" type="button" title="看图谱" @click.stop="onSelectIdea(item.id)"><Sparkles :size="14" /></button>
              </div>
            </div>
            <div class="fl-ip-idea-meta">
              <span class="fl-ip-time">{{ fmtTime(item.createdAt) }}</span>
              <span v-if="item.goalId && goalNameById[item.goalId]" class="fl-ip-tag fl-ip-tag-goal">{{ goalNameById[item.goalId] }}</span>
              <span v-if="linkCountFor(item.id)" class="fl-ip-tag fl-ip-tag-link">{{ linkCountFor(item.id) }} 个连接</span>
              <span v-if="item.verification === 'needs_check'" class="fl-ip-tag fl-ip-tag-warning">待复查</span>
              <span v-else-if="item.verification === 'possibly_wrong'" class="fl-ip-tag fl-ip-tag-warning">⚠ 可能错误</span>
              <span v-else-if="item.verification === 'verified'" class="fl-ip-tag fl-ip-tag-resolved"><Check :size="10" /> 已验证</span>
              <span v-else-if="item.verification === 'overturned'" class="fl-ip-tag fl-ip-tag-overturned">✖ 已被推翻</span>
              <span v-else-if="item.verification === 'resolved'" class="fl-ip-tag fl-ip-tag-resolved"><Check :size="10" /> 纠偏已处理</span>
              <span v-if="item.convertedTaskId" class="fl-ip-tag fl-ip-tag-done"><Check :size="10" /> 已转任务</span>
              <span v-else-if="justSavedId === item.id" class="fl-ip-tag fl-ip-tag-new">最新</span>
              <button
                v-if="item.embeddingStatus === 'failed'"
                class="fl-ip-tag fl-ip-tag-retry"
                type="button"
                title="语义索引失败,点击重试 — AI 推荐依赖此索引"
                @click.stop="inspiration.retryEmbedding(item.id)"
              >
                <RefreshCw :size="10" /> 索引失败 · 重试
              </button>
              <span v-for="kw in (item.keywords || []).slice(0, 2)" :key="kw" class="fl-ip-tag fl-ip-tag-keyword">{{ kw }}</span>
            </div>
            <!-- 操作面板 -->
            <InspirationGoalPicker
              v-if="assigningItemId === item.id"
              :item="item"
              :goals="goals.goals"
              :goal-name-by-id="goalNameById"
              :suggestion="goalSuggestion[item.id] ?? null"
              :loading="goalSuggestionLoading[item.id]"
              @assign="onAssignGoal(item.id, $event)"
              @suggest="onSuggestGoal(item.id)"
              @accept-suggestion="onAcceptSuggestedGoal(item.id)"
            />
            <InspirationLinkModal
              v-if="linkingItemId === item.id"
              :current-item-id="item.id"
              :links="inspiration.linksById[item.id] ?? []"
              :candidates="linkCandidates"
              :query="linkQuery"
              :relation="linkRelation"
              :peer-content="contentForId"
              @update:query="linkQuery = $event"
              @update:relation="linkRelation = $event"
              @link="(targetId, relation) => onLink(item.id, targetId, relation)"
              @unlink="(peerId) => onUnlink(item.id, peerId)"
            />
            <!-- 纠偏/操作栏 -->
            <div class="fl-ip-idea-actions">
              <button v-if="!item.convertedTaskId" class="fl-ip-act-btn" type="button" :disabled="inspiration.saving" @click.stop="onConvert(item.id)">转为任务</button>
              <button v-if="item.verification !== 'needs_check'" class="fl-ip-act-btn fl-ip-act-warn" type="button" @click.stop="inspiration.updateVerification(item.id, 'needs_check')">标记待复查</button>
              <button v-if="item.verification === 'needs_check' || item.verification === 'possibly_wrong'" class="fl-ip-act-btn fl-ip-act-success" type="button" @click.stop="inspiration.updateVerification(item.id, 'verified')">标记已验证</button>
              <button v-if="item.verification === 'needs_check' || item.verification === 'possibly_wrong'" class="fl-ip-act-btn fl-ip-act-del" type="button" @click.stop="inspiration.updateVerification(item.id, 'overturned')">标记已推翻</button>
              <button v-if="item.verification !== 'none'" class="fl-ip-act-btn" type="button" @click.stop="inspiration.updateVerification(item.id, 'none')">清除状态</button>
              <button class="fl-ip-act-btn" type="button" @click.stop="onCreateFollowup(item.id)">创建后续实验</button>
              <button class="fl-ip-act-btn fl-ip-act-del" type="button" @click.stop="inspiration.remove(item.id)"><Trash2 :size="12" /></button>
            </div>
            <div v-if="inspiration.recommendationLoading[item.id]" class="fl-ip-reco fl-ip-reco-state">
              <div class="fl-ip-reco-head">
                <Sparkles :size="13" />
                正在生成 AI 推荐…
              </div>
            </div>
            <div v-else-if="inspiration.recommendationError[item.id]" class="fl-ip-reco fl-ip-reco-state is-error">
              <div class="fl-ip-reco-head">
                <Sparkles :size="13" />
                AI 推荐暂不可用
              </div>
              <p class="fl-ip-reco-reason">{{ inspiration.recommendationError[item.id] }}</p>
            </div>
            <div v-else-if="inspiration.pendingRecommendations[item.id]?.length" class="fl-ip-reco">
              <div class="fl-ip-reco-head">
                <Sparkles :size="13" />
                AI 发现 {{ inspiration.pendingRecommendations[item.id].length }} 条相关旧灵感
              </div>
              <div
                v-for="reco in inspiration.pendingRecommendations[item.id]"
                :key="reco.candidateId"
                class="fl-ip-reco-item"
                :class="{ 'is-warn': reco.relation === 'contradicts' }"
              >
                <p class="fl-ip-reco-old">{{ reco.candidateContent }}</p>
                <p class="fl-ip-reco-reason">{{ reco.reason }}</p>
                <div class="fl-ip-reco-actions">
                  <button class="fl-ip-reco-btn" type="button" @click.stop="inspiration.acceptRecommendation(item.id, reco)">
                    {{ reco.relation === 'contradicts' ? '建立修正连接' : '接受为相关' }}
                  </button>
                  <button v-if="reco.relation === 'contradicts'" class="fl-ip-reco-btn fl-ip-reco-btn-warn" type="button" @click.stop="inspiration.updateVerification(reco.candidateId, 'needs_check')">
                    标记待复查
                  </button>
                  <button
                    v-if="reco.relation === 'contradicts' && !correctionResult[correctionKey(item.id, reco.candidateId)]"
                    class="fl-ip-reco-btn fl-ip-reco-btn-ai"
                    type="button"
                    :disabled="correctionLoading[correctionKey(item.id, reco.candidateId)]"
                    @click.stop="onAnalyzeCorrection(item.id, reco.candidateId, reco.candidateContent)"
                  >
                    <Sparkles :size="11" />
                    {{ correctionLoading[correctionKey(item.id, reco.candidateId)] ? '分析中…' : 'AI 纠偏分析' }}
                  </button>
                  <button class="fl-ip-reco-btn fl-ip-reco-btn-ghost" type="button" @click.stop="onIgnoreRecommendation(item.id, reco.candidateId)">
                    忽略
                  </button>
                </div>
                <!-- AI 纠偏分析结果 -->
                <InspirationCorrectionPanel
                  v-if="correctionResult[correctionKey(item.id, reco.candidateId)]"
                  :result="correctionResult[correctionKey(item.id, reco.candidateId)]!"
                  @mark-needs-check="inspiration.updateVerification(reco.candidateId, 'needs_check')"
                  @mark-overturned="inspiration.updateVerification(reco.candidateId, 'overturned')"
                  @accept-correction="inspiration.acceptRecommendation(item.id, reco)"
                  @create-followup="onCreateFollowup(item.id)"
                />
              </div>
            </div>
            <!-- 已分析 + 无任何关联 → 右下角灰字提示;避免误以为"AI 还没跑" -->
            <div v-if="isAnalyzedNoLink(item.id)" class="fl-ip-noreco" :title="`分析时间: ${inspiration.recommendationAnalyzedAt[item.id]}`">
              当前无关联
            </div>
          </article>
        </div>
      </div>
    </div>

    <div v-else class="fl-ip-empty">
      <Lightbulb :size="36" class="fl-ip-empty-icon" />
      <h3>{{ inspiration.totalCount === 0 ? "还没有灵感" : "没有匹配的灵感" }}</h3>
      <p>
        {{
          inspiration.totalCount === 0
            ? "随手记下你的第一个想法"
            : "试试调整筛选或搜索关键词"
        }}
      </p>
    </div>

    <!-- 批量操作栏（独立于 v-if/v-else 链） -->
    <Transition name="fl-bar">
      <div v-if="selectMode && selectedIds.size" class="fl-ip-batch-bar">
        <span class="fl-ip-batch-count">已选 {{ selectedIds.size }} 条</span>
        <button class="fl-ip-batch-btn" type="button" @click="toggleSelectAll">
          {{ selectedIds.size === filteredItems.length ? '取消全选' : '全选' }}
        </button>
        <button class="fl-ip-batch-btn fl-ip-batch-btn-ai" type="button" @click="onBatchAiExplore">
          <Sparkles :size="14" /> AI 探索关联
        </button>
        <button class="fl-ip-batch-btn fl-ip-batch-btn-quit" type="button" @click="exitSelectMode">
          <X :size="14" /> 退出多选
        </button>
      </div>
    </Transition>
      </main>

      <!-- 右侧辅助栏 -->
      <aside class="fl-ip-sidebar">
        <!-- 灵感概览 -->
        <section class="fl-ip-side-card">
          <h3 class="fl-ip-side-title">灵感概览</h3>
          <div class="fl-ip-metrics">
            <div class="fl-ip-metric"><strong>{{ inspiration.totalCount }}</strong><span>总灵感</span></div>
            <div class="fl-ip-metric"><strong>{{ linkedCount }}</strong><span>已连接</span></div>
            <div class="fl-ip-metric"><strong>{{ needsCheckCount }}</strong><span>待复查</span></div>
          </div>
        </section>

        <!-- 局部图谱 -->
        <InspirationGraphView
          :item="focusedItem"
          :links="focusedLinks"
          :all-items="inspiration.items"
          :recommendations="focusedRecommendations"
          :analyzing="focusedAnalyzing"
          :analyzed="focusedAnalyzed"
          :analyze-error="focusedAnalyzeError"
          @analyze="onAnalyzeFocused"
          @expand="onExpandGraph"
          @accept-reco="onAcceptRecoFromGraph"
          @reject-reco="onIgnoreRecommendation"
          @node-click="onSelectIdea"
        />

        <!-- 待处理推荐 -->
        <section v-if="allPendingRecommendations.length" class="fl-ip-side-card">
          <h3 class="fl-ip-side-title">待处理推荐</h3>
          <div class="fl-ip-review-list">
            <div v-for="(entry, idx) in allPendingRecommendations" :key="idx" class="fl-ip-review-item">
              <span class="fl-ip-review-dot" :class="entry.reco.relation === 'contradicts' ? 'is-warn' : ''"></span>
              <p>{{ entry.reco.reason }}</p>
            </div>
          </div>
        </section>

        <!-- 按目标分布 -->
        <section v-if="goalDistribution.length" class="fl-ip-side-card">
          <h3 class="fl-ip-side-title">按目标分布</h3>
          <div class="fl-ip-goal-list">
            <div v-for="(g, idx) in goalDistribution" :key="idx" class="fl-ip-goal-row">
              <span class="fl-ip-goal-dot" :style="{ background: idx < goalDistribution.length - 1 ? 'var(--color-primary)' : 'var(--color-text-muted)' }"></span>
              <span class="fl-ip-goal-name">{{ g.name }}</span>
              <span class="fl-ip-goal-num">{{ g.count }}</span>
            </div>
          </div>
        </section>

        <!-- AI 索引状态 -->
        <section class="fl-ip-side-card">
          <h3 class="fl-ip-side-title">AI 索引状态</h3>
          <div class="fl-ip-progress-card">
            <div class="fl-ip-progress-caption">
              <span>灵感语义索引</span>
              <span>{{ embeddingDoneCount }} / {{ embeddingPendingTotal }}</span>
            </div>
            <div class="fl-ip-progress-bar">
              <div class="fl-ip-progress-fill" :style="{ width: embeddingPendingTotal ? (embeddingDoneCount / embeddingPendingTotal * 100) + '%' : '0%' }"></div>
            </div>
          </div>
        </section>
      </aside>
    </div>

    <!-- D2: AI/契约错误显式提示 toast,4 秒自动消失 -->
    <Transition name="fl-bar">
      <div v-if="aiToast" class="fl-ip-toast" :class="`is-${aiToast.kind}`">
        <span class="fl-ip-toast-text">{{ aiToast.text }}</span>
        <button class="fl-ip-toast-close" type="button" @click="aiToast = null">
          <X :size="14" />
        </button>
      </div>
    </Transition>

    <!-- 全屏图谱 Modal:从右侧小图右上角"⤢"按钮触发 -->
    <InspirationGraphFullscreenModal
      v-model:open="fullscreenGraphOpen"
      :items="inspiration.items"
      :links="inspiration.linksById"
      :recommendations="inspiration.pendingRecommendations"
      :focused-id="focusedItemId"
      :analyzing="focusedAnalyzing"
      :analyzed="focusedAnalyzed"
      @analyze="onAnalyzeFocused"
      @accept-reco="onAcceptRecoFromGraph"
      @reject-reco="onIgnoreRecommendation"
      @node-click="onSelectIdea"
      @create-link="onCreateLink"
    />
  </section>
</template>

<style scoped>
.fl-inspirations {
  max-width: 1380px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: var(--sp-5);
}

/* ---------- 双栏工作区 ---------- */
.fl-ip-workspace {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 340px;
  gap: var(--sp-5);
  align-items: start;
}
@media (max-width: 1020px) {
  .fl-ip-workspace { grid-template-columns: 1fr; }
}
.fl-ip-main { display: flex; flex-direction: column; gap: var(--sp-4); min-width: 0; }
.fl-ip-sidebar {
  display: flex;
  flex-direction: column;
  gap: var(--sp-4);
  position: sticky;
  top: var(--sp-6);
}

/* ---------- 右侧面板通用 ---------- */
.fl-ip-side-card {
  padding: var(--sp-4);
  border: 1px solid var(--color-border);
  border-radius: 24px;
  background: var(--color-bg-elevated);
  box-shadow: var(--shadow-card);
}
.fl-ip-side-title {
  margin: 0 0 var(--sp-3);
  font-size: var(--fs-13, 13px);
  font-weight: 700;
}

/* 灵感概览 */
.fl-ip-metrics { display: grid; grid-template-columns: repeat(3, 1fr); gap: var(--sp-2); }
.fl-ip-metric {
  padding: var(--sp-3) var(--sp-2);
  border-radius: 15px;
  background: var(--color-bg-subtle);
  text-align: center;
}
.fl-ip-metric strong { display: block; font-size: 22px; letter-spacing: -0.5px; }
.fl-ip-metric span { display: block; margin-top: 2px; font-size: 11px; color: var(--color-text-muted); }

/* 待处理推荐 */
.fl-ip-review-list { display: flex; flex-direction: column; }
.fl-ip-review-item {
  display: flex;
  gap: var(--sp-2);
  padding: var(--sp-2) 0;
  border-top: 1px solid var(--color-divider);
}
.fl-ip-review-item:first-child { border-top: 0; padding-top: 0; }
.fl-ip-review-dot {
  width: 8px;
  height: 8px;
  margin-top: 6px;
  border-radius: 50%;
  background: var(--color-primary);
  flex-shrink: 0;
}
.fl-ip-review-dot.is-warn { background: var(--color-warning); }
.fl-ip-review-item p { margin: 0; font-size: 12px; line-height: 1.6; color: var(--color-text-secondary); }

/* 目标分布 */
.fl-ip-goal-list { display: flex; flex-direction: column; }
.fl-ip-goal-row {
  display: flex;
  align-items: center;
  gap: var(--sp-2);
  padding: 8px 0;
  border-top: 1px solid var(--color-divider);
}
.fl-ip-goal-row:first-child { border-top: 0; }
.fl-ip-goal-dot { width: 9px; height: 9px; border-radius: 50%; flex-shrink: 0; }
.fl-ip-goal-name { font-size: 12px; flex: 1; }
.fl-ip-goal-num { font-size: 11px; color: var(--color-text-muted); }

/* AI 索引状态 */
.fl-ip-progress-card {
  padding: var(--sp-3);
  border-radius: 16px;
  background: var(--color-primary-soft, color-mix(in srgb, var(--color-primary) 8%, white));
  border: 1px solid color-mix(in srgb, var(--color-primary) 20%, transparent);
}
.fl-ip-progress-caption { display: flex; justify-content: space-between; font-size: 11px; color: var(--color-text-secondary); }
.fl-ip-progress-bar {
  margin-top: var(--sp-2);
  height: 5px;
  border-radius: 999px;
  background: var(--color-bg-subtle);
  overflow: hidden;
}
.fl-ip-progress-fill {
  height: 100%;
  border-radius: 999px;
  background: linear-gradient(90deg, var(--color-primary), color-mix(in srgb, var(--color-primary) 60%, white));
  transition: width 0.3s ease;
}

/* ---------- 标题 ---------- */
.fl-ip-head {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: var(--sp-4);
}
.fl-ip-head h1 {
  margin: 0;
  font-size: 30px;
  font-weight: var(--fw-semibold);
  letter-spacing: -0.8px;
  display: flex;
  align-items: center;
  gap: var(--sp-3);
}
.fl-ip-mark {
  width: 38px;
  height: 38px;
  border-radius: 14px;
  display: grid;
  place-items: center;
  color: white;
  background: linear-gradient(145deg, var(--color-primary), color-mix(in srgb, var(--color-primary) 60%, white));
  box-shadow: 0 14px 34px color-mix(in srgb, var(--color-primary) 26%, transparent);
}
.fl-ip-icon { color: var(--color-primary); }
.fl-ip-sub {
  margin: var(--sp-2) 0 0;
  color: var(--color-text-secondary);
  font-size: var(--fs-14);
  line-height: 1.7;
  max-width: 660px;
}
.fl-ip-head-right { display: flex; align-items: center; gap: var(--sp-2); }
.fl-ip-ai-pill {
  height: 36px;
  padding: 0 12px;
  display: inline-flex;
  align-items: center;
  gap: 8px;
  border-radius: 999px;
  font-size: 12px;
  color: var(--color-text-secondary);
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  box-shadow: var(--shadow-card);
  cursor: help;
}
.fl-ip-pill-info {
  font-size: 11px;
  opacity: 0.55;
  margin-left: 1px;
}
.fl-ip-hint-privacy {
  cursor: help;
  border: 1px dashed color-mix(in srgb, var(--color-primary) 35%, transparent);
  background: color-mix(in srgb, var(--color-primary) 5%, var(--color-bg-subtle));
  color: var(--color-primary-dark) !important;
}
.fl-ip-pulse {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: var(--color-primary);
  box-shadow: 0 0 0 5px color-mix(in srgb, var(--color-primary) 12%, transparent);
  animation: fl-pulse 2s ease-in-out infinite;
}
@keyframes fl-pulse {
  0%, 100% { box-shadow: 0 0 0 3px color-mix(in srgb, var(--color-primary) 12%, transparent); }
  50% { box-shadow: 0 0 0 6px color-mix(in srgb, var(--color-primary) 8%, transparent); }
}

/* ---------- 输入区 ---------- */
.fl-ip-input-wrap {
  border: 1px solid var(--color-border);
  border-radius: 24px;
  background: var(--color-bg-elevated);
  box-shadow: 0 22px 60px rgba(20, 28, 48, 0.06);
  position: relative;
  overflow: hidden;
  backdrop-filter: blur(18px);
  transition: border-color var(--dur-fast) var(--ease-smooth),
    box-shadow var(--dur-fast) var(--ease-smooth);
}
.fl-ip-input-wrap::before {
  content: "";
  position: absolute;
  inset: 0 0 auto;
  height: 2px;
  background: linear-gradient(
    90deg,
    var(--color-primary),
    color-mix(in srgb, var(--color-primary) 20%, transparent)
  );
}
.fl-ip-textarea {
  width: 100%;
  padding: var(--sp-4) var(--sp-4) var(--sp-2);
  border: none;
  outline: none;
  box-shadow: none;
  resize: vertical;
  background: transparent;
  color: var(--color-text-primary);
  font-size: var(--fs-14);
  line-height: 1.7;
  font-family: var(--font-sans);
  min-height: 96px;
  max-height: 240px;
}
.fl-ip-textarea::placeholder { color: var(--color-text-muted); }
.fl-ip-textarea:focus-visible { box-shadow: none; }
.fl-ip-compose-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--sp-3) var(--sp-4) 0;
}
.fl-ip-compose-title { font-size: 13px; font-weight: 700; }
.fl-ip-compose-shortcut { font-size: 12px; color: var(--color-text-muted); }

/* ---------- AI 三候选 ---------- */
.fl-ip-drafts {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: var(--sp-3);
}
@media (max-width: 900px) { .fl-ip-drafts { grid-template-columns: 1fr; } }
.fl-ip-drafts-loading, .fl-ip-drafts-error {
  grid-column: 1 / -1;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: var(--sp-4);
  border-radius: 16px;
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  font-size: 13px;
  color: var(--color-text-secondary);
}
.fl-ip-drafts-error { color: var(--color-danger, #e53e3e); }
.fl-ip-draft-card {
  border-radius: 16px;
  padding: var(--sp-3);
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  box-shadow: var(--shadow-card);
  display: flex;
  flex-direction: column;
  gap: var(--sp-2);
}
.fl-ip-draft-card.is-primary {
  border-color: color-mix(in srgb, var(--color-primary) 22%, transparent);
  background: linear-gradient(180deg, color-mix(in srgb, var(--color-primary) 6%, transparent), var(--color-bg-elevated));
}
.fl-ip-draft-top {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: var(--sp-2);
}
.fl-ip-draft-kind { font-size: 12px; font-weight: 800; }
.fl-ip-draft-tone {
  font-size: 10px;
  color: var(--color-text-muted);
  padding: 3px 7px;
  border-radius: 999px;
  background: var(--color-bg-subtle);
}
.fl-ip-draft-text {
  font-size: 12px;
  line-height: 1.68;
  color: var(--color-text-primary);
  margin: 0;
  min-height: 80px;
}
.fl-ip-draft-foot { display: flex; gap: var(--sp-2); flex-wrap: wrap; }
.fl-ip-draft-btn {
  height: 28px;
  padding: 0 10px;
  border-radius: 9px;
  font-size: 12px;
  font-weight: 700;
  border: 1px solid var(--color-border);
  color: var(--color-text-secondary);
  background: var(--color-bg-elevated);
  cursor: pointer;
}
.fl-ip-draft-btn-primary {
  background: var(--color-primary);
  color: white;
  border-color: var(--color-primary);
}
.fl-ip-draft-btn-success {
  color: var(--color-success-text, #2f8b5b);
  background: color-mix(in srgb, var(--color-success, #2f8b5b) 10%, transparent);
}
.fl-ip-draft-btn-chat {
  display: flex;
  align-items: center;
  gap: 4px;
  color: var(--color-primary);
  border-color: var(--color-primary);
  background: color-mix(in srgb, var(--color-primary) 8%, transparent);
}
.fl-ip-draft-btn-copy {
  display: flex;
  align-items: center;
  gap: 4px;
  margin-right: auto;
}
.fl-ip-input-foot {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--sp-3);
  padding: var(--sp-2) var(--sp-3);
  border-top: 1px solid var(--color-divider);
  flex-wrap: wrap;
}
.fl-ip-hints { display: flex; flex-wrap: wrap; gap: 8px; }
.fl-ip-hint-tag {
  font-size: 11px;
  color: var(--color-text-muted);
  background: var(--color-bg-subtle);
  border-radius: 999px;
  padding: 4px 8px;
}
.fl-ip-input-actions { display: flex; gap: var(--sp-2); }
.fl-ip-btn {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 6px 12px;
  border-radius: var(--r-sm);
  border: 1px solid transparent;
  font-size: 12px;
  font-weight: var(--fw-medium);
  cursor: pointer;
  transition: all var(--dur-fast) var(--ease-smooth);
}
.fl-ip-btn-secondary {
  background: var(--color-bg-hover);
  color: var(--color-text-primary);
  border-color: var(--color-border);
}
.fl-ip-btn-secondary:hover:not(:disabled) {
  border-color: var(--color-primary);
  color: var(--color-primary);
}
.fl-ip-btn-primary {
  background: var(--color-primary);
  color: #fff;
}
.fl-ip-btn-primary:hover:not(:disabled) { opacity: 0.9; }
.fl-ip-btn:disabled { opacity: 0.45; cursor: not-allowed; }

/* ---------- 工具栏 ---------- */
.fl-ip-toolbar {
  display: flex;
  align-items: center;
  gap: var(--sp-3);
  flex-wrap: wrap;
}
.fl-ip-search {
  flex: 1;
  min-width: 200px;
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  border: 1px solid var(--color-border);
  border-radius: var(--r-pill);
  background: var(--color-bg-elevated);
  color: var(--color-text-muted);
  transition: border-color var(--dur-fast) var(--ease-smooth);
}
.fl-ip-search:focus-within {
  border-color: var(--color-primary);
  color: var(--color-primary);
}
.fl-ip-search input {
  flex: 1;
  border: none;
  outline: none;
  background: transparent;
  font-size: var(--fs-13, 13px);
  color: var(--color-text-primary);
  font-family: var(--font-sans);
}
.fl-ip-search input::placeholder { color: var(--color-text-muted); }

.fl-ip-filters { display: flex; gap: var(--sp-1); flex-wrap: wrap; }
.fl-ip-pill {
  padding: 5px 12px;
  border: 1px solid var(--color-border);
  background: var(--color-bg-elevated);
  border-radius: var(--r-pill);
  font-size: 12px;
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all var(--dur-fast) var(--ease-smooth);
}
.fl-ip-pill:hover { border-color: var(--color-primary); color: var(--color-primary); }
.fl-ip-pill.is-active {
  background: var(--color-primary-soft);
  border-color: var(--color-primary-soft);
  color: var(--color-primary-dark);
  font-weight: var(--fw-medium);
}
.fl-ip-pill-select { display: flex; align-items: center; gap: 4px; }

/* ---------- 时间线 ---------- */
.fl-ip-groups { display: flex; flex-direction: column; gap: var(--sp-4); }
.fl-ip-group { display: flex; flex-direction: column; }
.fl-ip-group-label {
  font-size: 12px;
  font-weight: 800;
  color: var(--color-text-muted);
  letter-spacing: 0.08em;
  padding-left: 46px;
  margin-bottom: var(--sp-2);
}
.fl-ip-timeline {
  position: relative;
  padding-left: 46px;
}
.fl-ip-timeline::before {
  content: "";
  position: absolute;
  left: 24px;
  top: 22px;
  bottom: 18px;
  width: 1px;
  background: linear-gradient(color-mix(in srgb, var(--color-primary) 22%, transparent), transparent);
}
.fl-ip-idea {
  position: relative;
  margin-bottom: var(--sp-3);
  padding: var(--sp-4);
  border-radius: 18px;
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  box-shadow: var(--shadow-card);
  transition: border-color var(--dur-fast) var(--ease-smooth), box-shadow var(--dur-fast) var(--ease-smooth);
}
.fl-ip-idea::before {
  content: "";
  position: absolute;
  left: -30px;
  top: 22px;
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background: var(--color-primary);
  box-shadow: 0 0 0 5px color-mix(in srgb, var(--color-primary) 10%, transparent);
}
.fl-ip-idea:hover {
  border-color: color-mix(in srgb, var(--color-primary) 40%, var(--color-border));
  box-shadow: 0 12px 30px rgba(20, 28, 48, 0.06);
}
.fl-ip-idea.is-warn::before {
  background: var(--color-warning);
  box-shadow: 0 0 0 5px color-mix(in srgb, var(--color-warning) 10%, transparent);
}
.fl-ip-idea.is-resolved::before {
  background: var(--color-success, #2f8b5b);
  box-shadow: 0 0 0 5px color-mix(in srgb, var(--color-success, #2f8b5b) 10%, transparent);
}
.fl-ip-tag-resolved {
  background: color-mix(in srgb, var(--color-success, #2f8b5b) 15%, transparent);
  color: var(--color-success-text, #2f8b5b);
}
.fl-ip-tag-overturned {
  background: color-mix(in srgb, var(--color-q1, #ef4444) 12%, transparent);
  color: var(--color-q1, #b91c1c);
  border: 1px solid color-mix(in srgb, var(--color-q1, #ef4444) 26%, transparent);
}
.fl-ip-idea.is-converted { opacity: 0.62; }
.fl-ip-idea.is-new {
  border-color: var(--color-primary);
  background: linear-gradient(135deg, color-mix(in srgb, var(--color-primary) 6%, var(--color-bg-elevated)), var(--color-bg-elevated) 60%);
}
.fl-ip-idea-head {
  display: flex;
  justify-content: space-between;
  gap: var(--sp-3);
  align-items: flex-start;
}
.fl-ip-idea-text {
  margin: 0;
  font-size: var(--fs-14);
  line-height: 1.74;
  color: var(--color-text-primary);
  max-width: 740px;
}
/* ---- 多选勾选框 ---- */
.fl-ip-check {
  background: none;
  border: none;
  color: var(--color-text-muted);
  cursor: pointer;
  padding: 0;
  display: flex;
  align-items: center;
  flex-shrink: 0;
}
.fl-ip-check-on { color: var(--color-primary); }
.fl-ip-idea.is-selected { border-color: var(--color-primary); background: color-mix(in srgb, var(--color-primary) 6%, var(--color-bg-elevated)); }

/* ---- 批量操作栏 ---- */
.fl-ip-batch-bar {
  position: sticky;
  bottom: var(--sp-4);
  display: flex;
  align-items: center;
  gap: var(--sp-3);
  padding: var(--sp-3) var(--sp-4);
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-primary);
  border-radius: var(--r-lg);
  box-shadow: 0 4px 16px color-mix(in srgb, black 12%, transparent);
  z-index: 10;
}
.fl-ip-batch-count { font-size: var(--fs-13); color: var(--color-text-secondary); font-weight: var(--fw-medium); }
.fl-ip-batch-btn {
  height: 30px;
  padding: 0 12px;
  border-radius: var(--r-md);
  border: 1px solid var(--color-border);
  background: var(--color-bg);
  color: var(--color-text-secondary);
  font-size: var(--fs-12);
  cursor: pointer;
}
.fl-ip-batch-btn:hover { border-color: var(--color-primary); color: var(--color-primary); }
.fl-ip-batch-btn-ai { display: flex; align-items: center; gap: 4px; background: var(--color-primary); color: #fff; border-color: var(--color-primary); }
.fl-ip-batch-btn-ai:hover { opacity: 0.9; }
.fl-ip-batch-btn-quit { margin-left: auto; display: flex; align-items: center; gap: 4px; color: var(--color-text-muted); }
.fl-bar-enter-active, .fl-bar-leave-active { transition: all 200ms var(--ease-smooth); }
.fl-bar-enter-from, .fl-bar-leave-to { opacity: 0; transform: translateY(8px); }

/* D2: AI/契约错误 toast */
.fl-ip-toast {
  position: fixed;
  bottom: 24px;
  left: 50%;
  transform: translateX(-50%);
  z-index: 200;
  display: flex;
  align-items: center;
  gap: var(--sp-2);
  padding: 10px 14px;
  border-radius: var(--r-md);
  font-size: var(--fs-13, 13px);
  max-width: 520px;
  box-shadow: 0 8px 24px rgba(20, 28, 48, 0.18);
}
.fl-ip-toast.is-error {
  background: color-mix(in srgb, var(--color-q1, #ef4444) 12%, var(--color-bg-elevated));
  border: 1px solid color-mix(in srgb, var(--color-q1, #ef4444) 40%, transparent);
  color: var(--color-q1, #b91c1c);
}
.fl-ip-toast.is-info {
  background: color-mix(in srgb, var(--color-primary) 8%, var(--color-bg-elevated));
  border: 1px solid color-mix(in srgb, var(--color-primary) 35%, transparent);
  color: var(--color-primary-dark);
}
.fl-ip-toast-text { flex: 1; line-height: 1.55; }
.fl-ip-toast-close {
  background: none;
  border: none;
  color: inherit;
  opacity: 0.7;
  cursor: pointer;
  display: grid;
  place-items: center;
  border-radius: var(--r-sm);
  padding: 2px;
}
.fl-ip-toast-close:hover { opacity: 1; background: rgba(0, 0, 0, 0.06); }

/* ---- 卡片工具栏 ---- */
.fl-ip-tool {
  width: 28px;
  height: 28px;
  border-radius: 9px;
  display: grid;
  place-items: center;
  color: var(--color-text-muted);
  border: none;
  background: none;
  cursor: pointer;
}
.fl-ip-tool:hover { background: var(--color-bg-hover); color: var(--color-primary); }
.fl-ip-tool.is-active { background: var(--color-primary-soft); color: var(--color-primary-dark); }
.fl-ip-idea-meta {
  margin-top: var(--sp-2);
  display: flex;
  align-items: center;
  gap: 7px;
  flex-wrap: wrap;
}
.fl-ip-idea-actions {
  margin-top: var(--sp-3);
  display: flex;
  gap: var(--sp-2);
  flex-wrap: wrap;
}
.fl-ip-act-btn {
  height: 28px;
  padding: 0 10px;
  border-radius: 9px;
  font-size: 12px;
  font-weight: 600;
  border: 1px solid var(--color-border);
  color: var(--color-text-secondary);
  background: var(--color-bg-elevated);
  cursor: pointer;
}
.fl-ip-act-btn:hover { border-color: var(--color-primary); color: var(--color-primary); }
.fl-ip-act-btn:disabled { opacity: 0.4; cursor: not-allowed; }
.fl-ip-act-warn {
  color: var(--color-warning-text, #b45309);
  background: color-mix(in srgb, var(--color-warning) 10%, transparent);
  border-color: color-mix(in srgb, var(--color-warning) 18%, transparent);
}
.fl-ip-act-success {
  color: var(--color-success-text, #2f8b5b);
  background: color-mix(in srgb, var(--color-success, #2f8b5b) 10%, transparent);
  border-color: color-mix(in srgb, var(--color-success, #2f8b5b) 18%, transparent);
}
.fl-ip-act-del {
  color: var(--color-text-muted);
  display: inline-flex;
  align-items: center;
}
.fl-ip-act-del:hover { color: var(--color-danger, #ef4444); }


/* ---------- tag ---------- */
.fl-ip-time { font-size: 11px; color: var(--color-text-muted); }
.fl-ip-tag {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  height: 22px;
  padding: 0 8px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 600;
}
.fl-ip-tag-goal {
  color: var(--color-primary-dark);
  background: color-mix(in srgb, var(--color-primary) 10%, transparent);
}
.fl-ip-tag-done {
  background: color-mix(in srgb, var(--color-success, #22c55e) 15%, transparent);
  color: var(--color-success-text, #16a34a);
}
.fl-ip-tag-new {
  background: color-mix(in srgb, var(--color-warning, #f59e0b) 18%, transparent);
  color: color-mix(in srgb, var(--color-warning, #f59e0b) 80%, #000);
}

.fl-ip-reco-btn-ai {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  border: 1px dashed color-mix(in srgb, var(--color-primary) 50%, transparent);
  background: color-mix(in srgb, var(--color-primary) 6%, transparent);
  color: var(--color-primary);
}
.fl-ip-reco-btn-ai:hover:not(:disabled) {
  background: color-mix(in srgb, var(--color-primary) 14%, transparent);
}

.fl-ip-tag-warning {
  background: color-mix(in srgb, var(--color-warning) 16%, transparent);
  color: var(--color-warning-text, #b45309);
}
.fl-ip-tag-retry {
  background: color-mix(in srgb, var(--color-warning) 14%, transparent);
  color: var(--color-warning-text, #b45309);
  border: 1px dashed color-mix(in srgb, var(--color-warning) 35%, transparent);
  cursor: pointer;
  transition: all 120ms ease;
}
.fl-ip-tag-retry:hover {
  background: color-mix(in srgb, var(--color-warning) 22%, transparent);
  border-color: var(--color-warning);
}
.fl-ip-tag-link {
  color: var(--color-success-text, #2f8b5b);
  background: color-mix(in srgb, var(--color-success, #2f8b5b) 12%, transparent);
}
.fl-ip-tag-keyword {
  color: var(--color-text-muted);
  background: var(--color-bg-subtle);
}
.fl-ip-idea.is-focused {
  border-color: var(--color-primary);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--color-primary) 12%, transparent);
}
.fl-ip-reco {
  margin-top: var(--sp-3);
  border-top: 1px solid var(--color-divider);
  padding-top: var(--sp-3);
  display: flex;
  flex-direction: column;
  gap: var(--sp-2);
}

/* 已分析无关联 — 卡片右下角浅灰小字 */
.fl-ip-noreco {
  margin-top: var(--sp-2);
  text-align: right;
  font-size: 11px;
  color: var(--color-text-muted);
  font-style: italic;
  user-select: none;
}
.fl-ip-reco-head {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  color: var(--color-primary-dark);
  font-size: 12px;
  font-weight: var(--fw-medium);
}
.fl-ip-reco-item {
  padding: var(--sp-2);
  border-radius: var(--r-sm);
  background: var(--color-bg-subtle);
  border: 1px solid var(--color-divider);
}
.fl-ip-reco-reason {
  font-size: 12px;
  line-height: 1.6;
  color: var(--color-text-secondary);
  margin: 0;
  padding: 8px 10px;
  border-radius: 10px;
  background: color-mix(in srgb, var(--color-primary) 5%, transparent);
}
.fl-ip-reco-old {
  font-size: 12px;
  line-height: 1.62;
  color: var(--color-text-primary);
  margin: 0 0 var(--sp-2);
}
.fl-ip-reco-actions {
  margin-top: var(--sp-2);
  display: flex;
  gap: var(--sp-2);
}
.fl-ip-reco-btn {
  height: 26px;
  padding: 0 10px;
  border-radius: var(--r-sm);
  border: 1px solid color-mix(in srgb, var(--color-primary) 20%, transparent);
  background: var(--color-primary-soft);
  color: var(--color-primary-dark);
  font-size: 12px;
  font-weight: var(--fw-medium);
}
.fl-ip-reco-btn-ghost {
  border-color: var(--color-border);
  background: var(--color-bg-elevated);
  color: var(--color-text-secondary);
}
.fl-ip-reco-btn-warn {
  color: var(--color-warning-text, #b45309);
  background: color-mix(in srgb, var(--color-warning) 10%, transparent);
  border-color: color-mix(in srgb, var(--color-warning) 18%, transparent);
}
.fl-ip-reco-item.is-warn {
  border-color: color-mix(in srgb, var(--color-warning) 30%, transparent);
  background: linear-gradient(180deg, color-mix(in srgb, var(--color-warning) 8%, transparent), var(--color-bg-subtle));
}
.fl-ip-reco-state {
  justify-content: center;
}
.fl-ip-reco-state.is-error {
  border-color: color-mix(in srgb, var(--color-warning) 28%, var(--color-border));
  background: color-mix(in srgb, var(--color-warning) 5%, var(--color-bg-subtle));
}

.fl-ip-empty {
  text-align: center;
  padding: var(--sp-8) var(--sp-4);
  color: var(--color-text-muted);
}
.fl-ip-empty-icon {
  color: var(--color-text-muted);
  opacity: 0.4;
  margin-bottom: var(--sp-3);
}
.fl-ip-empty h3 {
  margin: 0 0 4px;
  font-size: var(--fs-16);
  font-weight: var(--fw-semibold);
  color: var(--color-text-primary);
}
.fl-ip-empty p {
  margin: 0;
  font-size: var(--fs-13, 13px);
  line-height: 1.6;
}

</style>
