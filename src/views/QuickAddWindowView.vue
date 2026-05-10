<script setup lang="ts">
import { emit } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { onMounted } from "vue";

import QuickAddModal from "@/components/task/QuickAddModal.vue";
import { useGoalStore } from "@/stores/useGoalStore";
import { TASKS_CHANGED_EVENT } from "@/toolWindows";

const goals = useGoalStore();
const appWindow = getCurrentWindow();

async function closeWindow() {
  await appWindow.close();
}

async function handleCreated() {
  await emit(TASKS_CHANGED_EVENT);
}

onMounted(() => {
  goals.loadGoals().catch((err) => {
    console.error("[quick-add-window] load goals failed", err);
  });
});
</script>

<template>
  <QuickAddModal :visible="true" standalone @close="closeWindow" @created="handleCreated" />
</template>
