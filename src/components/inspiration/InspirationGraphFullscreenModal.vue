<script setup lang="ts">
/**
 * InspirationGraphFullscreenModal · 灵感图谱全屏视图
 *
 * - 沿用 TaskEditModal 的 .fl-modal-mask + .fl-modal-card + Transition fl-fade 模式
 * - card 撑满 100vw/100vh
 * - 头部带搜索框: 命中节点高亮, 其它降透明度
 * - ESC / 点 mask 关闭
 * - 内嵌 InspirationGraphCore (mode='full'),开启拖拽/缩放/MiniMap/Controls
 */

import { RefreshCw, Search, Sparkles, X } from "lucide-vue-next";
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";

import type { InspirationItem } from "@/stores/useInspirationStore";
import type { InspirationLink, InspirationRecommendation } from "@/types";

import InspirationGraphCore from "./InspirationGraphCore.vue";

const props = defineProps<{
  open: boolean;
  items: InspirationItem[];
  links: Record<string, InspirationLink[]>;
  recommendations: Record<string, InspirationRecommendation[]>;
  /** 打开时的中心节点(用于初始 focus + 「分析」按钮目标) */
  focusedId?: string | null;
  /** 当前 focused 节点是否在跑分析 */
  analyzing?: boolean;
  /** 当前 focused 节点是否已分析过 */
  analyzed?: boolean;
}>();

const emit = defineEmits<{
  "update:open": [boolean];
  "accept-reco": [sourceId: string, reco: InspirationRecommendation];
  "reject-reco": [sourceId: string, candidateId: string];
  "node-click": [id: string];
  "create-link": [sourceId: string, targetId: string, relation: "related" | "contradicts"];
  "delete-link": [sourceId: string, targetId: string];
  analyze: [];
}>();

const searchQuery = ref("");
const searchInput = ref<HTMLInputElement | null>(null);

const focusedItem = computed<InspirationItem | null>(() =>
  props.focusedId ? props.items.find((i) => i.id === props.focusedId) ?? null : null,
);

const analyzeBtnLabel = computed(() => {
  if (props.analyzing) return "分析中…";
  return props.analyzed ? "重新分析" : "分析关联";
});

function truncate(text: string, max: number) {
  return text.length > max ? text.slice(0, max) + "…" : text;
}

const focusedShortLabel = computed(() => {
  if (!focusedItem.value) return "";
  return truncate(focusedItem.value.summary || focusedItem.value.content, 12);
});

function close() {
  emit("update:open", false);
}

function onKeydown(e: KeyboardEvent) {
  if (!props.open) return;
  if (e.key === "Escape") {
    // 如果搜索框有内容,先清空,再次按才关闭
    if (searchQuery.value) {
      searchQuery.value = "";
      e.preventDefault();
      return;
    }
    e.preventDefault();
    close();
  }
}

onMounted(() => {
  document.addEventListener("keydown", onKeydown);
});

onBeforeUnmount(() => {
  document.removeEventListener("keydown", onKeydown);
});

watch(
  () => props.open,
  (open) => {
    if (open) {
      searchQuery.value = "";
      // 等 modal 完成 transition 再 focus
      requestAnimationFrame(() => {
        requestAnimationFrame(() => searchInput.value?.focus());
      });
    }
  },
);
</script>

<template>
  <Transition name="fl-fade">
    <div
      v-if="open"
      class="fl-modal-mask fl-graph-modal-mask"
      role="presentation"
      @click.self="close"
    >
      <div
        class="fl-modal-card fl-graph-modal-card"
        role="dialog"
        aria-modal="true"
        aria-labelledby="fl-graph-modal-title"
      >
        <header class="fl-graph-modal-head">
          <h2 id="fl-graph-modal-title">灵感关联图谱</h2>
          <div class="fl-graph-modal-search">
            <Search :size="14" class="fl-graph-search-icon" />
            <input
              ref="searchInput"
              v-model="searchQuery"
              class="fl-graph-search-input"
              type="search"
              placeholder="搜索灵感内容、摘要或关键词…"
              maxlength="80"
            />
            <button
              v-if="searchQuery"
              class="fl-graph-search-clear"
              type="button"
              title="清空"
              @click="searchQuery = ''"
            >
              <X :size="12" />
            </button>
          </div>
          <button
            v-if="focusedItem"
            type="button"
            class="fl-graph-modal-analyze"
            :class="{ 'is-loading': analyzing, 'is-redo': analyzed && !analyzing }"
            :disabled="analyzing"
            :title="`${analyzed ? '重新跑' : '让 AI 找'}「${focusedShortLabel}」的相关灵感`"
            @click="emit('analyze')"
          >
            <RefreshCw v-if="analyzing || analyzed" :size="14" :class="{ 'is-spinning': analyzing }" />
            <Sparkles v-else :size="14" />
            <span>{{ analyzeBtnLabel }}</span>
            <span v-if="focusedShortLabel" class="fl-graph-modal-analyze-target">「{{ focusedShortLabel }}」</span>
          </button>
          <button class="fl-icon-btn" type="button" aria-label="关闭" @click="close">
            <X :size="16" />
          </button>
        </header>

        <div class="fl-graph-modal-body">
          <InspirationGraphCore
            v-if="open"
            mode="full"
            :items="items"
            :links="links"
            :recommendations="recommendations"
            :focused-id="focusedId ?? null"
            :search-query="searchQuery"
            @accept-reco="(sourceId, reco) => emit('accept-reco', sourceId, reco)"
            @reject-reco="(sourceId, candidateId) => emit('reject-reco', sourceId, candidateId)"
            @node-click="(id) => emit('node-click', id)"
            @create-link="(sourceId, targetId, relation) => emit('create-link', sourceId, targetId, relation)"
            @delete-link="(sourceId, targetId) => emit('delete-link', sourceId, targetId)"
          />
        </div>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
/* 沿用 TaskEditModal 的命名,但样式独立 */
.fl-modal-mask {
  position: fixed;
  inset: 0;
  background: color-mix(in srgb, var(--color-text-primary) 50%, transparent);
  z-index: var(--z-modal, 1000);
  display: grid;
  place-items: center;
}

.fl-modal-card {
  width: 100vw;
  height: 100vh;
  max-width: 100vw;
  max-height: 100vh;
  background: var(--color-bg);
  display: flex;
  flex-direction: column;
  border-radius: 0;
}

.fl-graph-modal-head {
  display: flex;
  align-items: center;
  gap: var(--sp-3);
  padding: var(--sp-3) var(--sp-5);
  border-bottom: 1px solid var(--color-border);
  background: var(--color-bg-elevated);
  flex-shrink: 0;
}

.fl-graph-modal-head h2 {
  margin: 0;
  font-size: var(--fs-16);
  font-weight: var(--fw-semibold);
  color: var(--color-text-primary);
  flex-shrink: 0;
}

.fl-graph-modal-search {
  flex: 1;
  max-width: 420px;
  position: relative;
  display: flex;
  align-items: center;
  margin-left: var(--sp-3);
}

.fl-graph-search-icon {
  position: absolute;
  left: 10px;
  color: var(--color-text-muted);
  pointer-events: none;
}

.fl-graph-search-input {
  flex: 1;
  padding: 8px 32px 8px 32px;
  border-radius: var(--r-md);
  border: 1px solid var(--color-border);
  background: var(--color-bg-subtle);
  color: var(--color-text-primary);
  font-size: var(--fs-12);
  outline: none;
  width: 100%;
}

.fl-graph-search-input:focus {
  border-color: var(--color-primary);
  box-shadow:
    var(--shadow-focus, 0 0 0 3px color-mix(in srgb, var(--color-primary) 18%, transparent));
}

.fl-graph-search-clear {
  position: absolute;
  right: 6px;
  background: transparent;
  border: none;
  color: var(--color-text-muted);
  cursor: pointer;
  padding: 4px;
  border-radius: var(--r-sm);
  display: grid;
  place-items: center;
}

.fl-graph-search-clear:hover {
  background: var(--color-bg-hover);
  color: var(--color-text-primary);
}

.fl-icon-btn {
  background: none;
  border: none;
  color: var(--color-text-secondary);
  cursor: pointer;
  padding: var(--sp-2);
  border-radius: var(--r-sm);
}

.fl-icon-btn:hover {
  color: var(--color-text-primary);
  background: var(--color-bg-hover);
}

.fl-graph-modal-analyze {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 6px 12px;
  border-radius: 999px;
  border: 1px solid var(--color-primary);
  background: var(--color-primary-soft);
  color: var(--color-primary-dark);
  font-size: var(--fs-12);
  font-weight: var(--fw-medium);
  cursor: pointer;
  transition: all 120ms ease;
  white-space: nowrap;
  flex-shrink: 0;
}

.fl-graph-modal-analyze:hover:not(:disabled) {
  background: var(--color-primary);
  color: var(--color-text-on-primary);
}

.fl-graph-modal-analyze.is-loading {
  cursor: progress;
  opacity: 0.85;
}

.fl-graph-modal-analyze.is-redo {
  border-color: var(--color-border);
  background: transparent;
  color: var(--color-text-secondary);
}

.fl-graph-modal-analyze.is-redo:hover:not(:disabled) {
  border-color: var(--color-primary);
  background: var(--color-primary-soft);
  color: var(--color-primary-dark);
}

.fl-graph-modal-analyze:disabled {
  cursor: not-allowed;
}

.fl-graph-modal-analyze-target {
  font-weight: var(--fw-regular);
  opacity: 0.7;
  max-width: 120px;
  overflow: hidden;
  text-overflow: ellipsis;
}

.is-spinning {
  animation: fl-spin 800ms linear infinite;
}

@keyframes fl-spin {
  to { transform: rotate(360deg); }
}

.fl-graph-modal-body {
  flex: 1;
  overflow: hidden;
  position: relative;
  background: var(--color-bg);
}

.fl-fade-enter-active,
.fl-fade-leave-active {
  transition: opacity 200ms ease;
}
.fl-fade-enter-from,
.fl-fade-leave-to {
  opacity: 0;
}
</style>
