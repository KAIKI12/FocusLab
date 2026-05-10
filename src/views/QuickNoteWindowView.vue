<script setup lang="ts">
import { emit } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";

import QuickNoteModal from "@/components/common/QuickNoteModal.vue";
import { invokeCmd } from "@/composables/useTauriInvoke";
import { INSPIRATIONS_CHANGED_EVENT, setQuickAddPrefill } from "@/toolWindows";

const appWindow = getCurrentWindow();

async function closeWindow() {
  await appWindow.close();
}

async function handleSaved() {
  await emit(INSPIRATIONS_CHANGED_EVENT);
}

async function handleCreateTask(text: string, quadrant?: string) {
  setQuickAddPrefill({ title: text, quadrant });
  await invokeCmd("show_quick_add_window");
  await closeWindow();
}
</script>

<template>
  <QuickNoteModal
    :visible="true"
    standalone
    @close="closeWindow"
    @saved="handleSaved"
    @create-task="handleCreateTask"
  />
</template>
