<script setup lang="ts">
/**
 * GraphRecoEdge · vue-flow 自定义边组件,用于绘制 AI 推荐的"建议线"
 * - 视觉: 虚线 + opacity 与 confidence 正相关
 * - 交互: hover 显示 popover (reason / confidence% / 接受 / 忽略)
 * - hover 离开有 200ms 缓冲, click 边可锁定 popover
 */

import { BaseEdge, EdgeLabelRenderer, Position, getBezierPath, useVueFlow } from "@vue-flow/core";
import { computed, ref } from "vue";

import type { InspirationRecommendation } from "@/types";

interface RecoEdgeData {
  kind: "reco";
  sourceId: string;
  recommendation: InspirationRecommendation;
}

const props = defineProps<{
  id: string;
  sourceX: number;
  sourceY: number;
  targetX: number;
  targetY: number;
  sourcePosition: Position;
  targetPosition: Position;
  data: RecoEdgeData;
  markerEnd?: string;
}>();

const emit = defineEmits<{
  accept: [sourceId: string, reco: InspirationRecommendation];
  reject: [sourceId: string, candidateId: string];
}>();

const { onEdgeMouseEnter, onEdgeMouseLeave } = useVueFlow();
const hovered = ref(false);
const pinned = ref(false);
const popHovered = ref(false);
let leaveTimer: ReturnType<typeof setTimeout> | null = null;

onEdgeMouseEnter(({ edge }) => {
  if (edge.id !== props.id) return;
  if (leaveTimer) {
    clearTimeout(leaveTimer);
    leaveTimer = null;
  }
  hovered.value = true;
});

onEdgeMouseLeave(({ edge }) => {
  if (edge.id !== props.id) return;
  // 200ms 延时,允许鼠标平滑移到 popover
  if (leaveTimer) clearTimeout(leaveTimer);
  leaveTimer = setTimeout(() => {
    hovered.value = false;
    leaveTimer = null;
  }, 200);
});

const showPop = computed(() => hovered.value || pinned.value || popHovered.value);

const pathData = computed(() =>
  getBezierPath({
    sourceX: props.sourceX,
    sourceY: props.sourceY,
    sourcePosition: props.sourcePosition,
    targetX: props.targetX,
    targetY: props.targetY,
    targetPosition: props.targetPosition,
    curvature: 0.25,
  }),
);

const reco = computed(() => props.data.recommendation);
const isContradicts = computed(() => reco.value.relation === "contradicts");

const opacity = computed(() => {
  const conf = Math.max(0, Math.min(1, reco.value.confidence ?? 0.7));
  return 0.35 + 0.5 * conf;
});

const stroke = computed(() =>
  isContradicts.value ? "var(--color-warning)" : "var(--color-primary)",
);

const strokeWidth = computed(() => (showPop.value ? 2.6 : 1.8));

const confidencePct = computed(() => Math.round((reco.value.confidence ?? 0) * 100));

function onAccept() {
  pinned.value = false;
  emit("accept", props.data.sourceId, reco.value);
}

function onReject() {
  pinned.value = false;
  emit("reject", props.data.sourceId, reco.value.candidateId);
}

function togglePin() {
  pinned.value = !pinned.value;
}

function onPopEnter() {
  if (leaveTimer) {
    clearTimeout(leaveTimer);
    leaveTimer = null;
  }
  popHovered.value = true;
}

function onPopLeave() {
  popHovered.value = false;
}
</script>

<template>
  <BaseEdge
    :id="id"
    :path="pathData[0]"
    :marker-end="markerEnd"
    :style="{
      stroke,
      strokeOpacity: opacity,
      strokeWidth,
      strokeDasharray: '6 4',
      cursor: 'pointer',
      transition: 'stroke-width 120ms ease',
    }"
    :interaction-width="20"
    @click="togglePin"
  />
  <EdgeLabelRenderer>
    <div
      v-if="showPop"
      class="fl-reco-pop nodrag nopan"
      :style="{
        transform: `translate(-50%, -50%) translate(${pathData[1]}px, ${pathData[2]}px)`,
      }"
      @pointerenter="onPopEnter"
      @pointerleave="onPopLeave"
    >
      <div class="fl-reco-pop-head">
        <span class="fl-reco-tag" :class="{ 'is-warn': isContradicts }">
          {{ isContradicts ? 'AI 矛盾建议' : 'AI 相关建议' }}
        </span>
        <span class="fl-reco-conf">{{ confidencePct }}%</span>
      </div>
      <div class="fl-reco-conf-bar">
        <div
          class="fl-reco-conf-fill"
          :class="{ 'is-warn': isContradicts }"
          :style="{ width: confidencePct + '%' }"
        />
      </div>
      <p class="fl-reco-reason">{{ reco.reason || '(无说明)' }}</p>
      <div class="fl-reco-actions">
        <button type="button" class="fl-reco-btn fl-reco-btn-primary" @click="onAccept">
          {{ isContradicts ? '建立修正连接' : '接受为相关' }}
        </button>
        <button type="button" class="fl-reco-btn fl-reco-btn-ghost" @click="onReject">
          忽略
        </button>
      </div>
    </div>
  </EdgeLabelRenderer>
</template>

<style scoped>
.fl-reco-pop {
  position: absolute;
  pointer-events: all;
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--r-md);
  box-shadow: var(--shadow-card, 0 8px 24px rgba(20, 28, 48, 0.12));
  padding: 10px 12px;
  width: 240px;
  font-size: var(--fs-12);
  z-index: 100;
}

.fl-reco-pop-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 6px;
}

.fl-reco-tag {
  display: inline-block;
  padding: 2px 6px;
  border-radius: 999px;
  background: color-mix(in srgb, var(--color-primary) 14%, transparent);
  color: var(--color-primary-dark);
  font-weight: var(--fw-semibold);
  font-size: 10px;
}

.fl-reco-tag.is-warn {
  background: color-mix(in srgb, var(--color-warning) 14%, transparent);
  color: var(--color-warning-text);
}

.fl-reco-conf {
  font-family: ui-monospace, "SF Mono", monospace;
  color: var(--color-text-secondary);
  font-size: 11px;
}

.fl-reco-conf-bar {
  width: 100%;
  height: 3px;
  background: var(--color-bg-hover);
  border-radius: 2px;
  margin-bottom: 6px;
  overflow: hidden;
}

.fl-reco-conf-fill {
  height: 100%;
  background: var(--color-primary);
  transition: width 200ms ease;
}

.fl-reco-conf-fill.is-warn {
  background: var(--color-warning);
}

.fl-reco-reason {
  margin: 0 0 8px 0;
  color: var(--color-text-primary);
  line-height: 1.5;
  word-break: break-word;
}

.fl-reco-actions {
  display: flex;
  gap: 6px;
}

.fl-reco-btn {
  flex: 1;
  padding: 5px 8px;
  border-radius: var(--r-sm);
  border: 1px solid transparent;
  cursor: pointer;
  font-size: 11px;
  font-weight: var(--fw-medium);
  transition: background 120ms ease;
}

.fl-reco-btn-primary {
  background: var(--color-primary);
  color: var(--color-text-on-primary);
}

.fl-reco-btn-primary:hover {
  background: var(--color-primary-dark);
}

.fl-reco-btn-ghost {
  background: transparent;
  color: var(--color-text-secondary);
  border-color: var(--color-border);
}

.fl-reco-btn-ghost:hover {
  background: var(--color-bg-hover);
}
</style>
