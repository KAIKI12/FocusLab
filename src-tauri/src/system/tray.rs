//! 系统托盘 · 对齐原型 `prototype/screens/floating-ball.html` 的 8 项右键菜单。
//!
//! 架构:
//! - 菜单项点击 → 若需主窗,先唤起主窗;再 emit `focuslab:tray:action`/`focuslab:tray:navigate`
//! - 前端 App.vue 监听事件,委托给 useTimerStore / useUIStore / useSettlementStore / router
//! - 左键单击托盘 → 唤起主窗
//! - 菜单文字不动态切换(暂停/继续统一文字),避免 Rust 层同步 timer 状态的复杂性

use serde_json::json;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder,
};

const TRAY_EVENT_ACTION: &str = "focuslab:tray:action";
const TRAY_EVENT_NAVIGATE: &str = "focuslab:tray:navigate";

/// 构建并注册托盘图标 + 菜单。由 lib.rs setup 调用一次。
pub fn build(app: &AppHandle) -> tauri::Result<()> {
    let open_main = MenuItem::with_id(app, "open-main", "打开主窗口", true, Some("Ctrl+Shift+F"))?;
    let toggle_pause = MenuItem::with_id(app, "toggle-pause", "暂停 / 继续", true, Some("Space"))?;
    let switch_task = MenuItem::with_id(app, "switch-task", "切换任务", true, None::<&str>)?;
    let sep1 = PredefinedMenuItem::separator(app)?;
    let quick_add = MenuItem::with_id(app, "quick-add", "快速添加任务", true, Some("Ctrl+N"))?;
    let quick_note = MenuItem::with_id(app, "quick-note", "速记便签", true, Some("Ctrl+Shift+N"))?;
    let settle_today = MenuItem::with_id(app, "settle-today", "结束今天", true, None::<&str>)?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    let settings = MenuItem::with_id(app, "settings", "设置", true, Some("Ctrl+,"))?;
    let quit = MenuItem::with_id(app, "quit", "退出 FocusLab", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[
            &open_main,
            &toggle_pause,
            &switch_task,
            &sep1,
            &quick_add,
            &quick_note,
            &settle_today,
            &sep2,
            &settings,
            &quit,
        ],
    )?;

    let mut builder = TrayIconBuilder::with_id("main")
        .tooltip("FocusLab")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open-main" => show_main(app),
            "toggle-pause" => {
                show_main(app);
                let _ = app.emit(TRAY_EVENT_ACTION, json!({ "type": "toggle-pause" }));
            }
            "switch-task" => {
                show_main(app);
                let _ = app.emit(TRAY_EVENT_ACTION, json!({ "type": "switch-task" }));
            }
            "quick-add" => {
                show_main(app);
                let _ = app.emit(TRAY_EVENT_ACTION, json!({ "type": "quick-add" }));
            }
            "quick-note" => {
                show_main(app);
                let _ = app.emit(TRAY_EVENT_ACTION, json!({ "type": "quick-note" }));
            }
            "settle-today" => {
                show_main(app);
                let _ = app.emit(TRAY_EVENT_ACTION, json!({ "type": "settle-today" }));
            }
            "settings" => {
                show_main(app);
                let _ = app.emit(TRAY_EVENT_NAVIGATE, "/settings");
            }
            "quit" => app.exit(0),
            other => tracing::debug!("tray menu item {other} not handled"),
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main(tray.app_handle());
            }
        });

    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }

    builder.build(app)?;
    tracing::info!("system tray registered");
    Ok(())
}

/// 唤起主窗口 — 与 `commands::window_commands::show_main_window` 同语义的同步版本。
fn show_main(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.show();
        let _ = win.unminimize();
        let _ = win.set_focus();
        return;
    }

    let _ = WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
        .title("FocusLab")
        .inner_size(1000.0, 700.0)
        .min_inner_size(800.0, 600.0)
        .build();
}
