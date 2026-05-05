import type { ShortcutPlatform } from "@/shortcuts/normalize";
import { normalizeShortcut } from "@/shortcuts/normalize";

export interface ShortcutBinding {
  shortcut: string | null;
  enabled: boolean;
  scope: "app" | "global";
}

export type ShortcutBindingMap = Record<string, ShortcutBinding>;

export interface ShortcutConflict {
  shortcut: string;
  actionIds: string[];
}

export interface ReservedShortcutIssue {
  shortcut: string;
  platform: ShortcutPlatform;
  reason: "系统保留快捷键";
}

const RESERVED_SHORTCUTS: Record<ShortcutPlatform, Set<string>> = {
  macos: new Set([
    "Mod+Q",
    "Mod+H",
    "Mod+M",
    "Mod+Space",
    "Mod+Tab",
    "Mod+Alt+Escape",
  ]),
  windows: new Set([
    "Alt+F4",
    "Alt+Tab",
    "Ctrl+Alt+Delete",
    "Meta+L",
    "Meta+D",
    "Meta+Tab",
    "Meta+R",
  ]),
  linux: new Set([]),
};

export function detectShortcutConflicts(bindings: ShortcutBindingMap): ShortcutConflict[] {
  const grouped = new Map<string, string[]>();

  Object.entries(bindings).forEach(([actionId, binding]) => {
    if (!binding.enabled || !binding.shortcut) return;
    const shortcut = normalizeShortcut(binding.shortcut);
    if (!shortcut) return;
    const actionIds = grouped.get(shortcut) ?? [];
    actionIds.push(actionId);
    grouped.set(shortcut, actionIds);
  });

  return Array.from(grouped.entries())
    .filter(([, actionIds]) => actionIds.length > 1)
    .map(([shortcut, actionIds]) => ({
      shortcut,
      actionIds: [...actionIds].sort(),
    }))
    .sort((a, b) => a.shortcut.localeCompare(b.shortcut));
}

export function detectReservedShortcut(
  shortcut: string | null,
  platform: ShortcutPlatform,
): ReservedShortcutIssue | null {
  const normalized = normalizeShortcut(shortcut);
  if (!normalized) return null;
  if (!RESERVED_SHORTCUTS[platform].has(normalized)) return null;
  return {
    shortcut: normalized,
    platform,
    reason: "系统保留快捷键",
  };
}
