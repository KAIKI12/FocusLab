<script setup lang="ts">
/**
 * CalendarView · 月历视图 + day-detail 面板。
 * 左侧月历网格(评级色块 + 到期任务标记)，点击日期右侧展示当日详情。
 */

import { ChevronLeft, ChevronRight } from "lucide-vue-next";
import { computed, onMounted, ref, watch } from "vue";

import { invokeCmd } from "@/composables/useTauriInvoke";
import type { AssignmentWithTask, DaySummary, Task } from "@/types";

const today = new Date();
const year = ref(today.getFullYear());
const month = ref(today.getMonth());

const summaries = ref<DaySummary[]>([]);
const loading = ref(false);
const selectedDate = ref<string | null>(null);
const dayAssignments = ref<AssignmentWithTask[]>([]);
const dueTasks = ref<Task[]>([]);

const monthLabel = computed(() => {
  const d = new Date(year.value, month.value);
  return d.toLocaleDateString("zh-CN", { year: "numeric", month: "long" });
});

const calendarDays = computed(() => {
  const firstDay = new Date(year.value, month.value, 1);
  const lastDay = new Date(year.value, month.value + 1, 0);
  const startDow = firstDay.getDay();
  const totalDays = lastDay.getDate();
  const cells: Array<{ day: number; date: string } | null> = [];
  for (let i = 0; i < startDow; i++) cells.push(null);
  for (let d = 1; d <= totalDays; d++) {
    const date = `${year.value}-${String(month.value + 1).padStart(2, "0")}-${String(d).padStart(2, "0")}`;
    cells.push({ day: d, date });
  }
  return cells;
});

const summaryMap = computed(() => {
  const map = new Map<string, DaySummary>();
  for (const s of summaries.value) map.set(s.settleDate, s);
  return map;
});

/** 到期任务按日期索引 */
const dueByDate = computed(() => {
  const map = new Map<string, Task[]>();
  for (const t of dueTasks.value) {
    if (!t.due_date) continue;
    const d = t.due_date;
    if (!map.has(d)) map.set(d, []);
    map.get(d)!.push(t);
  }
  return map;
});

const todayStr = computed(() => {
  const d = new Date();
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
});

function gradeColor(grade: string): string {
  switch (grade) {
    case "S": return "var(--color-gold, #FAAD14)";
    case "A": return "var(--color-primary)";
    case "B": return "var(--color-q3)";
    case "C": return "var(--color-text-muted)";
    default: return "transparent";
  }
}

async function loadMonth() {
  loading.value = true;
  try {
    const from = `${year.value}-${String(month.value + 1).padStart(2, "0")}-01`;
    const lastDay = new Date(year.value, month.value + 1, 0).getDate();
    const to = `${year.value}-${String(month.value + 1).padStart(2, "0")}-${String(lastDay).padStart(2, "0")}`;
    summaries.value = await invokeCmd<DaySummary[]>("list_day_summaries", { from, to });

    // 加载当月有到期日的任务
    const allTasks = await invokeCmd<Task[]>("list_tasks", { statusFilter: null });
    dueTasks.value = allTasks.filter(
      (t) => t.due_date && t.due_date >= from && t.due_date <= to,
    );
  } finally {
    loading.value = false;
  }
}

async function selectDay(date: string) {
  selectedDate.value = date;
  try {
    dayAssignments.value = await invokeCmd<AssignmentWithTask[]>("list_assignments", {
      planDate: date,
    });
  } catch {
    dayAssignments.value = [];
  }
}

function prevMonth() {
  if (month.value === 0) { year.value--; month.value = 11; }
  else month.value--;
  selectedDate.value = null;
}

function nextMonth() {
  if (month.value === 11) { year.value++; month.value = 0; }
  else month.value++;
  selectedDate.value = null;
}

onMounted(loadMonth);
watch([year, month], loadMonth);
</script>

<template>
  <section class="fl-calendar-page">
    <div class="fl-cal-main">
      <header class="fl-cal-head">
        <h1>日历</h1>
        <div class="fl-cal-nav">
          <button class="fl-cal-arrow" type="button" @click="prevMonth">
            <ChevronLeft :size="18" />
          </button>
          <span class="fl-cal-month">{{ monthLabel }}</span>
          <button class="fl-cal-arrow" type="button" @click="nextMonth">
            <ChevronRight :size="18" />
          </button>
        </div>
      </header>

      <div class="fl-cal-grid">
        <div v-for="d in ['日','一','二','三','四','五','六']" :key="d" class="fl-cal-dow">
          {{ d }}
        </div>

        <div
          v-for="(cell, i) in calendarDays"
          :key="i"
          class="fl-cal-cell"
          :class="{
            'is-empty': !cell,
            'is-today': cell?.date === todayStr,
            'is-selected': cell?.date === selectedDate,
          }"
          @click="cell && selectDay(cell.date)"
        >
          <template v-if="cell">
            <span class="fl-cal-day">{{ cell.day }}</span>
            <div class="fl-cal-indicators">
              <span
                v-if="summaryMap.has(cell.date)"
                class="fl-cal-grade"
                :style="{ background: gradeColor(summaryMap.get(cell.date)!.grade) }"
              >
                {{ summaryMap.get(cell.date)!.grade }}
              </span>
              <span
                v-if="dueByDate.has(cell.date)"
                class="fl-cal-due"
                :title="`${dueByDate.get(cell.date)!.length} 个到期`"
              >
                ⏰
              </span>
            </div>
          </template>
        </div>
      </div>

      <div class="fl-cal-legend">
        <span v-for="g in ['S','A','B','C']" :key="g" class="fl-cal-legend-item">
          <span class="fl-cal-legend-dot" :style="{ background: gradeColor(g) }" />
          {{ g }}
        </span>
        <span class="fl-cal-legend-item">
          <span>⏰</span> 到期
        </span>
      </div>
    </div>

    <!-- Day Detail 面板 -->
    <aside v-if="selectedDate" class="fl-day-detail">
      <h2 class="fl-dd-title">
        {{ new Date(selectedDate + 'T00:00').toLocaleDateString('zh-CN', { month: 'long', day: 'numeric', weekday: 'short' }) }}
      </h2>

      <!-- 评级摘要 -->
      <div v-if="summaryMap.has(selectedDate)" class="fl-dd-summary">
        <span
          class="fl-dd-grade"
          :style="{ color: gradeColor(summaryMap.get(selectedDate)!.grade) }"
        >
          {{ summaryMap.get(selectedDate)!.grade }} 级
        </span>
        <span class="fl-dd-stat">
          {{ summaryMap.get(selectedDate)!.completedTasks }}/{{ summaryMap.get(selectedDate)!.totalTasks }} 完成
        </span>
        <span class="fl-dd-stat">
          {{ summaryMap.get(selectedDate)!.totalFocusMinutes }} 分钟专注
        </span>
        <div class="fl-dd-progress">
          <div
            class="fl-dd-progress-fill"
            :style="{
              width: (summaryMap.get(selectedDate)!.totalTasks > 0
                ? Math.round(summaryMap.get(selectedDate)!.completedTasks / summaryMap.get(selectedDate)!.totalTasks * 100)
                : 0) + '%'
            }"
          />
        </div>
      </div>
      <div v-else class="fl-dd-no-settle">未结算</div>

      <!-- 今日到期任务 -->
      <div v-if="dueByDate.has(selectedDate)" class="fl-dd-section">
        <h3>⏰ 到期任务</h3>
        <ul class="fl-dd-list">
          <li v-for="t in dueByDate.get(selectedDate)" :key="t.id" class="fl-dd-item fl-dd-due">
            {{ t.name }}
          </li>
        </ul>
      </div>

      <!-- 当日安排 -->
      <div class="fl-dd-section">
        <h3>📋 当日安排</h3>
        <ul v-if="dayAssignments.length" class="fl-dd-list">
          <li
            v-for="a in dayAssignments"
            :key="a.id"
            class="fl-dd-item"
            :class="{ 'is-done': a.dayStatus === 'completed' }"
          >
            <span>{{ a.taskName }}</span>
            <span class="fl-dd-status">{{ a.dayStatus }}</span>
          </li>
        </ul>
        <p v-else class="fl-dd-empty">无安排</p>
      </div>
    </aside>
  </section>
</template>

<style scoped>
.fl-calendar-page {
  max-width: 860px;
  margin: 0 auto;
  display: flex;
  gap: var(--sp-6);
}

.fl-cal-main {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: var(--sp-4);
}

.fl-cal-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.fl-cal-head h1 {
  font-size: var(--fs-24);
  font-weight: var(--fw-semibold);
  margin: 0;
  color: var(--color-text-primary);
}

.fl-cal-nav {
  display: flex;
  align-items: center;
  gap: var(--sp-2);
}

.fl-cal-arrow {
  background: none;
  border: 1px solid var(--color-border);
  border-radius: var(--r-sm);
  color: var(--color-text-secondary);
  cursor: pointer;
  padding: var(--sp-1);
  display: grid;
  place-items: center;
}
.fl-cal-arrow:hover {
  border-color: var(--color-primary);
  color: var(--color-primary);
}

.fl-cal-month {
  font-size: var(--fs-14);
  font-weight: var(--fw-medium);
  color: var(--color-text-primary);
  min-width: 100px;
  text-align: center;
}

.fl-cal-grid {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  gap: 2px;
}

.fl-cal-dow {
  text-align: center;
  font-size: 11px;
  color: var(--color-text-muted);
  padding: var(--sp-2) 0;
  font-weight: var(--fw-medium);
}

.fl-cal-cell {
  aspect-ratio: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 2px;
  border-radius: var(--r-sm);
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  cursor: pointer;
  transition: all var(--dur-fast) var(--ease-smooth);
}
.fl-cal-cell:hover:not(.is-empty) {
  border-color: var(--color-primary);
}
.fl-cal-cell.is-empty {
  background: transparent;
  border-color: transparent;
  cursor: default;
}
.fl-cal-cell.is-today {
  border-color: var(--color-primary);
  box-shadow: inset 0 0 0 1px var(--color-primary);
}
.fl-cal-cell.is-selected {
  background: var(--color-primary-soft);
  border-color: var(--color-primary);
}

.fl-cal-day {
  font-size: var(--fs-12);
  color: var(--color-text-secondary);
}

.fl-cal-indicators {
  display: flex;
  gap: 2px;
  align-items: center;
}

.fl-cal-grade {
  width: 16px;
  height: 16px;
  border-radius: 3px;
  display: grid;
  place-items: center;
  font-size: 9px;
  font-weight: var(--fw-semibold);
  color: #fff;
}

.fl-cal-due {
  font-size: 10px;
  line-height: 1;
}

.fl-cal-legend {
  display: flex;
  gap: var(--sp-4);
  justify-content: center;
}
.fl-cal-legend-item {
  display: flex;
  align-items: center;
  gap: var(--sp-1);
  font-size: 11px;
  color: var(--color-text-muted);
}
.fl-cal-legend-dot {
  width: 10px;
  height: 10px;
  border-radius: 2px;
}

/* ---------- Day Detail ---------- */
.fl-day-detail {
  width: 260px;
  flex-shrink: 0;
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--r-lg);
  padding: var(--sp-4);
  display: flex;
  flex-direction: column;
  gap: var(--sp-4);
  align-self: flex-start;
}

.fl-dd-title {
  font-size: var(--fs-14);
  font-weight: var(--fw-semibold);
  margin: 0;
  color: var(--color-text-primary);
}

.fl-dd-summary {
  display: flex;
  flex-wrap: wrap;
  gap: var(--sp-2);
  align-items: center;
}
.fl-dd-grade {
  font-size: var(--fs-16);
  font-weight: var(--fw-semibold);
}
.fl-dd-stat {
  font-size: var(--fs-12);
  color: var(--color-text-secondary);
  padding: 2px var(--sp-2);
  background: var(--color-bg-subtle);
  border-radius: var(--r-pill);
}

.fl-dd-progress {
  width: 100%; height: 4px; border-radius: 2px;
  background: var(--color-bg-hover); margin-top: var(--sp-2);
}
.fl-dd-progress-fill {
  height: 100%; border-radius: 2px;
  background: var(--color-primary);
  transition: width 0.3s var(--ease-smooth);
}

.fl-dd-no-settle {
  font-size: var(--fs-12);
  color: var(--color-text-muted);
}

.fl-dd-section h3 {
  font-size: var(--fs-12);
  font-weight: var(--fw-medium);
  color: var(--color-text-secondary);
  margin: 0 0 var(--sp-2);
}

.fl-dd-list {
  list-style: none;
  padding: 0;
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: var(--sp-1);
}

.fl-dd-item {
  font-size: var(--fs-12);
  color: var(--color-text-primary);
  padding: var(--sp-1) var(--sp-2);
  background: var(--color-bg-subtle);
  border-radius: var(--r-sm);
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.fl-dd-item.is-done {
  color: var(--color-text-muted);
  text-decoration: line-through;
}
.fl-dd-item.fl-dd-due {
  border-left: 2px solid var(--color-q3);
}

.fl-dd-status {
  font-size: 10px;
  color: var(--color-text-muted);
}

.fl-dd-empty {
  font-size: var(--fs-12);
  color: var(--color-text-muted);
  margin: 0;
}
</style>
