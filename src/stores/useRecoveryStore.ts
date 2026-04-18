/**
 * useRecoveryStore · 承载 RecoveryDialog 的可见性与上下文。
 *
 * 启动时由 useRecovery.checkOnMount() 填充,关闭 Dialog 时清空。
 */

import { defineStore } from "pinia";
import { ref } from "vue";

import type { RecoveryInfo } from "@/types";

export const useRecoveryStore = defineStore("recovery", () => {
  const info = ref<RecoveryInfo | null>(null);
  const visible = ref(false);

  function show(payload: RecoveryInfo) {
    info.value = payload;
    visible.value = true;
  }

  function hide() {
    visible.value = false;
    info.value = null;
  }

  return { info, visible, show, hide };
});
