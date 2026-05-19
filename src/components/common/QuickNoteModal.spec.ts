import { describe, expect, it } from "vitest";

import source from "./QuickNoteModal.vue?raw";
import inspirationPanelSource from "../inspiration/InspirationPanel.vue?raw";

describe("QuickNoteModal", () => {
  it("does not close standalone quick-note windows from the child save path", () => {
    expect(source).toContain("function finishAfterSave()");
    expect(source).toContain("if (!props.standalone) emit(\"close\");");
  });

  it("does not enforce a 500-character cap in inspiration inputs", () => {
    expect(source).not.toContain("maxlength=\"500\"");
    expect(inspirationPanelSource).not.toContain("maxlength=\"500\"");
    expect(source).not.toContain("/ 500");
    expect(inspirationPanelSource).not.toContain("/ 500");
  });
});
