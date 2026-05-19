<script setup lang="ts">
import {
  AlertTriangle,
  ArrowUpRight,
  Check,
  Eye,
  GitBranch,
  History,
  Sparkles,
  X,
} from "lucide-vue-next";
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";

import InspirationCorrectionPanel from "@/components/inspiration/InspirationCorrectionPanel.vue";
import type { InspirationRecommendation } from "@/types";

type ReviewFilter = "all" | "contradicts" | "related";

interface ReviewInboxEntry {
  key: string;
  itemId: string;
  sourceContent: string;
  sourceGoalName: string | null;
  sourceLinkCount: number;
  candidateId: string;
  candidateContent: string;
  candidateGoalName: string | null;
  candidateLinkCount: number;
  reco: InspirationRecommendation;
}

interface ReviewCorrectionResult {
  summary: string;
  oldJudgment: string;
  newEvidence: string;
  suggestion: string;
}

const props = defineProps<{
  entries: ReviewInboxEntry[];
  activeKey: string | null;
  busyKey?: string | null;
  deferredKeys?: string[];
  correctionLoadingKey?: string | null;
  correctionResults?: Record<string, ReviewCorrectionResult | null>;
}>();

const emit = defineEmits<{
  "select-entry": [key: string];
  accept: [itemId: string, reco: InspirationRecommendation];
  ignore: [itemId: string, candidateId: string];
  defer: [itemId: string, candidateId: string, relation: InspirationRecommendation["relation"]];
  "focus-source": [itemId: string];
  "focus-candidate": [candidateId: string];
  "analyze-correction": [itemId: string, candidateId: string, candidateContent: string];
  "mark-needs-check": [candidateId: string];
  "mark-overturned": [candidateId: string];
  "create-followup": [itemId: string];
  "restore-deferred": [key: string];
  "restore-all-deferred": [keys: string[]];
}>();

const filter = ref<ReviewFilter>("all");
const showDeferred = ref(false);
const deferredKeySet = computed(() => new Set(props.deferredKeys ?? []));

const filteredEntries = computed(() => {
  if (filter.value === "all") return props.entries;
  return props.entries.filter((entry) => entry.reco.relation === filter.value);
});

const actionableEntries = computed(() =>
  filteredEntries.value.filter((entry) => !deferredKeySet.value.has(entry.key)),
);

const deferredEntries = computed(() =>
  filteredEntries.value.filter((entry) => deferredKeySet.value.has(entry.key)),
);

const activeEntry = computed(
  () => actionableEntries.value.find((entry) => entry.key === props.activeKey) ?? actionableEntries.value[0] ?? null,
);

const activeIndex = computed(() => {
  if (!activeEntry.value) return 0;
  return actionableEntries.value.findIndex((entry) => entry.key === activeEntry.value?.key) + 1;
});

watch(
  actionableEntries,
  (entries) => {
    if (!entries.length) return;
    if (!props.activeKey || !entries.some((entry) => entry.key === props.activeKey)) {
      emit("select-entry", entries[0].key);
    }
  },
  { immediate: true },
);

const correctionKey = computed(() =>
  activeEntry.value ? `${activeEntry.value.itemId}::${activeEntry.value.candidateId}` : "",
);

const activeCorrection = computed(() =>
  correctionKey.value ? props.correctionResults?.[correctionKey.value] ?? null : null,
);

const correctionBusy = computed(() => props.correctionLoadingKey === correctionKey.value);

function relationLabel(relation: InspirationRecommendation["relation"]) {
  return relation === "contradicts" ? "矛盾/纠偏" : "相关";
}

function relationActionLabel(relation: InspirationRecommendation["relation"]) {
  return relation === "contradicts" ? "建立修正连接" : "接受为相关";
}

function confidenceLabel(confidence: number) {
  return `${Math.round(confidence * 100)}%`;
}

function isBusy(entry: ReviewInboxEntry) {
  return props.busyKey === entry.key;
}

function deferEntry(entry: ReviewInboxEntry | null) {
  if (!entry) return;
  emit("defer", entry.itemId, entry.candidateId, entry.reco.relation);
  const next = actionableEntries.value.find((candidate) => candidate.key !== entry.key);
  if (next) emit("select-entry", next.key);
}

function restoreDeferred(key: string) {
  if (!deferredKeySet.value.has(key)) return;
  emit("restore-deferred", key);
  emit("select-entry", key);
}

function restoreAllDeferred() {
  if (!deferredEntries.value.length) return;
  emit("restore-all-deferred", deferredEntries.value.map((entry) => entry.key));
  if (!props.activeKey && deferredEntries.value[0]) {
    emit("select-entry", deferredEntries.value[0].key);
  }
}

function selectOffset(offset: number) {
  if (!actionableEntries.value.length) return;
  const currentIndex = actionableEntries.value.findIndex((entry) => entry.key === activeEntry.value?.key);
  const baseIndex = currentIndex >= 0 ? currentIndex : 0;
  const nextIndex = Math.min(
    actionableEntries.value.length - 1,
    Math.max(0, baseIndex + offset),
  );
  const nextEntry = actionableEntries.value[nextIndex];
  if (nextEntry) emit("select-entry", nextEntry.key);
}

function acceptActive() {
  if (!activeEntry.value || isBusy(activeEntry.value)) return;
  emit("accept", activeEntry.value.itemId, activeEntry.value.reco);
}

function ignoreActive() {
  if (!activeEntry.value || isBusy(activeEntry.value)) return;
  emit("ignore", activeEntry.value.itemId, activeEntry.value.candidateId);
}

function deferActive() {
  if (!activeEntry.value || isBusy(activeEntry.value)) return;
  deferEntry(activeEntry.value);
}

function isEditableTarget(target: EventTarget | null) {
  if (!(target instanceof HTMLElement)) return false;
  if (target.isContentEditable) return true;
  const tagName = target.tagName.toLowerCase();
  return tagName === "input" || tagName === "textarea" || tagName === "select";
}

function onWindowKeydown(event: KeyboardEvent) {
  if (!actionableEntries.value.length) return;
  if (event.defaultPrevented) return;
  if (event.metaKey || event.ctrlKey || event.altKey) return;
  if (isEditableTarget(event.target)) return;

  const key = event.key.toLowerCase();
  if (key === "j" || event.key === "ArrowDown") {
    event.preventDefault();
    selectOffset(1);
    return;
  }
  if (key === "k" || event.key === "ArrowUp") {
    event.preventDefault();
    selectOffset(-1);
    return;
  }
  if (event.key === "Enter") {
    event.preventDefault();
    acceptActive();
    return;
  }
  if (key === "i") {
    event.preventDefault();
    ignoreActive();
    return;
  }
  if (key === "s") {
    event.preventDefault();
    deferActive();
  }
}

onMounted(() => {
  window.addEventListener("keydown", onWindowKeydown);
});

onBeforeUnmount(() => {
  window.removeEventListener("keydown", onWindowKeydown);
});
</script>

<template>
  <section v-if="entries.length" class="fl-review-inbox">
    <header class="fl-review-head">
      <div class="fl-review-head-copy">
        <h3>关联收件箱</h3>
        <p>集中处理 AI 推荐，不用在卡片墙里来回翻找。</p>
      </div>
      <div class="fl-review-count">
        <strong>{{ actionableEntries.length }}</strong>
        <span>当前待处理</span>
        <small v-if="deferredEntries.length">另有 {{ deferredEntries.length }} 条稍后</small>
      </div>
    </header>

    <div class="fl-review-toolbar">
      <div class="fl-review-filters">
        <button
          class="fl-review-filter"
          :class="{ 'is-active': filter === 'all' }"
          type="button"
          @click="filter = 'all'"
        >
          全部
        </button>
        <button
          class="fl-review-filter"
          :class="{ 'is-active': filter === 'contradicts' }"
          type="button"
          @click="filter = 'contradicts'"
        >
          矛盾/纠偏
        </button>
        <button
          class="fl-review-filter"
          :class="{ 'is-active': filter === 'related' }"
          type="button"
          @click="filter = 'related'"
        >
          相关
        </button>
      </div>
      <button
        v-if="deferredEntries.length"
        class="fl-review-deferred-toggle"
        :class="{ 'is-active': showDeferred }"
        type="button"
        @click="showDeferred = !showDeferred"
      >
        <History :size="12" />
        稍后处理 {{ deferredEntries.length }}
      </button>
      <p v-if="activeEntry" class="fl-review-progress">
        {{ activeIndex }} / {{ actionableEntries.length }}
      </p>
    </div>

    <div class="fl-review-shortcuts">
      <span class="fl-review-shortcut-key">J/K</span><span>切换</span>
      <span class="fl-review-shortcut-key">Enter</span><span>接受</span>
      <span class="fl-review-shortcut-key">I</span><span>忽略</span>
      <span class="fl-review-shortcut-key">S</span><span>稍后</span>
    </div>

    <div class="fl-review-body">
      <aside class="fl-review-list">
        <button
          v-for="entry in filteredEntries"
          :key="entry.key"
          class="fl-review-list-item"
          :class="{
            'is-active': activeEntry?.key === entry.key,
            'is-warn': entry.reco.relation === 'contradicts',
          }"
          type="button"
          @click="emit('select-entry', entry.key)"
        >
          <div class="fl-review-list-top">
            <span class="fl-review-pill" :class="{ 'is-warn': entry.reco.relation === 'contradicts' }">
              {{ relationLabel(entry.reco.relation) }}
            </span>
            <span class="fl-review-confidence">置信 {{ confidenceLabel(entry.reco.confidence) }}</span>
          </div>
          <p class="fl-review-snippet">
            <span>来源</span>
            {{ entry.sourceContent }}
          </p>
          <p class="fl-review-snippet">
            <span>候选</span>
            {{ entry.candidateContent }}
          </p>
        </button>

        <div v-if="!filteredEntries.length" class="fl-review-empty-list">
          <Sparkles :size="14" />
          当前筛选下没有待处理推荐
        </div>

        <div v-else-if="!actionableEntries.length" class="fl-review-empty-list">
          <History :size="14" />
          当前筛选下的推荐都已稍后
        </div>

        <section v-if="showDeferred && deferredEntries.length" class="fl-review-deferred-list">
          <div class="fl-review-deferred-head">
            <span>稍后处理（会保留）</span>
            <button type="button" @click="restoreAllDeferred()">恢复全部</button>
          </div>
          <div
            v-for="entry in deferredEntries"
            :key="entry.key"
            class="fl-review-deferred-item"
          >
            <div class="fl-review-deferred-copy">
              <span class="fl-review-pill" :class="{ 'is-warn': entry.reco.relation === 'contradicts' }">
                {{ relationLabel(entry.reco.relation) }}
              </span>
              <p>{{ entry.reco.reason }}</p>
            </div>
            <button type="button" @click="restoreDeferred(entry.key)">恢复</button>
          </div>
        </section>
      </aside>

      <section v-if="activeEntry" class="fl-review-detail">
        <div class="fl-review-detail-head">
          <span class="fl-review-detail-title">
            <GitBranch :size="14" />
            推荐上下文
          </span>
          <span class="fl-review-detail-tag" :class="{ 'is-warn': activeEntry.reco.relation === 'contradicts' }">
            {{ relationLabel(activeEntry.reco.relation) }}
          </span>
        </div>

        <div class="fl-review-compare">
          <button
            type="button"
            class="fl-review-note"
            @click="emit('focus-source', activeEntry.itemId)"
          >
            <div class="fl-review-note-head">
              <span>当前灵感</span>
              <ArrowUpRight :size="13" />
            </div>
            <p>{{ activeEntry.sourceContent }}</p>
            <div class="fl-review-note-meta">
              <span v-if="activeEntry.sourceGoalName">{{ activeEntry.sourceGoalName }}</span>
              <span>{{ activeEntry.sourceLinkCount }} 条连接</span>
            </div>
          </button>

          <button
            type="button"
            class="fl-review-note"
            @click="emit('focus-candidate', activeEntry.candidateId)"
          >
            <div class="fl-review-note-head">
              <span>候选灵感</span>
              <ArrowUpRight :size="13" />
            </div>
            <p>{{ activeEntry.candidateContent }}</p>
            <div class="fl-review-note-meta">
              <span v-if="activeEntry.candidateGoalName">{{ activeEntry.candidateGoalName }}</span>
              <span>{{ activeEntry.candidateLinkCount }} 条连接</span>
            </div>
          </button>
        </div>

        <section class="fl-review-reason">
          <div class="fl-review-reason-head">
            <Sparkles :size="14" />
            <span>AI 推荐理由</span>
          </div>
          <p>{{ activeEntry.reco.reason }}</p>
        </section>

        <div class="fl-review-actions">
          <button
            class="fl-review-action"
            type="button"
            @click="emit('focus-source', activeEntry.itemId)"
          >
            <Eye :size="13" />
            查看来源
          </button>
          <button
            class="fl-review-action"
            type="button"
            @click="emit('focus-candidate', activeEntry.candidateId)"
          >
            <Eye :size="13" />
            查看候选
          </button>
          <button
            v-if="activeEntry.reco.relation === 'contradicts'"
            class="fl-review-action fl-review-action-warn"
            type="button"
            @click="emit('mark-needs-check', activeEntry.candidateId)"
          >
            <AlertTriangle :size="13" />
            标记候选待复查
          </button>
          <button
            v-if="activeEntry.reco.relation === 'contradicts' && !activeCorrection"
            class="fl-review-action fl-review-action-ai"
            type="button"
            :disabled="correctionBusy"
            @click="emit('analyze-correction', activeEntry.itemId, activeEntry.candidateId, activeEntry.candidateContent)"
          >
            <Sparkles :size="13" />
            {{ correctionBusy ? "分析中…" : "AI 纠偏分析" }}
          </button>
          <button
            class="fl-review-action"
            type="button"
            :disabled="isBusy(activeEntry)"
            @click="deferEntry(activeEntry)"
          >
            <History :size="13" />
            稍后处理
          </button>
          <button
            class="fl-review-action fl-review-action-primary"
            type="button"
            :disabled="isBusy(activeEntry)"
            @click="emit('accept', activeEntry.itemId, activeEntry.reco)"
          >
            <Check :size="13" />
            {{ isBusy(activeEntry) ? "处理中…" : relationActionLabel(activeEntry.reco.relation) }}
          </button>
          <button
            class="fl-review-action fl-review-action-danger"
            type="button"
            :disabled="isBusy(activeEntry)"
            @click="emit('ignore', activeEntry.itemId, activeEntry.candidateId)"
          >
            <X :size="13" />
            {{ isBusy(activeEntry) ? "处理中…" : "忽略这条" }}
          </button>
        </div>

        <InspirationCorrectionPanel
          v-if="activeCorrection"
          :result="activeCorrection"
          @mark-needs-check="emit('mark-needs-check', activeEntry.candidateId)"
          @mark-overturned="emit('mark-overturned', activeEntry.candidateId)"
          @accept-correction="emit('accept', activeEntry.itemId, activeEntry.reco)"
          @create-followup="emit('create-followup', activeEntry.itemId)"
        />
      </section>

      <section v-else class="fl-review-detail-empty">
        <Sparkles :size="16" />
        <p>
          {{ deferredEntries.length ? "当前筛选下都被稍后处理了。" : "当前筛选下没有待处理推荐。" }}
        </p>
      </section>
    </div>
  </section>
</template>

<style scoped>
.fl-review-inbox {
  padding: var(--sp-4);
  border: 1px solid var(--color-border);
  border-radius: 24px;
  background: var(--color-bg-elevated);
  box-shadow: var(--shadow-card);
}

.fl-review-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--sp-3);
}

.fl-review-head-copy h3 {
  margin: 0;
  font-size: 16px;
  font-weight: 700;
}

.fl-review-head-copy p {
  margin: 6px 0 0;
  color: var(--color-text-secondary);
  font-size: 12px;
  line-height: 1.6;
}

.fl-review-count {
  min-width: 80px;
  padding: 10px 12px;
  border-radius: 16px;
  background: var(--color-bg-subtle);
  text-align: center;
}

.fl-review-count strong {
  display: block;
  font-size: 22px;
}

.fl-review-count span {
  display: block;
  margin-top: 2px;
  color: var(--color-text-muted);
  font-size: 11px;
}

.fl-review-count small {
  display: block;
  margin-top: 4px;
  color: var(--color-text-muted);
  font-size: 10px;
}

.fl-review-toolbar {
  margin-top: var(--sp-3);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--sp-3);
  flex-wrap: wrap;
}

.fl-review-filters {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.fl-review-filter {
  height: 30px;
  padding: 0 11px;
  border: 1px solid var(--color-border);
  border-radius: 999px;
  background: transparent;
  color: var(--color-text-secondary);
  cursor: pointer;
  font-size: 11px;
  font-weight: var(--fw-medium);
}

.fl-review-filter.is-active {
  border-color: var(--color-primary);
  background: var(--color-primary-soft);
  color: var(--color-primary-dark);
}

.fl-review-progress {
  margin: 0;
  color: var(--color-text-muted);
  font-size: 11px;
}

.fl-review-shortcuts {
  margin: 10px 0 0;
  display: flex;
  align-items: center;
  gap: 6px 8px;
  flex-wrap: wrap;
  color: var(--color-text-muted);
  font-size: 11px;
  line-height: 1.5;
}

.fl-review-shortcut-key {
  min-width: 28px;
  height: 22px;
  padding: 0 7px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--color-border);
  border-radius: 8px;
  background: var(--color-bg-subtle);
  color: var(--color-text-secondary);
  font-size: 10px;
  font-weight: var(--fw-semibold);
}

.fl-review-deferred-toggle {
  height: 30px;
  padding: 0 11px;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  border: 1px solid var(--color-border);
  border-radius: 999px;
  background: transparent;
  color: var(--color-text-secondary);
  cursor: pointer;
  font-size: 11px;
  font-weight: var(--fw-medium);
}

.fl-review-deferred-toggle.is-active {
  border-color: var(--color-primary);
  background: var(--color-primary-soft);
  color: var(--color-primary-dark);
}

.fl-review-body {
  margin-top: var(--sp-3);
  display: grid;
  grid-template-columns: minmax(240px, 320px) minmax(0, 1fr);
  gap: var(--sp-3);
}

@media (max-width: 980px) {
  .fl-review-body {
    grid-template-columns: 1fr;
  }
}

.fl-review-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
  min-width: 0;
}

.fl-review-deferred-list {
  margin-top: 6px;
  padding-top: 12px;
  border-top: 1px solid var(--color-divider);
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.fl-review-deferred-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  color: var(--color-text-secondary);
  font-size: 11px;
  font-weight: var(--fw-semibold);
}

.fl-review-deferred-head button,
.fl-review-deferred-item button {
  height: 28px;
  padding: 0 10px;
  border: 1px solid var(--color-border);
  border-radius: 999px;
  background: transparent;
  color: var(--color-text-secondary);
  cursor: pointer;
  font-size: 10px;
}

.fl-review-deferred-item {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 10px;
  padding: 10px 12px;
  border: 1px dashed var(--color-border);
  border-radius: 16px;
}

.fl-review-deferred-copy {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.fl-review-deferred-copy p {
  margin: 0;
  color: var(--color-text-secondary);
  font-size: 12px;
  line-height: 1.55;
}

.fl-review-list-item {
  width: 100%;
  padding: 12px;
  border: 1px solid var(--color-border);
  border-radius: 18px;
  background: var(--color-bg-subtle);
  text-align: left;
  cursor: pointer;
  transition: border-color 140ms ease, background 140ms ease, transform 140ms ease;
}

.fl-review-list-item:hover {
  border-color: color-mix(in srgb, var(--color-primary) 30%, var(--color-border));
  transform: translateY(-1px);
}

.fl-review-list-item.is-active {
  border-color: var(--color-primary);
  background: color-mix(in srgb, var(--color-primary) 8%, var(--color-bg-elevated));
}

.fl-review-list-item.is-warn.is-active {
  border-color: color-mix(in srgb, var(--color-warning) 55%, var(--color-border));
}

.fl-review-list-top {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--sp-2);
}

.fl-review-pill,
.fl-review-detail-tag {
  display: inline-flex;
  align-items: center;
  padding: 3px 8px;
  border-radius: 999px;
  background: color-mix(in srgb, var(--color-primary) 12%, transparent);
  color: var(--color-primary-dark);
  font-size: 10px;
  font-weight: var(--fw-semibold);
}

.fl-review-pill.is-warn,
.fl-review-detail-tag.is-warn {
  background: color-mix(in srgb, var(--color-warning) 14%, transparent);
  color: var(--color-warning-text);
}

.fl-review-confidence {
  color: var(--color-text-muted);
  font-size: 10px;
}

.fl-review-snippet {
  margin: 8px 0 0;
  font-size: 12px;
  line-height: 1.55;
  color: var(--color-text-secondary);
  display: -webkit-box;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 2;
  overflow: hidden;
}

.fl-review-snippet span {
  margin-right: 6px;
  color: var(--color-text-muted);
  font-size: 10px;
}

.fl-review-empty-list,
.fl-review-detail-empty {
  min-height: 160px;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  border: 1px dashed var(--color-border);
  border-radius: 18px;
  color: var(--color-text-muted);
  font-size: 12px;
}

.fl-review-detail {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: var(--sp-3);
}

.fl-review-detail-head,
.fl-review-reason-head,
.fl-review-note-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--sp-2);
}

.fl-review-detail-title,
.fl-review-reason-head {
  color: var(--color-text-secondary);
  font-size: 12px;
  font-weight: var(--fw-semibold);
}

.fl-review-reason-head {
  justify-content: flex-start;
}

.fl-review-compare {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--sp-3);
}

@media (max-width: 760px) {
  .fl-review-compare {
    grid-template-columns: 1fr;
  }
}

.fl-review-note {
  padding: 14px;
  border: 1px solid var(--color-border);
  border-radius: 18px;
  background: var(--color-bg-subtle);
  text-align: left;
  cursor: pointer;
}

.fl-review-note-head span {
  color: var(--color-text-secondary);
  font-size: 11px;
  font-weight: var(--fw-semibold);
}

.fl-review-note p {
  margin: 10px 0 0;
  color: var(--color-text-primary);
  font-size: 13px;
  line-height: 1.65;
}

.fl-review-note-meta {
  margin-top: 10px;
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.fl-review-note-meta span {
  color: var(--color-text-muted);
  font-size: 10px;
}

.fl-review-reason {
  padding: 14px;
  border-radius: 18px;
  background: color-mix(in srgb, var(--color-primary) 5%, var(--color-bg-subtle));
  border: 1px solid color-mix(in srgb, var(--color-primary) 14%, var(--color-border));
}

.fl-review-reason p {
  margin: 10px 0 0;
  color: var(--color-text-secondary);
  font-size: 13px;
  line-height: 1.7;
}

.fl-review-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
}

.fl-review-action {
  height: 34px;
  padding: 0 12px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  border: 1px solid var(--color-border);
  border-radius: 999px;
  background: transparent;
  color: var(--color-text-secondary);
  cursor: pointer;
  font-size: 11px;
  font-weight: var(--fw-medium);
}

.fl-review-action:disabled {
  cursor: progress;
  opacity: 0.7;
}

.fl-review-action-primary {
  border-color: var(--color-primary);
  background: var(--color-primary-soft);
  color: var(--color-primary-dark);
}

.fl-review-action-danger {
  border-color: color-mix(in srgb, var(--color-danger) 24%, transparent);
  color: var(--color-danger);
}

.fl-review-action-warn {
  border-color: color-mix(in srgb, var(--color-warning) 30%, transparent);
  color: var(--color-warning-text);
}

.fl-review-action-ai {
  border-color: color-mix(in srgb, var(--color-primary) 26%, transparent);
  color: var(--color-primary-dark);
}
</style>
