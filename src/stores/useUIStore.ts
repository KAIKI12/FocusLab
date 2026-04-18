/**
 * useUIStore · 全局 UI 状态 — 侧边栏折叠 / 当前活跃视图。
 */

import { defineStore } from "pinia";
import { ref } from "vue";

export const useUIStore = defineStore("ui", () => {
  const sidebarCollapsed = ref(false);

  function toggleSidebar() {
    sidebarCollapsed.value = !sidebarCollapsed.value;
  }

  return { sidebarCollapsed, toggleSidebar };
});
