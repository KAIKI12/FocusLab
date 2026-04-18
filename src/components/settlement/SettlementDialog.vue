<script setup lang="ts">
/**
 * SettlementDialog · 日结算弹窗(3 步 flow)。
 *
 * Step 1: 数据摘要(任务/专注时长/番茄数)
 * Step 2: 评级展示 + AI 叙事占位
 * Step 3: 完成(关闭)
 */

import { computed, ref } from "vue";

import GradeBadge from "@/components/settlement/GradeBadge.vue";
import { useSettlementStore } from "@/stores/useSettlementStore";

const store = useSettlementStore();
const step = ref(1);

const s = computed(() => store.settlement);

function nextStep() {
  if (step.value < 3) {
    step.value++;
  } else {
    step.value = 1;
    store.closeDialog();
  }
}

function fmtMinutes(min: number): string {
  if (min < 60) return `${min} 分钟`;
  const h = Math.floor(min / 60);
  const m = min % 60;
  return m > 0 ? `${h} 小时 ${m} 分` : `${h} 小时`;
}

const rateText = computed(() => {
  if (!s.value) return "0%";
  return `${Math.round(s.value.completionRate * 100)}%`;
});
</script>

<template>
  <Transition name="fl-fade">
    <div
      v-if="store.showDialog && s"
      class="fl-sd-mask"
      role="presentation"
    >
      <div class="fl-sd-card" role="dialog" aria-modal="true">
        <!-- Step 1: 数据摘要 -->
        <template v-if="step === 1">
          <h2 class="fl-sd-title">今日回顾</h2>
          <div class="fl-sd-stats">
            <div class="fl-sd-stat">
              <span class="fl-sd-num">{{ s.completedTasks }}</span>
              <span class="fl-sd-label">完成任务</span>
            </div>
            <div class="fl-sd-stat">
              <span class="fl-sd-num">{{ fmtMinutes(s.totalFocusMinutes) }}</span>
              <span class="fl-sd-label">专注时长</span>
            </div>
            <div class="fl-sd-stat">
              <span class="fl-sd-num">{{ s.totalPomodoros }}</span>
              <span class="fl-sd-label">番茄钟</span>
            </div>
          </div>
          <p class="fl-sd-detail">
            计划 {{ s.totalTasks - s.extraTasks }} 项,完成率 {{ rateText }}
            <template v-if="s.extraTasks > 0">
              · 额外完成 {{ s.extraTasks }} 项
            </template>
          </p>
        </template>

        <!-- Step 2: 评级 -->
        <template v-else-if="step === 2">
          <h2 class="fl-sd-title">评级</h2>
          <div class="fl-sd-grade-area">
            <GradeBadge :grade="s.grade as 'S' | 'A' | 'B' | 'C'" />
            <div class="fl-sd-grade-text">
              <template v-if="s.grade === 'S'">太棒了!完成全部计划还做了额外任务</template>
              <template v-else-if="s.grade === 'A'">完美!今天计划全部完成</template>
              <template v-else-if="s.grade === 'B'">不错,完成了大部分计划</template>
              <template v-else>计划赶不上变化,调整一下明天继续</template>
            </div>
          </div>
          <div v-if="s.aiSummary" class="fl-sd-ai">
            {{ s.aiSummary }}
          </div>
        </template>

        <!-- Step 3: 完成 -->
        <template v-else>
          <h2 class="fl-sd-title">结算完成</h2>
          <p class="fl-sd-done">
            未完成的计划任务已自动带入明天。休息好,明天继续加油!
          </p>
        </template>

        <button class="fl-sd-next" type="button" @click="nextStep">
          {{ step < 3 ? '下一步' : '关闭' }}
        </button>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.fl-sd-mask {
  position: fixed;
  inset: 0;
  background: color-mix(in srgb, var(--color-text-primary) 36%, transparent);
  display: grid;
  place-items: center;
  z-index: var(--z-modal);
  padding: var(--sp-4);
}

.fl-sd-card {
  width: min(440px, 100%);
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--r-lg);
  box-shadow: var(--shadow-modal);
  padding: var(--sp-6);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--sp-5);
  text-align: center;
}

.fl-sd-title {
  margin: 0;
  font-size: var(--fs-20, 20px);
  font-weight: var(--fw-semibold);
  color: var(--color-text-primary);
}

.fl-sd-stats {
  display: flex;
  gap: var(--sp-5);
}
.fl-sd-stat {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--sp-1);
}
.fl-sd-num {
  font-size: var(--fs-20, 20px);
  font-weight: var(--fw-semibold);
  color: var(--color-primary);
}
.fl-sd-label {
  font-size: var(--fs-12);
  color: var(--color-text-muted);
}

.fl-sd-detail {
  margin: 0;
  font-size: var(--fs-12);
  color: var(--color-text-secondary);
}

.fl-sd-grade-area {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--sp-3);
}
.fl-sd-grade-text {
  font-size: var(--fs-14);
  color: var(--color-text-secondary);
  max-width: 300px;
}

.fl-sd-ai {
  font-size: var(--fs-12);
  color: var(--color-text-secondary);
  padding: var(--sp-3) var(--sp-4);
  background: var(--color-bg-subtle);
  border-radius: var(--r-md);
  text-align: left;
  line-height: 1.6;
}

.fl-sd-done {
  margin: 0;
  font-size: var(--fs-14);
  color: var(--color-text-secondary);
}

.fl-sd-next {
  padding: var(--sp-3) var(--sp-6);
  border-radius: var(--r-md);
  border: none;
  background: var(--color-primary);
  color: var(--color-text-on-primary);
  font-size: var(--fs-14);
  font-weight: var(--fw-medium);
  cursor: pointer;
  transition: background var(--dur-fast) var(--ease-smooth);
}
.fl-sd-next:hover {
  background: var(--color-primary-dark);
}

.fl-fade-enter-active,
.fl-fade-leave-active {
  transition: opacity var(--dur-base) var(--ease-smooth);
}
.fl-fade-enter-from,
.fl-fade-leave-to {
  opacity: 0;
}
</style>
