import { describe, expect, it } from "vitest";

import {
  DEFAULT_SHORTCUT_CONFIG,
  SHORTCUT_ACTIONS,
  SHORTCUT_SETTING_KEY,
  getDefaultShortcutBinding,
  getShortcutDefinition,
} from "@/shortcuts/definitions";

describe("shortcut definitions", () => {
  it("exposes a stable settings key", () => {
    expect(SHORTCUT_SETTING_KEY).toBe("shortcuts_config_v1");
  });

  it("contains defaults for both app and global actions", () => {
    expect(DEFAULT_SHORTCUT_CONFIG.bindings["ui.commandPalette"]).toEqual({
      shortcut: "Mod+/",
      enabled: true,
      scope: "global",
    });
    expect(DEFAULT_SHORTCUT_CONFIG.bindings["focus.togglePause"]).toEqual({
      shortcut: "Space",
      enabled: true,
      scope: "app",
    });
  });

  it("returns cloned bindings from helper", () => {
    const a = getDefaultShortcutBinding("task.quickAdd");
    const b = getDefaultShortcutBinding("task.quickAdd");
    expect(a).toEqual({ shortcut: "Mod+N", enabled: true, scope: "global" });
    expect(a).not.toBe(b);
  });

  it("exposes ordered shortcut action definitions", () => {
    expect(SHORTCUT_ACTIONS[0]).toMatchObject({
      id: "focus.togglePause",
      label: "暂停 / 继续",
      group: "焦点/番茄钟",
      defaultShortcut: "Space",
    });
    expect(getShortcutDefinition("task.quickNote")).toMatchObject({
      label: "速记便签",
      defaultScope: "global",
      allowGlobal: true,
    });
  });
});
