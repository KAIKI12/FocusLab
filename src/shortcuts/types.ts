export type ShortcutScope = "app" | "global";

export interface ShortcutBinding {
  shortcut: string | null;
  enabled: boolean;
  scope: ShortcutScope;
}

export interface ShortcutConfigV1 {
  version: 1;
  bindings: Record<string, ShortcutBinding>;
}
