use std::io;

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder, webview::PageLoadEvent};

pub struct DshUrl(pub tauri::Url);

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
    let mut window_config = app
        .config()
        .app
        .windows
        .iter()
        .find(|window| window.label == "main")
        .cloned()
        .ok_or_else(|| io::Error::other("main window configuration is missing"))?;

    window_config.url = WebviewUrl::External(app.state::<DshUrl>().0.clone());
    WebviewWindowBuilder::from_config(app, &window_config)?
        .visible(false)
        .on_page_load(|window, payload| {
            if payload.event() == PageLoadEvent::Finished {
                if let Err(error) = window.show() {
                    eprintln!("Failed to show main window: {error}");
                }
                if let Err(error) = window.set_focus() {
                    eprintln!("Failed to focus main window: {error}");
                }
            }
        })
        .build()?;

    Ok(())
}
