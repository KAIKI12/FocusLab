<script setup lang="ts">
/**
 * CalendarView · 月历视图,展示每日结算热力图。
 */

import { ChevronLeft, ChevronRight } from "lucide-vue-next";
import { computed, onMounted, ref, watch } from "vue";

import { invokeCmd } from "@/composables/useTauriInvoke";
import type { DaySummary } from "@/types";

const today = new Date();
const year = ref(today.getFullYear());
const month = ref(today.getMonth()); // 0-indexed

const summaries = ref<DaySummary[]>([]);
const loading = ref(false);

const monthLabel = computed(() => {
  const d = new Date(year.value, month.value);
  return d.toLocaleDateString("zh-CN", { year: "numeric", month: "long" });
});

/** 当月日历网格(含前置空白) */
const calendarDays = computed(() => {
  const firstDay = new Date(year.value, month.value, 1);
  const lastDay = new Date(year.value, month.value + 1, 0);
  const startDow = firstDay.getDay(); // 0=Sun
  const totalDays = lastDay.getDate();

  const cells: Array<{ day: number; date: string } | null> = [];
  for (let i = 0; i < startDow; i++) cells.push(null);
  for (let d = 1; d <= totalDays; d++) {
    const date = `${year.value}-${String(month.value + 1).padStart(2, "0")}-${String(d).padStart(2, "0")}`;
    cells.push({ day: d, date });
  }
  return cells;
});

/** date -> DaySummary 映射 */
const summaryMap = computed(() => {
  const map = new Map<string, DaySummary>();
  for (const s of summaries.value) {
    map.set(s.settleDate, s);
  }
  return map;
});

function gradeColor(grade: string): string {
  switch (grade) {
    case "S": return "var(--color-success)";
    case "A": return "var(--color-primary)";
    case "B": return "var(--color-q3)";
    case "C": return "var(--color-q1)";
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
  } finally {
    loading.value = false;
  }
}

function prevMonth() {
  if (month.value === 0) { year.value--; month.value = 11; }
  else month.value--;
}

function nextMonth() {
  if (month.value === 11) { year.value++; month.value = 0; }
  else month.value++;
}

onMounted(loadMonth);
watch([year, month], loadMonth);
</script>

<template>
  <section class="fl-calendar">
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
        :class="{ 'is-empty': !cell }"
      >
        <template v-if="cell">
          <span class="fl-cal-day">{{ cell.day }}</span>
          <div
            v-if="summaryMap.has(cell.date)"
            class="fl-cal-dot"
            :style="{ background: gradeColor(summaryMap.get(cell.date)!.grade) }"
            :title="`${summaryMap.get(cell.date)!.grade} 级 · ${summaryMap.get(cell.date)!.completedTasks}/${summaryMap.get(cell.date)!.totalTasks} 完成 · ${summaryMap.get(cell.date)!.totalFocusMinutes} 分钟`"
          >
            {{ summaryMap.get(cell.date)!.grade }}
          </div>
        </template>
      </div>
    </div>

    <!-- 图例 -->
    <div class="fl-cal-legend">
      <span v-for="g in ['S','A','B','C']" :key="g" class="fl-cal-legend-item">
        <span class="fl-cal-legend-dot" :style="{ background: gradeColor(g) }" />
        {{ g }}
      </span>
    </div>
  </section>
</template>

<style scoped>
.fl-calendar {
  max-width: 540px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: var(--sp-5);
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
  position: relative;
}
.fl-cal-cell.is-empty {
  background: transparent;
  border-color: transparent;
}

.fl-cal-day {
  font-size: var(--fs-12);
  color: var(--color-text-secondary);
}

.fl-cal-dot {
  width: 20px;
  height: 20px;
  border-radius: var(--r-sm);
  display: grid;
  place-items: center;
  font-size: 10px;
  font-weight: var(--fw-semibold);
  color: #fff;
  cursor: default;
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
</style>
