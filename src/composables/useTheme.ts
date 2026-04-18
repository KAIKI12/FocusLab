/**
 * useTheme · 主题切换 composable
 *
 * 两个正交轴:
 *   - mode: 'light' | 'dark' | 'auto'   (`data-theme` 属性)
 *   - accent: 主题色调 id  ('default' 表示不设,走 :root)
 *
 * 持久化:
 *   - localStorage.fl-theme
 *   - localStorage.fl-accent
 * 跨窗口同步靠 storage 事件(悬浮球 / 快捷面板启用后自动生效)。
 *
 * 主题初次应用在 index.html 的 fl-theme-init 内联脚本(避免首屏闪白);
 * 本 composable 用于应用内程序化切换。
 */

import { readonly, ref } from "vue";

export type ThemeMode = "light" | "dark" | "auto";

const STORAGE_KEY_MODE = "fl-theme";
const STORAGE_KEY_ACCENT = "fl-accent";

function readMode(): ThemeMode {
  const v = (localStorage.getItem(STORAGE_KEY_MODE) as ThemeMode | null) ?? "light";
  return v === "light" || v === "dark" || v === "auto" ? v : "light";
}

function readAccent(): string {
  return localStorage.getItem(STORAGE_KEY_ACCENT) ?? "default";
}

function prefersDark(): boolean {
  return window.matchMedia?.("(prefers-color-scheme: dark)")?.matches ?? false;
}

function applyDom(mode: ThemeMode, accent: string) {
  const d = document.documentElement;
  if (mode === "auto") {
    d.dataset.theme = prefersDark() ? "dark" : "light";
  } else {
    d.dataset.theme = mode;
  }
  if (accent && accent !== "default") {
    d.dataset.accentTheme = accent;
  } else {
    delete d.dataset.accentTheme;
  }
}

// 单例状态,避免多个组件各自持有不同副本
const mode = ref<ThemeMode>(readMode());
const accent = ref<string>(readAccent());

function setMode(next: ThemeMode) {
  mode.value = next;
  localStorage.setItem(STORAGE_KEY_MODE, next);
  applyDom(next, accent.value);
}

function setAccent(next: string) {
  accent.value = next;
  if (next && next !== "default") {
    localStorage.setItem(STORAGE_KEY_ACCENT, next);
  } else {
    localStorage.removeItem(STORAGE_KEY_ACCENT);
  }
  applyDom(mode.value, next);
}

// 监听 storage 事件 — 来自其他窗口 / tab 的变更
if (typeof window !== "undefined") {
  window.addEventListener("storage", (e) => {
    if (e.key === STORAGE_KEY_MODE) {
      mode.value = readMode();
      applyDom(mode.value, accent.value);
    } else if (e.key === STORAGE_KEY_ACCENT) {
      accent.value = readAccent();
      applyDom(mode.value, accent.value);
    }
  });

  // auto 模式下跟随系统主题变化
  window.matchMedia?.("(prefers-color-scheme: dark)")?.addEventListener("change", () => {
    if (mode.value === "auto") applyDom(mode.value, accent.value);
  });
}

export function useTheme() {
  return {
    mode: readonly(mode),
    accent: readonly(accent),
    setMode,
    setAccent,
  };
}
