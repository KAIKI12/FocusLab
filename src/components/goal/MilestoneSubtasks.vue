<script setup lang="ts">
/**
 * MilestoneSubtasks · 里程碑子任务列表 + 今日关联任务 banner。
 *
 * 对齐 prototype/goals/milestones.html:488-544。
 * 数据源:tasks 表(通过 useMilestoneSubtasks 派生),子任务 = tasks WHERE milestone_id = ?。
 */

import { Check, CircleDashed, Flame, Plus, Circle } from "lucide-vue-next";
import { computed, ref } from "vue";

import { useMilestoneSubtasks } from "@/composables/useMilestoneSubtasks";
import { useTaskStore } from "@/stores/useTaskStore";
import type { Task } from "@/types";

const props = defineProps<{
  milestoneId: string;
}>();

const tasks = useTaskStore();
const { subtasksOf, todayActiveOf, progressOf } = useMilestoneSubtasks();

const newTitle = ref("");
const adding = ref(false);

const subtasks = computed(() => subtasksOf(props.milestoneId));
const todayActive = computed(() => todayActiveOf(props.milestoneId));
const progress = computed(() => progressOf(props.milestoneId));

function formatEstimate(t: Task): string {
  if (t.status === "completed") return "已完成";
  if (t.status === "in_progress") return "进行中";
  if (!t.estimated_minutes) return "待启动";
  // 粗粒度:≥ 8h 显示天数,否则分钟
  if (t.estimated_minutes >= 480) {
    const days = Math.round(t.estimated_minutes / 480);
    return `${days} 天`;
  }
  return `${t.estimated_minutes}m`;
}

async function onAdd() {
  const name = newTitle.value.trim();
  if (!name) return;
  adding.value = true;
  try {
    const created = await tasks.create({ name, quadrant: "important_not_urgent" });
    await tasks.update({ id: created.id, milestoneId: props.milestoneId });
    newTitle.value = "";
  } finally {
    adding.value = false;
  }
}

async function onToggleStatus(t: Task) {
  const next = t.status === "completed" ? "pending" : "completed";
  await tasks.update({ id: t.id, status: next as "pending" | "in_progress" | "completed" });
}

function formatSeconds(s: number): string {
  const m = Math.floor(s / 60);
  const h = Math.floor(m / 60);
  const mm = m % 60;
  if (h > 0) return `${h}h ${mm}m`;
  return `${mm}m`;
}
</script>

<template>
  <div class="fl-ms-sub">
    <div class="fl-ms-sub-head">
      <span>子任务 · {{ progress.done }}/{{ progress.total }}</span>
    </div>

    <!-- 今日关联 banner -->
    <div v-if="todayActive" class="fl-ms-today">
      <Flame :size="14" />
      <span>
        今天正在推进 · <strong>{{ todayActive.task.name }}</strong>
        <template v-if="todayActive.isFocusing">
          · 已专注 {{ formatSeconds(todayActive.focusingSeconds) }}
        </template>
      </span>
    </div>

    <!-- 子任务行 -->
    <div v-if="subtasks.length" class="fl-ms-sub-list">
      <div
        v-for="t in subtasks"
        :key="t.id"
        class="fl-ms-sub-row"
        :class="{
          'is-done': t.status === 'completed',
          'is-doing': t.status === 'in_progress',
        }"
      >
        <button
          class="fl-ms-sub-status"
          :aria-label="t.status === 'completed' ? '取消完成' : '标记完成'"
          @click="onToggleStatus(t)"
        >
          <Check v-if="t.status === 'completed'" :size="14" class="fl-ms-sub-done-icon" />
          <CircleDashed v-else-if="t.status === 'in_progress'" :size="14" class="fl-ms-sub-doing-icon" />
          <Circle v-else :size="14" class="fl-ms-sub-pending-icon" />
        </button>
        <span class="fl-ms-sub-title">{{ t.name }}</span>
        <span class="fl-ms-sub-meta">{{ formatEstimate(t) }}</span>
      </div>
    </div>
    <div v-else class="fl-ms-sub-empty">尚无子任务 · 用下方输入添加</div>

    <!-- 添加 -->
    <form class="fl-ms-sub-add" @submit.prevent="onAdd">
      <input
        v-model="newTitle"
        class="fl-ms-sub-input"
        type="text"
        placeholder="添加子任务…"
        maxlength="80"
      />
      <button
        class="fl-ms-sub-add-btn"
        type="submit"
        :disabled="!newTitle.trim() || adding"
      >
        <Plus :size="12" />
      </button>
    </form>
  </div>
</template>

<style scoped>
.fl-ms-sub { display: flex; flex-direction: column; gap: var(--sp-3); }

.fl-ms-sub-head {
  font-size: var(--fs-12); color: var(--color-text-muted);
  text-transform: uppercase; letter-spacing: 0.5px;
  font-weight: var(--fw-medium);
}

.fl-ms-today {
  display: flex; align-items: center; gap: var(--sp-2);
  padding: var(--sp-3);
  background: var(--color-primary-soft);
  border: 1px dashed color-mix(in srgb, var(--color-primary) 30%, transparent);
  border-radius: var(--r-sm);
  font-size: var(--fs-13, 13px);
  color: var(--color-primary);
}
.fl-ms-today strong { color: var(--color-text-primary); }

.fl-ms-sub-list { display: flex; flex-direction: column; gap: 2px; }

.fl-ms-sub-row {
  display: flex; align-items: center; gap: var(--sp-2);
  padding: 6px 8px;
  border-radius: var(--r-sm);
  transition: background var(--dur-fast);
}
.fl-ms-sub-row:hover { background: var(--color-bg-hover); }
.fl-ms-sub-row.is-done { opacity: 0.55; }
.fl-ms-sub-row.is-done .fl-ms-sub-title {
  text-decoration: line-through;
  color: var(--color-text-muted);
}

.fl-ms-sub-status {
  width: 20px; height: 20px;
  display: grid; place-items: center;
  flex-shrink: 0;
  background: none; border: none;
  cursor: pointer; padding: 0;
}
.fl-ms-sub-done-icon { color: var(--color-success); }
.fl-ms-sub-doing-icon { color: var(--color-primary); }
.fl-ms-sub-pending-icon { color: var(--color-text-muted); }

.fl-ms-sub-title {
  flex: 1; min-width: 0;
  font-size: var(--fs-13, 13px);
  color: var(--color-text-primary);
  white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
}

.fl-ms-sub-meta {
  font-size: 11px; color: var(--color-text-muted);
  flex-shrink: 0;
}

.fl-ms-sub-empty {
  font-size: 11px; color: var(--color-text-muted);
  padding: var(--sp-3) 0; text-align: center;
}

.fl-ms-sub-add { display: flex; gap: var(--sp-2); }
.fl-ms-sub-input {
  flex: 1;
  padding: 6px 10px;
  border-radius: var(--r-md);
  border: 1px solid var(--color-border);
  background: var(--color-bg-subtle);
  color: var(--color-text-primary);
  font-size: 12px;
  outline: none;
}
.fl-ms-sub-input:focus { border-color: var(--color-primary); }
.fl-ms-sub-add-btn {
  padding: 0 10px;
  border-radius: var(--r-md);
  border: none;
  background: var(--color-primary);
  color: var(--color-text-on-primary);
  cursor: pointer;
  display: grid; place-items: center;
}
.fl-ms-sub-add-btn:disabled { opacity: 0.4; cursor: not-allowed; }
</style>
