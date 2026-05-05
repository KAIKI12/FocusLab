import type { ShortcutPlatform } from "@/shortcuts/normalize";

export function detectShortcutPlatform(platform: string): ShortcutPlatform {
  const normalized = platform.toLowerCase();
  if (normalized.includes("mac")) return "macos";
  if (normalized.includes("win")) return "windows";
  return "linux";
}
