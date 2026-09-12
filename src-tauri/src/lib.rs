mod drop;
mod dsh;
mod tray;
mod window;

use std::{
    process::Child,
    sync::{Arc, Mutex},
};

use tauri::Manager;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // GtkFileChooserNative 依赖此变量决定是否走 xdg-desktop-portal 文件选择器
    // （portal 不可用时 GTK 自动回退本地对话框）；必须早于 GTK 初始化设置，故 unsafe 安全。
    #[cfg(target_os = "linux")]
    unsafe {
        std::env::set_var("GTK_USE_PORTAL", "1");
    }

    let dsh_process = Arc::new(Mutex::new(None::<Child>));
    let dsh_process_for_setup = Arc::clone(&dsh_process);

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Err(error) = window::show_or_create_main(app) {
                eprintln!("Failed to show main window: {error}");
            }
        }))
        .on_window_event(|window, event| {
            if window.label() != "main" {
                return;
            }

            if let tauri::WindowEvent::DragDrop(tauri::DragDropEvent::Drop { paths, .. }) = event {
                if let Some(webview) = window.app_handle().get_webview_window("main") {
                    drop::forward_dropped_files(&webview, paths);
                }
                return;
            }

            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();

                if let Err(error) = window.destroy() {
                    eprintln!("Failed to destroy main window: {error}");
                }
            }
        })
        .setup(move |app| {
            if !dsh::is_available() {
                let app_handle = app.handle().clone();

                app.dialog()
                    .message(
                        "DeepSeek Harness (dsh) was not found.\n\n\
                         Please install the DSH command-line tool first, \
                         then restart DSH Desktop.",
                    )
                    .title("DSH Not Found")
                    .kind(MessageDialogKind::Warning)
                    .buttons(MessageDialogButtons::OkCancelCustom(
                        "Open Website".into(),
                        "Quit".into(),
                    ))
                    .show(move |open_website| {
                        if open_website
                            && let Err(error) = tauri_plugin_opener::open_url(
                                "https://deepseek.com/harness/",
                                None::<&str>,
                            )
                        {
                            eprintln!("Failed to open DeepSeek Harness website: {error}");
                        }

                        app_handle.exit(0);
                    });

                return Ok(());
            }

            let server = dsh::start()?;
            app.manage(window::DshUrl(server.url));
            *dsh_process_for_setup.lock().unwrap() = Some(server.child);

            tray::create_tray(app)?;
            window::show_or_create_main(app.handle())?;

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building Tauri application");

    // Keep the tray process alive after the main window is destroyed.
    let exit_code = app.run_return(|_, event| {
        if let tauri::RunEvent::ExitRequested { api, code, .. } = event
            && code.is_none()
        {
            api.prevent_exit();
        }
    });

    if let Some(mut child) = dsh_process.lock().unwrap().take() {
        dsh::stop(&mut child);
    }

    std::process::exit(exit_code);
}
