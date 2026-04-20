<script setup lang="ts">
/**
 * AvailableTimePanel · 本周每日可用工作时间预估。
 * 对齐 prototype/screens/calendar.html:692。
 *
 * 算法:每天给一个 base(工作日 9h / 周末 5h),减去该天的固定日程总时长,
 * 得到"理论上可用于深度工作"的小时数。仅展示参考,不联动 DTA。
 */

import { computed } from "vue";

import { useFixedSchedule } from "@/composables/useFixedSchedule";

const { totalMinutesForWeekday } = useFixedSchedule();

const WEEKDAY_LABELS: Record<number, string> = {
  0: "周日", 1: "周一", 2: "周二", 3: "周三", 4: "周四", 5: "周五", 6: "周六",
};
const DISPLAY_ORDER = [1, 2, 3, 4, 5, 6, 0];
const WEEKEND = new Set([0, 6]);
const todayWeekday = new Date().getDay();

function baseMinutes(weekday: number): number {
  return WEEKEND.has(weekday) ? 5 * 60 : 9 * 60;
}

interface Row {
  weekday: number;
  availableMin: number;
  baseMin: number;
}

const rows = computed<Row[]>(() =>
  DISPLAY_ORDER.map((w) => {
    const base = baseMinutes(w);
    const fixed = totalMinutesForWeekday(w);
    return { weekday: w, availableMin: Math.max(0, base - fixed), baseMin: base };
  }),
);

const totalHours = computed(() =>
  (rows.value.reduce((sum, r) => sum + r.availableMin, 0) / 60).toFixed(1),
);

function fmtHour(min: number): string {
  const h = min / 60;
  return h >= 10 ? `${h.toFixed(0)}h` : `${h.toFixed(1)}h`;
}
</script>

<template>
  <div class="fl-at-panel">
    <div class="fl-at-head">
      <span class="fl-at-title">本周可用工作时间</span>
      <span class="fl-at-total">{{ totalHours }}h</span>
    </div>
    <p class="fl-at-hint">去掉固定日程 · 理论上可用于深度工作的时间</p>

    <div class="fl-at-rows">
      <div
        v-for="r in rows"
        :key="r.weekday"
        class="fl-at-row"
        :class="{
          'is-today': r.weekday === todayWeekday,
          'is-weekend': r.weekday === 0 || r.weekday === 6,
        }"
      >
        <span class="fl-at-day">
          {{ WEEKDAY_LABELS[r.weekday] }}<template v-if="r.weekday === todayWeekday"> · 今天</template>
        </span>
        <div class="fl-at-bar">
          <div
            class="fl-at-fill"
            :style="{ width: `${Math.round((r.availableMin / r.baseMin) * 100)}%` }"
          />
        </div>
        <span class="fl-at-value">{{ fmtHour(r.availableMin) }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.fl-at-panel {
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--r-lg);
  padding: var(--sp-4);
  display: flex;
  flex-direction: column;
  gap: var(--sp-2);
}

.fl-at-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.fl-at-title {
  font-size: var(--fs-14);
  font-weight: var(--fw-semibold);
  color: var(--color-text-primary);
}
.fl-at-total {
  font-family: var(--font-mono);
  font-weight: var(--fw-bold);
  color: var(--color-primary);
  font-size: var(--fs-14);
}
.fl-at-hint {
  font-size: 11px;
  color: var(--color-text-muted);
  margin: 0 0 var(--sp-2);
  line-height: 1.5;
}

.fl-at-rows {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.fl-at-row {
  display: grid;
  grid-template-columns: 64px 1fr auto;
  align-items: center;
  gap: var(--sp-2);
  padding: 4px 6px;
  border-radius: var(--r-sm);
  font-size: 11px;
}
.fl-at-row.is-today {
  background: var(--color-primary-soft);
}

.fl-at-day {
  color: var(--color-text-secondary);
}
.fl-at-row.is-today .fl-at-day {
  color: var(--color-primary);
  font-weight: var(--fw-semibold);
}
.fl-at-row.is-weekend:not(.is-today) .fl-at-day {
  color: var(--color-warning-text, var(--color-warning));
}

.fl-at-bar {
  height: 6px;
  background: var(--color-bg-hover);
  border-radius: 3px;
  overflow: hidden;
  min-width: 0;
}
.fl-at-fill {
  height: 100%;
  background: var(--color-primary);
  border-radius: 3px;
  transition: width var(--dur-base) var(--ease-smooth);
}

.fl-at-value {
  font-family: var(--font-mono);
  font-weight: var(--fw-medium);
  color: var(--color-text-primary);
  font-size: 11px;
  min-width: 32px;
  text-align: right;
}
</style>
