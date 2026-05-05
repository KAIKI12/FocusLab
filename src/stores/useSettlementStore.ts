/**
 * useSettlementStore · 日结算 store。
 */

import { defineStore } from "pinia";
import { ref } from "vue";

import { invokeCmd } from "@/composables/useTauriInvoke";
import type { Settlement, SettleInput, UnsettledYesterday, YesterdaySummary } from "@/types";

export const useSettlementStore = defineStore("settlement", () => {
  const settlement = ref<Settlement | null>(null);
  const yesterday = ref<YesterdaySummary | null>(null);
  const showDialog = ref(false);
  const settling = ref(false);

  // 心情打卡前置流程(exp_mood_checkin 开启时)
  const showMoodPrompt = ref(false);
  const pendingInput = ref<SettleInput | null>(null);

  // 昨日未结算补打卡：本次会话内 dismissed 后不再弹
  const unsettledYesterday = ref<UnsettledYesterday | null>(null);
  const unsettledDismissed = ref(false);

  function readMorningIntent(): number | null {
    const today = new Date().toISOString().slice(0, 10);
    const v = localStorage.getItem(`fl-mood-morning-${today}`);
    return v ? Number(v) || null : null;
  }

  async function isMoodEnabled(): Promise<boolean> {
    try {
      const v = await invokeCmd<string | null>("get_setting", { key: "exp_mood_checkin" });
      return v !== "0"; // 默认开启(未设置时按 on 处理)
    } catch {
      return true;
    }
  }

  async function settle(input: SettleInput = {}) {
    // 若已提供 eveningMood(例如测试场景)或开关关闭,直接 settle
    if (input.eveningMood !== undefined || !(await isMoodEnabled())) {
      await settleInternal({
        ...input,
        morningIntent: input.morningIntent ?? readMorningIntent(),
      });
      return;
    }
    // 先弹 evening mood,选择/跳过后再 settle
    pendingInput.value = input;
    showMoodPrompt.value = true;
  }

  async function confirmMood(eveningMood: number | null) {
    const base = pendingInput.value ?? {};
    showMoodPrompt.value = false;
    pendingInput.value = null;
    await settleInternal({
      ...base,
      eveningMood,
      morningIntent: base.morningIntent ?? readMorningIntent(),
    });
  }

  async function settleInternal(input: SettleInput) {
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

  /** 打开昨日结算弹窗(用于"查看昨日结算") */
  async function openYesterdayDialog() {
    if (!yesterday.value) await loadYesterday();
    if (!yesterday.value) return;
    await loadSettlement(yesterday.value.settleDate);
    if (settlement.value) showDialog.value = true;
  }

  function closeDialog() {
    showDialog.value = false;
  }

  /** 启动时检测昨日是否未结算（本次会话已 dismiss 则跳过） */
  async function checkUnsettledYesterday() {
    if (unsettledDismissed.value) return;
    unsettledYesterday.value = await invokeCmd<UnsettledYesterday | null>(
      "check_unsettled_yesterday",
    );
  }

  /** 用户点"稍后再说":本次会话内不再弹 */
  function dismissUnsettled() {
    unsettledDismissed.value = true;
    unsettledYesterday.value = null;
  }

  /** 补做昨日结算:走 settle 流程并指定 planDate */
  async function settleYesterday() {
    const target = unsettledYesterday.value;
    if (!target) return;
    await settle({ planDate: target.settleDate, triggerType: "makeup" });
    unsettledYesterday.value = null;
  }

  return {
    settlement,
    yesterday,
    showDialog,
    settling,
    showMoodPrompt,
    unsettledYesterday,
    unsettledDismissed,
    settle,
    confirmMood,
    loadSettlement,
    loadYesterday,
    openYesterdayDialog,
    closeDialog,
    checkUnsettledYesterday,
    dismissUnsettled,
    settleYesterday,
  };
});
