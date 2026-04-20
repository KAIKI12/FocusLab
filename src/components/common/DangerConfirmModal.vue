<script setup lang="ts">
/**
 * DangerConfirmModal · 通用危险操作确认弹窗。
 * 对齐 prototype/screens/modals.html §4 Danger Confirm。
 * 需要用户输入指定文字才能确认。
 */

import { AlertTriangle } from "lucide-vue-next";
import { computed, ref, watch } from "vue";

const props = defineProps<{
  visible: boolean;
  title: string;
  description: string;
  items?: Array<{ label: string; count: string }>;
  confirmText?: string;
}>();

const emit = defineEmits<{ close: []; confirmed: [] }>();

const input = ref("");
const requiredText = computed(() => props.confirmText ?? "RESET");
const isMatch = computed(() => input.value === requiredText.value);

watch(() => props.visible, (v) => { if (!v) input.value = ""; });

function onConfirm() {
  if (!isMatch.value) return;
  emit("confirmed");
}
</script>

<template>
  <Transition name="fl-fade">
    <div v-if="visible" class="fl-modal-mask">
      <div class="fl-dc" role="alertdialog" aria-modal="true">
        <header class="fl-dc-head">
          <div class="fl-dc-icon"><AlertTriangle :size="24" /></div>
          <h2>{{ title }}</h2>
          <p>{{ description }}</p>
        </header>

        <div v-if="items?.length" class="fl-dc-items">
          <div v-for="(it, i) in items" :key="i" class="fl-dc-item">
            <span>{{ it.label }}</span>
            <span class="fl-dc-count">{{ it.count }}</span>
          </div>
        </div>

        <div class="fl-dc-confirm">
          <label class="fl-dc-label">
            请输入 <code>{{ requiredText }}</code> 以确认操作
          </label>
          <input
            v-model="input"
            class="fl-dc-input"
            type="text"
            :placeholder="requiredText"
            spellcheck="false"
            autocomplete="off"
          />
        </div>

        <footer class="fl-dc-foot">
          <button class="fl-btn fl-btn-ghost" @click="emit('close')">取消</button>
          <button class="fl-btn fl-btn-danger" :disabled="!isMatch" @click="onConfirm">
            确认删除
          </button>
        </footer>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.fl-modal-mask {
  position: fixed; inset: 0;
  background: color-mix(in srgb, var(--color-text-primary) 40%, transparent);
  display: grid; place-items: center;
  z-index: var(--z-modal); padding: var(--sp-4);
}

.fl-dc {
  width: min(420px, 100%);
  max-height: calc(100vh - 32px);
  overflow-x: hidden;
  overflow-y: auto;
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--r-lg);
  box-shadow: var(--shadow-modal);
}

.fl-dc-head {
  padding: var(--sp-5); text-align: center;
  background: linear-gradient(180deg, color-mix(in srgb, #ef4444 8%, var(--color-bg-elevated)), var(--color-bg-elevated));
}
.fl-dc-icon {
  width: 48px; height: 48px; border-radius: 50%;
  background: color-mix(in srgb, #ef4444 12%, transparent);
  color: #ef4444; display: grid; place-items: center;
  margin: 0 auto var(--sp-3);
}
.fl-dc-head h2 { font-size: var(--fs-16); font-weight: var(--fw-semibold); margin: 0 0 var(--sp-1); color: #ef4444; }
.fl-dc-head p { font-size: var(--fs-12); color: var(--color-text-muted); margin: 0; }

.fl-dc-items {
  padding: 0 var(--sp-5); display: flex; flex-direction: column; gap: 4px;
}
.fl-dc-item {
  display: flex; justify-content: space-between; align-items: center;
  padding: 6px var(--sp-3); border-radius: var(--r-sm);
  background: var(--color-bg-subtle); font-size: var(--fs-12);
}
.fl-dc-count { font-weight: var(--fw-semibold); color: var(--color-text-primary); }

.fl-dc-confirm { padding: var(--sp-4) var(--sp-5); }
.fl-dc-label { font-size: var(--fs-12); color: var(--color-text-secondary); display: block; margin-bottom: var(--sp-2); }
.fl-dc-label code {
  background: var(--color-bg-subtle); padding: 2px 6px;
  border-radius: 3px; font-family: var(--font-mono, monospace);
  font-weight: var(--fw-semibold); color: #ef4444;
}
.fl-dc-input {
  width: 100%; padding: var(--sp-2) var(--sp-3);
  border-radius: var(--r-md); border: 1px solid var(--color-border);
  background: var(--color-bg-subtle); color: var(--color-text-primary);
  font-size: var(--fs-14); font-family: var(--font-mono, monospace);
  outline: none; text-align: center;
}
.fl-dc-input:focus { border-color: #ef4444; }

.fl-dc-foot {
  display: flex; justify-content: flex-end; gap: var(--sp-2);
  padding: var(--sp-3) var(--sp-5);
  border-top: 1px solid var(--color-border);
}
.fl-btn {
  padding: 8px 16px; border-radius: var(--r-md);
  font-size: var(--fs-12); font-weight: var(--fw-medium);
  border: 1px solid transparent; cursor: pointer;
}
.fl-btn-ghost { background: transparent; color: var(--color-text-secondary); border-color: var(--color-border); }
.fl-btn-ghost:hover { background: var(--color-bg-hover); }
.fl-btn-danger { background: #ef4444; color: #fff; }
.fl-btn-danger:hover:not(:disabled) { background: #dc2626; }
.fl-btn-danger:disabled { opacity: 0.4; cursor: not-allowed; }

.fl-fade-enter-active, .fl-fade-leave-active { transition: opacity var(--dur-base) var(--ease-smooth); }
.fl-fade-enter-from, .fl-fade-leave-to { opacity: 0; }
</style>
