use std::io;

use tauri::{
    AppHandle, Manager, WebviewUrl, WebviewWindowBuilder,
    webview::NewWindowResponse,
};

pub struct DshUrl(pub tauri::Url);

/// Web 内容要求打开新窗口时，把 http(s) 链接交给系统默认浏览器。
///
/// DSH Web UI 用 `window.open(url, "_blank")` 打开外部链接；WebKitGTK 自身没有
/// 浏览器窗口，未注册本处理器时该请求被静默丢弃，表现为点击链接没有反应。
/// 返回 `Deny` 阻止 WebView 创建内嵌窗口，避免应用内出现无法导航的空窗口。
fn open_in_system_browser(url: tauri::Url) -> NewWindowResponse<tauri::Wry> {
    if matches!(url.scheme(), "http" | "https")
        && let Err(error) = tauri_plugin_opener::open_url(url.as_str(), None::<&str>)
    {
        eprintln!("Failed to open {url} in the system browser: {error}");
    }

    NewWindowResponse::Deny
}

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
    let window = WebviewWindowBuilder::from_config(app, &window_config)?
        .on_new_window(|url, _features| open_in_system_browser(url))
        .build()?;

    window.show()?;
    window.set_focus()?;
    Ok(())
}
