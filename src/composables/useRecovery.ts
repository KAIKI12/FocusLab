/**
 * useRecovery · 启动时崩溃恢复检查的编排 composable。
 *
 * 分发规则(对齐 docs/04 §11.2):
 *   AutoResume  → 调 `resume_from_crash`,后端把内存 RunningTimer 拉起并 spawn tick
 *   AskUser     → 推进 recoveryStore,由 RecoveryDialog 弹窗
 *   AutoEnd     → 调 `abandon_from_crash`,后端把 session 标为 abandoned + 清 timer_state
 *
 * 本 composable 单例,只应在 App.vue onMounted 调用一次。
 */

import { invokeCmd } from "@/composables/useTauriInvoke";
import { useRecoveryStore } from "@/stores/useRecoveryStore";
import { useTimerStore } from "@/stores/useTimerStore";
import type { RecoveryInfo } from "@/types";

function fmtGap(sec: number): string {
  if (sec < 60) return `${sec}s`;
  if (sec < 3600) return `${Math.round(sec / 60)} 分钟`;
  return `${(sec / 3600).toFixed(1)} 小时`;
}

export function useRecovery() {
  const recoveryStore = useRecoveryStore();
  const timerStore = useTimerStore();

  async function checkOnMount() {
    // 先让前端 timer store 挂上事件监听,避免 resume_from_crash 发出的事件漏接
    await timerStore.init();

    const info = await invokeCmd<RecoveryInfo | null>("check_crash_recovery");
    if (!info) return;

    switch (info.recommendation) {
      case "AutoResume":
        console.info(
          `[recovery] 自动恢复:计时器在 ${fmtGap(info.gapSeconds)} 前中断`,
        );
        try {
          await invokeCmd<unknown>("resume_from_crash");
        } catch (e) {
          console.error("[recovery] resume_from_crash 失败", e);
        }
        break;

      case "AskUser":
        recoveryStore.show(info);
        break;

      case "AutoEnd":
        console.warn(
          `[recovery] 自动结束:中断已超 ${fmtGap(info.gapSeconds)}`,
        );
        try {
          await invokeCmd<void>("abandon_from_crash");
        } catch (e) {
          console.error("[recovery] abandon_from_crash 失败", e);
        }
        break;
    }
  }

  return { checkOnMount };
}
