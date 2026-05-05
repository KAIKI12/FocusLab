# Popup Windows Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 将 quick-add、quick-note、command-palette 从主窗口内 modal 拆为独立 Tauri 窗口，并让全局快捷键/托盘对应入口不再唤起主窗口。

**Architecture:** 保留 `main` 作为完整应用窗口，新增 `quick-add`、`quick-note`、`command-palette` 三个轻量窗口。Tauri 后端负责按 label 创建/显示窗口，前端新增 popup routes 复用现有表单组件；导航类动作继续走主窗口语义，弹窗类动作直接调用独立窗口命令。

**Tech Stack:** Tauri 2, Rust, Vue 3, Pinia, Vue Router, Vitest

---

### Task 1: 为窗口命令写失败测试

**Files:**
- Modify: `src-tauri/src/commands/window_commands.rs`
- Test: `src/composables/useShortcutRuntime.spec.ts`

**Step 1: Write the failing test**

在 `src/composables/useShortcutRuntime.spec.ts` 新增 3 个用例：

```ts
it("opens quick add window without showing main window", async () => {
  // 触发 task.quickAdd
  expect(mocks.invokeCmdMock).toHaveBeenCalledWith("show_quick_add_window");
  expect(mocks.invokeCmdMock).not.toHaveBeenCalledWith("show_main_window");
});

it("opens quick note window without showing main window", async () => {
  expect(mocks.invokeCmdMock).toHaveBeenCalledWith("show_quick_note_window");
  expect(mocks.invokeCmdMock).not.toHaveBeenCalledWith("show_main_window");
});

it("opens command palette window without showing main window", async () => {
  expect(mocks.invokeCmdMock).toHaveBeenCalledWith("show_command_palette_window");
  expect(mocks.invokeCmdMock).not.toHaveBeenCalledWith("show_main_window");
});
```

**Step 2: Run test to verify it fails**

Run: `pnpm vitest run src/composables/useShortcutRuntime.spec.ts`
Expected: FAIL，提示仍在调用 `show_main_window` 或缺少新命令

**Step 3: Write minimal implementation**

先不要改完整窗口实现，只把快捷键动作切到新命令名，制造 Rust 侧未实现失败：

```ts
await invokeCmd("show_quick_add_window");
await invokeCmd("show_quick_note_window");
await invokeCmd("show_command_palette_window");
```

**Step 4: Run test to verify it passes/fails at the right boundary**

Run: `pnpm vitest run src/composables/useShortcutRuntime.spec.ts`
Expected: 前端测试通过，但 Rust 编译仍未接线

**Step 5: Commit**

```bash
git add src/composables/useShortcutRuntime.spec.ts src/composables/useShortcutRuntime.ts
git commit -m "test: cover popup shortcut window commands"
```

### Task 2: 在 Tauri 后端新增独立小窗命令

**Files:**
- Modify: `src-tauri/src/commands/window_commands.rs:1-29`
- Modify: `src-tauri/src/lib.rs:177-178`

**Step 1: Write the failing test**

如果仓库暂无 Rust 单测，就用编译失败作为红灯：先在 `lib.rs` 注册尚不存在的命令。

```rust
commands::window_commands::show_quick_add_window,
commands::window_commands::show_quick_note_window,
commands::window_commands::show_command_palette_window,
```

**Step 2: Run test to verify it fails**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: FAIL，提示找不到新增命令函数

**Step 3: Write minimal implementation**

在 `window_commands.rs` 提供一个共享 helper，例如：

```rust
fn show_or_create_popup(app: &tauri::AppHandle, label: &str, route: &str, title: &str, width: f64, height: f64) -> Result<(), String>
```

并实现：
- 若窗口已存在：`show + unminimize + set_focus`
- 若不存在：`WebviewWindowBuilder::new(..., WebviewUrl::App(route.into()))`
- 新增 3 个命令：
  - `show_quick_add_window`
  - `show_quick_note_window`
  - `show_command_palette_window`

窗口建议：
- quick-add: 520x640
- quick-note: 640x720
- command-palette: 720x560

**Step 4: Run test to verify it passes**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/src/commands/window_commands.rs src-tauri/src/lib.rs
git commit -m "feat: add popup window commands"
```

### Task 3: 为 popup route 写失败测试

**Files:**
- Modify: `src/router/index.ts`
- Test: `src/__tests__/setup.ts` or create `src/router/index.spec.ts`

**Step 1: Write the failing test**

为新路由写断言：

```ts
expect(routes).toContainEqual(expect.objectContaining({ path: "/popup/quick-add" }));
expect(routes).toContainEqual(expect.objectContaining({ path: "/popup/quick-note" }));
expect(routes).toContainEqual(expect.objectContaining({ path: "/popup/command-palette" }));
```

并校验 `hideLayout: true`。

**Step 2: Run test to verify it fails**

Run: `pnpm vitest run src/router/index.spec.ts`
Expected: FAIL，提示 route 不存在

**Step 3: Write minimal implementation**

在 `src/router/index.ts` 新增 3 个 popup route，全部 `meta.hideLayout = true`，分别指向专用 popup view。

**Step 4: Run test to verify it passes**

Run: `pnpm vitest run src/router/index.spec.ts`
Expected: PASS

**Step 5: Commit**

```bash
git add src/router/index.ts src/router/index.spec.ts
git commit -m "feat: add popup routes"
```

### Task 4: 抽出可复用的 quick-add / quick-note / command-palette 视图容器

**Files:**
- Create: `src/views/popup/QuickAddPopupView.vue`
- Create: `src/views/popup/QuickNotePopupView.vue`
- Create: `src/views/popup/CommandPalettePopupView.vue`
- Modify: `src/components/task/QuickAddModal.vue`
- Modify: `src/components/common/QuickNoteModal.vue`
- Modify: `src/components/common/CommandPalette.vue`
- Modify: `src/views/TodayView.vue:573-581`
- Modify: `src/App.vue:134-145`

**Step 1: Write the failing test**

为每个 popup view 写最小渲染测试，验证：
- 能独立渲染表单/命令列表
- 成功提交后关闭当前窗口，而不是修改 `ui.showQuickAdd/showQuickNote`

示例：

```ts
it("closes popup window after quick note submit", async () => {
  expect(closeWindowMock).toHaveBeenCalled();
});
```

**Step 2: Run test to verify it fails**

Run: `pnpm vitest run src/views/popup/*.spec.ts`
Expected: FAIL，popup view 不存在

**Step 3: Write minimal implementation**

实现原则：
- 复用现有业务组件，避免复制表单逻辑
- 若现有组件强依赖 `visible` prop / modal shell，则把内容层抽成共享子组件
- popup view 内直接渲染内容，不包 modal mask
- 提交成功后调用 Tauri window API 关闭当前窗口
- `TodayView.vue` 保留原主窗口入口，或逐步移除主窗口内 quick-add/quick-note modal 挂载
- `App.vue` 移除 command palette 的全局 modal 挂载，改由独立窗口使用

**Step 4: Run test to verify it passes**

Run: `pnpm vitest run src/views/popup/*.spec.ts`
Expected: PASS

**Step 5: Commit**

```bash
git add src/views/popup src/components/task/QuickAddModal.vue src/components/common/QuickNoteModal.vue src/components/common/CommandPalette.vue src/views/TodayView.vue src/App.vue
git commit -m "refactor: move lightweight popups into standalone windows"
```

### Task 5: 改托盘行为，弹窗入口不再唤起主窗口

**Files:**
- Modify: `src-tauri/src/system/tray.rs:51-77`
- Modify: `src/App.vue:52-77,103-118`

**Step 1: Write the failing test**

若暂无托盘自动化测试，先用编译红灯 + 人工可验证日志断言思路：
- `quick-add` / `quick-note` 不再调用 `show_main`
- 改为直接调用新增 popup command 或 emit 新事件给后端窗口层

**Step 2: Run test to verify it fails**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: FAIL 或行为未达预期

**Step 3: Write minimal implementation**

在 `tray.rs` 中：
- `open-main` / `settings` / `toggle-pause` / `switch-task` 保持主窗口语义
- `quick-add` / `quick-note` 直接打开 popup window
- `settle-today` 暂时保留主窗口语义（方案 C 第一阶段）

在 `App.vue` 中：
- 删除对 `quick-add` / `quick-note` 的主窗口内 tray 分发依赖
- 保留其他 tray action 处理

**Step 4: Run test to verify it passes**

Run: `cargo check --manifest-path src-tauri/Cargo.toml && pnpm vitest run src/composables/useShortcutRuntime.spec.ts`
Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/src/system/tray.rs src/App.vue
git commit -m "feat: open popup windows from tray without main window"
```

### Task 6: 完整验证

**Files:**
- Verify only

**Step 1: Run focused frontend tests**

Run: `pnpm vitest run src/composables/useShortcutRuntime.spec.ts src/router/index.spec.ts`
Expected: PASS

**Step 2: Run popup-related tests**

Run: `pnpm vitest run src/views/popup/*.spec.ts`
Expected: PASS

**Step 3: Run Rust compile check**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: PASS

**Step 4: Manual verification**

手动验证：
- 全局快捷键 quick-note 只打开 `quick-note` 窗口
- quick-add 只打开 `quick-add` 窗口
- command-palette 只打开独立命令面板窗口
- 主窗口隐藏时，popup 仍可正常打开
- popup 提交成功后关闭自身，不唤起主窗口
- 设置/导航类入口仍正常唤起主窗口

**Step 5: Commit**

```bash
git add -A
git commit -m "test: verify standalone popup window flow"
```
