<script setup lang="ts">
/**
 * YesterdayCard · 昨日回顾卡片(打开应用时展示)。
 */

import { Clock, Target, X } from "lucide-vue-next";
import { onMounted, ref } from "vue";

import GradeBadge from "@/components/settlement/GradeBadge.vue";
import { useSettlementStore } from "@/stores/useSettlementStore";

const store = useSettlementStore();
const dismissed = ref(false);

onMounted(() => {
  store.loadYesterday();
});

function fmtMinutes(min: number): string {
  if (min < 60) return `${min} 分钟`;
  const h = Math.floor(min / 60);
  const m = min % 60;
  return m > 0 ? `${h}h${m}m` : `${h}h`;
}
</script>

<template>
  <div
    v-if="store.yesterday && !dismissed"
    class="fl-yc"
  >
    <div class="fl-yc-head">
      <span class="fl-yc-title">昨日回顾</span>
      <button class="fl-yc-close" type="button" @click="dismissed = true">
        <X :size="14" />
      </button>
    </div>

    <div class="fl-yc-body">
      <GradeBadge :grade="store.yesterday.grade" />

      <div class="fl-yc-stats">
        <div class="fl-yc-row">
          <Target :size="12" />
          <span>{{ store.yesterday.completedTasks }} / {{ store.yesterday.totalTasks }} 任务完成</span>
        </div>
        <div class="fl-yc-row">
          <Clock :size="12" />
          <span>专注 {{ fmtMinutes(store.yesterday.totalFocusMinutes) }} · {{ store.yesterday.totalPomodoros }} 个番茄</span>
        </div>
        <div v-if="store.yesterday.longestFocusTaskName" class="fl-yc-row">
          <span class="fl-yc-muted">最长专注: {{ store.yesterday.longestFocusTaskName }}</span>
        </div>
        <div v-if="store.yesterday.carriedOverCount > 0" class="fl-yc-row fl-yc-carry">
          {{ store.yesterday.carriedOverCount }} 项任务已带入今天
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.fl-yc {
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--r-lg);
  box-shadow: var(--shadow-card);
  overflow: hidden;
}

.fl-yc-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--sp-3) var(--sp-4);
  border-bottom: 1px solid var(--color-border);
}
.fl-yc-title {
  font-size: var(--fs-12);
  font-weight: var(--fw-semibold);
  color: var(--color-text-secondary);
}
.fl-yc-close {
  background: none;
  border: none;
  color: var(--color-text-muted);
  cursor: pointer;
  padding: 2px;
}

.fl-yc-body {
  display: flex;
  align-items: center;
  gap: var(--sp-4);
  padding: var(--sp-4);
}

.fl-yc-stats {
  display: flex;
  flex-direction: column;
  gap: var(--sp-1);
}
.fl-yc-row {
  display: flex;
  align-items: center;
  gap: var(--sp-2);
  font-size: var(--fs-12);
  color: var(--color-text-primary);
}
.fl-yc-muted {
  color: var(--color-text-muted);
}
.fl-yc-carry {
  color: var(--color-warning-text, var(--color-q2));
  font-weight: var(--fw-medium);
}
</style>
