<script setup lang="ts">
/**
 * FixedSchedulePanel · 每周固定日程展示/编辑。
 * 对齐 prototype/screens/calendar.html:630。
 */

import { Plus, Trash2, X } from "lucide-vue-next";
import { computed, ref } from "vue";

import { useFixedSchedule } from "@/composables/useFixedSchedule";

const { byWeekday, add, remove } = useFixedSchedule();

const WEEKDAY_LABELS: Record<number, string> = {
  0: "周日", 1: "周一", 2: "周二", 3: "周三", 4: "周四", 5: "周五", 6: "周六",
};
const DISPLAY_ORDER = [1, 2, 3, 4, 5, 6, 0];

const todayWeekday = new Date().getDay();

// ----- 新增表单 -----
const adding = ref(false);
const formWeekday = ref<number>(todayWeekday);
const formStart = ref("09:00");
const formEnd = ref("10:30");
const formTitle = ref("");

const canSubmit = computed(
  () => formTitle.value.trim().length > 0 && formStart.value < formEnd.value,
);

function openForm(weekday: number) {
  formWeekday.value = weekday;
  formStart.value = "09:00";
  formEnd.value = "10:30";
  formTitle.value = "";
  adding.value = true;
}

function cancelForm() {
  adding.value = false;
}

function submitForm() {
  if (!canSubmit.value) return;
  add({
    weekday: formWeekday.value,
    startTime: formStart.value,
    endTime: formEnd.value,
    title: formTitle.value.trim(),
  });
  adding.value = false;
}
</script>

<template>
  <div class="fl-fs-panel">
    <div class="fl-fs-head">
      <span class="fl-fs-title">每周固定日程</span>
      <button class="fl-fs-add" type="button" :disabled="adding" @click="openForm(todayWeekday)">
        <Plus :size="12" /> 添加
      </button>
    </div>
    <p class="fl-fs-hint">课程、组会等每周重复的日程 · 仅本地保存</p>

    <!-- 内联添加表单 -->
    <div v-if="adding" class="fl-fs-form">
      <div class="fl-fs-form-row">
        <select v-model.number="formWeekday" class="fl-fs-input">
          <option v-for="w in DISPLAY_ORDER" :key="w" :value="w">{{ WEEKDAY_LABELS[w] }}</option>
        </select>
        <input v-model="formStart" type="time" class="fl-fs-input" />
        <span class="fl-fs-sep">→</span>
        <input v-model="formEnd" type="time" class="fl-fs-input" />
      </div>
      <input
        v-model="formTitle"
        type="text"
        class="fl-fs-input fl-fs-title-input"
        placeholder="例:📘 机器学习课程"
        maxlength="40"
      />
      <div class="fl-fs-form-actions">
        <button type="button" class="fl-fs-btn fl-fs-btn-ghost" @click="cancelForm">
          <X :size="12" /> 取消
        </button>
        <button type="button" class="fl-fs-btn fl-fs-btn-primary" :disabled="!canSubmit" @click="submitForm">
          添加
        </button>
      </div>
    </div>

    <!-- 7 天分组(按周一 -> 周日顺序) -->
    <div class="fl-fs-groups">
      <div v-for="w in DISPLAY_ORDER" :key="w" class="fl-fs-group">
        <div class="fl-fs-day-label" :class="{ 'is-today': w === todayWeekday }">
          {{ WEEKDAY_LABELS[w] }}<template v-if="w === todayWeekday"> · 今天</template>
        </div>
        <div v-if="byWeekday[w].length" class="fl-fs-rows">
          <div v-for="s in byWeekday[w]" :key="s.id" class="fl-fs-row">
            <span class="fl-fs-time">{{ s.startTime }}–{{ s.endTime }}</span>
            <span class="fl-fs-row-title">{{ s.title }}</span>
            <button
              type="button"
              class="fl-fs-row-del"
              :title="`删除「${s.title}」`"
              @click="remove(s.id)"
            >
              <Trash2 :size="12" />
            </button>
          </div>
        </div>
        <div v-else class="fl-fs-empty">(无固定日程)</div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.fl-fs-panel {
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--r-lg);
  padding: var(--sp-4);
  display: flex;
  flex-direction: column;
  gap: var(--sp-2);
}

.fl-fs-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.fl-fs-title {
  font-size: var(--fs-14);
  font-weight: var(--fw-semibold);
  color: var(--color-text-primary);
}
.fl-fs-add {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  background: none;
  border: none;
  color: var(--color-primary);
  font-size: 11px;
  cursor: pointer;
  padding: 2px 4px;
  border-radius: var(--r-sm);
}
.fl-fs-add:hover:not(:disabled) { background: var(--color-primary-soft); }
.fl-fs-add:disabled { opacity: 0.4; cursor: not-allowed; }

.fl-fs-hint {
  font-size: 11px;
  color: var(--color-text-muted);
  margin: 0 0 var(--sp-1);
  line-height: 1.5;
}

/* ---------- 表单 ---------- */
.fl-fs-form {
  display: flex;
  flex-direction: column;
  gap: var(--sp-2);
  padding: var(--sp-3);
  border-radius: var(--r-md);
  background: var(--color-primary-soft);
  border: 1px solid color-mix(in srgb, var(--color-primary) 30%, transparent);
}
.fl-fs-form-row {
  display: flex;
  align-items: center;
  gap: 4px;
}
.fl-fs-sep { color: var(--color-text-muted); font-size: 11px; }
.fl-fs-input {
  padding: 4px 6px;
  font-size: 11px;
  border: 1px solid var(--color-border);
  border-radius: var(--r-sm);
  background: var(--color-bg-elevated);
  color: var(--color-text-primary);
  outline: none;
  flex: 1;
  min-width: 0;
}
.fl-fs-input:focus { border-color: var(--color-primary); }
.fl-fs-title-input { flex: 1 1 100%; }
.fl-fs-form-actions {
  display: flex;
  gap: 6px;
  justify-content: flex-end;
}
.fl-fs-btn {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  padding: 3px 10px;
  border-radius: var(--r-sm);
  font-size: 11px;
  cursor: pointer;
  border: 1px solid transparent;
}
.fl-fs-btn-ghost {
  background: transparent;
  color: var(--color-text-secondary);
  border-color: var(--color-border);
}
.fl-fs-btn-primary {
  background: var(--color-primary);
  color: #fff;
}
.fl-fs-btn-primary:disabled { opacity: 0.4; cursor: not-allowed; }

/* ---------- 分组 ---------- */
.fl-fs-groups {
  display: flex;
  flex-direction: column;
  gap: var(--sp-2);
  margin-top: var(--sp-1);
}

.fl-fs-group { display: flex; flex-direction: column; gap: 4px; }

.fl-fs-day-label {
  font-size: 11px;
  font-weight: var(--fw-medium);
  color: var(--color-text-muted);
  padding: 2px 6px;
  border-radius: var(--r-sm);
  width: fit-content;
}
.fl-fs-day-label.is-today {
  color: var(--color-primary);
  background: var(--color-primary-soft);
  font-weight: var(--fw-semibold);
}

.fl-fs-rows { display: flex; flex-direction: column; gap: 2px; }
.fl-fs-row {
  display: grid;
  grid-template-columns: auto 1fr auto;
  gap: 6px;
  align-items: center;
  padding: 4px 6px;
  border-radius: var(--r-sm);
  font-size: 11px;
  transition: background var(--dur-fast) var(--ease-smooth);
}
.fl-fs-row:hover { background: var(--color-bg-hover); }

.fl-fs-time {
  font-family: var(--font-mono);
  color: var(--color-text-secondary);
  font-size: 10px;
  padding: 1px 6px;
  border-radius: var(--r-pill);
  background: var(--color-bg-subtle);
}
.fl-fs-row-title {
  color: var(--color-text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.fl-fs-row-del {
  background: none;
  border: none;
  color: var(--color-text-muted);
  cursor: pointer;
  padding: 2px;
  border-radius: var(--r-sm);
  opacity: 0;
  transition: all var(--dur-fast) var(--ease-smooth);
}
.fl-fs-row:hover .fl-fs-row-del { opacity: 1; }
.fl-fs-row-del:hover { color: var(--color-q1); }

.fl-fs-empty {
  font-size: 11px;
  color: var(--color-text-muted);
  font-style: italic;
  padding: 2px 6px;
}
</style>
