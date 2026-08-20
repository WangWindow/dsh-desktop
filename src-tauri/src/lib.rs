mod dsh;
mod tray;
mod window;

use std::{
    process::Child,
    sync::{Arc, Mutex},
};

use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Some(child):
    //   DSH 是由当前应用启动的，我们负责它的生命周期。
    //
    // None:
    //   当前没有我们管理的 DSH 进程。
    let dsh_process = Arc::new(Mutex::new(None::<Child>));
    let dsh_process_for_setup = Arc::clone(&dsh_process);

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        //
        // 单实例。
        //
        // 用户再次启动程序时，不创建第二个实例，
        // 而是显示已有窗口；如果窗口已经被销毁，则重新创建。
        //
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Err(error) = window::show_or_create_main(app) {
                eprintln!("Failed to show main window: {error}");
            }
        }))
        //
        // 主窗口点击 × 时，不退出整个程序。
        //
        // 我们不再使用 hide()，而是直接 destroy()。
        // 这样下次显示时创建一个全新的 GTK/WebView 窗口，
        // 避免 GNOME Wayland 下 hide -> show 后标题栏失效的问题。
        //
        .on_window_event(|window, event| {
            if window.label() != "main" {
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
            //
            // 1. 检查 DSH 是否已安装。
            //
            // 没有 DSH 时不创建主窗口和托盘，避免用户只看到一个
            // 无法加载的 WebView。
            //
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
                        if open_website {
                            if let Err(error) = tauri_plugin_opener::open_url(
                                "https://deepseek.com/harness/",
                                None::<&str>,
                            ) {
                                eprintln!("Failed to open DeepSeek Harness website: {error}");
                            }
                        }

                        app_handle.exit(0);
                    });

                return Ok(());
            }

            //
            // 2. 启动 DSH，或者复用已经运行的 DSH Web。
            //

            if dsh::port_is_open() {
                println!("DSH Web port is already in use");
            } else {
                let child = dsh::start()?;

                *dsh_process_for_setup.lock().unwrap() = Some(child);
            }

            //
            // 3. 创建系统托盘
            //

            tray::create_tray(app)?;

            //
            // 4. 创建主窗口
            //

            window::show_or_create_main(app.handle())?;

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building Tauri application");

    //
    // 即使最后一个窗口被 destroy，也不能退出应用。
    //
    // Tray -> Quit 使用 app.exit(0)，此时 code == Some(0)，
    // 因此允许真正退出。
    //
    let exit_code = app.run_return(|_, event| {
        if let tauri::RunEvent::ExitRequested { api, code, .. } = event {
            if code.is_none() {
                api.prevent_exit();
            }
        }
    });

    //
    // 应用真正退出之后，清理我们启动的 DSH。
    //

    if let Some(mut child) = dsh_process.lock().unwrap().take() {
        dsh::stop(&mut child);
    }

    std::process::exit(exit_code);
}
