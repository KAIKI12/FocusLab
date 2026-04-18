/**
 * useSettlementStore · 日结算 store。
 */

import { defineStore } from "pinia";
import { ref } from "vue";

import { invokeCmd } from "@/composables/useTauriInvoke";
import type { Settlement, SettleInput, YesterdaySummary } from "@/types";

export const useSettlementStore = defineStore("settlement", () => {
  const settlement = ref<Settlement | null>(null);
  const yesterday = ref<YesterdaySummary | null>(null);
  const showDialog = ref(false);
  const settling = ref(false);

  async function settle(input: SettleInput = {}) {
    settling.value = true;
    try {
      settlement.value = await invokeCmd<Settlement>("settle_day", { input });
      showDialog.value = true;
    } finally {
      settling.value = false;
    }
  }

  async function loadSettlement(planDate?: string) {
    settlement.value = await invokeCmd<Settlement | null>("get_settlement", {
      planDate: planDate ?? null,
    });
  }

  async function loadYesterday() {
    yesterday.value = await invokeCmd<YesterdaySummary | null>(
      "get_yesterday_summary",
    );
  }

  function closeDialog() {
    showDialog.value = false;
  }

  return {
    settlement,
    yesterday,
    showDialog,
    settling,
    settle,
    loadSettlement,
    loadYesterday,
    closeDialog,
  };
});
