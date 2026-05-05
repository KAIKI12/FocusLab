/**
 * usePersonaFlipCard — 人格图鉴翻卡解锁机制。
 *
 * 规则:
 *   - 累计日结算 < 7: 全部锁定,不可翻卡
 *   - 累计日结算 = 7: 自动揭示用户匹配人格(免费)
 *   - 之后按已收集数阶梯给翻卡机会: 0-10 张 → 3 天/次, 11-25 张 → 2 天/次, 26+ → 1 天/次
 *   - 翻卡机会不累积,最多 1 次可用
 *
 * 持久化: settings KV (persona_revealed_cards / persona_last_flip_date)
 */

import { computed, ref } from "vue";

import { invokeCmd } from "@/composables/useTauriInvoke";
import { PERSONA_META, type BasePersonaMeta } from "@/data/personas";
import type { DaySummary } from "@/types";

const STORAGE_KEYS = {
  revealedCards: "persona_revealed_cards",
  lastFlipDate: "persona_last_flip_date",
} as const;

const revealedCards = ref<Set<string>>(new Set());
const lastFlipDate = ref<string | null>(null);
const matchedPersona = ref<string | null>(null);
const initialized = ref(false);

function todayLocal(): string {
  const d = new Date();
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
}

function daysBetween(a: string, b: string): number {
  const da = new Date(`${a}T00:00:00`);
  const db = new Date(`${b}T00:00:00`);
  return Math.floor((db.getTime() - da.getTime()) / 86_400_000);
}

function flipInterval(revealedCount: number): number {
  if (revealedCount >= 26) return 1;
  if (revealedCount >= 11) return 2;
  return 3;
}

const revealedCount = computed(() => revealedCards.value.size);

const currentInterval = computed(() => flipInterval(revealedCount.value));

const flipAvailable = computed(() => {
  if (!lastFlipDate.value) return true;
  return daysBetween(lastFlipDate.value, todayLocal()) >= currentInterval.value;
});

const daysUntilFlip = computed(() => {
  if (flipAvailable.value) return 0;
  if (!lastFlipDate.value) return 0;
  const passed = daysBetween(lastFlipDate.value, todayLocal());
  return Math.max(currentInterval.value - passed, 0);
});

function isRevealed(name: string): boolean {
  return revealedCards.value.has(name);
}

async function persistState() {
  const arr = Array.from(revealedCards.value);
  await invokeCmd("set_setting", {
    key: STORAGE_KEYS.revealedCards,
    value: JSON.stringify(arr),
  });
  if (lastFlipDate.value) {
    await invokeCmd("set_setting", {
      key: STORAGE_KEYS.lastFlipDate,
      value: lastFlipDate.value,
    });
  }
}

async function useFlipCard(name: string): Promise<boolean> {
  if (!flipAvailable.value) return false;
  revealedCards.value = new Set([...revealedCards.value, name]);
  lastFlipDate.value = todayLocal();
  await persistState();
  return true;
}

// ---- 简化人格匹配: 5 维画像 → 欧几里得距离 ----

function clamp(v: number, lo: number, hi: number): number {
  return Math.max(lo, Math.min(hi, v));
}

function computeUserDims(summaries: DaySummary[]): [number, number, number, number, number] {
  if (!summaries.length) return [3, 3, 3, 3, 3];

  const rates = summaries.map((s) => (s.totalTasks > 0 ? s.completedTasks / s.totalTasks : 0));
  const focusMins = summaries.map((s) => s.totalFocusMinutes);
  const taskCounts = summaries.map((s) => s.totalTasks);

  const avgRate = rates.reduce((a, b) => a + b, 0) / rates.length;
  const avgFocus = focusMins.reduce((a, b) => a + b, 0) / focusMins.length;
  const avgTasks = taskCounts.reduce((a, b) => a + b, 0) / taskCounts.length;

  const rateStd = Math.sqrt(
    rates.reduce((sum, r) => sum + (r - avgRate) ** 2, 0) / rates.length,
  );

  const hasLateCompletion = summaries.some(
    (s) => s.grade === "S" || s.grade === "A",
  );
  const lowEarlyRate = avgRate < 0.5;

  // dim1: 时段偏好 (无 sessions 时段数据,暂用中性值 3)
  const dim1 = 3;

  // dim2: DDL模式 (高完成率且稳定 → 提前型; 低稳定高爆发 → 临阵型)
  const dim2 = clamp(Math.round(1 + (rateStd * 10) + (lowEarlyRate ? 1 : 0) + (hasLateCompletion ? 0.5 : 0)), 1, 5);

  // dim3: 稳定性 (标准差越低越稳定)
  const dim3 = clamp(Math.round(5 - rateStd * 12), 1, 5);

  // dim4: 强度 (日均专注分钟 → 1-5: <30=1, 30-60=2, 60-120=3, 120-240=4, >240=5)
  const intensityMap = [30, 60, 120, 240];
  let dim4 = 1;
  for (const threshold of intensityMap) {
    if (avgFocus >= threshold) dim4++;
  }

  // dim5: 多线程 (日均任务数 → 1-5: <2=1, 2-3=2, 3-5=3, 5-8=4, >8=5)
  const multiMap = [2, 3, 5, 8];
  let dim5 = 1;
  for (const threshold of multiMap) {
    if (avgTasks >= threshold) dim5++;
  }

  return [dim1, dim2, dim3, dim4 as number, dim5 as number];
}

function euclideanDist(a: number[], b: number[]): number {
  return Math.sqrt(a.reduce((sum, v, i) => sum + (v - b[i]) ** 2, 0));
}

async function matchPersona(): Promise<string> {
  const today = todayLocal();
  const d = new Date();
  d.setDate(d.getDate() - 7);
  const from = `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;

  let summaries: DaySummary[] = [];
  try {
    summaries = await invokeCmd<DaySummary[]>("list_day_summaries", { from, to: today });
  } catch {
    // 数据不足时使用默认值
  }

  const userDims = computeUserDims(summaries);

  let bestName = "";
  let bestDist = Infinity;

  for (const [name, meta] of Object.entries(PERSONA_META)) {
    if (meta.kind !== "base") continue;
    const dist = euclideanDist(userDims, (meta as BasePersonaMeta).dims);
    if (dist < bestDist) {
      bestDist = dist;
      bestName = name;
    }
  }

  return bestName || "全能异常体";
}

async function init(hatchDay: number) {
  if (initialized.value) return;

  try {
    const raw = await invokeCmd<string | null>("get_setting", {
      key: STORAGE_KEYS.revealedCards,
    });
    if (raw) {
      const arr: string[] = JSON.parse(raw);
      revealedCards.value = new Set(arr);
    }
  } catch {
    // 首次使用,空集合
  }

  try {
    const d = await invokeCmd<string | null>("get_setting", {
      key: STORAGE_KEYS.lastFlipDate,
    });
    if (d) lastFlipDate.value = d;
  } catch {
    // ignore
  }

  if (hatchDay >= 7) {
    const name = await matchPersona();
    matchedPersona.value = name;
    if (!revealedCards.value.has(name)) {
      revealedCards.value = new Set([...revealedCards.value, name]);
      await persistState();
    }
  }

  initialized.value = true;
}

export function usePersonaFlipCard() {
  return {
    revealedCards,
    revealedCount,
    matchedPersona,
    flipAvailable,
    daysUntilFlip,
    currentInterval,
    initialized,
    isRevealed,
    useFlipCard,
    init,
  };
}
