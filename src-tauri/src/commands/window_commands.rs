//! 窗口管理命令 — 供悬浮球 / 快捷键唤起主窗口与页面内动作。

use serde_json::json;
use tauri::{Emitter, Manager};

const WINDOW_ACTION_EVENT: &str = "focuslab://window-action";

async fn focus_main_window(app: &tauri::AppHandle) -> Result<(), String> {
    if let Some(win) = app.get_webview_window("main") {
        win.show().map_err(|e| e.to_string())?;
        win.unminimize().map_err(|e| e.to_string())?;
        win.set_focus().map_err(|e| e.to_string())?;
        return Ok(());
    }

    use tauri::WebviewUrl;
    tauri::WebviewWindowBuilder::new(
        app,
        "main",
        WebviewUrl::App("index.html".into()),
    )
    .title("FocusLab")
    .inner_size(1000.0, 700.0)
    .min_inner_size(800.0, 600.0)
    .build()
    .map_err(|e| e.to_string())?;

    Ok(())
}

/// 显示主窗口。如果窗口已关闭(被销毁),则重新创建。
#[tauri::command]
pub async fn show_main_window(app: tauri::AppHandle) -> Result<(), String> {
    focus_main_window(&app).await
}

#[tauri::command]
pub async fn show_quick_add_window(app: tauri::AppHandle) -> Result<(), String> {
    focus_main_window(&app).await?;
    let _ = app.emit(WINDOW_ACTION_EVENT, json!({ "type": "quick-add" }));
    Ok(())
}

#[tauri::command]
pub async fn show_quick_note_window(app: tauri::AppHandle) -> Result<(), String> {
    focus_main_window(&app).await?;
    let _ = app.emit(WINDOW_ACTION_EVENT, json!({ "type": "quick-note" }));
    Ok(())
}

#[tauri::command]
pub async fn show_command_palette_window(app: tauri::AppHandle) -> Result<(), String> {
    focus_main_window(&app).await?;
    let _ = app.emit(WINDOW_ACTION_EVENT, json!({ "type": "command-palette" }));
    Ok(())
}
