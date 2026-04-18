/**
 * useSound · Web Audio API 提示音(无外部音频文件依赖)。
 *
 * - focusComplete: 两音阶上行(C5→E5)
 * - breakComplete: 柔和单音(G4)
 *
 * 遵守 useUIStore.soundEnabled 开关。
 * AudioContext 在首次用户交互后懒创建(浏览器策略)。
 */

import { useUIStore } from "@/stores/useUIStore";

let ctx: AudioContext | null = null;

function getCtx(): AudioContext {
  if (!ctx) {
    ctx = new AudioContext();
  }
  return ctx;
}

function playTone(freq: number, duration: number, startDelay = 0) {
  const ac = getCtx();
  const osc = ac.createOscillator();
  const gain = ac.createGain();
  osc.type = "sine";
  osc.frequency.value = freq;
  gain.gain.setValueAtTime(0.3, ac.currentTime + startDelay);
  gain.gain.exponentialRampToValueAtTime(0.001, ac.currentTime + startDelay + duration);
  osc.connect(gain);
  gain.connect(ac.destination);
  osc.start(ac.currentTime + startDelay);
  osc.stop(ac.currentTime + startDelay + duration);
}

export function useSound() {
  const ui = useUIStore();

  function playFocusComplete() {
    if (!ui.soundEnabled) return;
    // C5 (523Hz) → E5 (659Hz)
    playTone(523, 0.2, 0);
    playTone(659, 0.25, 0.18);
  }

  function playBreakComplete() {
    if (!ui.soundEnabled) return;
    // G4 (392Hz) gentle
    playTone(392, 0.35, 0);
  }

  return { playFocusComplete, playBreakComplete };
}
