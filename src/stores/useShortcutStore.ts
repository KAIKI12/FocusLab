import type { ShortcutBindingMap } from "@/shortcuts/conflicts";
import type { ShortcutBinding } from "@/shortcuts/types";
import { defineStore } from "pinia";
import { computed, ref } from "vue";

import { invokeCmd } from "@/composables/useTauriInvoke";
import {
  detectReservedShortcut,
  detectShortcutConflicts,
} from "@/shortcuts/conflicts";
import {
  cloneShortcutBindings,
  DEFAULT_SHORTCUT_CONFIG,
  SHORTCUT_SETTING_KEY,
} from "@/shortcuts/definitions";
import { normalizeShortcut, type ShortcutPlatform } from "@/shortcuts/normalize";

export interface ShortcutReservedIssue {
  actionId: string;
  shortcut: string;
  platform: ShortcutPlatform;
  reason: "系统保留快捷键";
}

export interface ShortcutRegistrationIssue {
  actionId: string;
  shortcut: string;
  reason: "可能被系统或其他软件占用";
}

function normalizeBinding(binding: ShortcutBinding): ShortcutBinding {
  return {
    shortcut: normalizeShortcut(binding.shortcut),
    enabled: binding.enabled,
    scope: binding.scope,
  };
}

export const useShortcutStore = defineStore("shortcut", () => {
  const bindings = ref<ShortcutBindingMap>(cloneShortcutBindings(DEFAULT_SHORTCUT_CONFIG.bindings));
  const platform = ref<ShortcutPlatform>("windows");
  const lastSavedAt = ref<string | null>(null);
  const registrationIssues = ref<ShortcutRegistrationIssue[]>([]);

  const conflicts = computed(() => detectShortcutConflicts(bindings.value));
  const reservedIssues = computed<ShortcutReservedIssue[]>(() => Object.entries(bindings.value)
    .map(([actionId, binding]) => {
      const issue = detectReservedShortcut(binding.shortcut, platform.value);
      return issue ? { actionId, ...issue } : null;
    })
    .filter((issue): issue is ShortcutReservedIssue => issue !== null));
  const canSave = computed(() => conflicts.value.length === 0 && reservedIssues.value.length === 0);

  async function load() {
    const raw = await invokeCmd<string | null>("get_setting", { key: SHORTCUT_SETTING_KEY });
    const next = cloneShortcutBindings(DEFAULT_SHORTCUT_CONFIG.bindings);
    if (raw) {
      const parsed = JSON.parse(raw) as { version?: number; bindings?: Record<string, ShortcutBinding> };
      Object.entries(parsed.bindings ?? {}).forEach(([actionId, binding]) => {
        next[actionId] = normalizeBinding({
          shortcut: binding.shortcut,
          enabled: binding.enabled,
          scope: binding.scope,
        });
      });
    }
    bindings.value = next;
  }

  function setPlatform(next: ShortcutPlatform) {
    platform.value = next;
  }

  function setRegistrationIssues(next: ShortcutRegistrationIssue[]) {
    registrationIssues.value = [...next];
  }

  function updateBinding(actionId: string, patch: Partial<ShortcutBinding>) {
    const current = bindings.value[actionId] ?? { shortcut: null, enabled: true, scope: "app" };
    bindings.value[actionId] = normalizeBinding({
      shortcut: patch.shortcut ?? current.shortcut,
      enabled: patch.enabled ?? current.enabled,
      scope: patch.scope ?? current.scope,
    });
    registrationIssues.value = registrationIssues.value.filter((issue) => issue.actionId !== actionId);
  }

  async function save() {
    if (!canSave.value) throw new Error("shortcut validation failed");
    const value = JSON.stringify({
      version: 1,
      bindings: bindings.value,
    });
    await invokeCmd("set_setting", { key: SHORTCUT_SETTING_KEY, value });
    lastSavedAt.value = new Date().toISOString();
  }

  return {
    bindings,
    platform,
    conflicts,
    reservedIssues,
    registrationIssues,
    canSave,
    lastSavedAt,
    load,
    save,
    setPlatform,
    setRegistrationIssues,
    updateBinding,
  };
});
