<script setup lang="ts">
/**
 * ManualSessionModal · 手动补录专注记录弹窗。
 * visible=false 时不渲染。
 */

import { Clock, Save, X } from "lucide-vue-next";
import { computed, ref, watch } from "vue";

import { invokeCmd } from "@/composables/useTauriInvoke";
import { useTaskStore } from "@/stores/useTaskStore";
import type { ManualSessionInput } from "@/types";

const props = defineProps<{ visible: boolean }>();
const emit = defineEmits<{ close: [] }>();

const tasks = useTaskStore();

const taskId = ref("");
const startDate = ref("");
const startTime = ref("09:00");
const durationMinutes = ref(25);
const saving = ref(false);

watch(
  () => props.visible,
  (v) => {
    if (v) {
      // 默认今天日期
      const d = new Date();
      startDate.value = d.toISOString().slice(0, 10);
      startTime.value = "09:00";
      durationMinutes.value = 25;
      taskId.value = tasks.tasks[0]?.id ?? "";
    }
  },
);

const canSave = computed(
  () => taskId.value && startDate.value && durationMinutes.value > 0,
);

async function onSave() {
  if (!canSave.value) return;
  saving.value = true;
  try {
    const startISO = new Date(`${startDate.value}T${startTime.value}`).toISOString();
    const input: ManualSessionInput = {
      taskId: taskId.value,
      startTime: startISO,
      durationMinutes: durationMinutes.value,
    };
    await invokeCmd("create_manual_session", { input });
    emit("close");
  } catch (e) {
    console.error("[manual-session] save failed", e);
  } finally {
    saving.value = false;
  }
}
</script>

<template>
  <Transition name="fl-fade">
    <div
      v-if="visible"
      class="fl-modal-mask"
      role="presentation"
      @click.self="emit('close')"
    >
      <div class="fl-modal-card" role="dialog" aria-modal="true">
        <header class="fl-modal-head">
          <h2><Clock :size="16" /> 补录专注记录</h2>
          <button class="fl-icon-btn" type="button" aria-label="关闭" @click="emit('close')">
            <X :size="16" />
          </button>
        </header>

        <form class="fl-modal-body" @submit.prevent="onSave">
          <label class="fl-field">
            <span class="fl-label">任务</span>
            <select v-model="taskId" class="fl-input">
              <option v-for="t in tasks.tasks" :key="t.id" :value="t.id">
                {{ t.name }}
              </option>
            </select>
          </label>

          <div class="fl-row">
            <label class="fl-field fl-half">
              <span class="fl-label">日期</span>
              <input v-model="startDate" class="fl-input" type="date" required />
            </label>
            <label class="fl-field fl-half">
              <span class="fl-label">开始时间</span>
              <input v-model="startTime" class="fl-input" type="time" required />
            </label>
          </div>

          <label class="fl-field">
            <span class="fl-label">时长 (分钟)</span>
            <input
              v-model.number="durationMinutes"
              class="fl-input"
              type="number"
              min="1"
              max="480"
              required
            />
          </label>

          <footer class="fl-modal-foot">
            <button
              class="fl-btn fl-btn-primary"
              type="submit"
              :disabled="!canSave || saving"
            >
              <Save :size="14" /> {{ saving ? '保存中…' : '补录' }}
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
  width: min(420px, 100%);
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--r-lg);
  box-shadow: var(--shadow-modal);
}

.fl-modal-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--sp-4) var(--sp-5);
  border-bottom: 1px solid var(--color-border);
}
.fl-modal-head h2 {
  display: flex;
  align-items: center;
  gap: var(--sp-2);
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
.fl-icon-btn:hover { color: var(--color-text-primary); }

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

.fl-row { display: flex; gap: var(--sp-3); }
.fl-half { flex: 1; }

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
.fl-btn-primary:hover:not(:disabled) { background: var(--color-primary-dark); }
.fl-btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }
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
.fl-fade-leave-to { opacity: 0; }
</style>
