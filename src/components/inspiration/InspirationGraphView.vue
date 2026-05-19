<script setup lang="ts">
/**
 * InspirationGraphView · 灵感局部图谱 (mini 模式薄壳)
 *
 * 接口向后兼容:依然接收 item/links/allItems,新增可选 recommendations + 分析状态。
 * 实际渲染委托给 InspirationGraphCore (mode='mini')。
 * 用户点击右上角"放大"按钮时 emit('expand') ,由父组件打开全屏 Modal。
 *
 * 方案 A 之后: AI 推荐改为用户主动触发,头部加「✨ 分析关联 / 🔄 重新分析」按钮。
 * 父组件接 @analyze 事件后调 inspiration.loadRecommendations(item.id)。
 */

import { RefreshCw, Sparkles, X } from "lucide-vue-next";
import { computed } from "vue";

import type { InspirationItem } from "@/stores/useInspirationStore";
import type { InspirationLink, InspirationRecommendation } from "@/types";

import InspirationGraphCore from "./InspirationGraphCore.vue";

const props = defineProps<{
  /** 中心节点 */
  item: InspirationItem | null;
  /** 该中心节点的所有已确认 link */
  links: InspirationLink[];
  /** 完整 items 列表(用于查找邻居 label) */
  allItems?: InspirationItem[];
  /** 该中心节点的 AI 推荐(可选,缺省则不画建议线) */
  recommendations?: InspirationRecommendation[];
  /** 当前是否正在跑分析(loading) */
  analyzing?: boolean;
  /** 该中心节点是否已分析过(决定按钮文案: 分析关联 vs 重新分析) */
  analyzed?: boolean;
  /** 上次分析的错误信息(若有) */
  analyzeError?: string;
}>();

const emit = defineEmits<{
  expand: [];
  analyze: [];
  "accept-reco": [sourceId: string, reco: InspirationRecommendation];
  "reject-reco": [sourceId: string, candidateId: string];
  "node-click": [id: string];
  unlink: [sourceId: string, targetId: string];
}>();

const linksMap = computed<Record<string, InspirationLink[]>>(() => {
  if (!props.item) return {};
  return { [props.item.id]: props.links };
});

const recoMap = computed<Record<string, InspirationRecommendation[]>>(() => {
  if (!props.item) return {};
  return { [props.item.id]: props.recommendations ?? [] };
});

const itemsList = computed<InspirationItem[]>(
  () => props.allItems ?? (props.item ? [props.item] : []),
);

const analyzeBtnLabel = computed(() => {
  if (props.analyzing) return "分析中…";
  return props.analyzed ? "重新分析" : "分析关联";
});

function relationLabel(link: InspirationLink) {
  return link.relation === "contradicts" ? "矛盾/纠偏" : "相关";
}

function sourceLabel(link: InspirationLink) {
  return link.sourceType === "ai_accepted" ? "AI 确认" : "手动连接";
}

function linkReason(link: InspirationLink) {
  const raw = link.reason?.trim();
  if (raw) return raw;
  return link.sourceType === "ai_accepted" ? "已接受 AI 推荐建立该连接" : "手动建立的连接";
}

function peerIdOf(link: InspirationLink) {
  if (!props.item) return link.targetId;
  return link.sourceId === props.item.id ? link.targetId : link.sourceId;
}

function contentForId(id: string) {
  const item = props.allItems?.find((entry) => entry.id === id);
  if (!item) return "(已删除)";
  return item.content.trim() || "图片灵感";
}

const linkRows = computed(() =>
  props.links.map((link) => {
    const peerId = peerIdOf(link);
    return {
      link,
      peerId,
      peerContent: contentForId(peerId),
    };
  }),
);

function onUnlink(peerId: string) {
  if (!props.item) return;
  emit("unlink", props.item.id, peerId);
}
</script>

<template>
  <section class="fl-graph">
    <header class="fl-graph-head">
      <div class="fl-graph-head-left">
        <h3>局部图谱视图</h3>
        <span v-if="item" class="fl-graph-meta">
          {{ links.length }} 条关系
          <template v-if="recommendations && recommendations.length">
            · {{ recommendations.length }} AI 建议
          </template>
        </span>
      </div>

      <button
        v-if="item"
        type="button"
        class="fl-graph-analyze-btn"
        :class="{ 'is-loading': analyzing, 'is-redo': analyzed && !analyzing }"
        :disabled="analyzing"
        :title="analyzed ? '重新跑一遍 AI 关联分析(会消耗 API 配额)' : '让 AI 找出与当前灵感相关的其它灵感'"
        @click="emit('analyze')"
      >
        <RefreshCw v-if="analyzing || analyzed" :size="13" :class="{ 'is-spinning': analyzing }" />
        <Sparkles v-else :size="13" />
        <span>{{ analyzeBtnLabel }}</span>
      </button>
    </header>

    <p v-if="analyzeError && !analyzing" class="fl-graph-err">
      ⚠ 分析失败: {{ analyzeError }}
    </p>

    <InspirationGraphCore
      mode="mini"
      :items="itemsList"
      :links="linksMap"
      :recommendations="recoMap"
      :focused-id="item?.id ?? null"
      @accept-reco="(sourceId, reco) => emit('accept-reco', sourceId, reco)"
      @reject-reco="(sourceId, candidateId) => emit('reject-reco', sourceId, candidateId)"
      @node-click="(id) => emit('node-click', id)"
      @delete-link="(sourceId, targetId) => emit('unlink', sourceId, targetId)"
      @expand="emit('expand')"
    />

    <div v-if="item && linkRows.length" class="fl-graph-links">
      <div class="fl-graph-links-head">
        <span>已连接</span>
        <span>{{ linkRows.length }}</span>
      </div>
      <div class="fl-graph-link-list">
        <div
          v-for="row in linkRows"
          :key="row.link.id"
          class="fl-graph-link-row"
          :class="{ 'is-warn': row.link.relation === 'contradicts' }"
        >
          <span class="fl-graph-link-dot" />
          <div class="fl-graph-link-body">
            <span class="fl-graph-link-text">{{ row.peerContent }}</span>
            <div class="fl-graph-link-meta">
              <span class="fl-graph-link-chip" :class="{ 'is-warn': row.link.relation === 'contradicts' }">
                {{ relationLabel(row.link) }}
              </span>
              <span class="fl-graph-link-chip is-source">
                {{ sourceLabel(row.link) }}
              </span>
            </div>
            <p class="fl-graph-link-reason">{{ linkReason(row.link) }}</p>
          </div>
          <button
            class="fl-graph-link-remove"
            type="button"
            title="删除连接"
            aria-label="删除连接"
            @click="onUnlink(row.peerId)"
          >
            <X :size="12" />
          </button>
        </div>
      </div>
    </div>

    <p class="fl-graph-tip">
      鼠标悬停虚线可查看 AI 建议详情;点击右上 ⤢ 放大查看全图。
    </p>
  </section>
</template>

<style scoped>
.fl-graph {
  border: 1px solid var(--color-border);
  border-radius: 24px;
  background: var(--color-bg-elevated);
  box-shadow: var(--shadow-card);
  padding: var(--sp-4);
  overflow: hidden;
}

.fl-graph-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--sp-2);
  margin-bottom: var(--sp-3);
  flex-wrap: wrap;
}

.fl-graph-head-left {
  display: flex;
  align-items: baseline;
  gap: var(--sp-2);
  min-width: 0;
}

.fl-graph-head h3 {
  margin: 0;
  font-size: var(--fs-13, 13px);
  font-weight: 700;
}

.fl-graph-meta {
  font-size: 11px;
  color: var(--color-text-muted);
}

.fl-graph-analyze-btn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px 10px;
  border-radius: 999px;
  border: 1px solid var(--color-primary);
  background: var(--color-primary-soft);
  color: var(--color-primary-dark);
  font-size: 11px;
  font-weight: var(--fw-medium);
  cursor: pointer;
  transition: all 120ms ease;
  white-space: nowrap;
}

.fl-graph-analyze-btn:hover:not(:disabled) {
  background: var(--color-primary);
  color: var(--color-text-on-primary);
}

.fl-graph-analyze-btn.is-loading {
  cursor: progress;
  opacity: 0.8;
}

.fl-graph-analyze-btn.is-redo {
  border-color: var(--color-border);
  background: transparent;
  color: var(--color-text-secondary);
}

.fl-graph-analyze-btn.is-redo:hover:not(:disabled) {
  border-color: var(--color-primary);
  background: var(--color-primary-soft);
  color: var(--color-primary-dark);
}

.fl-graph-analyze-btn:disabled {
  cursor: not-allowed;
}

.is-spinning {
  animation: fl-spin 800ms linear infinite;
}

@keyframes fl-spin {
  to { transform: rotate(360deg); }
}

.fl-graph-err {
  margin: 0 0 var(--sp-2) 0;
  padding: 6px 10px;
  border-radius: var(--r-sm, 8px);
  background: color-mix(in srgb, var(--color-warning) 12%, transparent);
  color: var(--color-warning-text);
  font-size: 11px;
  border: 1px solid color-mix(in srgb, var(--color-warning) 24%, transparent);
}

.fl-graph-links {
  margin-top: var(--sp-3);
  border-top: 1px solid var(--color-divider);
  padding-top: var(--sp-3);
}

.fl-graph-links-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  color: var(--color-text-secondary);
  font-size: 11px;
  font-weight: var(--fw-semibold);
  margin-bottom: var(--sp-2);
}

.fl-graph-link-list {
  display: flex;
  flex-direction: column;
  gap: var(--sp-2);
}

.fl-graph-link-row {
  display: grid;
  grid-template-columns: 8px minmax(0, 1fr) 24px;
  align-items: start;
  gap: var(--sp-2);
  padding: 8px 6px;
  border-radius: var(--r-sm);
  background: var(--color-bg-subtle);
}

.fl-graph-link-dot {
  width: 7px;
  height: 7px;
  border-radius: 999px;
  background: var(--color-primary);
}

.fl-graph-link-row.is-warn .fl-graph-link-dot {
  background: var(--color-warning);
}

.fl-graph-link-body {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 5px;
}

.fl-graph-link-text {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--color-text-primary);
  font-size: 11px;
  font-weight: var(--fw-medium);
}

.fl-graph-link-meta {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
}

.fl-graph-link-chip {
  display: inline-flex;
  align-items: center;
  padding: 2px 6px;
  border-radius: 999px;
  background: color-mix(in srgb, var(--color-primary) 12%, transparent);
  color: var(--color-primary-dark);
  font-size: 10px;
  font-weight: var(--fw-medium);
}

.fl-graph-link-chip.is-warn {
  background: color-mix(in srgb, var(--color-warning) 14%, transparent);
  color: var(--color-warning-text);
}

.fl-graph-link-chip.is-source {
  background: color-mix(in srgb, var(--color-text-secondary) 10%, transparent);
  color: var(--color-text-secondary);
}

.fl-graph-link-reason {
  margin: 0;
  color: var(--color-text-secondary);
  font-size: 11px;
  line-height: 1.45;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.fl-graph-link-remove {
  width: 24px;
  height: 24px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  margin-top: 1px;
  border: 1px solid transparent;
  border-radius: var(--r-sm);
  background: transparent;
  color: var(--color-text-muted);
  cursor: pointer;
}

.fl-graph-link-remove:hover {
  color: var(--color-danger);
  background: color-mix(in srgb, var(--color-danger) 10%, transparent);
}

.fl-graph-tip {
  margin: var(--sp-3) 0 0 0;
  font-size: 11px;
  color: var(--color-text-secondary);
  line-height: 1.5;
}
</style>
