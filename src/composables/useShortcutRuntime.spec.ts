import { register, unregisterAll, type ShortcutEvent } from "@tauri-apps/plugin-global-shortcut";
import { beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  pushMock: vi.fn(),
  invokeCmdMock: vi.fn(),
  setModeMock: vi.fn(),
  settlementMock: {
    settle: vi.fn(),
  },
  timerMock: {
    isPaused: false,
    isRunning: false,
    pause: vi.fn(),
    resume: vi.fn(),
    abandon: vi.fn(),
  },
  uiMock: {
    showQuickAdd: false,
    showQuickNote: false,
  },
  shortcutStoreMock: {
    bindings: {
      "task.quickAdd": { shortcut: "Mod+N", enabled: true, scope: "global" as const },
      "task.quickNote": { shortcut: "Mod+Shift+N", enabled: true, scope: "global" as const },
      "ui.commandPalette": { shortcut: "Mod+/", enabled: true, scope: "global" as const },
    },
    setRegistrationIssues: vi.fn(),
  },
}));

vi.mock("vue", async () => {
  const actual = await vi.importActual<typeof import("vue")>("vue");
  return {
    ...actual,
    watch: vi.fn((_source, _callback, _options) => vi.fn()),
  };
});

vi.mock("@/composables/useTauriInvoke", () => ({
  invokeCmd: mocks.invokeCmdMock,
}));

vi.mock("@/composables/useTheme", () => ({
  useTheme: () => ({ mode: { value: "dark" }, setMode: mocks.setModeMock }),
}));

vi.mock("@/router", () => ({
  default: {
    currentRoute: { value: { path: "/today", meta: {} } },
    push: mocks.pushMock,
  },
}));

vi.mock("@/stores/useSettlementStore", () => ({
  useSettlementStore: () => mocks.settlementMock,
}));

vi.mock("@/stores/useTimerStore", () => ({
  useTimerStore: () => mocks.timerMock,
}));

vi.mock("@/stores/useUIStore", () => ({
  useUIStore: () => mocks.uiMock,
}));

vi.mock("@/stores/useShortcutStore", () => ({
  useShortcutStore: () => mocks.shortcutStoreMock,
}));

import { useShortcutRuntime } from "@/composables/useShortcutRuntime";

describe("useShortcutRuntime", () => {
  beforeEach(() => {
    vi.mocked(register).mockReset();
    vi.mocked(unregisterAll).mockReset();
    mocks.shortcutStoreMock.setRegistrationIssues.mockReset();
    mocks.pushMock.mockReset();
    mocks.invokeCmdMock.mockReset();
    mocks.uiMock.showQuickAdd = false;
    mocks.uiMock.showQuickNote = false;
  });

  it("records global shortcut registration failures per action", async () => {
    vi.mocked(unregisterAll).mockResolvedValue(undefined);
    vi.mocked(register)
      .mockRejectedValueOnce(new Error("busy"))
      .mockResolvedValueOnce(undefined)
      .mockResolvedValueOnce(undefined);

    const runtime = useShortcutRuntime();
    await runtime.syncGlobalShortcuts();

    expect(mocks.shortcutStoreMock.setRegistrationIssues).toHaveBeenCalledWith([
      {
        actionId: "task.quickAdd",
        shortcut: "Mod+N",
        reason: "可能被系统或其他软件占用",
      },
    ]);
  });

  it("clears global shortcut registration issues after successful sync", async () => {
    vi.mocked(unregisterAll).mockResolvedValue(undefined);
    vi.mocked(register).mockResolvedValue(undefined);

    const runtime = useShortcutRuntime();
    await runtime.syncGlobalShortcuts();

    expect(mocks.shortcutStoreMock.setRegistrationIssues).toHaveBeenCalledWith([]);
  });

  it("opens quick add window without showing main window", async () => {
    vi.mocked(unregisterAll).mockResolvedValue(undefined);
    let quickAddHandler: ((event: ShortcutEvent) => void | Promise<void>) | undefined;
    vi.mocked(register).mockImplementation(async (accelerator, handler) => {
      if (accelerator === "CommandOrControl+N") quickAddHandler = handler;
    });

    const runtime = useShortcutRuntime();
    await runtime.syncGlobalShortcuts();
    await quickAddHandler?.({ state: "Pressed" } as ShortcutEvent);

    expect(mocks.invokeCmdMock).toHaveBeenCalledWith("show_quick_add_window");
    expect(mocks.invokeCmdMock).not.toHaveBeenCalledWith("show_main_window");
  });

  it("opens quick note window without showing main window", async () => {
    vi.mocked(unregisterAll).mockResolvedValue(undefined);
    let quickNoteHandler: ((event: ShortcutEvent) => void | Promise<void>) | undefined;
    vi.mocked(register).mockImplementation(async (accelerator, handler) => {
      if (accelerator === "CommandOrControl+Shift+N") quickNoteHandler = handler;
    });

    const runtime = useShortcutRuntime();
    await runtime.syncGlobalShortcuts();
    await quickNoteHandler?.({ state: "Pressed" } as ShortcutEvent);

    expect(mocks.invokeCmdMock).toHaveBeenCalledWith("show_quick_note_window");
    expect(mocks.invokeCmdMock).not.toHaveBeenCalledWith("show_main_window");
  });

  it("opens command palette window without showing main window", async () => {
    vi.mocked(unregisterAll).mockResolvedValue(undefined);
    let commandPaletteHandler: ((event: ShortcutEvent) => void | Promise<void>) | undefined;
    vi.mocked(register).mockImplementation(async (accelerator, handler) => {
      if (accelerator === "CommandOrControl+/") commandPaletteHandler = handler;
    });

    const runtime = useShortcutRuntime();
    await runtime.syncGlobalShortcuts();
    await commandPaletteHandler?.({ state: "Pressed" } as ShortcutEvent);

    expect(mocks.invokeCmdMock).toHaveBeenCalledWith("show_command_palette_window");
    expect(mocks.invokeCmdMock).not.toHaveBeenCalledWith("show_main_window");
  });
});

