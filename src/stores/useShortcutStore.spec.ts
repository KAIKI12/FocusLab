import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { useShortcutStore } from "@/stores/useShortcutStore";

describe("useShortcutStore", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  it("loads defaults when no persisted config exists", async () => {
    vi.mocked(invoke).mockResolvedValueOnce(null);

    const store = useShortcutStore();
    await store.load();

    expect(invoke).toHaveBeenCalledWith("get_setting", { key: "shortcuts_config_v1" });
    expect(store.bindings["ui.commandPalette"]).toEqual({
      shortcut: "Mod+/",
      enabled: true,
      scope: "global",
    });
  });

  it("loads persisted bindings and normalizes shortcuts", async () => {
    vi.mocked(invoke).mockResolvedValueOnce(JSON.stringify({
      version: 1,
      bindings: {
        "task.quickAdd": { shortcut: "cmd+n", enabled: true, scope: "global" },
      },
    }));

    const store = useShortcutStore();
    await store.load();

    expect(store.bindings["task.quickAdd"]).toEqual({
      shortcut: "Mod+N",
      enabled: true,
      scope: "global",
    });
  });

  it("reports internal conflicts after updating a binding", () => {
    const store = useShortcutStore();

    store.updateBinding("day.settle", { shortcut: "Mod+N" });

    expect(store.conflicts).toEqual([
      {
        shortcut: "Mod+N",
        actionIds: ["day.settle", "task.quickAdd"],
      },
    ]);
    expect(store.canSave).toBe(false);
  });

  it("reports reserved shortcuts on the active platform", () => {
    const store = useShortcutStore();

    store.setPlatform("windows");
    store.updateBinding("task.quickAdd", { shortcut: "Alt+F4" });

    expect(store.reservedIssues).toEqual([
      {
        actionId: "task.quickAdd",
        shortcut: "Alt+F4",
        platform: "windows",
        reason: "系统保留快捷键",
      },
    ]);
    expect(store.canSave).toBe(false);
  });

  it("tracks global shortcut registration issues separately from validation", () => {
    const store = useShortcutStore();

    store.setRegistrationIssues([
      {
        actionId: "task.quickAdd",
        shortcut: "Mod+N",
        reason: "可能被系统或其他软件占用",
      },
    ]);

    expect(store.registrationIssues).toEqual([
      {
        actionId: "task.quickAdd",
        shortcut: "Mod+N",
        reason: "可能被系统或其他软件占用",
      },
    ]);
    expect(store.canSave).toBe(true);
  });

  it("saves normalized config when validation passes", async () => {
    vi.mocked(invoke).mockResolvedValueOnce(undefined);

    const store = useShortcutStore();
    store.updateBinding("task.quickAdd", { shortcut: " cmd + shift + n ", scope: "global" });
    store.updateBinding("task.quickNote", { shortcut: "Alt+Shift+N", scope: "app" });

    await store.save();

    expect(invoke).toHaveBeenCalledWith("set_setting", {
      key: "shortcuts_config_v1",
      value: JSON.stringify({
        version: 1,
        bindings: store.bindings,
      }),
    });
    expect(store.lastSavedAt).not.toBeNull();
  });
});

