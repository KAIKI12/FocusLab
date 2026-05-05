import { watch } from "vue";
import { register, unregisterAll } from "@tauri-apps/plugin-global-shortcut";
import { invokeCmd } from "@/composables/useTauriInvoke";
import { useTheme } from "@/composables/useTheme";
import router from "@/router";
import { useSettlementStore } from "@/stores/useSettlementStore";
import { useTimerStore } from "@/stores/useTimerStore";
import { useShortcutStore } from "@/stores/useShortcutStore";
import { eventMatchesShortcut, toGlobalShortcut } from "@/shortcuts/runtime";

function shouldIgnoreGlobalEvent(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  const tagName = target.tagName.toLowerCase();
  return tagName === "input" || tagName === "textarea" || target.isContentEditable;
}

async function ensureMainWindow() {
  await invokeCmd("show_main_window");
}

export function useShortcutRuntime() {
  const timer = useTimerStore();
  const settlement = useSettlementStore();
  const shortcutStore = useShortcutStore();
  const { mode: themeMode, setMode } = useTheme();
  let stopSyncWatcher: (() => void) | null = null;

  const actions: Record<string, () => void | Promise<void>> = {
    "focus.togglePause": async () => {
      if (timer.isPaused) await timer.resume();
      else if (timer.isRunning) await timer.pause();
    },
    "focus.abandonPomodoro": async () => {
      await timer.abandon();
    },
    "task.quickAdd": async () => {
      await invokeCmd("show_quick_add_window");
    },
    "task.quickNote": async () => {
      await invokeCmd("show_quick_note_window");
    },
    "day.settle": async () => {
      if (router.currentRoute.value.meta.hideLayout) await router.push("/today");
      await settlement.settle();
    },
    "nav.today": async () => { await ensureMainWindow(); await router.push("/today"); },
    "nav.goals": async () => { await ensureMainWindow(); await router.push("/goals"); },
    "nav.calendar": async () => { await ensureMainWindow(); await router.push("/calendar"); },
    "nav.stats": async () => { await ensureMainWindow(); await router.push("/stats"); },
    "nav.settings": async () => { await ensureMainWindow(); await router.push("/settings"); },
    "ui.commandPalette": async () => {
      await invokeCmd("show_command_palette_window");
    },
    "ui.toggleTheme": () => {
      setMode(themeMode.value === "dark" ? "light" : "dark");
    },
  };

  const onKeydown = async (event: KeyboardEvent) => {
    if (shouldIgnoreGlobalEvent(event.target)) return;
    for (const [actionId, binding] of Object.entries(shortcutStore.bindings)) {
      if (!binding.enabled || binding.scope !== "app") continue;
      if (!eventMatchesShortcut(event, binding.shortcut)) continue;
      event.preventDefault();
      await actions[actionId]?.();
      return;
    }
  };

  async function syncGlobalShortcuts() {
    await unregisterAll();
    const globalBindings = Object.entries(shortcutStore.bindings)
      .filter(([, binding]) => binding.enabled && binding.scope === "global")
      .map(([actionId, binding]) => ({ actionId, shortcut: binding.shortcut, globalShortcut: toGlobalShortcut(binding.shortcut) }))
      .filter((item): item is { actionId: string; shortcut: string; globalShortcut: string } => Boolean(item.shortcut && item.globalShortcut));
    const nextIssues = [] as Array<{ actionId: string; shortcut: string; reason: "可能被系统或其他软件占用" }>;

    for (const binding of globalBindings) {
      try {
        await register(binding.globalShortcut, async (event) => {
          if (event.state !== "Pressed") return;
          await actions[binding.actionId]?.();
        });
      } catch {
        nextIssues.push({
          actionId: binding.actionId,
          shortcut: binding.shortcut,
          reason: "可能被系统或其他软件占用",
        });
      }
    }

    shortcutStore.setRegistrationIssues(nextIssues);
  }

  function mount() {
    window.addEventListener("keydown", onKeydown);
    stopSyncWatcher = watch(
      () => JSON.stringify(shortcutStore.bindings),
      () => {
        void syncGlobalShortcuts();
      },
      { immediate: true },
    );
  }

  function unmount() {
    window.removeEventListener("keydown", onKeydown);
    stopSyncWatcher?.();
    stopSyncWatcher = null;
    void unregisterAll();
  }

  return {
    mount,
    unmount,
    syncGlobalShortcuts,
  };
}
