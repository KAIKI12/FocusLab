<script setup lang="ts">
/**
 * GraphLinkEdge · 已确认灵感连接的交互边。
 *
 * 保持实线视觉,并在 hover / click 时提供删除入口。
 */

import { BaseEdge, EdgeLabelRenderer, Position, getBezierPath, useVueFlow } from "@vue-flow/core";
import { Trash2 } from "lucide-vue-next";
import { computed, ref } from "vue";

import type { InspirationLink } from "@/types";

interface LinkEdgeData {
  kind: "link";
  link: InspirationLink;
}

const props = defineProps<{
  id: string;
  sourceX: number;
  sourceY: number;
  targetX: number;
  targetY: number;
  sourcePosition: Position;
  targetPosition: Position;
  data: LinkEdgeData;
  markerEnd?: string;
}>();

const emit = defineEmits<{
  delete: [sourceId: string, targetId: string];
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
  if (leaveTimer) clearTimeout(leaveTimer);
  leaveTimer = setTimeout(() => {
    hovered.value = false;
    leaveTimer = null;
  }, 180);
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

const link = computed(() => props.data.link);
const isContradicts = computed(() => link.value.relation === "contradicts");
const stroke = computed(() =>
  isContradicts.value ? "var(--color-warning)" : "var(--color-primary)",
);
const label = computed(() => (isContradicts.value ? "矛盾/纠偏" : "相关"));
const sourceLabel = computed(() => (link.value.sourceType === "ai_accepted" ? "AI 确认" : "手动连接"));
const linkReason = computed(() => {
  const raw = link.value.reason?.trim();
  if (raw) return raw;
  return link.value.sourceType === "ai_accepted" ? "已接受 AI 推荐建立该连接" : "手动建立的连接";
});
const strokeWidth = computed(() => (showPop.value ? 2.8 : 2));

function togglePin() {
  pinned.value = !pinned.value;
}

function onDelete() {
  pinned.value = false;
  emit("delete", link.value.sourceId, link.value.targetId);
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
      strokeWidth,
      strokeOpacity: showPop ? 0.95 : 0.85,
      cursor: 'pointer',
      transition: 'stroke-width 120ms ease, stroke-opacity 120ms ease',
    }"
    :interaction-width="18"
    @click="togglePin"
  />
  <EdgeLabelRenderer>
    <div
      v-if="showPop"
      class="fl-link-edge-pop nodrag nopan"
      :style="{
        transform: `translate(-50%, -50%) translate(${pathData[1]}px, ${pathData[2]}px)`,
      }"
      @pointerenter="onPopEnter"
      @pointerleave="onPopLeave"
    >
      <div class="fl-link-edge-head">
        <span class="fl-link-edge-tag" :class="{ 'is-warn': isContradicts }">
          {{ label }}
        </span>
        <span class="fl-link-edge-source">{{ sourceLabel }}</span>
      </div>
      <p class="fl-link-edge-reason">{{ linkReason }}</p>
      <button type="button" class="fl-link-edge-delete" @click="onDelete">
        <Trash2 :size="12" />
        <span>删除连接</span>
      </button>
    </div>
  </EdgeLabelRenderer>
</template>

<style scoped>
.fl-link-edge-pop {
  position: absolute;
  pointer-events: all;
  width: 178px;
  padding: 9px 10px;
  border: 1px solid var(--color-border);
  border-radius: var(--r-md);
  background: var(--color-bg-elevated);
  box-shadow: var(--shadow-card, 0 8px 24px rgba(20, 28, 48, 0.12));
  z-index: 100;
}

.fl-link-edge-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--sp-2);
  margin-bottom: 8px;
}

.fl-link-edge-tag {
  display: inline-flex;
  align-items: center;
  padding: 2px 6px;
  border-radius: 999px;
  background: color-mix(in srgb, var(--color-primary) 14%, transparent);
  color: var(--color-primary-dark);
  font-size: 10px;
  font-weight: var(--fw-semibold);
}

.fl-link-edge-tag.is-warn {
  background: color-mix(in srgb, var(--color-warning) 14%, transparent);
  color: var(--color-warning-text);
}

.fl-link-edge-source {
  color: var(--color-text-muted);
  font-size: 10px;
}

.fl-link-edge-reason {
  margin: 0 0 8px;
  color: var(--color-text-secondary);
  font-size: 11px;
  line-height: 1.45;
  word-break: break-word;
}

.fl-link-edge-delete {
  width: 100%;
  height: 28px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 5px;
  border: 1px solid color-mix(in srgb, var(--color-danger) 26%, transparent);
  border-radius: var(--r-sm);
  background: color-mix(in srgb, var(--color-danger) 8%, transparent);
  color: var(--color-danger);
  cursor: pointer;
  font-size: 11px;
  font-weight: var(--fw-medium);
}

.fl-link-edge-delete:hover {
  background: color-mix(in srgb, var(--color-danger) 14%, transparent);
}
</style>
