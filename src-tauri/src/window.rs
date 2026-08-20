use std::io;

use tauri::{AppHandle, Manager, WebviewWindowBuilder};

pub fn show_or_create_main(app: &AppHandle) -> tauri::Result<()> {
    //
    // 如果窗口仍然存在，直接显示并聚焦。
    //
    if let Some(window) = app.get_webview_window("main") {
        window.show()?;
        window.set_focus()?;

        return Ok(());
    }

    //
    // 窗口已经被 destroy：
    // 从 tauri.conf.json 中找到 main 窗口配置。
    //
    let window_config = app
        .config()
        .app
        .windows
        .iter()
        .find(|window| window.label == "main")
        .ok_or_else(|| io::Error::other("main window configuration is missing"))?;

    //
    // 根据配置重新创建完整的 WebViewWindow。
    //
    let window = WebviewWindowBuilder::from_config(app, window_config)?.build()?;
    window.show()?;
    window.set_focus()?;

    Ok(())
}
