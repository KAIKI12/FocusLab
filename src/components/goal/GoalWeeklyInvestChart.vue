<script setup lang="ts">
/**
 * GoalWeeklyInvestChart · 目标本周 7 柱时间投入图。
 *
 * 对齐 prototype/goals/milestones.html:566-580。
 * 数据源:useGoalStore.weeklyInvest(后端 get_goal_weekly_invest 聚合结果)。
 * 高度按本周最大分钟数归一化,今天柱高亮 + tooltip。
 */

import { computed } from "vue";

import { useGoalStore } from "@/stores/useGoalStore";

const goals = useGoalStore();

const WEEKDAY_LABELS = ["周一", "周二", "周三", "周四", "周五", "周六", "周日"];

const todayWeekday = computed(() => {
  const d = new Date().getDay(); // 0=周日..6=周六
  return d === 0 ? 6 : d - 1;    // 转换为 0=周一..6=周日
});

const chart = computed(() => {
  const inv = goals.weeklyInvest;
  if (!inv) return { cols: [], maxMinutes: 0 };
  const maxMinutes = Math.max(1, ...inv.buckets.map((b) => b.minutes));
  return {
    cols: inv.buckets.map((b) => ({
      weekday: b.weekday,
      minutes: b.minutes,
      heightPct: Math.round((b.minutes / maxMinutes) * 95), // 留 5% 上边距
      label: WEEKDAY_LABELS[b.weekday] ?? `D${b.weekday}`,
      isToday: b.weekday === todayWeekday.value,
      isFuture: b.weekday > todayWeekday.value,
    })),
    maxMinutes,
  };
});

function formatMinutes(m: number): string {
  if (m <= 0) return "0m";
  const h = Math.floor(m / 60);
  const mm = m % 60;
  if (h > 0) return mm ? `${h}h ${mm}m` : `${h}h`;
  return `${mm}m`;
}
</script>

<template>
  <div class="fl-ms-invest">
    <div class="fl-ms-invest-head">
      <span>本目标本周时间投入</span>
      <span class="fl-ms-invest-total">{{ formatMinutes(goals.weeklyInvest?.totalMinutes ?? 0) }}</span>
    </div>

    <div v-if="chart.cols.length" class="fl-ms-invest-bar">
      <div
        v-for="c in chart.cols"
        :key="c.weekday"
        class="fl-ms-invest-col"
        :class="{ 'is-today': c.isToday, 'is-future': c.isFuture }"
        :style="{ height: c.isFuture ? '0%' : c.heightPct + '%' }"
        :data-tip="`${c.label}${c.isToday ? ' · 今天' : ''} · ${formatMinutes(c.minutes)}`"
      />
    </div>
    <div v-else class="fl-ms-invest-empty">还没有本周投入数据</div>

    <div class="fl-ms-invest-axis">
      <span
        v-for="c in chart.cols"
        :key="c.weekday"
        :class="{ 'is-today': c.isToday }"
      >{{ c.label }}</span>
    </div>
  </div>
</template>

<style scoped>
.fl-ms-invest {
  margin-top: var(--sp-5);
  padding-top: var(--sp-5);
  border-top: 1px solid var(--color-divider);
  display: flex; flex-direction: column; gap: var(--sp-2);
}

.fl-ms-invest-head {
  display: flex; justify-content: space-between; align-items: baseline;
  font-size: var(--fs-12); color: var(--color-text-muted);
  text-transform: uppercase; letter-spacing: 0.5px;
  font-weight: var(--fw-medium);
}
.fl-ms-invest-total {
  color: var(--color-primary);
  font-family: var(--font-mono);
  font-weight: var(--fw-semibold);
  text-transform: none;
  font-size: var(--fs-13, 13px);
}

.fl-ms-invest-bar {
  display: flex; gap: 3px;
  height: 100px; align-items: flex-end;
  padding: var(--sp-2) 0;
}

.fl-ms-invest-col {
  flex: 1;
  min-height: 2px;
  background: var(--color-primary-soft);
  border-radius: 2px 2px 0 0;
  transition: background var(--dur-base), height var(--dur-base);
  position: relative;
  cursor: default;
}
.fl-ms-invest-col:hover { background: var(--color-primary-dark, var(--color-primary)); }
.fl-ms-invest-col.is-today { background: var(--color-primary); }
.fl-ms-invest-col.is-future {
  background: var(--color-bg-hover);
  min-height: 2px;
}
.fl-ms-invest-col:hover::after {
  content: attr(data-tip);
  position: absolute;
  bottom: calc(100% + 4px); left: 50%; transform: translateX(-50%);
  background: var(--color-text-primary);
  color: var(--color-bg-elevated);
  padding: 4px 8px;
  font-size: 11px;
  border-radius: var(--r-xs);
  white-space: nowrap;
  z-index: 1;
  pointer-events: none;
}

.fl-ms-invest-empty {
  height: 100px;
  display: grid; place-items: center;
  color: var(--color-text-muted); font-size: 12px;
}

.fl-ms-invest-axis {
  display: flex;
  font-size: 11px;
  color: var(--color-text-muted);
  padding-top: 4px;
}
.fl-ms-invest-axis span {
  flex: 1; text-align: center;
}
.fl-ms-invest-axis span.is-today {
  color: var(--color-primary);
  font-weight: var(--fw-medium);
}
</style>
