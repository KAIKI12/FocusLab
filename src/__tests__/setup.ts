import { isRegistered, register, unregisterAll } from "@tauri-apps/plugin-global-shortcut";
import { createPinia, setActivePinia } from "pinia";
import { beforeEach, vi } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
  emit: vi.fn(),
}));

vi.mock("@tauri-apps/plugin-global-shortcut", () => ({
  register: vi.fn(),
  unregister: vi.fn(),
  unregisterAll: vi.fn(),
  isRegistered: vi.fn(),
}));

vi.mock("@/composables/useSound", () => ({
  useSound: () => ({ play: vi.fn() }),
}));

beforeEach(() => {
  setActivePinia(createPinia());
  vi.mocked(register).mockReset();
  vi.mocked(unregisterAll).mockReset();
  vi.mocked(isRegistered).mockReset();
});
