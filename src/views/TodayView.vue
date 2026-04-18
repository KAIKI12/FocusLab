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

import { Calendar, Check, Plus, X } from "lucide-vue-next";
import { computed, onMounted, ref } from "vue";

import { useAssignmentStore } from "@/stores/useAssignmentStore";
import { useTaskStore } from "@/stores/useTaskStore";

const tasks = useTaskStore();
const assignments = useAssignmentStore();

const name = ref("");

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
</script>

<template>
  <section class="fl-today">
    <header class="fl-page-head">
      <h1>今日</h1>
      <p class="fl-page-sub">
        Week 1b 验证页 · 任务池 + 今日计划(DTA) + 当前逻辑日由后端解析
      </p>
    </header>

    <!-- 任务池 -->
    <div class="fl-section">
      <div class="fl-section-head">
        <span class="fl-section-title">任务池</span>
        <span class="fl-section-count">{{ tasks.tasks.length }}</span>
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

      <div v-if="tasks.loading" class="fl-empty">载入中…</div>
      <ul v-else-if="tasks.tasks.length" class="fl-list">
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
            class="fl-mini-btn"
            type="button"
            :disabled="assignedTaskIds.has(t.id)"
            :title="assignedTaskIds.has(t.id) ? '已在今日计划' : '加入今日计划'"
            @click="addToPlan(t.id)"
          >
            <Calendar :size="12" />
            {{ assignedTaskIds.has(t.id) ? '已加入' : '今日' }}
          </button>
        </li>
      </ul>
      <div v-else class="fl-empty">还没有任务 · 写下第一件 ↑</div>
    </div>

    <!-- 今日计划 -->
    <div class="fl-section">
      <div class="fl-section-head">
        <span class="fl-section-title">今日计划</span>
        <span class="fl-section-count">
          {{ assignments.assignments.filter((a) => a.dayStatus === 'completed').length }}
          /
          {{ assignments.assignments.length }}
        </span>
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

.fl-empty {
  text-align: center;
  padding: var(--sp-8) var(--sp-4);
  color: var(--color-text-muted);
  font-size: var(--fs-14);
  background: var(--color-bg-subtle);
  border-radius: var(--r-md);
}
</style>
