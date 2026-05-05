import { describe, expect, it } from "vitest";

import { normalizeShortcut } from "@/shortcuts/normalize";

describe("normalizeShortcut", () => {
  it("normalizes modifier order and uppercase keys", () => {
    expect(normalizeShortcut(" Shift + Mod + n ")).toBe("Mod+Shift+N");
  });

  it("normalizes mac aliases to logical modifiers", () => {
    expect(normalizeShortcut("cmd+shift+n")).toBe("Mod+Shift+N");
    expect(normalizeShortcut("option+/" )).toBe("Alt+/");
  });

  it("normalizes windows aliases", () => {
    expect(normalizeShortcut("control+1")).toBe("Ctrl+1");
    expect(normalizeShortcut("win+l")).toBe("Meta+L");
  });

  it("returns null for empty shortcuts", () => {
    expect(normalizeShortcut("   ")).toBeNull();
  });
});
