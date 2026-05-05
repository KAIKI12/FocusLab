<script setup lang="ts">
/**
 * GraphInspirationNode · 自定义灵感节点
 *
 * 设计目标:
 * - 4 方向 handle (top/right/bottom/left),每边一个连接锚点
 * - handle 在节点不悬停时小而淡,悬停时放大变明显
 * - 关键 vue-flow 约定:
 *   1. 每个 handle 必须有唯一 id (因为我们有多个同类型 handle)
 *   2. 不能用 v-if/display:none 隐藏 handle,只能 opacity:0
 *   3. 既要 source 又要 target 才能双向拖拽连接 (Loose mode 也不例外)
 *
 * 仅 full 模式渲染连接锚点。
 */

import { Handle, Position, type NodeProps } from "@vue-flow/core";
import { computed } from "vue";

import type { InspirationItem } from "@/stores/useInspirationStore";

interface NodeData {
  label: string;
  item: InspirationItem;
  mode: "mini" | "full";
}

const props = defineProps<NodeProps<NodeData>>();

const isFull = computed(() => props.data.mode === "full");
</script>

<template>
  <div class="fl-graph-inode" :class="{ 'is-full': isFull }">
    <!-- 4 方向 × source/target 各一对 handle (共 8 个),每个唯一 id
         Loose mode 仍要求双类型存在,否则反向拖拽会失效。
         非 full 模式: opacity:0 (不能 display:none,会影响 edge 计算) -->
    <Handle id="t-top" type="target" :position="Position.Top" :class="['fl-graph-h fl-graph-h-top', { 'is-hidden': !isFull }]" />
    <Handle id="s-top" type="source" :position="Position.Top" :class="['fl-graph-h fl-graph-h-top', { 'is-hidden': !isFull }]" />
    <Handle id="t-right" type="target" :position="Position.Right" :class="['fl-graph-h fl-graph-h-right', { 'is-hidden': !isFull }]" />
    <Handle id="s-right" type="source" :position="Position.Right" :class="['fl-graph-h fl-graph-h-right', { 'is-hidden': !isFull }]" />
    <Handle id="t-bottom" type="target" :position="Position.Bottom" :class="['fl-graph-h fl-graph-h-bottom', { 'is-hidden': !isFull }]" />
    <Handle id="s-bottom" type="source" :position="Position.Bottom" :class="['fl-graph-h fl-graph-h-bottom', { 'is-hidden': !isFull }]" />
    <Handle id="t-left" type="target" :position="Position.Left" :class="['fl-graph-h fl-graph-h-left', { 'is-hidden': !isFull }]" />
    <Handle id="s-left" type="source" :position="Position.Left" :class="['fl-graph-h fl-graph-h-left', { 'is-hidden': !isFull }]" />

    <span class="fl-graph-inode-label">{{ data.label }}</span>
  </div>
</template>

<style scoped>
.fl-graph-inode {
  position: relative;
  display: grid;
  place-items: center;
  min-width: 80px;
  max-width: 180px;
  padding: 8px 12px;
  border-radius: 14px;
  background: var(--color-bg-elevated);
  border: 1.5px solid var(--color-border);
  font-size: 12px;
  line-height: 1.3;
  text-align: center;
  color: var(--color-text-primary);
  box-shadow: 0 4px 14px rgba(20, 28, 48, 0.06);
  word-break: break-word;
  cursor: grab;
  transition: border-color 120ms ease, box-shadow 120ms ease;
}

.fl-graph-inode.is-full:hover {
  border-color: var(--color-primary);
  box-shadow:
    0 4px 16px rgba(20, 28, 48, 0.1),
    0 0 0 3px color-mix(in srgb, var(--color-primary) 14%, transparent);
}

.fl-graph-inode-label {
  pointer-events: none;
}

/* Handle 默认小且半透明 — 不挡视觉
 * 悬停节点时变大变明显,提示用户可以拖
 * source 和 target 重叠在同一坐标 — 视觉上看起来是单点
 */
.fl-graph-inode :deep(.fl-graph-h) {
  width: 14px;
  height: 14px;
  background: var(--color-primary);
  border: 2px solid var(--color-bg-elevated);
  border-radius: 50%;
  opacity: 0.55;
  transition: opacity 120ms ease, transform 120ms ease, background 120ms ease;
  z-index: 2;
}

/* mini 模式: handle 不可见但保留 DOM (vue-flow edge 计算依赖 handle 真实位置) */
.fl-graph-inode :deep(.fl-graph-h.is-hidden) {
  opacity: 0;
  pointer-events: none;
}

/* hover 节点 → handle 放大变亮,更易瞄准 (仅 full 模式) */
.fl-graph-inode.is-full:hover :deep(.fl-graph-h:not(.is-hidden)) {
  opacity: 1;
  transform: scale(1.2);
}

/* hover 单个 handle 自身 → 进一步放大 */
.fl-graph-inode.is-full :deep(.fl-graph-h:not(.is-hidden):hover) {
  opacity: 1;
  transform: scale(1.6);
  background: var(--color-primary-dark);
}

/* 拖拽中的 handle 视觉 */
.fl-graph-inode :deep(.fl-graph-h.connectingfrom),
.fl-graph-inode :deep(.fl-graph-h.connectingto) {
  opacity: 1;
  transform: scale(1.5);
  background: var(--color-primary-dark);
}

/* 增大 hit area: ::before 透明扩展 8px 范围,让用户不必精确瞄 14px 圆点 */
.fl-graph-inode.is-full :deep(.fl-graph-h:not(.is-hidden)::before) {
  content: "";
  position: absolute;
  inset: -8px;
  border-radius: 50%;
}
</style>
