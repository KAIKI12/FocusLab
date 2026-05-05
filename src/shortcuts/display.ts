import type { ShortcutPlatform } from "@/shortcuts/normalize";

const MAC_LABELS: Record<string, string> = {
  Mod: "⌘",
  Shift: "⇧",
  Alt: "⌥",
  Ctrl: "⌃",
  Meta: "⌃",
};

const WINDOWS_LABELS: Record<string, string> = {
  Mod: "Ctrl",
  Shift: "Shift",
  Alt: "Alt",
  Ctrl: "Ctrl",
  Meta: "Win",
};

export function formatShortcutForPlatform(shortcut: string | null, platform: ShortcutPlatform): string {
  if (!shortcut) return "";
  const parts = shortcut.split("+");
  if (platform === "macos") {
    return parts.map((part) => MAC_LABELS[part] ?? part).join("");
  }
  return parts.map((part) => WINDOWS_LABELS[part] ?? part).join("+");
}
