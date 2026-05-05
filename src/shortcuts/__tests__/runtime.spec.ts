import { describe, expect, it } from "vitest";

import {
  eventMatchesShortcut,
  toGlobalShortcut,
} from "@/shortcuts/runtime";

function makeKeyboardEvent(init: Partial<KeyboardEvent>): KeyboardEvent {
  return {
    key: "",
    ctrlKey: false,
    metaKey: false,
    altKey: false,
    shiftKey: false,
    ...init,
  } as KeyboardEvent;
}

describe("eventMatchesShortcut", () => {
  it("matches logical Mod shortcuts on ctrl keyboards", () => {
    expect(eventMatchesShortcut(makeKeyboardEvent({
      key: "n",
      ctrlKey: true,
      shiftKey: true,
    }), "Mod+Shift+N")).toBe(true);
  });

  it("matches logical Mod shortcuts on meta keyboards", () => {
    expect(eventMatchesShortcut(makeKeyboardEvent({
      key: "n",
      metaKey: true,
      shiftKey: true,
    }), "Mod+Shift+N")).toBe(true);
  });

  it("does not match when required modifiers are missing", () => {
    expect(eventMatchesShortcut(makeKeyboardEvent({
      key: "n",
      ctrlKey: true,
    }), "Mod+Shift+N")).toBe(false);
  });

  it("matches bare Space", () => {
    expect(eventMatchesShortcut(makeKeyboardEvent({ key: " " }), "Space")).toBe(true);
  });
});

describe("toGlobalShortcut", () => {
  it("maps logical Mod to CommandOrControl", () => {
    expect(toGlobalShortcut("Mod+Shift+N")).toBe("CommandOrControl+Shift+N");
  });

  it("keeps regular modifiers readable for the plugin", () => {
    expect(toGlobalShortcut("Alt+F4")).toBe("Alt+F4");
  });
});
