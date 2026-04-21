/**
 * Smoke test:确认 vitest 基础设施 + @tauri-apps/api/core mock 生效。
 */

import { describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";

describe("vitest setup", () => {
  it("invoke is mocked", () => {
    expect(vi.isMockFunction(invoke)).toBe(true);
  });

  it("invoke returns whatever we resolve", async () => {
    vi.mocked(invoke).mockResolvedValueOnce("hello");
    const result = await invoke("anything");
    expect(result).toBe("hello");
  });
});
