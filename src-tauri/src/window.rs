use std::io;

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

pub struct DshUrl(pub tauri::Url);

pub fn show_or_create_main(app: &AppHandle) -> tauri::Result<()> {
    // 如果窗口仍然存在，直接显示并聚焦。
    if let Some(window) = app.get_webview_window("main") {
        window.show()?;
        window.set_focus()?;
        return Ok(());
    }

    // 如果窗口不存在，则创建一个新的窗口。
    let webview_url = WebviewUrl::External(app.state::<DshUrl>().0.clone());
    let window_config = app
        .config()
        .app
        .windows
        .iter()
        .find(|window| window.label == "main")
        .cloned()
        .map(|mut config| {
            config.url = webview_url;
            config
        })
        .ok_or_else(|| io::Error::other("main window configuration is missing"))?;
    let window = WebviewWindowBuilder::from_config(app, &window_config)?.build()?;

    window.show()?;
    window.set_focus()?;
    Ok(())
}
