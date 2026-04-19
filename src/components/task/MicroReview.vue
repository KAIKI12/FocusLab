<script setup lang="ts">
/**
 * MicroReview · 任务微复盘卡片 — 对齐 prototype/screens/micro-review.html。
 * 番茄完成后弹出，记录时间偏差原因和经验。
 */

import { ref } from "vue";

const props = defineProps<{
  visible: boolean;
  taskName: string;
  estimatedMinutes: number | null;
  actualMinutes: number;
}>();

const emit = defineEmits<{ close: []; submit: [data: { reason: string; note: string }] }>();

const reason = ref("");
const note = ref("");

const REASONS = [
  { value: "harder", label: "比预想难" },
  { value: "interrupted", label: "被打断了" },
  { value: "scope", label: "做多了" },
  { value: "distracted", label: "走神了" },
];

const deviation = props.estimatedMinutes
  ? Math.round(((props.actualMinutes - props.estimatedMinutes) / props.estimatedMinutes) * 100)
  : null;

function onSubmit() {
  emit("submit", { reason: reason.value, note: note.value });
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
      <div class="fl-mr-card">
        <h3>快速复盘</h3>
        <div class="fl-mr-task">{{ taskName }}</div>

        <!-- 时间偏差 -->
        <div v-if="deviation !== null" class="fl-mr-time">
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
            class="fl-mr-badge"
            :class="{ 'is-over': deviation > 0, 'is-under': deviation < 0 }"
          >
            {{ deviation > 0 ? '+' : '' }}{{ deviation }}%
          </span>
        </div>

        <!-- 原因(偏差>30%时显示) -->
        <div v-if="deviation !== null && Math.abs(deviation) > 30" class="fl-mr-reasons">
          <div class="fl-mr-label">是什么原因?</div>
          <div class="fl-mr-chips">
            <button
              v-for="r in REASONS"
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
          <input
            v-model="note"
            type="text"
            placeholder="一句话经验 (可选)"
            maxlength="100"
          />
        </div>

        <div class="fl-mr-foot">
          <button class="fl-mr-btn fl-mr-ghost" @click="onSkip">跳过</button>
          <button class="fl-mr-btn fl-mr-primary" @click="onSubmit">记录</button>
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
  width: min(400px, 100%);
  background: var(--color-bg-elevated); border: 1px solid var(--color-border);
  border-radius: var(--r-lg); box-shadow: var(--shadow-modal);
  padding: var(--sp-5); display: flex; flex-direction: column; gap: var(--sp-4);
}
.fl-mr-card h3 { font-size: var(--fs-16); font-weight: var(--fw-semibold); margin: 0; }
.fl-mr-task { font-size: var(--fs-14); color: var(--color-text-secondary); }

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

.fl-mr-label { font-size: var(--fs-12); color: var(--color-text-muted); margin-bottom: var(--sp-2); }
.fl-mr-chips { display: grid; grid-template-columns: 1fr 1fr; gap: var(--sp-2); }
.fl-mr-chip {
  padding: var(--sp-2); border: 1px solid var(--color-border); border-radius: var(--r-md);
  background: transparent; font-size: var(--fs-12); cursor: pointer;
  transition: all var(--dur-fast);
}
.fl-mr-chip:hover { background: var(--color-bg-hover); }
.fl-mr-chip.is-selected { border-color: var(--color-primary); background: var(--color-primary-soft); color: var(--color-primary); }

.fl-mr-note input {
  width: 100%; padding: var(--sp-2) var(--sp-3);
  border: 1px solid var(--color-border); border-radius: var(--r-md);
  background: var(--color-bg-subtle); color: var(--color-text-primary);
  font-size: var(--fs-14); outline: none;
}
.fl-mr-note input:focus { border-color: var(--color-primary); }

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
