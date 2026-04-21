/**
 * 全局测试准备:
 *   - Pinia 全局实例
 *   - Tauri 全家桶 mock(invoke / event listen)
 */

import { createPinia, setActivePinia } from "pinia";
import { beforeEach, vi } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
  emit: vi.fn(),
}));

vi.mock("@/composables/useSound", () => ({
  useSound: () => ({ play: vi.fn() }),
}));

beforeEach(() => {
  setActivePinia(createPinia());
});
