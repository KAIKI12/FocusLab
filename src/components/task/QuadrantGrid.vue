<script setup lang="ts">
/**
 * QuadrantGrid · 2×2 四象限分组任务视图 — 支持跨象限拖拽。
 *
 * 拖拽实现:HTML5 Drag & Drop API
 *   dragstart → 记录 taskId + 原象限
 *   dragover/dragenter → 目标象限高亮
 *   drop → emit changeQuadrant(taskId, newQuadrant)
 */

import { Play } from "lucide-vue-next";
import { ref } from "vue";

import type { Task } from "@/types";

defineProps<{
  tasksByQuadrant: Record<string, Task[]>;
  timerIdle: boolean;
}>();

const emit = defineEmits<{
  edit: [task: Task];
  start: [taskId: string];
  changeQuadrant: [taskId: string, quadrant: string];
}>();

const quadrantMeta: Record<string, { label: string; cls: string }> = {
  important_urgent: { label: "紧急重要", cls: "q1" },
  important_not_urgent: { label: "重要不紧急", cls: "q2" },
  not_important_urgent: { label: "紧急不重要", cls: "q3" },
  not_important_not_urgent: { label: "不紧急不重要", cls: "q4" },
};

const quadrantOrder = [
  "important_urgent",
  "important_not_urgent",
  "not_important_urgent",
  "not_important_not_urgent",
];

// ---------- 拖拽状态 ----------

const dragOverQuadrant = ref<string | null>(null);

function onDragStart(e: DragEvent, task: Task) {
  if (!e.dataTransfer) return;
  e.dataTransfer.effectAllowed = "move";
  e.dataTransfer.setData("text/plain", task.id);
  e.dataTransfer.setData("application/x-quadrant", task.quadrant);
}

function onDragOver(e: DragEvent, quadrant: string) {
  e.preventDefault();
  if (e.dataTransfer) e.dataTransfer.dropEffect = "move";
  dragOverQuadrant.value = quadrant;
}

function onDragLeave(quadrant: string) {
  if (dragOverQuadrant.value === quadrant) {
    dragOverQuadrant.value = null;
  }
}

function onDrop(e: DragEvent, targetQuadrant: string) {
  e.preventDefault();
  dragOverQuadrant.value = null;
  if (!e.dataTransfer) return;

  const taskId = e.dataTransfer.getData("text/plain");
  const fromQuadrant = e.dataTransfer.getData("application/x-quadrant");

  if (!taskId || fromQuadrant === targetQuadrant) return;
  emit("changeQuadrant", taskId, targetQuadrant);
}

function onDragEnd() {
  dragOverQuadrant.value = null;
}
</script>

<template>
  <div class="fl-qgrid">
    <div
      v-for="key in quadrantOrder"
      :key="key"
      class="fl-qcell"
      :class="[
        quadrantMeta[key].cls,
        { 'is-drag-over': dragOverQuadrant === key },
      ]"
      @dragover="onDragOver($event, key)"
      @dragenter.prevent
      @dragleave="onDragLeave(key)"
      @drop="onDrop($event, key)"
    >
      <div class="fl-qcell-head">
        <span class="fl-qcell-label">{{ quadrantMeta[key].label }}</span>
        <span class="fl-qcell-count">{{ tasksByQuadrant[key]?.length ?? 0 }}</span>
      </div>
      <ul v-if="tasksByQuadrant[key]?.length" class="fl-qcell-list">
        <li
          v-for="t in tasksByQuadrant[key]"
          :key="t.id"
          class="fl-qcell-item"
          draggable="true"
          @dragstart="onDragStart($event, t)"
          @dragend="onDragEnd"
          @click="emit('edit', t)"
        >
          <span class="fl-qcell-name">{{ t.name }}</span>
          <button
            class="fl-qcell-play"
            type="button"
            :disabled="!timerIdle"
            :aria-label="`开始「${t.name}」`"
            @click.stop="emit('start', t.id)"
          >
            <Play :size="10" />
          </button>
        </li>
      </ul>
      <div v-else class="fl-qcell-empty">拖入任务或暂无</div>
    </div>
  </div>
</template>

<style scoped>
.fl-qgrid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--sp-3);
}

.fl-qcell {
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--r-md);
  padding: var(--sp-3);
  border-left: 3px solid var(--color-border);
  min-height: 100px;
  transition:
    background var(--dur-fast) var(--ease-smooth),
    border-color var(--dur-fast) var(--ease-smooth),
    box-shadow var(--dur-fast) var(--ease-smooth);
}
.fl-qcell.q1 { border-left-color: var(--color-q1); }
.fl-qcell.q2 { border-left-color: var(--color-q2); }
.fl-qcell.q3 { border-left-color: var(--color-q3); }
.fl-qcell.q4 { border-left-color: var(--color-q4); }

/* 拖拽目标高亮 */
.fl-qcell.is-drag-over {
  background: color-mix(in srgb, var(--color-primary) 8%, var(--color-bg-elevated));
  border-color: var(--color-primary);
  box-shadow: inset 0 0 0 1px var(--color-primary);
}
.fl-qcell.q1.is-drag-over { border-color: var(--color-q1); box-shadow: inset 0 0 0 1px var(--color-q1); background: color-mix(in srgb, var(--color-q1) 6%, var(--color-bg-elevated)); }
.fl-qcell.q2.is-drag-over { border-color: var(--color-q2); box-shadow: inset 0 0 0 1px var(--color-q2); background: color-mix(in srgb, var(--color-q2) 6%, var(--color-bg-elevated)); }
.fl-qcell.q3.is-drag-over { border-color: var(--color-q3); box-shadow: inset 0 0 0 1px var(--color-q3); background: color-mix(in srgb, var(--color-q3) 6%, var(--color-bg-elevated)); }
.fl-qcell.q4.is-drag-over { border-color: var(--color-q4); box-shadow: inset 0 0 0 1px var(--color-q4); background: color-mix(in srgb, var(--color-q4) 6%, var(--color-bg-elevated)); }

.fl-qcell-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  margin-bottom: var(--sp-2);
}

.fl-qcell-label {
  font-size: var(--fs-12);
  font-weight: var(--fw-semibold);
  color: var(--color-text-primary);
}
.fl-qcell-count {
  font-size: 10px;
  color: var(--color-text-muted);
}

.fl-qcell-list {
  list-style: none;
  padding: 0;
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.fl-qcell-item {
  display: flex;
  align-items: center;
  gap: var(--sp-2);
  padding: 4px var(--sp-2);
  border-radius: var(--r-sm);
  cursor: grab;
  transition: background var(--dur-fast) var(--ease-smooth);
}
.fl-qcell-item:hover {
  background: var(--color-bg-hover);
}
.fl-qcell-item:active {
  cursor: grabbing;
  opacity: 0.6;
}

.fl-qcell-name {
  flex: 1;
  font-size: var(--fs-12);
  color: var(--color-text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.fl-qcell-play {
  flex: 0 0 20px;
  width: 20px;
  height: 20px;
  border-radius: 50%;
  border: 1px solid var(--color-border);
  background: transparent;
  color: var(--color-text-muted);
  display: grid;
  place-items: center;
  cursor: pointer;
  opacity: 0;
  transition: all var(--dur-fast) var(--ease-smooth);
}
.fl-qcell-item:hover .fl-qcell-play {
  opacity: 1;
}
.fl-qcell-play:hover:not(:disabled) {
  border-color: var(--color-success);
  color: var(--color-success);
}
.fl-qcell-play:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.fl-qcell-empty {
  font-size: 11px;
  color: var(--color-text-muted);
  text-align: center;
  padding: var(--sp-4) 0;
}
</style>
