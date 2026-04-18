<script setup lang="ts">
/**
 * InterruptionDialog · 暂停时弹出的中断原因选择。
 *
 * 6 个预设原因 pill + 可选 note 输入 + [记录]/[跳过]。
 * 非阻塞:跳过不创建中断记录。
 */

import { ref } from "vue";

import { invokeCmd } from "@/composables/useTauriInvoke";
import { useUIStore } from "@/stores/useUIStore";
import type { InterruptionReason } from "@/types";

const props = defineProps<{
  sessionId: string | null;
}>();

const emit = defineEmits<{
  recorded: [interruptionId: string];
}>();

const ui = useUIStore();

const selectedReason = ref<InterruptionReason | null>(null);
const note = ref("");

const reasons: { value: InterruptionReason; label: string; icon: string }[] = [
  { value: "phone_message", label: "电话/消息", icon: "📞" },
  { value: "colleague", label: "同学/导师找", icon: "🧑‍🏫" },
  { value: "rest", label: "主动休息", icon: "🚶" },
  { value: "distraction", label: "走神了", icon: "💭" },
  { value: "errand", label: "临时事务", icon: "📋" },
  { value: "other", label: "其他", icon: "📝" },
];

function onSelect(r: InterruptionReason) {
  selectedReason.value = selectedReason.value === r ? null : r;
}

async function onRecord() {
  if (!props.sessionId) return;
  try {
    const id = await invokeCmd<string>("create_interruption", {
      input: {
        sessionId: props.sessionId,
        reason: selectedReason.value ?? undefined,
        note: note.value.trim() || undefined,
      },
    });
    emit("recorded", id);
  } catch (e) {
    console.error("[interruption] create failed", e);
  }
  reset();
}

function onSkip() {
  reset();
}

function reset() {
  selectedReason.value = null;
  note.value = "";
  ui.showInterruptionDialog = false;
}
</script>

<template>
  <Transition name="fl-fade">
    <div
      v-if="ui.showInterruptionDialog && sessionId"
      class="fl-int-mask"
      role="presentation"
      @click.self="onSkip"
    >
      <div class="fl-int-card" role="dialog" aria-modal="true" aria-labelledby="fl-int-title">
        <h3 id="fl-int-title" class="fl-int-head">记录中断原因</h3>

        <div class="fl-int-pills">
          <button
            v-for="r in reasons"
            :key="r.value"
            class="fl-pill"
            :class="{ 'is-active': selectedReason === r.value }"
            type="button"
            @click="onSelect(r.value)"
          >
            <span class="fl-pill-icon">{{ r.icon }}</span>
            {{ r.label }}
          </button>
        </div>

        <textarea
          v-if="selectedReason"
          v-model="note"
          class="fl-int-note"
          placeholder="补充说明(可选)…"
          rows="2"
          maxlength="100"
        />

        <div class="fl-int-actions">
          <button class="fl-act fl-act-primary" type="button" @click="onRecord">
            记录
          </button>
          <button class="fl-act fl-act-ghost" type="button" @click="onSkip">
            跳过
          </button>
        </div>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.fl-int-mask {
  position: fixed;
  inset: 0;
  background: color-mix(in srgb, var(--color-text-primary) 24%, transparent);
  display: grid;
  place-items: center;
  z-index: var(--z-modal);
  padding: var(--sp-4);
}

.fl-int-card {
  width: min(380px, 100%);
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--r-lg);
  box-shadow: var(--shadow-modal);
  padding: var(--sp-5);
  display: flex;
  flex-direction: column;
  gap: var(--sp-4);
}

.fl-int-head {
  margin: 0;
  font-size: var(--fs-14);
  font-weight: var(--fw-semibold);
  color: var(--color-text-primary);
}

.fl-int-pills {
  display: flex;
  flex-wrap: wrap;
  gap: var(--sp-2);
}

.fl-pill {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: var(--sp-2) var(--sp-3);
  border-radius: var(--r-pill);
  border: 1px solid var(--color-border);
  background: var(--color-bg-subtle);
  color: var(--color-text-secondary);
  font-size: var(--fs-12);
  cursor: pointer;
  transition: all var(--dur-fast) var(--ease-smooth);
}
.fl-pill:hover {
  border-color: var(--color-primary);
  color: var(--color-primary);
}
.fl-pill.is-active {
  background: var(--color-primary);
  color: var(--color-text-on-primary);
  border-color: var(--color-primary);
}
.fl-pill-icon {
  font-size: 14px;
}

.fl-int-note {
  padding: var(--sp-2) var(--sp-3);
  border-radius: var(--r-md);
  border: 1px solid var(--color-border);
  background: var(--color-bg-subtle);
  color: var(--color-text-primary);
  font-size: var(--fs-12);
  font-family: inherit;
  resize: none;
  outline: none;
}
.fl-int-note:focus {
  border-color: var(--color-primary);
}

.fl-int-actions {
  display: flex;
  gap: var(--sp-2);
  justify-content: flex-end;
}

.fl-act {
  padding: var(--sp-2) var(--sp-4);
  border-radius: var(--r-md);
  font-size: var(--fs-12);
  font-weight: var(--fw-medium);
  border: none;
  cursor: pointer;
}
.fl-act-primary {
  background: var(--color-primary);
  color: var(--color-text-on-primary);
}
.fl-act-primary:hover {
  background: var(--color-primary-dark);
}
.fl-act-ghost {
  background: transparent;
  color: var(--color-text-secondary);
}
.fl-act-ghost:hover {
  color: var(--color-text-primary);
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
