<script setup lang="ts">
/**
 * StatsView · 数据洞察 — 概览卡片 + 热力图 + 完成率趋势 + 时间分类。
 * 纯 CSS/Canvas 实现，不引入 ECharts。
 */

import { computed, onMounted, ref } from "vue";

import { invokeCmd } from "@/composables/useTauriInvoke";
import { useAIStore } from "@/stores/useAIStore";
import type { CategoryTime, GoalWeeklyInvest, HeatmapCell, StatsOverview, TrendPoint } from "@/types";

const ai = useAIStore();
const overview = ref<StatsOverview | null>(null);
const heatmap = ref<HeatmapCell[]>([]);
const trend = ref<TrendPoint[]>([]);
const categories = ref<CategoryTime[]>([]);
const goalInvests = ref<GoalWeeklyInvest[]>([]);
const weekSummary = ref("");
const loadingWeekly = ref(false);
const loading = ref(true);
const rangeDays = ref(30);

const DOW_LABELS = ["日", "一", "二", "三", "四", "五", "六"];
const HOUR_LABELS = Array.from({ length: 24 }, (_, i) => `${i}`);

async function loadAll() {
  loading.value = true;
  try {
    const days = rangeDays.value;
    const [o, h, t, c, gi] = await Promise.all([
      invokeCmd<StatsOverview>("get_stats_overview", { days }),
      invokeCmd<HeatmapCell[]>("get_focus_heatmap", { days }),
      invokeCmd<TrendPoint[]>("get_completion_trend", { days }),
      invokeCmd<CategoryTime[]>("get_time_by_category", { days }),
      invokeCmd<GoalWeeklyInvest[]>("list_goal_weekly_invests"),
    ]);
    overview.value = o;
    heatmap.value = h;
    trend.value = t;
    categories.value = c;
    goalInvests.value = gi;
  } finally {
    loading.value = false;
  }
}

onMounted(loadAll);

// ---------- 热力图 ----------

/** 7×24 矩阵, heatmapGrid[dow][hour] = minutes */
const heatmapGrid = computed(() => {
  const grid: number[][] = Array.from({ length: 7 }, () => Array(24).fill(0));
  for (const c of heatmap.value) {
    grid[c.dayOfWeek][c.hour] = c.minutes;
  }
  return grid;
});

const heatmapMax = computed(() => {
  let max = 1;
  for (const c of heatmap.value) if (c.minutes > max) max = c.minutes;
  return max;
});

function heatColor(minutes: number): string {
  if (minutes === 0) return "var(--color-bg-subtle)";
  const intensity = Math.min(minutes / heatmapMax.value, 1);
  const alpha = 0.15 + intensity * 0.85;
  return `color-mix(in srgb, var(--color-primary) ${Math.round(alpha * 100)}%, transparent)`;
}

// ---------- 趋势图 ----------

const trendMax = computed(() => {
  let max = 1;
  for (const p of trend.value) if (p.focusMinutes > max) max = p.focusMinutes;
  return max;
});

// ---------- 分类 ----------

const categoryTotal = computed(() =>
  categories.value.reduce((sum, c) => sum + c.minutes, 0) || 1,
);

// ---------- 黄金专注时段 ----------

/** 按小时聚合 7 天总分钟,扫出连续活跃段,取 top 3 */
interface PeakWindow {
  startHour: number; // inclusive
  endHour: number;   // exclusive
  totalMin: number;
}

const peakWindows = computed<PeakWindow[]>(() => {
  const hourTotals = Array.from({ length: 24 }, (_, h) =>
    heatmapGrid.value.reduce((sum, row) => sum + row[h], 0),
  );
  const wins: PeakWindow[] = [];
  let cur: PeakWindow | null = null;
  for (let h = 0; h < 24; h++) {
    if (hourTotals[h] > 0) {
      if (!cur) cur = { startHour: h, endHour: h + 1, totalMin: hourTotals[h] };
      else { cur.endHour = h + 1; cur.totalMin += hourTotals[h]; }
    } else if (cur) {
      wins.push(cur);
      cur = null;
    }
  }
  if (cur) wins.push(cur);
  return wins.sort((a, b) => b.totalMin - a.totalMin).slice(0, 3);
});

function fmtHour(h: number): string {
  return `${String(h).padStart(2, "0")}:00`;
}

function labelForWindow(w: PeakWindow): { emoji: string; label: string } {
  const s = w.startHour;
  if (s < 6) return { emoji: "🌌", label: "深夜时段" };
  if (s < 11) return { emoji: "🌅", label: "早晨清晰时段" };
  if (s < 14) return { emoji: "☀️", label: "正午专注时段" };
  if (s < 17) return { emoji: "☕", label: "午后恢复时段" };
  if (s < 20) return { emoji: "🌇", label: "傍晚时段" };
  return { emoji: "🌙", label: "晚间深度时段" };
}

// ---------- 长线目标投入 ----------

const GI_COLORS = [
  "var(--color-primary)",
  "var(--color-q2)",
  "var(--color-gold)",
  "var(--color-q3)",
  "var(--color-q1)",
  "var(--color-text-muted)",
];

const activeGoalInvests = computed(() =>
  goalInvests.value.filter((g) => g.totalMinutes > 0),
);

const maxGoalInvest = computed(() =>
  activeGoalInvests.value.reduce((m, g) => Math.max(m, g.totalMinutes), 0) || 1,
);

function giColor(i: number): string {
  return GI_COLORS[i % GI_COLORS.length];
}

const QUADRANT_LABELS: Record<string, string> = {
  important_urgent: "紧急重要",
  important_not_urgent: "重要不紧急",
  not_important_urgent: "紧急不重要",
  not_important_not_urgent: "不紧急不重要",
  unknown: "未分类",
};

const QUADRANT_COLORS: Record<string, string> = {
  important_urgent: "var(--color-q1)",
  important_not_urgent: "var(--color-q2)",
  not_important_urgent: "var(--color-q3)",
  not_important_not_urgent: "var(--color-q4)",
  unknown: "var(--color-text-muted)",
};

function fmtMin(m: number): string {
  if (m < 60) return `${m} 分钟`;
  const h = Math.floor(m / 60);
  const r = m % 60;
  return r > 0 ? `${h}h${r}m` : `${h}h`;
}

function changeRange(days: number) {
  rangeDays.value = days;
  loadAll();
}
</script>

<template>
  <section class="fl-stats">
    <header class="fl-stats-head">
      <div>
        <h1>数据洞察</h1>
        <p class="fl-stats-sub">专注模式 · 完成趋势 · 时间分布</p>
      </div>
      <div class="fl-range-btns">
        <button
          v-for="d in [7, 30, 90]"
          :key="d"
          class="fl-range-btn"
          :class="{ 'is-active': rangeDays === d }"
          type="button"
          @click="changeRange(d)"
        >
          {{ d }} 天
        </button>
      </div>
    </header>

    <div v-if="loading" class="fl-stats-loading">载入中…</div>

    <template v-else>
      <!-- 概览卡片 -->
      <div v-if="overview" class="fl-overview">
        <div class="fl-ov-card">
          <span class="fl-ov-num">{{ fmtMin(overview.totalFocusMinutes) }}</span>
          <span class="fl-ov-label">总专注</span>
        </div>
        <div class="fl-ov-card">
          <span class="fl-ov-num">{{ overview.totalPomodoros }}</span>
          <span class="fl-ov-label">番茄钟</span>
        </div>
        <div class="fl-ov-card">
          <span class="fl-ov-num">{{ overview.totalTasksCompleted }}</span>
          <span class="fl-ov-label">完成任务</span>
        </div>
        <div class="fl-ov-card">
          <span class="fl-ov-num">{{ Math.round(overview.avgDailyFocus) }}m</span>
          <span class="fl-ov-label">日均专注</span>
        </div>
        <div class="fl-ov-card">
          <span class="fl-ov-num">{{ overview.bestGradeCount }}</span>
          <span class="fl-ov-label">S 级</span>
        </div>
        <div class="fl-ov-card">
          <span class="fl-ov-num">{{ overview.currentStreak }} 天</span>
          <span class="fl-ov-label">连续结算</span>
        </div>
      </div>

      <!-- AI 周度小结 -->
      <div class="fl-section">
        <div class="fl-ai-weekly">
          <div class="fl-ai-weekly-head">
            <div class="fl-ai-weekly-avatar">✨</div>
            <div>
              <div class="fl-ai-weekly-title">AI 周度小结</div>
              <div v-if="loadingWeekly" class="fl-ai-weekly-text">正在生成…</div>
              <div v-else-if="weekSummary" class="fl-ai-weekly-text">{{ weekSummary }}</div>
              <div v-else class="fl-ai-weekly-text">点击生成本周总结</div>
            </div>
          </div>
          <button
            v-if="!weekSummary && !loadingWeekly"
            class="fl-ai-weekly-btn"
            @click="loadingWeekly = true; ai.weeklySummary().then(r => weekSummary = r).catch(() => weekSummary = '暂时无法生成，请先配置 AI 服务').finally(() => loadingWeekly = false)"
          >
            生成周报
          </button>
        </div>
      </div>

      <!-- 专注热力图 -->
      <div class="fl-section">
        <h2 class="fl-section-title">专注时段热力图</h2>
        <div class="fl-heatmap">
          <div class="fl-hm-corner" />
          <div v-for="h in HOUR_LABELS" :key="'h'+h" class="fl-hm-hour">{{ h }}</div>
          <template v-for="(dow, di) in DOW_LABELS" :key="'d'+di">
            <div class="fl-hm-dow">{{ dow }}</div>
            <div
              v-for="hi in 24"
              :key="`${di}-${hi}`"
              class="fl-hm-cell"
              :style="{ background: heatColor(heatmapGrid[di][hi - 1]) }"
              :title="`${dow} ${hi - 1}:00 — ${heatmapGrid[di][hi - 1]} 分钟`"
            />
          </template>
        </div>
      </div>

      <!-- 完成率趋势 -->
      <div class="fl-section">
        <h2 class="fl-section-title">完成率趋势</h2>
        <div v-if="trend.length" class="fl-trend">
          <div class="fl-trend-chart">
            <div
              v-for="(p, i) in trend"
              :key="i"
              class="fl-trend-bar"
              :style="{ height: `${Math.max(p.focusMinutes / trendMax * 100, 4)}%` }"
              :title="`${p.date}\n完成率 ${Math.round(p.completionRate * 100)}%\n专注 ${p.focusMinutes}m`"
            >
              <span class="fl-trend-rate">{{ Math.round(p.completionRate * 100) }}</span>
            </div>
          </div>
          <div class="fl-trend-labels">
            <span v-for="(p, i) in trend" :key="i" class="fl-trend-date">
              {{ p.date.slice(5) }}
            </span>
          </div>
        </div>
        <div v-else class="fl-empty">暂无结算数据</div>
      </div>

      <!-- 时间分类 -->
      <div class="fl-section">
        <h2 class="fl-section-title">专注时间分布</h2>
        <div v-if="categories.length" class="fl-categories">
          <div class="fl-cat-bar-container">
            <div
              v-for="c in categories"
              :key="c.quadrant"
              class="fl-cat-segment"
              :style="{
                width: `${(c.minutes / categoryTotal) * 100}%`,
                background: QUADRANT_COLORS[c.quadrant] || 'var(--color-text-muted)',
              }"
              :title="`${QUADRANT_LABELS[c.quadrant] || c.quadrant}: ${fmtMin(c.minutes)}`"
            />
          </div>
          <ul class="fl-cat-legend">
            <li v-for="c in categories" :key="c.quadrant" class="fl-cat-item">
              <span
                class="fl-cat-dot"
                :style="{ background: QUADRANT_COLORS[c.quadrant] || 'var(--color-text-muted)' }"
              />
              <span class="fl-cat-label">{{ QUADRANT_LABELS[c.quadrant] || c.quadrant }}</span>
              <span class="fl-cat-value">{{ fmtMin(c.minutes) }} · {{ c.sessionCount }} 次</span>
            </li>
          </ul>
        </div>
        <div v-else class="fl-empty">暂无分类数据</div>
      </div>

      <!-- 黄金专注时段 -->
      <div class="fl-section">
        <h2 class="fl-section-title">黄金专注时段</h2>
        <div v-if="peakWindows.length" class="fl-peak-list">
          <div
            v-for="(w, i) in peakWindows"
            :key="`${w.startHour}-${w.endHour}`"
            class="fl-peak-row"
            :class="{ 'is-top': i === 0 }"
          >
            <span class="fl-peak-emoji">{{ labelForWindow(w).emoji }}</span>
            <div class="fl-peak-body">
              <div class="fl-peak-time">{{ fmtHour(w.startHour) }} — {{ fmtHour(w.endHour) }}</div>
              <div class="fl-peak-desc">
                {{ labelForWindow(w).label }} · 专注 {{ fmtMin(w.totalMin) }}
              </div>
            </div>
            <span class="fl-peak-rank" :class="{ 'is-top': i === 0 }">#{{ i + 1 }}</span>
          </div>
        </div>
        <div v-else class="fl-empty">暂无时段数据 — 完成几个专注会话后会显示你的黄金时段</div>
      </div>

      <!-- 长线目标投入 -->
      <div class="fl-section">
        <h2 class="fl-section-title">长线目标投入 · 本周</h2>
        <div v-if="activeGoalInvests.length" class="fl-gi-list">
          <div v-for="(g, i) in activeGoalInvests" :key="g.goalId" class="fl-gi-row">
            <span class="fl-gi-dot" :style="{ background: giColor(i) }" />
            <span class="fl-gi-name" :title="g.goalName">{{ g.goalName }}</span>
            <div class="fl-gi-bar">
              <div
                class="fl-gi-fill"
                :style="{
                  width: `${(g.totalMinutes / maxGoalInvest) * 100}%`,
                  background: giColor(i),
                }"
              />
            </div>
            <span class="fl-gi-time">{{ fmtMin(g.totalMinutes) }}</span>
          </div>
        </div>
        <div v-else class="fl-empty">本周还没有目标时间投入</div>
      </div>
    </template>
  </section>
</template>

<style scoped>
.fl-stats {
  max-width: 720px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: var(--sp-6);
}

.fl-stats-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
}
.fl-stats-head h1 {
  font-size: var(--fs-24);
  font-weight: var(--fw-semibold);
  margin: 0;
  color: var(--color-text-primary);
}
.fl-stats-sub {
  margin: var(--sp-1) 0 0;
  color: var(--color-text-secondary);
  font-size: var(--fs-12);
}

.fl-range-btns {
  display: flex;
  gap: 2px;
  background: var(--color-bg-subtle);
  padding: 3px;
  border-radius: var(--r-md);
}
.fl-range-btn {
  padding: var(--sp-1) var(--sp-3);
  border: none;
  background: transparent;
  border-radius: var(--r-sm);
  color: var(--color-text-secondary);
  font-size: var(--fs-12);
  cursor: pointer;
}
.fl-range-btn.is-active {
  background: var(--color-bg-elevated);
  color: var(--color-text-primary);
  box-shadow: var(--shadow-card);
}

.fl-stats-loading {
  text-align: center;
  padding: var(--sp-10);
  color: var(--color-text-muted);
}

/* ---------- 概览 ---------- */
.fl-overview {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: var(--sp-3);
}
.fl-ov-card {
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--r-md);
  padding: var(--sp-4);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--sp-1);
}
.fl-ov-num {
  font-size: var(--fs-20, 20px);
  font-weight: var(--fw-semibold);
  color: var(--color-primary);
}
.fl-ov-label {
  font-size: var(--fs-12);
  color: var(--color-text-muted);
}

/* ---------- 通用 section ---------- */
.fl-section {
  display: flex;
  flex-direction: column;
  gap: var(--sp-3);
}
.fl-section-title {
  font-size: var(--fs-14);
  font-weight: var(--fw-semibold);
  margin: 0;
  color: var(--color-text-primary);
}

/* ---------- 热力图 ---------- */
.fl-heatmap {
  display: grid;
  grid-template-columns: 28px repeat(24, 1fr);
  gap: 2px;
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--r-md);
  padding: var(--sp-3);
}
.fl-hm-corner { /* 空角 */ }
.fl-hm-hour {
  font-size: 9px;
  color: var(--color-text-muted);
  text-align: center;
}
.fl-hm-dow {
  font-size: 10px;
  color: var(--color-text-muted);
  display: flex;
  align-items: center;
  justify-content: center;
}
.fl-hm-cell {
  aspect-ratio: 1;
  border-radius: 2px;
  cursor: default;
  min-height: 12px;
}

/* ---------- 趋势 ---------- */
.fl-trend {
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--r-md);
  padding: var(--sp-4);
}
.fl-trend-chart {
  display: flex;
  align-items: flex-end;
  gap: 2px;
  height: 120px;
}
.fl-trend-bar {
  flex: 1;
  background: var(--color-primary);
  border-radius: 2px 2px 0 0;
  position: relative;
  min-width: 8px;
  cursor: default;
  transition: background var(--dur-fast) var(--ease-smooth);
}
.fl-trend-bar:hover {
  background: var(--color-primary-dark);
}
.fl-trend-rate {
  position: absolute;
  top: -14px;
  left: 50%;
  transform: translateX(-50%);
  font-size: 8px;
  color: var(--color-text-muted);
  white-space: nowrap;
}
.fl-trend-labels {
  display: flex;
  gap: 2px;
  margin-top: var(--sp-1);
}
.fl-trend-date {
  flex: 1;
  font-size: 8px;
  color: var(--color-text-muted);
  text-align: center;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* ---------- 分类 ---------- */
.fl-categories {
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--r-md);
  padding: var(--sp-4);
  display: flex;
  flex-direction: column;
  gap: var(--sp-3);
}
.fl-cat-bar-container {
  display: flex;
  height: 24px;
  border-radius: var(--r-sm);
  overflow: hidden;
}
.fl-cat-segment {
  min-width: 4px;
  transition: width var(--dur-base) var(--ease-smooth);
}
.fl-cat-legend {
  list-style: none;
  padding: 0;
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: var(--sp-2);
}
.fl-cat-item {
  display: flex;
  align-items: center;
  gap: var(--sp-2);
  font-size: var(--fs-12);
}
.fl-cat-dot {
  width: 10px;
  height: 10px;
  border-radius: 2px;
  flex-shrink: 0;
}
.fl-cat-label {
  color: var(--color-text-primary);
  flex: 1;
}
.fl-cat-value {
  color: var(--color-text-muted);
}

.fl-empty {
  text-align: center;
  padding: var(--sp-6);
  color: var(--color-text-muted);
  font-size: var(--fs-14);
  background: var(--color-bg-subtle);
  border-radius: var(--r-md);
}

/* ---------- 黄金专注时段 ---------- */
.fl-peak-list {
  display: flex;
  flex-direction: column;
  gap: var(--sp-2);
}
.fl-peak-row {
  display: flex;
  align-items: center;
  gap: var(--sp-2);
  padding: 10px 12px;
  background: var(--color-bg-subtle);
  border-radius: var(--r-sm);
}
.fl-peak-row.is-top {
  background: var(--color-primary-soft);
}
.fl-peak-emoji {
  font-size: 20px;
  flex-shrink: 0;
}
.fl-peak-body {
  flex: 1;
  min-width: 0;
}
.fl-peak-time {
  font-size: var(--fs-13, 13px);
  font-weight: var(--fw-semibold);
  color: var(--color-text-primary);
}
.fl-peak-row.is-top .fl-peak-time {
  color: var(--color-primary-dark, var(--color-primary));
}
.fl-peak-desc {
  font-size: 11px;
  color: var(--color-text-secondary);
  margin-top: 2px;
}
.fl-peak-rank {
  font-size: var(--fs-12);
  padding: 2px var(--sp-2);
  border-radius: var(--r-sm);
  background: var(--color-bg-elevated);
  color: var(--color-text-muted);
  flex-shrink: 0;
}
.fl-peak-rank.is-top {
  background: var(--color-primary);
  color: #fff;
}

/* ---------- 长线目标投入 ---------- */
.fl-gi-list {
  display: flex;
  flex-direction: column;
  gap: var(--sp-2);
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--r-md);
  padding: var(--sp-4);
}
.fl-gi-row {
  display: grid;
  grid-template-columns: 10px minmax(80px, 120px) 1fr auto;
  align-items: center;
  gap: var(--sp-2);
  font-size: var(--fs-12);
}
.fl-gi-dot {
  width: 10px;
  height: 10px;
  border-radius: 2px;
}
.fl-gi-name {
  color: var(--color-text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.fl-gi-bar {
  height: 6px;
  background: var(--color-bg-subtle);
  border-radius: 3px;
  overflow: hidden;
}
.fl-gi-fill {
  height: 100%;
  border-radius: 3px;
  transition: width var(--dur-base) var(--ease-smooth);
}
.fl-gi-time {
  color: var(--color-text-muted);
  font-variant-numeric: tabular-nums;
}

/* AI 周度小结 */
.fl-ai-weekly {
  background: linear-gradient(180deg, var(--color-primary-soft), var(--color-bg-elevated));
  border: 1px solid color-mix(in srgb, var(--color-primary) 20%, var(--color-border));
  border-radius: var(--r-md); padding: var(--sp-4);
  display: flex; flex-direction: column; gap: var(--sp-3);
}
.fl-ai-weekly-head { display: flex; gap: var(--sp-3); }
.fl-ai-weekly-avatar {
  width: 32px; height: 32px; border-radius: var(--r-sm); flex-shrink: 0;
  background: var(--color-primary-soft); display: grid; place-items: center; font-size: 16px;
}
.fl-ai-weekly-title { font-weight: var(--fw-medium); margin-bottom: 2px; font-size: var(--fs-14); }
.fl-ai-weekly-text { font-size: var(--fs-12); color: var(--color-text-secondary); line-height: 1.6; }
.fl-ai-weekly-btn {
  align-self: flex-start; padding: var(--sp-2) var(--sp-4);
  border-radius: var(--r-md); border: none;
  background: var(--color-primary); color: #fff;
  font-size: var(--fs-12); cursor: pointer;
}
</style>
