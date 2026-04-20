/**
 * useFixedSchedule · 每周固定日程(课程/组会/例会等)的存取。
 *
 * 对齐 prototype/screens/calendar.html 右栏"每周固定日程"和"本周可用工作时间"。
 * 存储:localStorage(单机本地,不跨设备同步)。
 * 用 module-level ref 确保多组件引用同一份数据。
 */

import { computed, ref } from "vue";

export interface FixedSchedule {
  id: string;
  /** 0=周日 .. 6=周六(匹配 Date.getDay()) */
  weekday: number;
  /** "HH:mm" */
  startTime: string;
  /** "HH:mm" */
  endTime: string;
  /** 用户直接写,建议前缀 emoji(📘 机器学习课程 / 🧑‍🤝‍🧑 组会) */
  title: string;
}

const STORAGE_KEY = "fl-fixed-schedules";

function load(): FixedSchedule[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw);
    return Array.isArray(parsed) ? parsed : [];
  } catch {
    return [];
  }
}

function persist(list: FixedSchedule[]) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(list));
}

function diffMinutes(start: string, end: string): number {
  const [sh, sm] = start.split(":").map(Number);
  const [eh, em] = end.split(":").map(Number);
  return eh * 60 + em - (sh * 60 + sm);
}

// 模块级单例 — 所有组件共享同一份响应式数据
const items = ref<FixedSchedule[]>(load());

export function useFixedSchedule() {
  function add(input: Omit<FixedSchedule, "id">): FixedSchedule {
    const full: FixedSchedule = { ...input, id: crypto.randomUUID() };
    items.value.push(full);
    persist(items.value);
    return full;
  }

  function remove(id: string) {
    items.value = items.value.filter((i) => i.id !== id);
    persist(items.value);
  }

  /** 按星期分组,组内按开始时间升序 */
  const byWeekday = computed<Record<number, FixedSchedule[]>>(() => {
    const map: Record<number, FixedSchedule[]> = { 0: [], 1: [], 2: [], 3: [], 4: [], 5: [], 6: [] };
    for (const it of items.value) {
      if (it.weekday >= 0 && it.weekday <= 6) map[it.weekday].push(it);
    }
    for (const k of Object.keys(map)) {
      map[+k].sort((a, b) => a.startTime.localeCompare(b.startTime));
    }
    return map;
  });

  /** 某星期的固定日程总分钟数 */
  function totalMinutesForWeekday(weekday: number): number {
    return items.value
      .filter((i) => i.weekday === weekday)
      .reduce((sum, i) => sum + diffMinutes(i.startTime, i.endTime), 0);
  }

  return { items, add, remove, byWeekday, totalMinutesForWeekday };
}
