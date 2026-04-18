/**
 * useRecovery · 启动时崩溃恢复检查的编排 composable。
 *
 * 分发规则(对齐 docs/04 §11.2):
 *   AutoResume  → 控制台 + toast 打日志(Week 1b 不做真实恢复,Week 2 接状态机)
 *   AskUser     → 推进 recoveryStore,由 RecoveryDialog 弹窗
 *   AutoEnd     → 后端 reset_timer_state + toast(Week 2 会顺带把 session
 *                 标为 abandoned,本轮先只复位 timer_state)
 *
 * 本 composable 单例,只应在 App.vue onMounted 调用一次。
 */

import { invokeCmd } from "@/composables/useTauriInvoke";
import { useRecoveryStore } from "@/stores/useRecoveryStore";
import { useTimerStateStore } from "@/stores/useTimerStateStore";
import type { RecoveryInfo } from "@/types";

function fmtGap(sec: number): string {
  if (sec < 60) return `${sec}s`;
  if (sec < 3600) return `${Math.round(sec / 60)} 分钟`;
  return `${(sec / 3600).toFixed(1)} 小时`;
}

export function useRecovery() {
  const recoveryStore = useRecoveryStore();
  const timerStore = useTimerStateStore();

  async function checkOnMount() {
    const info = await invokeCmd<RecoveryInfo | null>("check_crash_recovery");
    if (!info) return; // idle 态,无事发生

    switch (info.recommendation) {
      case "AutoResume":
        console.info(
          `[recovery] 自动恢复:计时器在 ${fmtGap(info.gapSeconds)} 前中断`,
          info,
        );
        // TODO(Week 2): 这里真正恢复 ticking;本轮只保留 timer_state 原样
        break;

      case "AskUser":
        recoveryStore.show(info);
        break;

      case "AutoEnd":
        console.warn(
          `[recovery] 自动结束:中断已超 ${fmtGap(info.gapSeconds)}`,
          info,
        );
        await timerStore.reset();
        break;
    }
  }

  return { checkOnMount };
}
