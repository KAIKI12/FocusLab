//! 窗口管理命令 — 供悬浮球 / 快捷键唤起主窗口与独立工具窗口。

use tauri::{Manager, WebviewUrl, WebviewWindow};

async fn focus_main_window(app: &tauri::AppHandle) -> Result<(), String> {
    if let Some(win) = app.get_webview_window("main") {
        focus_window(&win)?;
        return Ok(());
    }

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

fn focus_window(window: &WebviewWindow) -> Result<(), String> {
    window.show().map_err(|e| e.to_string())?;
    window.unminimize().map_err(|e| e.to_string())?;
    window.set_focus().map_err(|e| e.to_string())?;
    Ok(())
}

fn ensure_tool_window(
    app: &tauri::AppHandle,
    label: &str,
    url: &str,
    title: &str,
    width: f64,
    height: f64,
    min_width: f64,
    min_height: f64,
) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(label) {
        return focus_window(&window);
    }

    let window = tauri::WebviewWindowBuilder::new(app, label, WebviewUrl::App(url.into()))
        .title(title)
        .inner_size(width, height)
        .min_inner_size(min_width, min_height)
        .resizable(true)
        .build()
        .map_err(|e| e.to_string())?;

    focus_window(&window)
}

/// 显示主窗口。如果窗口已关闭(被销毁),则重新创建。
#[tauri::command]
pub async fn show_main_window(app: tauri::AppHandle) -> Result<(), String> {
    focus_main_window(&app).await
}

#[tauri::command]
pub async fn show_quick_add_window(app: tauri::AppHandle) -> Result<(), String> {
    ensure_tool_window(
        &app,
        "quick-add",
        "quick-add.html",
        "快速添加任务",
        640.0,
        560.0,
        520.0,
        420.0,
    )
}

#[tauri::command]
pub async fn show_quick_note_window(app: tauri::AppHandle) -> Result<(), String> {
    ensure_tool_window(
        &app,
        "quick-note",
        "quick-note.html",
        "速记便签",
        760.0,
        760.0,
        680.0,
        560.0,
    )
}

#[tauri::command]
pub async fn show_command_palette_window(app: tauri::AppHandle) -> Result<(), String> {
    ensure_tool_window(
        &app,
        "command-palette",
        "command-palette.html",
        "命令面板",
        720.0,
        540.0,
        560.0,
        420.0,
    )
}
