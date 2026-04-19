/**
 * useUIStore · 全局 UI 状态 — 侧边栏 / 中断弹窗 / 音效开关。
 */

import { defineStore } from "pinia";
import { ref } from "vue";

export const useUIStore = defineStore("ui", () => {
  const sidebarCollapsed = ref(false);
  const showInterruptionDialog = ref(false);
  const showQuickAdd = ref(false);
  const soundEnabled = ref(true);

  function toggleSidebar() {
    sidebarCollapsed.value = !sidebarCollapsed.value;
  }

  function toggleSound() {
    soundEnabled.value = !soundEnabled.value;
  }

  return { sidebarCollapsed, toggleSidebar, showInterruptionDialog, showQuickAdd, soundEnabled, toggleSound };
});
