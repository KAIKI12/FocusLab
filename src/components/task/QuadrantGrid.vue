<script setup lang="ts">
/**
 * QuadrantGrid · 2×2 四象限分组任务视图。
 * 每个象限格子展示该象限下的任务列表(紧凑模式)。
 */

import { Play } from "lucide-vue-next";

import type { Task } from "@/types";

defineProps<{
  tasksByQuadrant: Record<string, Task[]>;
  timerIdle: boolean;
}>();

const emit = defineEmits<{
  edit: [task: Task];
  start: [taskId: string];
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
</script>

<template>
  <div class="fl-qgrid">
    <div
      v-for="key in quadrantOrder"
      :key="key"
      class="fl-qcell"
      :class="quadrantMeta[key].cls"
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
      <div v-else class="fl-qcell-empty">暂无任务</div>
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
}
.fl-qcell.q1 { border-left-color: var(--color-q1); }
.fl-qcell.q2 { border-left-color: var(--color-q2); }
.fl-qcell.q3 { border-left-color: var(--color-q3); }
.fl-qcell.q4 { border-left-color: var(--color-q4); }

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
  cursor: pointer;
  transition: background var(--dur-fast) var(--ease-smooth);
}
.fl-qcell-item:hover {
  background: var(--color-bg-hover);
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
