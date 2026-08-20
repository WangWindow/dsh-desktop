use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
};

pub fn create_tray(app: &tauri::App) -> tauri::Result<()> {
    //
    // 系统托盘图标。
    //
    let tray_icon = tauri::include_image!("./icons/icon.png");

    //
    // Show
    //
    let show_item = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;

    //
    // Quit
    //
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    //
    // Linux 下需要给 tray 设置 menu。
    // 没有 menu 时某些 AppIndicator 环境不会显示托盘。
    //
    let tray_menu = Menu::with_items(app, &[&show_item, &quit_item])?;

    TrayIconBuilder::new()
        .icon(tray_icon)
        .menu(&tray_menu)
        .on_menu_event(|app, event| {
            match event.id().as_ref() {
                //
                // 显示窗口。
                //
                // 如果之前点击 × 已经 destroy，
                // 此处会重新创建一个全新的窗口。
                //
                "show" => {
                    if let Err(error) = crate::window::show_or_create_main(app) {
                        eprintln!("Failed to show main window: {error}");
                    }
                }

                //
                // 真正退出整个应用。
                //
                // 这会让 run_return() 返回，
                // 随后 lib.rs 会清理 DSH。
                //
                "quit" => {
                    app.exit(0);
                }

                _ => {}
            }
        })
        .build(app)?;

    Ok(())
}
