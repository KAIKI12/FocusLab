import { normalizeShortcut } from "@/shortcuts/normalize";

function normalizeEventKey(key: string): string {
  if (key === " ") return "Space";
  if (key.length === 1) return key.toUpperCase();
  return key.charAt(0).toUpperCase() + key.slice(1).toLowerCase();
}

export function eventMatchesShortcut(event: KeyboardEvent, shortcut: string | null): boolean {
  const normalized = normalizeShortcut(shortcut);
  if (!normalized) return false;

  const parts = normalized.split("+");
  const key = parts[parts.length - 1];
  const modifiers = new Set(parts.slice(0, -1));
  const requiresCtrl = modifiers.has("Ctrl");
  const requiresMeta = modifiers.has("Meta");
  const requiresMod = modifiers.has("Mod");

  if (normalizeEventKey(event.key) !== key) return false;
  if (event.altKey !== modifiers.has("Alt")) return false;
  if (event.shiftKey !== modifiers.has("Shift")) return false;
  if (requiresCtrl && !event.ctrlKey) return false;
  if (requiresMeta && !event.metaKey) return false;
  if (requiresMod && !(event.ctrlKey || event.metaKey)) return false;
  if (!requiresCtrl && !requiresMod && event.ctrlKey) return false;
  if (!requiresMeta && !requiresMod && event.metaKey) return false;
  return true;
}

export function toGlobalShortcut(shortcut: string | null): string | null {
  const normalized = normalizeShortcut(shortcut);
  if (!normalized) return null;
  return normalized
    .split("+")
    .map((part) => {
      if (part === "Mod") return "CommandOrControl";
      return part;
    })
    .join("+");
}
