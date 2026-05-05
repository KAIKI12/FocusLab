<script setup lang="ts">
import { computed, ref } from "vue";

import { shortcutFromKeyboardEvent } from "@/shortcuts/capture";
import {
  getDefaultShortcutBinding,
  SHORTCUT_ACTIONS,
} from "@/shortcuts/definitions";
import { formatShortcutForPlatform } from "@/shortcuts/display";
import type { ShortcutScope } from "@/shortcuts/types";
import { useShortcutStore } from "@/stores/useShortcutStore";

const store = useShortcutStore();
const capturingActionId = ref<string | null>(null);
const saveMessage = ref("");

const groupedActions = computed(() => {
  const groups = new Map<string, typeof SHORTCUT_ACTIONS>();
  SHORTCUT_ACTIONS.forEach((definition) => {
    const items = groups.get(definition.group) ?? [];
    items.push(definition);
    groups.set(definition.group, items);
  });
  return Array.from(groups.entries()).map(([group, actions]) => ({ group, actions }));
});

const conflictMap = computed(() => {
  const map = new Map<string, string>();
  store.conflicts.forEach((conflict) => {
    conflict.actionIds.forEach((actionId) => {
      map.set(actionId, `与其他快捷键重复：${conflict.shortcut}`);
    });
  });
  return map;
});

const reservedMap = computed(() => {
  const map = new Map<string, string>();
  store.reservedIssues.forEach((issue) => {
    map.set(issue.actionId, `${issue.reason}：${issue.shortcut}`);
  });
  return map;
});

const registrationMap = computed(() => {
  const map = new Map<string, string>();
  store.registrationIssues.forEach((issue) => {
    map.set(issue.actionId, `${issue.reason}：${issue.shortcut}`);
  });
  return map;
});

function displayShortcut(actionId: string): string {
  return formatShortcutForPlatform(store.bindings[actionId]?.shortcut ?? null, store.platform);
}

function errorText(actionId: string): string {
  return conflictMap.value.get(actionId)
    ?? reservedMap.value.get(actionId)
    ?? registrationMap.value.get(actionId)
    ?? "";
}

function updateScope(actionId: string, scope: ShortcutScope) {
  store.updateBinding(actionId, { scope });
  saveMessage.value = "";
}

function resetBinding(actionId: string) {
  store.updateBinding(actionId, getDefaultShortcutBinding(actionId));
  saveMessage.value = "";
}

function clearBinding(actionId: string) {
  store.updateBinding(actionId, { shortcut: null });
  saveMessage.value = "";
}

function startCapture(actionId: string) {
  capturingActionId.value = actionId;
  saveMessage.value = "按下新的快捷键，按 Escape 取消。";
}

function onCaptureKeydown(actionId: string, event: KeyboardEvent) {
  event.preventDefault();
  event.stopPropagation();

  if (event.key === "Escape") {
    capturingActionId.value = null;
    saveMessage.value = "";
    return;
  }

  const shortcut = shortcutFromKeyboardEvent(event);
  if (!shortcut) return;

  store.updateBinding(actionId, { shortcut });
  capturingActionId.value = null;
  saveMessage.value = "";
}

async function saveBindings() {
  try {
    await store.save();
    saveMessage.value = "已保存。";
  } catch {
    saveMessage.value = "存在冲突或系统保留快捷键，暂时不能保存。";
  }
}

function restoreDefaults() {
  SHORTCUT_ACTIONS.forEach((definition) => {
    store.updateBinding(definition.id, getDefaultShortcutBinding(definition.id));
  });
  saveMessage.value = "";
}
</script>

<template>
  <div class="fl-shortcuts-panel">
    <div v-for="section in groupedActions" :key="section.group" class="fl-sc-group">
      <div class="fl-sc-section">{{ section.group }}</div>
      <div
        v-for="action in section.actions"
        :key="action.id"
        class="fl-sc-row"
        :class="{ 'has-error': errorText(action.id) }"
      >
        <div class="fl-sc-main">
          <span class="fl-sc-label">{{ action.label }}</span>
          <span v-if="errorText(action.id)" class="fl-sc-error">{{ errorText(action.id) }}</span>
        </div>

        <div class="fl-sc-actions">
          <button
            class="fl-sc-capture"
            :class="{ 'is-capturing': capturingActionId === action.id }"
            type="button"
            tabindex="0"
            @click="startCapture(action.id)"
            @keydown="capturingActionId === action.id ? onCaptureKeydown(action.id, $event) : undefined"
          >
            {{ capturingActionId === action.id ? '按键中…' : (displayShortcut(action.id) || '未设置') }}
          </button>

          <select
            v-if="action.allowGlobal"
            class="fl-sc-scope"
            :value="store.bindings[action.id]?.scope"
            @change="updateScope(action.id, ($event.target as HTMLSelectElement).value as ShortcutScope)"
          >
            <option value="app">应用内</option>
            <option value="global">全局</option>
          </select>
          <span v-else class="fl-sc-scope fl-sc-scope-static">应用内</span>

          <button class="fl-sc-mini" type="button" @click="resetBinding(action.id)">恢复默认</button>
          <button class="fl-sc-mini" type="button" @click="clearBinding(action.id)">清空</button>
        </div>
      </div>
    </div>

    <div class="fl-sc-toolbar">
      <button class="fl-sc-btn" type="button" @click="restoreDefaults">恢复全部默认</button>
      <button class="fl-sc-btn is-primary" type="button" :disabled="!store.canSave" @click="saveBindings">保存</button>
    </div>
    <div v-if="saveMessage" class="fl-sc-status">{{ saveMessage }}</div>
  </div>
</template>

<style scoped>
.fl-shortcuts-panel { display: flex; flex-direction: column; gap: var(--sp-4); }
.fl-sc-group { display: flex; flex-direction: column; gap: 4px; }
.fl-sc-section { font-size: 11px; color: var(--color-text-muted); font-weight: var(--fw-medium); margin-top: var(--sp-2); }
.fl-sc-row {
  display: flex; align-items: center; justify-content: space-between; gap: var(--sp-3);
  padding: var(--sp-3) 0; border-bottom: 1px solid var(--color-border);
}
.fl-sc-row.has-error { border-bottom-color: #ef4444; }
.fl-sc-main { display: flex; flex-direction: column; gap: 4px; min-width: 0; }
.fl-sc-label { font-size: var(--fs-14); color: var(--color-text-primary); }
.fl-sc-error { font-size: var(--fs-12); color: #ef4444; }
.fl-sc-actions { display: flex; align-items: center; gap: var(--sp-2); flex-wrap: wrap; justify-content: flex-end; }
.fl-sc-capture,
.fl-sc-mini,
.fl-sc-btn,
.fl-sc-scope {
  border: 1px solid var(--color-border); background: var(--color-bg-elevated); color: var(--color-text-primary);
  border-radius: var(--r-sm); font-size: var(--fs-12); padding: 6px 10px;
}
.fl-sc-capture { min-width: 110px; font-family: var(--font-mono, monospace); }
.fl-sc-capture.is-capturing { border-color: var(--color-primary); color: var(--color-primary); }
.fl-sc-scope-static { color: var(--color-text-muted); background: var(--color-bg-subtle); }
.fl-sc-mini,
.fl-sc-btn,
.fl-sc-capture { cursor: pointer; }
.fl-sc-toolbar { display: flex; justify-content: flex-end; gap: var(--sp-2); }
.fl-sc-btn.is-primary { background: var(--color-primary); border-color: var(--color-primary); color: white; }
.fl-sc-btn:disabled { opacity: 0.5; cursor: not-allowed; }
.fl-sc-status { font-size: var(--fs-12); color: var(--color-text-muted); text-align: right; }
</style>
