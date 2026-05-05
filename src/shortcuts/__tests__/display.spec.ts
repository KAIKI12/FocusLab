import { describe, expect, it } from "vitest";

import { formatShortcutForPlatform } from "@/shortcuts/display";

describe("formatShortcutForPlatform", () => {
  it("formats logical Mod keys for macos", () => {
    expect(formatShortcutForPlatform("Mod+Shift+N", "macos")).toBe("⌘⇧N");
    expect(formatShortcutForPlatform("Mod+/", "macos")).toBe("⌘/");
  });

  it("formats logical Mod keys for windows", () => {
    expect(formatShortcutForPlatform("Mod+Shift+N", "windows")).toBe("Ctrl+Shift+N");
    expect(formatShortcutForPlatform("Mod+,", "windows")).toBe("Ctrl+,");
  });

  it("formats Space without altering it", () => {
    expect(formatShortcutForPlatform("Space", "macos")).toBe("Space");
  });
});
