<script setup lang="ts">
/**
 * TodayView · Week 1a 的 task CRUD + Week 1b 的今日计划(DTA)分配。
 *
 * 上半区 — 任务池(pending + in_progress 的全局任务)
 *   ・添加任务
 *   ・✓ 标完成(全局)
 *   ・📅 加入今日计划
 *
 * 下半区 — 今日计划(daily_task_assignments,当前逻辑日)
 *   ・✓ 标今日完成(day_status=completed,不影响全局 task 状态)
 *   ・✕ 从计划移除(删 dta,不影响 task)
 */

import { Calendar, Check, Grid2X2, List, Lock, Minimize2, Moon, Pencil, Play, Plus, Trash2, X } from "lucide-vue-next";
import { computed, onMounted, ref } from "vue";

import YesterdayCard from "@/components/settlement/YesterdayCard.vue";
import QuadrantGrid from "@/components/task/QuadrantGrid.vue";
import TaskEditModal from "@/components/task/TaskEditModal.vue";
import PresetSwitcher from "@/components/timer/PresetSwitcher.vue";
import TimerCard from "@/components/timer/TimerCard.vue";
import { useBubble } from "@/composables/useBubble";
import { useAssignmentStore } from "@/stores/useAssignmentStore";
import { useSettlementStore } from "@/stores/useSettlementStore";
import { useTaskStore } from "@/stores/useTaskStore";
import { useTimerStore } from "@/stores/useTimerStore";
import type { Task } from "@/types";

const tasks = useTaskStore();
const assignments = useAssignmentStore();
const timer = useTimerStore();
const settlement = useSettlementStore();
const { open: openBubble } = useBubble();

const name = ref("");
const viewMode = ref<"list" | "quadrant">("list");
const editingTask = ref<Task | null>(null);

onMounted(async () => {
  await Promise.all([tasks.load(), assignments.load()]);
});

/** 已经在今日计划里的 task_id 集合 — 用于上半区按钮禁用 */
const assignedTaskIds = computed(
  () => new Set(assignments.assignments.map((a) => a.taskId)),
);

async function onAdd() {
  const trimmed = name.value.trim();
  if (!trimmed) return;
  await tasks.create({ name: trimmed });
  name.value = "";
}

async function addToPlan(taskId: string) {
  try {
    await assignments.create({ taskId });
  } catch (e) {
    // 唯一键冲突在后端已转成 friendly 错误
    console.error(e);
  }
}

/** 启动番茄钟 — 仅在当前 idle 时允许,根据 selectedPreset 选择模式 */
async function onStartPomodoro(taskId: string) {
  if (!timer.isIdle) {
    console.warn("[timer] 已有计时进行中,忽略新启动请求");
    return;
  }
  try {
    if (timer.selectedPreset === "free") {
      await timer.startFree(taskId);
    } else {
      await timer.startPomodoro(taskId, timer.selectedPreset);
    }
  } catch (e) {
    console.error("[timer] start failed", e);
  }
}
</script>

<template>
  <section class="fl-today">
    <header class="fl-page-head">
      <div>
        <h1>今日</h1>
        <p class="fl-page-sub">
          任务池 + 今日计划 + 番茄钟 + 四象限
        </p>
      </div>
      <div class="fl-page-actions">
        <button class="fl-bubble-entry" type="button" title="悬浮球" @click="openBubble">
          <Minimize2 :size="14" /> 悬浮球
        </button>
        <button
          class="fl-settle-btn"
          type="button"
          :disabled="settlement.settling"
          @click="settlement.settle()"
        >
          <Moon :size="14" /> 结束今天
        </button>
      </div>
    </header>

    <!-- 昨日回顾(有数据时自动展示) -->
    <YesterdayCard />

    <!-- 当前计时卡(仅 non-idle 时渲染) -->
    <TimerCard />

    <!-- Preset 选择(仅 idle 时渲染) -->
    <PresetSwitcher />

    <!-- 任务池 -->
    <div class="fl-section">
      <div class="fl-section-head">
        <span class="fl-section-title">任务池</span>
        <div class="fl-section-right">
          <span class="fl-section-count">{{ tasks.tasks.length }}</span>
          <button
            class="fl-view-toggle"
            type="button"
            :title="viewMode === 'list' ? '象限视图' : '列表视图'"
            @click="viewMode = viewMode === 'list' ? 'quadrant' : 'list'"
          >
            <Grid2X2 v-if="viewMode === 'list'" :size="14" />
            <List v-else :size="14" />
          </button>
        </div>
      </div>

      <form class="fl-task-form" @submit.prevent="onAdd">
        <input
          v-model="name"
          class="fl-input"
          type="text"
          placeholder="添加任务…"
          maxlength="80"
        />
        <button class="fl-btn" type="submit" :disabled="!name.trim()">
          <Plus :size="14" />
          添加
        </button>
      </form>

      <!-- 象限视图 -->
      <QuadrantGrid
        v-if="viewMode === 'quadrant' && tasks.tasks.length"
        :tasks-by-quadrant="tasks.tasksByQuadrant"
        :timer-idle="timer.isIdle"
        @edit="editingTask = $event"
        @start="onStartPomodoro($event)"
      />

      <!-- 列表视图 -->
      <div v-else-if="tasks.loading" class="fl-empty">载入中…</div>
      <ul v-else-if="tasks.tasks.length && viewMode === 'list'" class="fl-list">
        <li v-for="t in tasks.tasks" :key="t.id" class="fl-item">
          <button
            class="fl-check"
            type="button"
            :aria-label="`完成「${t.name}」`"
            @click="tasks.complete(t.id)"
          >
            <Check :size="14" />
          </button>
          <span class="fl-name">{{ t.name }}</span>
          <button
            class="fl-mini-btn fl-mini-edit"
            type="button"
            title="编辑"
            @click="editingTask = t"
          >
            <Pencil :size="10" />
          </button>
          <button
            class="fl-mini-btn fl-mini-play"
            type="button"
            :disabled="!timer.isIdle"
            :title="timer.isIdle ? '开始番茄钟' : '已有计时进行中'"
            :aria-label="`开始番茄钟「${t.name}」`"
            @click="onStartPomodoro(t.id)"
          >
            <Play :size="12" />
          </button>
          <button
            class="fl-mini-btn"
            type="button"
            :disabled="assignedTaskIds.has(t.id)"
            :title="assignedTaskIds.has(t.id) ? '已在今日计划' : '加入今日计划'"
            @click="addToPlan(t.id)"
          >
            <Calendar :size="12" />
            {{ assignedTaskIds.has(t.id) ? '已加入' : '今日' }}
          </button>
          <button
            class="fl-mini-btn fl-mini-danger"
            type="button"
            title="删除"
            @click="tasks.remove(t.id)"
          >
            <Trash2 :size="10" />
          </button>
        </li>
      </ul>
      <div v-else class="fl-empty">还没有任务 · 写下第一件 ↑</div>
    </div>

    <!-- 今日计划 -->
    <div class="fl-section">
      <div class="fl-section-head">
        <span class="fl-section-title">今日计划</span>
        <div class="fl-section-right">
          <span v-if="assignments.stats" class="fl-stats-badge">
            {{ assignments.stats.completedCount }}/{{ assignments.stats.plannedCount }}
            <template v-if="assignments.stats.extraCompleted > 0">
              +{{ assignments.stats.extraCompleted }}
            </template>
          </span>
          <button
            v-if="!assignments.stats?.isLocked"
            class="fl-view-toggle"
            type="button"
            title="锁定今日计划"
            @click="assignments.lockPlan()"
          >
            <Lock :size="12" />
          </button>
          <span v-else class="fl-locked-badge" title="计划已锁定">
            <Lock :size="10" /> 已锁定
          </span>
        </div>
      </div>

      <ul v-if="assignments.assignments.length" class="fl-list">
        <li
          v-for="a in assignments.assignments"
          :key="a.id"
          class="fl-item"
          :class="{ 'is-done': a.dayStatus === 'completed' }"
        >
          <button
            class="fl-check"
            :class="{ 'is-checked': a.dayStatus === 'completed' }"
            type="button"
            :aria-label="`完成「${a.taskName}」`"
            @click="
              assignments.setStatus(
                a.id,
                a.dayStatus === 'completed' ? 'pending' : 'completed',
              )
            "
          >
            <Check :size="14" />
          </button>
          <span class="fl-name">{{ a.taskName }}</span>
          <span class="fl-badge">{{ a.source }}</span>
          <button
            class="fl-mini-btn fl-mini-play"
            type="button"
            :disabled="!timer.isIdle || a.dayStatus === 'completed'"
            :title="
              a.dayStatus === 'completed'
                ? '今日已完成'
                : timer.isIdle
                  ? '开始番茄钟'
                  : '已有计时进行中'
            "
            :aria-label="`开始番茄钟「${a.taskName}」`"
            @click="onStartPomodoro(a.taskId)"
          >
            <Play :size="12" />
          </button>
          <button
            class="fl-mini-btn fl-mini-danger"
            type="button"
            :aria-label="`从今日计划移除「${a.taskName}」`"
            @click="assignments.remove(a.id)"
          >
            <X :size="12" />
          </button>
        </li>
      </ul>
      <div v-else class="fl-empty">今日还没排任何任务 · 从上方点「今日」加入</div>
    </div>

    <!-- 编辑弹窗 -->
    <TaskEditModal :task="editingTask" @close="editingTask = null" />
  </section>
</template>

<style scoped>
.fl-today {
  max-width: 720px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: var(--sp-5);
}

.fl-page-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
}

.fl-page-actions {
  display: flex;
  gap: var(--sp-2);
}

.fl-page-head h1 {
  font-size: var(--fs-24);
  font-weight: var(--fw-semibold);
  margin: 0;
  color: var(--color-text-primary);
}

.fl-page-sub {
  margin: var(--sp-1) 0 0;
  color: var(--color-text-secondary);
  font-size: var(--fs-12);
}

.fl-section {
  display: flex;
  flex-direction: column;
  gap: var(--sp-3);
}

.fl-section-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
}

.fl-section-right {
  display: flex;
  align-items: center;
  gap: var(--sp-2);
}

.fl-section-title {
  font-size: var(--fs-16);
  font-weight: var(--fw-semibold);
  color: var(--color-text-primary);
}

.fl-section-count {
  font-size: var(--fs-12);
  color: var(--color-text-muted);
}

.fl-task-form {
  display: flex;
  gap: var(--sp-2);
}

.fl-input {
  flex: 1;
  padding: var(--sp-3) var(--sp-4);
  border-radius: var(--r-md);
  border: 1px solid var(--color-border);
  background: var(--color-bg-elevated);
  color: var(--color-text-primary);
  font-size: var(--fs-14);
  outline: none;
  transition:
    border-color var(--dur-fast) var(--ease-smooth),
    box-shadow var(--dur-fast) var(--ease-smooth);
}

.fl-input::placeholder {
  color: var(--color-text-muted);
}

.fl-input:focus {
  border-color: var(--color-primary);
  box-shadow: var(--shadow-focus);
}

.fl-btn {
  display: inline-flex;
  align-items: center;
  gap: var(--sp-1);
  padding: 0 var(--sp-4);
  border-radius: var(--r-md);
  background: var(--color-primary);
  color: var(--color-text-on-primary);
  border: none;
  font-size: var(--fs-12);
  font-weight: var(--fw-medium);
  cursor: pointer;
  transition: background var(--dur-fast) var(--ease-smooth);
}

.fl-btn:hover:not(:disabled) {
  background: var(--color-primary-dark);
}

.fl-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.fl-list {
  list-style: none;
  padding: 0;
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: var(--sp-2);
}

.fl-item {
  display: flex;
  align-items: center;
  gap: var(--sp-3);
  padding: var(--sp-3) var(--sp-4);
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--r-md);
  box-shadow: var(--shadow-card);
}

.fl-item.is-done .fl-name {
  color: var(--color-text-muted);
  text-decoration: line-through;
}

.fl-check {
  width: 22px;
  height: 22px;
  flex: 0 0 22px;
  border-radius: var(--r-pill);
  border: 1.5px solid var(--color-border-strong);
  background: transparent;
  color: transparent;
  cursor: pointer;
  display: grid;
  place-items: center;
  transition: all var(--dur-fast) var(--ease-smooth);
}

.fl-check:hover,
.fl-check.is-checked {
  border-color: var(--color-success);
  background: var(--color-success);
  color: var(--color-text-on-primary);
}

.fl-name {
  flex: 1;
  color: var(--color-text-primary);
}

.fl-badge {
  font-size: 10px;
  color: var(--color-text-muted);
  padding: 2px var(--sp-2);
  border-radius: var(--r-pill);
  background: var(--color-bg-subtle);
  text-transform: uppercase;
  letter-spacing: 0.3px;
}

.fl-mini-btn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px var(--sp-2);
  border: 1px solid var(--color-border);
  background: transparent;
  color: var(--color-text-secondary);
  border-radius: var(--r-sm);
  font-size: 11px;
  cursor: pointer;
  transition: all var(--dur-fast) var(--ease-smooth);
}

.fl-mini-btn:hover:not(:disabled) {
  border-color: var(--color-primary);
  color: var(--color-primary);
}

.fl-mini-btn:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

.fl-mini-danger:hover {
  border-color: var(--color-q1);
  color: var(--color-q1);
}

.fl-mini-play:hover:not(:disabled) {
  border-color: var(--color-success);
  color: var(--color-success);
}

.fl-mini-edit:hover:not(:disabled) {
  border-color: var(--color-primary);
  color: var(--color-primary);
}

.fl-view-toggle {
  background: none;
  border: 1px solid var(--color-border);
  border-radius: var(--r-sm);
  padding: 4px;
  color: var(--color-text-muted);
  cursor: pointer;
  display: grid;
  place-items: center;
  transition: all var(--dur-fast) var(--ease-smooth);
}
.fl-view-toggle:hover {
  border-color: var(--color-primary);
  color: var(--color-primary);
}

.fl-stats-badge {
  font-size: 11px;
  color: var(--color-text-secondary);
  padding: 2px var(--sp-2);
  background: var(--color-bg-subtle);
  border-radius: var(--r-pill);
  font-weight: var(--fw-medium);
}

.fl-locked-badge {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  font-size: 10px;
  color: var(--color-success);
  font-weight: var(--fw-medium);
}

.fl-bubble-entry {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: var(--sp-2) var(--sp-3);
  border-radius: var(--r-md);
  border: 1px solid var(--color-border);
  background: var(--color-bg-elevated);
  color: var(--color-text-secondary);
  font-size: 11px;
  cursor: pointer;
  transition: all var(--dur-fast) var(--ease-smooth);
}
.fl-bubble-entry:hover {
  border-color: var(--color-primary);
  color: var(--color-primary);
}

.fl-settle-btn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: var(--sp-2) var(--sp-3);
  border-radius: var(--r-md);
  border: 1px solid var(--color-border);
  background: var(--color-bg-elevated);
  color: var(--color-text-secondary);
  font-size: 11px;
  cursor: pointer;
  transition: all var(--dur-fast) var(--ease-smooth);
}
.fl-settle-btn:hover {
  border-color: var(--color-q2);
  color: var(--color-q2);
}
.fl-settle-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.fl-empty {
  text-align: center;
  padding: var(--sp-8) var(--sp-4);
  color: var(--color-text-muted);
  font-size: var(--fs-14);
  background: var(--color-bg-subtle);
  border-radius: var(--r-md);
}
</style>
