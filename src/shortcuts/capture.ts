import { normalizeShortcut } from "@/shortcuts/normalize";

const MODIFIER_KEYS = new Set(["Shift", "Control", "Ctrl", "Meta", "Alt"]);

function normalizeKeyboardKey(key: string): string {
  if (key === " ") return "Space";
  if (key.length === 1) return key.toUpperCase();
  return key;
}

export function shortcutFromKeyboardEvent(event: KeyboardEvent): string | null {
  const key = normalizeKeyboardKey(event.key);
  if (MODIFIER_KEYS.has(key)) return null;

  const parts: string[] = [];
  if (event.ctrlKey || event.metaKey) parts.push("Mod");
  if (event.altKey) parts.push("Alt");
  if (event.shiftKey) parts.push("Shift");
  parts.push(key);

  return normalizeShortcut(parts.join("+"));
}
