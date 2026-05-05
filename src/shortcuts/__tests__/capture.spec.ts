import { describe, expect, it } from "vitest";

import { shortcutFromKeyboardEvent } from "@/shortcuts/capture";

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

describe("shortcutFromKeyboardEvent", () => {
  it("captures logical Mod shortcuts on ctrl keyboards", () => {
    expect(shortcutFromKeyboardEvent(makeKeyboardEvent({
      key: "n",
      ctrlKey: true,
      shiftKey: true,
    }))).toBe("Mod+Shift+N");
  });

  it("captures logical Mod shortcuts on meta keyboards", () => {
    expect(shortcutFromKeyboardEvent(makeKeyboardEvent({
      key: "n",
      metaKey: true,
      shiftKey: true,
    }))).toBe("Mod+Shift+N");
  });

  it("returns Space for bare space", () => {
    expect(shortcutFromKeyboardEvent(makeKeyboardEvent({ key: " " }))).toBe("Space");
  });

  it("ignores pure modifier presses", () => {
    expect(shortcutFromKeyboardEvent(makeKeyboardEvent({ key: "Shift", shiftKey: true }))).toBeNull();
    expect(shortcutFromKeyboardEvent(makeKeyboardEvent({ key: "Meta", metaKey: true }))).toBeNull();
  });
});
