<script setup lang="ts">
/**
 * MicroReview · 任务微复盘卡片 — 对齐 prototype/screens/micro-review.html。
 *
 * 4 种触发场景 (由调用方传 scenario):
 *   - deviation: 时间偏差 > 30% (必弹)
 *   - q1:        紧急重要任务完成 (必弹,留痕关键节点)
 *   - milestone: 关联里程碑子任务完成 (必弹,进科研日志)
 *   - (静默):    Q3/Q4 事务性 / 偏差 < 30% / 日内已弹 ≥3 次 — 调用方直接不渲染
 */

import { computed, ref } from "vue";

const props = withDefaults(defineProps<{
  visible: boolean;
  taskName: string;
  estimatedMinutes: number | null;
  actualMinutes: number;
  scenario?: "deviation" | "q1" | "milestone";
  milestoneName?: string | null;
}>(), {
  scenario: "deviation",
  milestoneName: null,
});

const emit = defineEmits<{
  close: [];
  submit: [data: { reason: string; note: string; scenario: string }];
}>();

const reason = ref("");
const note = ref("");

const deviation = computed(() => {
  if (!props.estimatedMinutes) return null;
  return Math.round(((props.actualMinutes - props.estimatedMinutes) / props.estimatedMinutes) * 100);
});

const REASONS_DEVIATION = [
  { value: "harder", label: "📚 比预想难" },
  { value: "interrupted", label: "🔔 被打断了" },
  { value: "scope", label: "➕ 做多了" },
  { value: "distracted", label: "📱 走神了" },
];

const REASONS_Q1 = [
  { value: "faster", label: "⚡ 可以更快" },
  { value: "ok", label: "✓ 节奏刚好" },
  { value: "batch", label: "🎯 下次批量处理" },
  { value: "improve", label: "🤷 可以改进" },
];

const chipOptions = computed(() => props.scenario === "q1" ? REASONS_Q1 : REASONS_DEVIATION);

const headerTitle = computed(() => {
  switch (props.scenario) {
    case "q1": return "📝 快速复盘 · Q1 任务";
    case "milestone": return "📝 里程碑进度 +1";
    default: return "📝 快速复盘";
  }
});

const notePlaceholder = computed(() => {
  switch (props.scenario) {
    case "q1": return "结果 / 备注 (可选) — 导师怎么说?";
    case "milestone": return "有什么发现值得记到科研日志里?";
    default: return "一句话经验 (可选)";
  }
});

/** 是否渲染 chip 组: deviation 场景只在偏差绝对值 > 30% 才出,q1 始终出,milestone 不出 */
const showReasons = computed(() => {
  if (props.scenario === "q1") return true;
  if (props.scenario === "milestone") return false;
  return deviation.value !== null && Math.abs(deviation.value) > 30;
});

const showTimeBar = computed(() => props.scenario !== "milestone" && deviation.value !== null);

function onSubmit() {
  emit("submit", { reason: reason.value, note: note.value, scenario: props.scenario });
  reason.value = "";
  note.value = "";
}

function onSkip() {
  emit("close");
  reason.value = "";
  note.value = "";
}
</script>

<template>
  <Transition name="fl-fade">
    <div v-if="visible" class="fl-mr-mask" @click.self="onSkip">
      <div class="fl-mr-card" :class="[`is-${scenario}`]">
        <h3>{{ headerTitle }}</h3>
        <div class="fl-mr-task">
          {{ taskName }}
          <span v-if="scenario === 'q1'" class="fl-mr-ctx">🔴 紧急重要</span>
          <span v-else-if="scenario === 'milestone' && milestoneName" class="fl-mr-ctx">🎯 {{ milestoneName }}</span>
        </div>

        <!-- 时间偏差条 -->
        <div v-if="showTimeBar" class="fl-mr-time">
          <div class="fl-mr-time-row">
            <span>预估</span>
            <span class="fl-mr-val">{{ estimatedMinutes }}m</span>
          </div>
          <span class="fl-mr-arrow">→</span>
          <div class="fl-mr-time-row">
            <span>实际</span>
            <span class="fl-mr-val">{{ actualMinutes }}m</span>
          </div>
          <span
            v-if="deviation !== null"
            class="fl-mr-badge"
            :class="{ 'is-over': deviation > 0, 'is-under': deviation < 0 }"
          >
            {{ deviation > 0 ? '+' : '' }}{{ deviation }}%
          </span>
        </div>

        <!-- 里程碑场景: 显示关联信息 -->
        <div v-if="scenario === 'milestone'" class="fl-mr-ms-banner">
          <span class="fl-mr-ms-label">里程碑推进</span>
          <span class="fl-mr-ms-val">{{ actualMinutes }}m 投入</span>
        </div>

        <!-- 经验 chip -->
        <div v-if="showReasons" class="fl-mr-reasons">
          <div class="fl-mr-label">
            {{ scenario === 'q1' ? '下次类似任务的经验?' : '是什么原因?' }}
          </div>
          <div class="fl-mr-chips">
            <button
              v-for="r in chipOptions"
              :key="r.value"
              class="fl-mr-chip"
              :class="{ 'is-selected': reason === r.value }"
              @click="reason = reason === r.value ? '' : r.value"
            >
              {{ r.label }}
            </button>
          </div>
        </div>

        <!-- 笔记 -->
        <div class="fl-mr-note">
          <textarea
            v-if="scenario === 'milestone'"
            v-model="note"
            rows="3"
            :placeholder="notePlaceholder"
            maxlength="300"
          />
          <input
            v-else
            v-model="note"
            type="text"
            :placeholder="notePlaceholder"
            maxlength="120"
          />
        </div>

        <div class="fl-mr-foot">
          <button class="fl-mr-btn fl-mr-ghost" @click="onSkip">跳过</button>
          <button class="fl-mr-btn fl-mr-primary" @click="onSubmit">
            {{ scenario === 'milestone' ? '保存 · 继续' : '记录' }}
          </button>
        </div>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.fl-mr-mask {
  position: fixed; inset: 0;
  background: color-mix(in srgb, var(--color-text-primary) 28%, transparent);
  display: grid; place-items: center;
  z-index: var(--z-modal); padding: var(--sp-4);
}
.fl-mr-card {
  width: min(420px, 100%);
  max-height: calc(100vh - 32px);
  overflow-y: auto;
  background: var(--color-bg-elevated); border: 1px solid var(--color-border);
  border-radius: var(--r-lg); box-shadow: var(--shadow-modal);
  padding: var(--sp-5); display: flex; flex-direction: column; gap: var(--sp-4);
}
.fl-mr-card.is-q1 { border-top: 3px solid var(--color-q1, #FF4D4F); }
.fl-mr-card.is-milestone { border-top: 3px solid var(--color-primary); }
.fl-mr-card h3 { font-size: var(--fs-16); font-weight: var(--fw-semibold); margin: 0; }
.fl-mr-task {
  font-size: var(--fs-14); color: var(--color-text-primary); font-weight: var(--fw-medium);
  display: flex; align-items: center; gap: var(--sp-2); flex-wrap: wrap;
}
.fl-mr-ctx {
  font-size: 11px; color: var(--color-text-secondary); font-weight: var(--fw-normal);
  padding: 2px 8px; background: var(--color-bg-subtle); border-radius: var(--r-pill);
}

.fl-mr-time {
  display: flex; align-items: center; gap: var(--sp-3);
  padding: var(--sp-3); background: var(--color-bg-subtle); border-radius: var(--r-md);
}
.fl-mr-time-row { display: flex; flex-direction: column; align-items: center; gap: 2px; font-size: var(--fs-12); color: var(--color-text-muted); }
.fl-mr-val { font-family: var(--font-mono); font-size: var(--fs-16); font-weight: var(--fw-semibold); color: var(--color-text-primary); }
.fl-mr-arrow { color: var(--color-text-muted); }
.fl-mr-badge {
  padding: 2px 8px; border-radius: var(--r-pill); font-size: 11px; font-weight: var(--fw-semibold);
  margin-left: auto;
}
.fl-mr-badge.is-over { background: color-mix(in srgb, var(--color-q3) 12%, transparent); color: var(--color-q3); }
.fl-mr-badge.is-under { background: color-mix(in srgb, var(--color-success) 12%, transparent); color: var(--color-success); }

.fl-mr-ms-banner {
  display: flex; align-items: center; justify-content: space-between;
  padding: var(--sp-3); background: var(--color-primary-soft);
  border: 1px solid color-mix(in srgb, var(--color-primary) 25%, transparent);
  border-radius: var(--r-md); font-size: var(--fs-13, 13px);
}
.fl-mr-ms-label { color: var(--color-text-secondary); }
.fl-mr-ms-val { font-family: var(--font-mono); color: var(--color-primary); font-weight: var(--fw-semibold); }

.fl-mr-label { font-size: var(--fs-12); color: var(--color-text-muted); margin-bottom: var(--sp-2); }
.fl-mr-chips { display: grid; grid-template-columns: 1fr 1fr; gap: var(--sp-2); }
.fl-mr-chip {
  padding: var(--sp-2); border: 1px solid var(--color-border); border-radius: var(--r-md);
  background: transparent; font-size: var(--fs-12); cursor: pointer;
  transition: all var(--dur-fast);
}
.fl-mr-chip:hover { background: var(--color-bg-hover); }
.fl-mr-chip.is-selected { border-color: var(--color-primary); background: var(--color-primary-soft); color: var(--color-primary); }

.fl-mr-note input,
.fl-mr-note textarea {
  width: 100%; padding: var(--sp-2) var(--sp-3);
  border: 1px solid var(--color-border); border-radius: var(--r-md);
  background: var(--color-bg-subtle); color: var(--color-text-primary);
  font-size: var(--fs-14); outline: none; font-family: inherit; resize: vertical;
}
.fl-mr-note input:focus,
.fl-mr-note textarea:focus { border-color: var(--color-primary); }

.fl-mr-foot { display: flex; gap: var(--sp-2); justify-content: flex-end; }
.fl-mr-btn {
  padding: var(--sp-2) var(--sp-4); border-radius: var(--r-md);
  font-size: var(--fs-12); font-weight: var(--fw-medium); cursor: pointer; border: none;
}
.fl-mr-primary { background: var(--color-primary); color: #fff; }
.fl-mr-ghost { background: transparent; color: var(--color-text-secondary); border: 1px solid var(--color-border); }

.fl-fade-enter-active, .fl-fade-leave-active { transition: opacity var(--dur-base) var(--ease-smooth); }
.fl-fade-enter-from, .fl-fade-leave-to { opacity: 0; }
</style>
