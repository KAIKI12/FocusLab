<script setup lang="ts">
/**
 * InspirationGraphCore · 灵感关联图谱核心渲染
 *
 * 同一个组件服务两个场景:
 * - mini (右侧 240px 小图): 中心节点 + 一度邻居,最多 30 节点
 * - full (Modal 全屏): 全部 items, 可拖拽/缩放/搜索高亮
 *
 * AI 推荐边走自定义 edge type 'reco' (GraphRecoEdge.vue) - 虚线 + popover
 * 已确认 link 走默认 edge - 实线 + 颜色区分 manual/ai_accepted/contradicts
 */

import { Background } from "@vue-flow/background";
import { Controls } from "@vue-flow/controls";
import {
  ConnectionMode,
  VueFlow,
  useVueFlow,
  type Connection,
  type Edge,
  type Node,
} from "@vue-flow/core";
import { MiniMap } from "@vue-flow/minimap";
import { computed, onMounted, onUnmounted, ref, watch } from "vue";

import { useGraphForceLayout } from "@/composables/useGraphForceLayout";
import type { InspirationItem } from "@/stores/useInspirationStore";
import type { InspirationLink, InspirationRecommendation } from "@/types";

import GraphInspirationNode from "./GraphInspirationNode.vue";
import GraphRecoEdge from "./GraphRecoEdge.vue";

const props = defineProps<{
  items: InspirationItem[];
  links: Record<string, InspirationLink[]>;
  recommendations: Record<string, InspirationRecommendation[]>;
  mode?: "mini" | "full";
  /** 中心节点 ID,mini 模式必填;full 模式可选(用于高亮) */
  focusedId?: string | null;
  /** 全屏模式下的搜索查询 */
  searchQuery?: string;
}>();

const emit = defineEmits<{
  "accept-reco": [sourceId: string, reco: InspirationRecommendation];
  "reject-reco": [sourceId: string, candidateId: string];
  "node-click": [id: string];
  "create-link": [sourceId: string, targetId: string, relation: "related" | "contradicts"];
  expand: [];
}>();

const mode = props.mode ?? "mini";
const isFull = mode === "full";
const flowId = `fl-graph-${mode}`;

const stageEl = ref<HTMLElement | null>(null);
const stageWidth = ref(isFull ? 1000 : 320);
const stageHeight = ref(isFull ? 700 : 240);

// ---------- 数据派生 ----------

/** mini = 中心 + 一度邻居 (上限 30); full = 全部 items */
const visibleItems = computed<InspirationItem[]>(() => {
  if (isFull) return props.items;
  const focusId = props.focusedId;
  if (!focusId) return [];
  const center = props.items.find((i) => i.id === focusId);
  if (!center) return [];
  const neighborIds = new Set<string>([focusId]);
  for (const link of props.links[focusId] ?? []) {
    neighborIds.add(link.sourceId === focusId ? link.targetId : link.sourceId);
  }
  for (const reco of props.recommendations[focusId] ?? []) {
    neighborIds.add(reco.candidateId);
  }
  const seen = new Set<string>([focusId]);
  const result: InspirationItem[] = [center];
  for (const item of props.items) {
    if (seen.has(item.id)) continue;
    if (neighborIds.has(item.id)) {
      result.push(item);
      seen.add(item.id);
      if (result.length >= 30) break;
    }
  }
  return result;
});

const itemMap = computed(() => new Map(visibleItems.value.map((i) => [i.id, i])));

/** 已确认 link, 按 link.id 去重, 过滤孤儿 */
const allLinks = computed<InspirationLink[]>(() => {
  const seen = new Set<string>();
  const result: InspirationLink[] = [];
  for (const item of visibleItems.value) {
    for (const link of props.links[item.id] ?? []) {
      if (!itemMap.value.has(link.sourceId) || !itemMap.value.has(link.targetId)) continue;
      if (seen.has(link.id)) continue;
      seen.add(link.id);
      result.push(link);
    }
  }
  return result;
});

interface RecoEdgeRaw {
  sourceId: string;
  reco: InspirationRecommendation;
}

/** AI 推荐边 (排除已有手动 link 的对) */
const recoEdges = computed<RecoEdgeRaw[]>(() => {
  const result: RecoEdgeRaw[] = [];
  for (const [sourceId, recos] of Object.entries(props.recommendations)) {
    if (!itemMap.value.has(sourceId)) continue;
    for (const reco of recos) {
      if (!itemMap.value.has(reco.candidateId)) continue;
      const dup = (props.links[sourceId] ?? []).some(
        (l) =>
          (l.sourceId === sourceId && l.targetId === reco.candidateId) ||
          (l.sourceId === reco.candidateId && l.targetId === sourceId),
      );
      if (dup) continue;
      result.push({ sourceId, reco });
    }
  }
  return result;
});

// ---------- 力布局 ----------

const { positions, pin, unpin, resize } = useGraphForceLayout(
  () => visibleItems.value.map((i) => ({ id: i.id })),
  () => {
    const out: { source: string; target: string }[] = [];
    for (const link of allLinks.value) {
      out.push({ source: link.sourceId, target: link.targetId });
    }
    for (const re of recoEdges.value) {
      out.push({ source: re.sourceId, target: re.reco.candidateId });
    }
    return out;
  },
  {
    width: stageWidth.value,
    height: stageHeight.value,
    linkDistance: isFull ? 130 : 80,
    charge: isFull ? -300 : -180,
    collideRadius: isFull ? 40 : 28,
  },
);

// ---------- vue-flow nodes/edges ----------

function truncate(text: string, max: number) {
  return text.length > max ? text.slice(0, max) + "…" : text;
}

function matchesSearch(item: InspirationItem, query: string): boolean {
  const q = query.trim().toLowerCase();
  if (!q) return false;
  if (item.content.toLowerCase().includes(q)) return true;
  if (item.summary && item.summary.toLowerCase().includes(q)) return true;
  return item.keywords.some((k) => k.toLowerCase().includes(q));
}

const flowNodes = computed<Node[]>(() => {
  const labelMax = isFull ? 18 : 12;
  const q = props.searchQuery ?? "";
  const hasQuery = q.trim().length > 0;
  return visibleItems.value.map((item): Node => {
    const pos = positions.value.get(item.id) ?? {
      x: stageWidth.value / 2,
      y: stageHeight.value / 2,
    };
    const isCenter = !isFull && props.focusedId === item.id;
    const hasContradicts = (props.links[item.id] ?? []).some(
      (l) => l.relation === "contradicts",
    );
    const matched = hasQuery && matchesSearch(item, q);
    const dimmed = hasQuery && !matched;
    const classes = [
      "fl-graph-node",
      isCenter ? "is-center" : "",
      hasContradicts ? "is-contradicts" : "",
      matched ? "is-match" : "",
      dimmed ? "is-dim" : "",
    ]
      .filter(Boolean)
      .join(" ");
    return {
      id: item.id,
      type: "inspiration",
      position: { x: pos.x, y: pos.y },
      data: {
        label: truncate(item.summary || item.content, labelMax),
        item,
        mode,
      },
      class: classes,
      draggable: isFull,
      selectable: true,
      connectable: isFull,
    };
  });
});

const flowEdges = computed<Edge[]>(() => {
  const result: Edge[] = [];
  for (const link of allLinks.value) {
    const isContradicts = link.relation === "contradicts";
    result.push({
      id: `link:${link.id}`,
      source: link.sourceId,
      target: link.targetId,
      type: "default",
      animated: false,
      style: {
        stroke: isContradicts ? "var(--color-warning)" : "var(--color-primary)",
        strokeWidth: 2,
        strokeOpacity: 0.85,
      },
      data: { kind: "link", link },
    });
  }
  for (const re of recoEdges.value) {
    result.push({
      id: `reco:${re.sourceId}:${re.reco.candidateId}`,
      source: re.sourceId,
      target: re.reco.candidateId,
      type: "reco",
      data: {
        kind: "reco",
        sourceId: re.sourceId,
        recommendation: re.reco,
      },
    });
  }
  return result;
});

// ---------- vue-flow API ----------

const { fitView, onConnect, onConnectStart, onConnectEnd } = useVueFlow(flowId);

// 全屏模式下:用户拖拽 handle 连接两节点 → 触发 onConnect (默认路径,落在 handle 上)
// 默认 relation=related;同节点自连 / 重复连线 由 store linkManually 内部去重
onConnect((params: Connection) => {
  if (!isFull) return;
  const source = params.source;
  const target = params.target;
  if (!source || !target || source === target) return;
  emit("create-link", source, target, "related");
});

// Excalidraw 风格:用户从 handle 拖出后,即使没有精准落到目标 handle 上,
// 只要落在另一个节点的"身体"任意位置 → 也连接成功。
// 步骤:onConnectStart 记 source,onConnectEnd 用 elementFromPoint 找节点 div,
// 通过 .vue-flow__node 的 data-id 解析目标节点 id。
let pendingSourceId: string | null = null;

onConnectStart((event) => {
  if (!isFull) return;
  pendingSourceId = event.nodeId ?? null;
});

onConnectEnd((event) => {
  if (!isFull) {
    pendingSourceId = null;
    return;
  }
  const sourceId = pendingSourceId;
  pendingSourceId = null;
  if (!sourceId) return;

  // 如果 onConnect 已经处理过(精准落到 handle 上),vue-flow 会先触发 onConnect
  // 此时 onConnectEnd 后置走的逻辑做"是否需要兜底"判断:
  // 用 elementFromPoint 找鼠标下方元素,向上爬到最近 .vue-flow__node
  // 如果找到的目标节点 id 与 sourceId 不同,且没有刚刚的 onConnect 已建立 → 兜底建立
  if (!event) return;
  const point =
    event instanceof MouseEvent
      ? { x: event.clientX, y: event.clientY }
      : event.changedTouches?.[0]
        ? { x: event.changedTouches[0].clientX, y: event.changedTouches[0].clientY }
        : null;
  if (!point) return;

  const elem = document.elementFromPoint(point.x, point.y);
  if (!elem) return;
  const nodeEl = (elem as HTMLElement).closest(".vue-flow__node") as HTMLElement | null;
  if (!nodeEl) return;
  const targetId = nodeEl.getAttribute("data-id");
  if (!targetId || targetId === sourceId) return;

  // 兜底连接(若 onConnect 已 emit 过同样的连接,store.linkManually 内部 DUPLICATE_LINK 会被吞)
  emit("create-link", sourceId, targetId, "related");
});

// 节点集合变化后,fitView (双 rAF 等位置稳定)
watch(
  () => visibleItems.value.map((i) => i.id).join("|"),
  async () => {
    await new Promise((r) => requestAnimationFrame(r));
    await new Promise((r) => requestAnimationFrame(r));
    try {
      fitView({ padding: 0.2, duration: 200 });
    } catch {
      // fitView 在节点为空时可能抛异常,吞掉
    }
  },
);

// ---------- ResizeObserver (仅 full 模式) ----------

let ro: ResizeObserver | null = null;
let resizeTimer: ReturnType<typeof setTimeout> | null = null;

onMounted(() => {
  if (!stageEl.value) return;
  // 立刻同步真实容器尺寸 — 否则力布局拿到 props 默认值,节点位置错位
  const rect = stageEl.value.getBoundingClientRect();
  if (rect.width > 0 && rect.height > 0) {
    stageWidth.value = rect.width;
    stageHeight.value = rect.height;
    resize(rect.width, rect.height);
  }
  if (!isFull) return;
  ro = new ResizeObserver((entries) => {
    if (resizeTimer) clearTimeout(resizeTimer);
    resizeTimer = setTimeout(() => {
      const entry = entries[0];
      if (!entry) return;
      const { width, height } = entry.contentRect;
      if (width <= 0 || height <= 0) return;
      stageWidth.value = width;
      stageHeight.value = height;
      resize(width, height);
      try {
        fitView({ padding: 0.2, duration: 200 });
      } catch {
        // ignore
      }
    }, 120);
  });
  ro.observe(stageEl.value);
});

onUnmounted(() => {
  if (ro) ro.disconnect();
  if (resizeTimer) clearTimeout(resizeTimer);
});

// ---------- 节点交互 ----------

function onNodeClick(payload: { node: Node }) {
  emit("node-click", payload.node.id);
}

function onAcceptReco(sourceId: string, reco: InspirationRecommendation) {
  emit("accept-reco", sourceId, reco);
}

function onRejectReco(sourceId: string, candidateId: string) {
  emit("reject-reco", sourceId, candidateId);
}

function onNodeDragStart(payload: { node: Node }) {
  pin(payload.node.id, payload.node.position.x, payload.node.position.y);
}

function onNodeDrag(payload: { node: Node }) {
  pin(payload.node.id, payload.node.position.x, payload.node.position.y);
}

function onNodeDragStop(payload: { node: Node }) {
  unpin(payload.node.id);
}
</script>

<template>
  <div ref="stageEl" class="fl-graph-scope" :class="`fl-graph-scope-${mode}`">
    <VueFlow
      :id="flowId"
      :nodes="flowNodes"
      :edges="flowEdges"
      :nodes-draggable="isFull"
      :nodes-connectable="isFull"
      :connection-mode="ConnectionMode.Loose"
      :elements-selectable="true"
      :pan-on-drag="isFull"
      :zoom-on-scroll="isFull"
      :zoom-on-pinch="isFull"
      :pan-on-scroll="false"
      :prevent-scrolling="isFull"
      :fit-view-on-init="true"
      :min-zoom="0.2"
      :max-zoom="2"
      @node-click="onNodeClick"
      @node-drag-start="onNodeDragStart"
      @node-drag="onNodeDrag"
      @node-drag-stop="onNodeDragStop"
    >
      <template #edge-reco="recoProps">
        <GraphRecoEdge
          v-bind="recoProps"
          @accept="onAcceptReco"
          @reject="onRejectReco"
        />
      </template>
      <template #node-inspiration="nodeProps">
        <GraphInspirationNode v-bind="nodeProps" />
      </template>
      <Background v-if="isFull" pattern-color="#e8eaf0" :gap="24" :size="1" />
      <Controls v-if="isFull" :show-interactive="false" position="bottom-right" />
      <MiniMap v-if="isFull" pannable zoomable position="bottom-left" />
    </VueFlow>

    <button
      v-if="!isFull && visibleItems.length > 0"
      class="fl-graph-expand-btn"
      type="button"
      title="放大查看完整图谱"
      @click="emit('expand')"
    >
      <span aria-hidden="true">⤢</span>
    </button>

    <div v-if="visibleItems.length === 0" class="fl-graph-empty">
      <p v-if="isFull">还没有灵感,先在主面板上记一条吧。</p>
      <p v-else>选择一条灵感卡片查看其图谱。</p>
    </div>

    <div
      v-if="visibleItems.length > 0"
      class="fl-graph-legend"
      :class="{ 'is-mini': !isFull }"
    >
      <span class="fl-graph-legend-item">
        <span class="fl-graph-legend-line solid" /> 已确认
      </span>
      <span class="fl-graph-legend-item">
        <span class="fl-graph-legend-line dashed" /> AI 建议
      </span>
      <span class="fl-graph-legend-item">
        <span class="fl-graph-legend-line warn" /> 矛盾
      </span>
    </div>
  </div>
</template>

<style scoped>
.fl-graph-scope {
  position: relative;
  width: 100%;
  height: 100%;
  background: radial-gradient(
    circle at 50% 46%,
    color-mix(in srgb, var(--color-primary) 8%, transparent),
    transparent 55%
  );
  border-radius: var(--r-sm, 8px);
  overflow: hidden;
}

.fl-graph-scope-mini {
  height: 240px;
}

.fl-graph-scope-full {
  height: 100%;
  border-radius: 0;
}

/* 节点样式 - 穿透 scoped 覆盖 vue-flow 默认 */
.fl-graph-scope :deep(.vue-flow__node) {
  font-size: 11px;
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: 14px;
  padding: 6px 10px;
  min-width: 70px;
  max-width: 180px;
  text-align: center;
  line-height: 1.3;
  color: var(--color-text-primary);
  box-shadow: 0 4px 14px rgba(20, 28, 48, 0.06);
  transition:
    opacity 200ms ease,
    filter 200ms ease,
    box-shadow 200ms ease,
    border-color 200ms ease;
  word-break: break-word;
}

.fl-graph-scope-full :deep(.vue-flow__node) {
  font-size: 12px;
  padding: 8px 12px;
  min-width: 90px;
}

.fl-graph-scope :deep(.vue-flow__node.is-center) {
  background: var(--color-primary-soft);
  border-color: color-mix(in srgb, var(--color-primary) 40%, transparent);
  color: var(--color-primary-dark);
  font-weight: var(--fw-semibold);
}

.fl-graph-scope :deep(.vue-flow__node.is-contradicts) {
  border-color: color-mix(in srgb, var(--color-warning) 40%, transparent);
  color: var(--color-warning-text);
}

.fl-graph-scope :deep(.vue-flow__node.is-dim) {
  opacity: 0.25;
  filter: grayscale(0.6);
}

.fl-graph-scope :deep(.vue-flow__node.is-match) {
  border-color: var(--color-primary);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--color-primary) 24%, transparent);
}

.fl-graph-scope :deep(.vue-flow__node.selected) {
  border-color: var(--color-primary);
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--color-primary) 18%, transparent);
}

.fl-graph-scope :deep(.vue-flow__handle) {
  display: none;
}

/* full 模式下:节点四周显示连接锚点(浅蓝小点),hover 节点时强化显示 */
.fl-graph-scope-full :deep(.vue-flow__handle) {
  display: block;
  width: 8px;
  height: 8px;
  background: var(--color-primary);
  border: 2px solid var(--color-bg-elevated);
  opacity: 0.35;
  transition: opacity 120ms ease, transform 120ms ease;
}

.fl-graph-scope-full :deep(.vue-flow__node:hover .vue-flow__handle) {
  opacity: 1;
  transform: scale(1.15);
}

.fl-graph-scope-full :deep(.vue-flow__handle.connectingfrom),
.fl-graph-scope-full :deep(.vue-flow__handle.connectingto) {
  opacity: 1;
  background: var(--color-primary-dark);
}

.fl-graph-scope-full :deep(.vue-flow__connection-path) {
  stroke: var(--color-primary);
  stroke-width: 2;
  stroke-dasharray: 4 3;
  opacity: 0.7;
}

.fl-graph-scope :deep(.vue-flow__edge-path) {
  transition: stroke-width 120ms ease;
}

.fl-graph-scope :deep(.vue-flow__attribution) {
  display: none;
}

.fl-graph-expand-btn {
  position: absolute;
  top: 8px;
  right: 8px;
  z-index: 10;
  width: 28px;
  height: 28px;
  border-radius: var(--r-sm, 8px);
  border: 1px solid var(--color-border);
  background: var(--color-bg-elevated);
  color: var(--color-text-secondary);
  cursor: pointer;
  display: grid;
  place-items: center;
  font-size: 14px;
  line-height: 1;
  transition: all 120ms ease;
  box-shadow: 0 2px 6px rgba(20, 28, 48, 0.06);
}

.fl-graph-expand-btn:hover {
  background: var(--color-primary-soft);
  color: var(--color-primary-dark);
  border-color: color-mix(in srgb, var(--color-primary) 40%, transparent);
}

.fl-graph-empty {
  position: absolute;
  inset: 0;
  display: grid;
  place-items: center;
  color: var(--color-text-muted);
  font-size: 12px;
  pointer-events: none;
  text-align: center;
  padding: 16px;
}

.fl-graph-legend {
  position: absolute;
  bottom: 8px;
  left: 8px;
  display: flex;
  gap: 10px;
  align-items: center;
  background: color-mix(in srgb, var(--color-bg-elevated) 92%, transparent);
  border-radius: 999px;
  padding: 4px 10px;
  font-size: 10px;
  color: var(--color-text-secondary);
  border: 1px solid var(--color-border);
  z-index: 5;
  pointer-events: none;
  white-space: nowrap;
}

.fl-graph-legend.is-mini {
  font-size: 9px;
  gap: 6px;
  padding: 3px 8px;
}

.fl-graph-legend-item {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.fl-graph-legend-line {
  display: inline-block;
  width: 14px;
  height: 0;
  border-top: 2px solid var(--color-primary);
  border-radius: 1px;
}

.fl-graph-legend-line.dashed {
  border-top-style: dashed;
}

.fl-graph-legend-line.warn {
  border-top-color: var(--color-warning);
}
</style>
