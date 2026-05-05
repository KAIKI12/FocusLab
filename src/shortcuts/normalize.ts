export type ShortcutPlatform = "macos" | "windows" | "linux";

const MODIFIER_ORDER = ["Ctrl", "Mod", "Alt", "Shift", "Meta"] as const;

const MODIFIER_ALIAS: Record<string, (typeof MODIFIER_ORDER)[number]> = {
  cmd: "Mod",
  command: "Mod",
  mod: "Mod",
  ctrl: "Ctrl",
  control: "Ctrl",
  option: "Alt",
  alt: "Alt",
  shift: "Shift",
  win: "Meta",
  meta: "Meta",
  super: "Meta",
};

function normalizePart(raw: string): string {
  const trimmed = raw.trim();
  if (!trimmed) return "";
  const alias = MODIFIER_ALIAS[trimmed.toLowerCase()];
  if (alias) return alias;
  if (trimmed.length === 1) return trimmed.toUpperCase();
  if (trimmed.toLowerCase() === "space") return "Space";
  if (/^f\d{1,2}$/i.test(trimmed)) return trimmed.toUpperCase();
  return trimmed.charAt(0).toUpperCase() + trimmed.slice(1).toLowerCase();
}

export function normalizeShortcut(input: string | null | undefined): string | null {
  if (!input || !input.trim()) return null;

  const parts = input
    .split("+")
    .map(normalizePart)
    .filter(Boolean);

  if (parts.length === 0) return null;

  const modifiers = Array.from(new Set(parts.filter((part) => MODIFIER_ORDER.includes(part as never))));
  const keys = parts.filter((part) => !MODIFIER_ORDER.includes(part as never));
  const orderedModifiers = MODIFIER_ORDER.filter((modifier) => modifiers.includes(modifier));

  return [...orderedModifiers, ...keys].join("+");
}
