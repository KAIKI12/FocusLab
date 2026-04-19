//! 窗口管理命令 — show_main_window 供悬浮球唤起主窗口。

use tauri::Manager;

/// 显示主窗口。如果窗口已关闭(被销毁),则重新创建。
#[tauri::command]
pub async fn show_main_window(app: tauri::AppHandle) -> Result<(), String> {
    // 尝试找到现有主窗口
    if let Some(win) = app.get_webview_window("main") {
        win.show().map_err(|e| e.to_string())?;
        win.unminimize().map_err(|e| e.to_string())?;
        win.set_focus().map_err(|e| e.to_string())?;
        return Ok(());
    }

    // 主窗口已被关闭,重新创建
    use tauri::WebviewUrl;
    tauri::WebviewWindowBuilder::new(
        &app,
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
