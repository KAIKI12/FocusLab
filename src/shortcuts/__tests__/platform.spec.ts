import { describe, expect, it } from "vitest";

import { detectShortcutPlatform } from "@/shortcuts/platform";

describe("detectShortcutPlatform", () => {
  it("detects macos from navigator platform", () => {
    expect(detectShortcutPlatform("MacIntel")).toBe("macos");
  });

  it("detects windows from navigator platform", () => {
    expect(detectShortcutPlatform("Win32")).toBe("windows");
  });

  it("falls back to linux for other platforms", () => {
    expect(detectShortcutPlatform("Linux x86_64")).toBe("linux");
  });
});
