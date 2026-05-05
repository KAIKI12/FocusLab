import type { ShortcutBinding, ShortcutConfigV1, ShortcutScope } from "@/shortcuts/types";

export interface ShortcutActionDefinition {
  id: string;
  label: string;
  group: string;
  defaultShortcut: string | null;
  defaultScope: ShortcutScope;
  allowGlobal: boolean;
}

export const SHORTCUT_SETTING_KEY = "shortcuts_config_v1";

export const SHORTCUT_ACTIONS: ShortcutActionDefinition[] = [
  { id: "focus.togglePause", label: "暂停 / 继续", group: "焦点/番茄钟", defaultShortcut: "Space", defaultScope: "app", allowGlobal: false },
  { id: "focus.abandonPomodoro", label: "结束番茄钟", group: "焦点/番茄钟", defaultShortcut: "Mod+Shift+X", defaultScope: "app", allowGlobal: false },
  { id: "task.quickAdd", label: "快速添加任务", group: "任务/计划", defaultShortcut: "Mod+N", defaultScope: "global", allowGlobal: true },
  { id: "task.quickNote", label: "速记便签", group: "任务/计划", defaultShortcut: "Mod+Shift+N", defaultScope: "global", allowGlobal: true },
  { id: "day.settle", label: "结束今天", group: "任务/计划", defaultShortcut: "Mod+Shift+E", defaultScope: "app", allowGlobal: false },
  { id: "nav.today", label: "今日计划", group: "视图/导航", defaultShortcut: "Mod+1", defaultScope: "app", allowGlobal: false },
  { id: "nav.goals", label: "长线目标", group: "视图/导航", defaultShortcut: "Mod+2", defaultScope: "app", allowGlobal: false },
  { id: "nav.calendar", label: "日历视图", group: "视图/导航", defaultShortcut: "Mod+3", defaultScope: "app", allowGlobal: false },
  { id: "nav.stats", label: "数据分析", group: "视图/导航", defaultShortcut: "Mod+4", defaultScope: "app", allowGlobal: false },
  { id: "nav.settings", label: "打开设置", group: "设置/模式", defaultShortcut: "Mod+,", defaultScope: "app", allowGlobal: false },
  { id: "ui.commandPalette", label: "命令面板", group: "设置/模式", defaultShortcut: "Mod+/", defaultScope: "global", allowGlobal: true },
  { id: "ui.toggleTheme", label: "切换主题", group: "设置/模式", defaultShortcut: "Mod+Shift+T", defaultScope: "app", allowGlobal: false },
];

const SHORTCUT_ACTION_MAP = Object.fromEntries(
  SHORTCUT_ACTIONS.map((definition) => [definition.id, definition]),
) as Record<string, ShortcutActionDefinition>;

export const DEFAULT_SHORTCUT_CONFIG: ShortcutConfigV1 = {
  version: 1,
  bindings: Object.fromEntries(
    SHORTCUT_ACTIONS.map((definition) => [definition.id, {
      shortcut: definition.defaultShortcut,
      enabled: true,
      scope: definition.defaultScope,
    }]),
  ),
};

export function getShortcutDefinition(actionId: string): ShortcutActionDefinition | null {
  return SHORTCUT_ACTION_MAP[actionId] ?? null;
}

export function getDefaultShortcutBinding(actionId: string): ShortcutBinding {
  const binding = DEFAULT_SHORTCUT_CONFIG.bindings[actionId];
  if (!binding) {
    return { shortcut: null, enabled: true, scope: "app" };
  }
  return { ...binding };
}

export function cloneShortcutBindings(bindings: Record<string, ShortcutBinding>): Record<string, ShortcutBinding> {
  return Object.fromEntries(
    Object.entries(bindings).map(([actionId, binding]) => [actionId, { ...binding }]),
  );
}
