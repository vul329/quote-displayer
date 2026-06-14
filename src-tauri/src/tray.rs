use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{TrayIcon, TrayIconBuilder},
    AppHandle, Manager, Runtime,
};

use crate::db::AppData;
use crate::popup;

/// Holds the tray icon + pause item so we can toggle its label.
pub struct TrayHandle<R: Runtime> {
    icon: TrayIcon<R>,
    pause_item: MenuItem<R>,
}

// TrayIcon<R> is Send+Sync as long as R: Runtime
unsafe impl<R: Runtime> Send for TrayHandle<R> {}
unsafe impl<R: Runtime> Sync for TrayHandle<R> {}

pub fn setup_tray<R: Runtime>(app: &AppHandle<R>) {
    let icon_content = include_bytes!("../icons/tray-icon.png");
    let icon = Image::from_bytes(icon_content).unwrap_or_else(|_| {
        let fallback: &[u8] = &[0u8; 32 * 32 * 4];
        Image::new(fallback, 32, 32)
    });

    let show_item = tauri::menu::MenuItemBuilder::with_id("show", "立即顯示一則")
        .build(app)
        .unwrap();
    let pause_item = tauri::menu::MenuItemBuilder::with_id("pause", "暫停")
        .build(app)
        .unwrap();
    let edit_item = tauri::menu::MenuItemBuilder::with_id("edit", "編輯佳句")
        .build(app)
        .unwrap();
    let select_item = tauri::menu::MenuItemBuilder::with_id("select", "選擇主題")
        .build(app)
        .unwrap();
    let schedule_item =
        tauri::menu::MenuItemBuilder::with_id("schedule", "排程設定")
            .build(app)
            .unwrap();
    let autostart_item =
        tauri::menu::MenuItemBuilder::with_id("autostart", "隨系統啟動")
            .build(app)
            .unwrap();
    let ontop_item = tauri::menu::MenuItemBuilder::with_id("ontop", "佳句置頂")
        .build(app)
        .unwrap();
    let weight_item = tauri::menu::MenuItemBuilder::with_id("weight", "權重輪播")
        .build(app)
        .unwrap();
    let darkmode_item =
        tauri::menu::MenuItemBuilder::with_id("darkmode", "深色模式")
            .build(app)
            .unwrap();
    let close_item = tauri::menu::MenuItemBuilder::with_id("close", "關閉")
        .build(app)
        .unwrap();
    let sep = tauri::menu::PredefinedMenuItem::separator(app).unwrap();

    let menu = Menu::with_items(
        app,
        &[
            &show_item,
            &sep,
            &pause_item,
            &sep,
            &edit_item,
            &select_item,
            &schedule_item,
            &autostart_item,
            &sep,
            &ontop_item,
            &weight_item,
            &darkmode_item,
            &sep,
            &close_item,
        ],
    )
    .unwrap();

    let tray = TrayIconBuilder::new()
        .icon(icon)
        .tooltip("佳句隨機顯示器\n載入中...")
        .menu(&menu)
        .on_menu_event(move |app, event| {
            let data = app.state::<AppData>();
            match event.id().as_ref() {
                "show" => popup::show_quote_popup(app),
                "pause" => {
                    let mut settings = data.settings.lock().unwrap();
                    settings.paused = !settings.paused;
                    let paused = settings.paused;
                    drop(settings);
                    data.persist_all();
                    if let Some(handle) = app.try_state::<TrayHandle<R>>() {
                        let _ = handle.pause_item.set_text(if paused { "繼續" } else { "暫停" });
                    }
                    update_tray_tooltip_impl(app);
                }
                "edit" | "select" | "schedule" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "autostart" => {
                    let mut settings = data.settings.lock().unwrap();
                    settings.auto_start = !settings.auto_start;
                    let val = settings.auto_start;
                    drop(settings);
                    data.persist_all();
                    if val {
                        enable_autostart();
                    } else {
                        disable_autostart();
                    }
                }
                "ontop" => {
                    let mut settings = data.settings.lock().unwrap();
                    settings.always_on_top = !settings.always_on_top;
                    drop(settings);
                    data.persist_all();
                }
                "weight" => {
                    let mut settings = data.settings.lock().unwrap();
                    settings.weight_mode = !settings.weight_mode;
                    drop(settings);
                    data.persist_all();
                }
                "darkmode" => {
                    let mut settings = data.settings.lock().unwrap();
                    settings.dark_mode = !settings.dark_mode;
                    drop(settings);
                    data.persist_all();
                }
                "close" => {
                    data.persist_all();
                    app.exit(0);
                }
                _ => {}
            }
        })
        .build(app)
        .unwrap();

    // Set initial pause label based on saved state
    {
        let data = app.state::<AppData>();
        let paused = data.settings.lock().unwrap().paused;
        let _ = pause_item.set_text(if paused { "繼續" } else { "暫停" });
    }

    app.manage(TrayHandle { icon: tray, pause_item });

    update_tray_tooltip_impl(app);
}

fn update_tray_tooltip_impl<R: Runtime>(app: &AppHandle<R>) {
    let data = app.state::<AppData>();
    let paused = data.settings.lock().unwrap().paused;
    let today = data.stats.lock().unwrap().today_displayed;
    let quote_count = data.quotes.lock().unwrap().quotes.len();

    let status = if paused { "已暫停" } else { "啟用中" };
    let tooltip = format!(
        "佳句隨機顯示器\n共 {} 則佳句 | {}\n今日已顯示 {} 次",
        quote_count, status, today
    );

    if let Some(handle) = app.try_state::<TrayHandle<R>>() {
        let _ = handle.icon.set_tooltip(Some(tooltip));
    }
}

fn enable_autostart() {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        let exe = std::env::current_exe().unwrap_or_default();
        let _ = std::process::Command::new("reg")
            .args([
                "add",
                "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
                "/v",
                "QuoteDisplayer",
                "/t",
                "REG_SZ",
                "/d",
                &exe.to_string_lossy(),
                "/f",
            ])
            .creation_flags(0x08000000)
            .output();
    }
}

fn disable_autostart() {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        let _ = std::process::Command::new("reg")
            .args([
                "delete",
                "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
                "/v",
                "QuoteDisplayer",
                "/f",
            ])
            .creation_flags(0x08000000)
            .output();
    }
}

pub fn trigger_tooltip_update<R: Runtime>(app: &AppHandle<R>) {
    update_tray_tooltip_impl(app);
}
