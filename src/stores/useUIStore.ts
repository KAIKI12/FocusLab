/**
 * useUIStore · 全局 UI 状态 — 侧边栏 / 中断弹窗 / 音效开关 / 聊天面板。
 */

import { defineStore } from "pinia";
import { ref, watch } from "vue";

export const useUIStore = defineStore("ui", () => {
  const sidebarCollapsed = ref(false);
  const showInterruptionDialog = ref(false);
  const showQuickAdd = ref(false);
  const showQuickNote = ref(false);
  const quickNotePrefilledTitle = ref("");
  const quickNotePrefilledQuadrant = ref("");
  const quickNotePrefilledText = ref("");
  const soundEnabled = ref(true);

  // ---- 聊天面板 ----
  // 默认关闭，仅用户手动点击 FAB 才打开，不持久化
  const showChat = ref(false);
  const chatPanelPinned = ref(localStorage.getItem("fl-chat-pinned") !== "false");
  const chatPanelWidth = ref(Number(localStorage.getItem("fl-chat-width")) || 400);

  watch(chatPanelPinned, (v) => localStorage.setItem("fl-chat-pinned", String(v)));
  watch(chatPanelWidth, (v) => localStorage.setItem("fl-chat-width", String(v)));

  function toggleChat() {
    showChat.value = !showChat.value;
  }

  function toggleSidebar() {
    sidebarCollapsed.value = !sidebarCollapsed.value;
  }

  function toggleSound() {
    soundEnabled.value = !soundEnabled.value;
  }

  return {
    sidebarCollapsed, toggleSidebar, showInterruptionDialog,
    showQuickAdd, showQuickNote, quickNotePrefilledTitle,
    quickNotePrefilledQuadrant, quickNotePrefilledText,
    soundEnabled, toggleSound,
    showChat, chatPanelPinned, chatPanelWidth, toggleChat,
  };
});
