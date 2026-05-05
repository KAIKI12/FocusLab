import { describe, expect, it } from "vitest";

import {
  getDisplayHatchDay,
  getRemainingHatchDays,
  isHatchComplete,
} from "@/composables/personaHatchProgress";

const HATCH_TOTAL_DAYS = 7;

describe("personaHatchProgress", () => {
  it("treats invalid values like 0 settlement days", () => {
    for (const value of [
      Number.NaN,
      -1,
      -99,
      Number.NEGATIVE_INFINITY,
      Number.POSITIVE_INFINITY,
    ]) {
      expect(getDisplayHatchDay(value)).toBe(1);
      expect(getRemainingHatchDays(value)).toBe(HATCH_TOTAL_DAYS);
      expect(isHatchComplete(value)).toBe(false);
    }
  });

  it("floors non-integer settlement days consistently", () => {
    expect(getDisplayHatchDay(1.9)).toBe(1);
    expect(getRemainingHatchDays(1.9)).toBe(HATCH_TOTAL_DAYS - 1);
    expect(isHatchComplete(1.9)).toBe(false);

    expect(getDisplayHatchDay(6.9)).toBe(6);
    expect(getRemainingHatchDays(6.9)).toBe(1);
    expect(isHatchComplete(6.9)).toBe(false);
  });

  it("treats zero settlement days as day 1 and still locked", () => {
    expect(getDisplayHatchDay(0)).toBe(1);
    expect(getRemainingHatchDays(0)).toBe(HATCH_TOTAL_DAYS);
    expect(isHatchComplete(0)).toBe(false);
  });

  it("keeps day 1 after the first settlement day", () => {
    expect(getDisplayHatchDay(1)).toBe(1);
    expect(getRemainingHatchDays(1)).toBe(HATCH_TOTAL_DAYS - 1);
    expect(isHatchComplete(1)).toBe(false);
  });

  it("shows the current cumulative settlement day before unlock", () => {
    expect(getDisplayHatchDay(6)).toBe(6);
    expect(getRemainingHatchDays(6)).toBe(1);
    expect(isHatchComplete(6)).toBe(false);
  });

  it("unlocks at seven settlement days", () => {
    expect(getDisplayHatchDay(HATCH_TOTAL_DAYS)).toBe(HATCH_TOTAL_DAYS);
    expect(getRemainingHatchDays(HATCH_TOTAL_DAYS)).toBe(0);
    expect(isHatchComplete(HATCH_TOTAL_DAYS)).toBe(true);
  });

  it("caps the visible progress after unlock", () => {
    expect(getDisplayHatchDay(99)).toBe(HATCH_TOTAL_DAYS);
    expect(getRemainingHatchDays(99)).toBe(0);
    expect(isHatchComplete(99)).toBe(true);
  });
});
