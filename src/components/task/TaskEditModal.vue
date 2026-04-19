<script setup lang="ts">
/**
 * TaskEditModal · 编辑任务属性(name/desc/quadrant/估时/截止日)。
 * task=null 时不渲染(关闭状态)。
 */

import { Save, X } from "lucide-vue-next";
import { ref, watch } from "vue";

import { useTaskStore } from "@/stores/useTaskStore";
import type { Task } from "@/types";

const props = defineProps<{ task: Task | null }>();
const emit = defineEmits<{ close: [] }>();

const tasks = useTaskStore();

const name = ref("");
const description = ref("");
const quadrant = ref("important_not_urgent");
const estimatedMinutes = ref<number | null>(null);
const dueDate = ref("");
const recurrenceRule = ref("");

watch(
  () => props.task,
  (t) => {
    if (!t) return;
    name.value = t.name;
    description.value = t.description ?? "";
    quadrant.value = t.quadrant;
    estimatedMinutes.value = t.estimated_minutes;
    dueDate.value = t.due_date ?? "";
  },
  { immediate: true },
);

const quadrants = [
  { value: "important_urgent", label: "紧急重要", cls: "q1" },
  { value: "important_not_urgent", label: "重要不紧急", cls: "q2" },
  { value: "not_important_urgent", label: "紧急不重要", cls: "q3" },
  { value: "not_important_not_urgent", label: "不紧急不重要", cls: "q4" },
];

async function onSave() {
  if (!props.task || !name.value.trim()) return;
  try {
    await tasks.update({
      id: props.task.id,
      name: name.value.trim(),
      description: description.value || undefined,
      quadrant: quadrant.value,
      estimatedMinutes: estimatedMinutes.value ?? undefined,
      dueDate: dueDate.value || undefined,
    });
    emit("close");
  } catch (e) {
    console.error("[task] update failed", e);
  }
}
</script>

<template>
  <Transition name="fl-fade">
    <div
      v-if="task"
      class="fl-modal-mask"
      role="presentation"
      @click.self="emit('close')"
    >
      <div class="fl-modal-card" role="dialog" aria-modal="true" aria-labelledby="fl-edit-title">
        <header class="fl-modal-head">
          <h2 id="fl-edit-title">编辑任务</h2>
          <button class="fl-icon-btn" type="button" aria-label="关闭" @click="emit('close')">
            <X :size="16" />
          </button>
        </header>

        <form class="fl-modal-body" @submit.prevent="onSave">
          <label class="fl-field">
            <span class="fl-label">任务名</span>
            <input v-model="name" class="fl-input" type="text" maxlength="80" required />
          </label>

          <label class="fl-field">
            <span class="fl-label">描述</span>
            <textarea v-model="description" class="fl-input fl-textarea" rows="2" maxlength="500" />
          </label>

          <fieldset class="fl-field fl-quadrant-group">
            <legend class="fl-label">象限</legend>
            <div class="fl-quadrant-options">
              <label
                v-for="q in quadrants"
                :key="q.value"
                class="fl-q-option"
                :class="[q.cls, { 'is-active': quadrant === q.value }]"
              >
                <input v-model="quadrant" type="radio" :value="q.value" class="fl-sr-only" />
                {{ q.label }}
              </label>
            </div>
          </fieldset>

          <div class="fl-row">
            <label class="fl-field fl-half">
              <span class="fl-label">预估(分钟)</span>
              <input v-model.number="estimatedMinutes" class="fl-input" type="number" min="1" max="480" />
            </label>
            <label class="fl-field fl-half">
              <span class="fl-label">截止日期</span>
              <input v-model="dueDate" class="fl-input" type="date" />
            </label>
          </div>

          <label class="fl-field">
            <span class="fl-label">重复规则</span>
            <select v-model="recurrenceRule" class="fl-input">
              <option value="">不重复</option>
              <option value="daily">每天</option>
              <option value="weekdays">工作日</option>
              <option value="weekly">每周</option>
              <option value="monthly">每月</option>
            </select>
          </label>

          <footer class="fl-modal-foot">
            <button class="fl-btn fl-btn-primary" type="submit" :disabled="!name.trim()">
              <Save :size="14" /> 保存
            </button>
            <button class="fl-btn fl-btn-secondary" type="button" @click="emit('close')">
              取消
            </button>
          </footer>
        </form>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.fl-modal-mask {
  position: fixed;
  inset: 0;
  background: color-mix(in srgb, var(--color-text-primary) 32%, transparent);
  display: grid;
  place-items: center;
  z-index: var(--z-modal);
  padding: var(--sp-4);
}

.fl-modal-card {
  width: min(480px, 100%);
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--r-lg);
  box-shadow: var(--shadow-modal);
  display: flex;
  flex-direction: column;
}

.fl-modal-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--sp-4) var(--sp-5);
  border-bottom: 1px solid var(--color-border);
}

.fl-modal-head h2 {
  font-size: var(--fs-16);
  font-weight: var(--fw-semibold);
  margin: 0;
}

.fl-icon-btn {
  background: none;
  border: none;
  color: var(--color-text-secondary);
  cursor: pointer;
  padding: var(--sp-1);
  border-radius: var(--r-sm);
}
.fl-icon-btn:hover {
  color: var(--color-text-primary);
}

.fl-modal-body {
  padding: var(--sp-5);
  display: flex;
  flex-direction: column;
  gap: var(--sp-4);
}

.fl-field {
  display: flex;
  flex-direction: column;
  gap: var(--sp-1);
  border: none;
  padding: 0;
  margin: 0;
}

.fl-label {
  font-size: var(--fs-12);
  color: var(--color-text-secondary);
  font-weight: var(--fw-medium);
}

.fl-input {
  padding: var(--sp-2) var(--sp-3);
  border-radius: var(--r-md);
  border: 1px solid var(--color-border);
  background: var(--color-bg-subtle);
  color: var(--color-text-primary);
  font-size: var(--fs-14);
  outline: none;
}
.fl-input:focus {
  border-color: var(--color-primary);
  box-shadow: var(--shadow-focus);
}

.fl-textarea {
  resize: vertical;
  font-family: inherit;
}

.fl-row {
  display: flex;
  gap: var(--sp-3);
}
.fl-half {
  flex: 1;
}

.fl-quadrant-group {
  border: none;
  padding: 0;
}

.fl-quadrant-options {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--sp-2);
}

.fl-q-option {
  padding: var(--sp-2) var(--sp-3);
  border-radius: var(--r-md);
  border: 1.5px solid var(--color-border);
  font-size: var(--fs-12);
  cursor: pointer;
  text-align: center;
  transition: all var(--dur-fast) var(--ease-smooth);
}
.fl-q-option.q1 { border-color: var(--color-q1); color: var(--color-q1-text, var(--color-q1)); }
.fl-q-option.q2 { border-color: var(--color-q2); color: var(--color-q2-text, var(--color-q2)); }
.fl-q-option.q3 { border-color: var(--color-q3); color: var(--color-q3-text, var(--color-q3)); }
.fl-q-option.q4 { border-color: var(--color-q4); color: var(--color-q4-text, var(--color-q4)); }
.fl-q-option:not(.is-active) { opacity: 0.55; }
.fl-q-option.is-active { font-weight: var(--fw-semibold); opacity: 1; }

.fl-sr-only {
  position: absolute;
  width: 1px;
  height: 1px;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
}

.fl-modal-foot {
  display: flex;
  gap: var(--sp-2);
  justify-content: flex-end;
  padding-top: var(--sp-2);
}

.fl-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: var(--sp-2) var(--sp-4);
  border-radius: var(--r-md);
  font-size: var(--fs-12);
  font-weight: var(--fw-medium);
  border: 1px solid transparent;
  cursor: pointer;
  transition: background var(--dur-fast) var(--ease-smooth);
}
.fl-btn-primary {
  background: var(--color-primary);
  color: var(--color-text-on-primary);
}
.fl-btn-primary:hover:not(:disabled) {
  background: var(--color-primary-dark);
}
.fl-btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.fl-btn-secondary {
  background: var(--color-bg-hover);
  color: var(--color-text-primary);
  border-color: var(--color-border);
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
