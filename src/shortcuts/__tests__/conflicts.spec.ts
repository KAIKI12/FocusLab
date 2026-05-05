import { describe, expect, it } from "vitest";

import {
  detectReservedShortcut,
  detectShortcutConflicts,
  type ShortcutBindingMap,
} from "@/shortcuts/conflicts";

function makeBindings(overrides: Partial<ShortcutBindingMap> = {}): ShortcutBindingMap {
  return {
    "ui.commandPalette": { shortcut: "Mod+/", enabled: true, scope: "global" },
    "task.quickAdd": { shortcut: "Mod+N", enabled: true, scope: "global" },
    "focus.togglePause": { shortcut: "Space", enabled: true, scope: "app" },
    ...overrides,
  };
}

describe("detectShortcutConflicts", () => {
  it("flags duplicate shortcuts across enabled bindings", () => {
    const conflicts = detectShortcutConflicts(makeBindings({
      "day.settle": { shortcut: "Mod+N", enabled: true, scope: "app" },
    }));

    expect(conflicts).toEqual([
      {
        shortcut: "Mod+N",
        actionIds: ["day.settle", "task.quickAdd"],
      },
    ]);
  });

  it("ignores disabled or empty shortcuts", () => {
    const conflicts = detectShortcutConflicts(makeBindings({
      "day.settle": { shortcut: "Mod+N", enabled: false, scope: "app" },
      "nav.today": { shortcut: null, enabled: true, scope: "app" },
    }));

    expect(conflicts).toEqual([]);
  });
});

describe("detectReservedShortcut", () => {
  it("blocks common macos reserved shortcuts", () => {
    expect(detectReservedShortcut("Mod+Q", "macos")).toEqual({
      shortcut: "Mod+Q",
      platform: "macos",
      reason: "系统保留快捷键",
    });
  });

  it("blocks common windows reserved shortcuts", () => {
    expect(detectReservedShortcut("Alt+F4", "windows")).toEqual({
      shortcut: "Alt+F4",
      platform: "windows",
      reason: "系统保留快捷键",
    });
    expect(detectReservedShortcut("Meta+L", "windows")).toEqual({
      shortcut: "Meta+L",
      platform: "windows",
      reason: "系统保留快捷键",
    });
  });

  it("returns null for safe shortcuts", () => {
    expect(detectReservedShortcut("Mod+Shift+N", "windows")).toBeNull();
  });
});
