mod commands;
mod db;
mod keyboard;
mod models;
mod popup;
mod scheduler;
mod tray;

use db::AppData;
use popup::ActiveQuote;
use scheduler::{LastTriggerTime, QuoteScheduler};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_shell::init())
        .manage(AppData::new())
        .manage(LastTriggerTime::new())
        .manage(ActiveQuote::new())
        .setup(|app| {
            // Initialise keyboard hook (WH_KEYBOARD_LL on Windows)
            keyboard::init(app.handle());

            // Set up system tray
            tray::setup_tray(app.handle());

            // Start the scheduler
            let scheduler = QuoteScheduler::new(app.handle().clone());
            scheduler.start();

            // Intercept main window close → hide instead of destroy
            if let Some(window) = app.get_webview_window("main") {
                let w = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = w.hide();
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_quote,
            commands::get_all_quotes,
            commands::save_all_quotes,
            commands::add_quote,
            commands::update_quote,
            commands::delete_quote,
            commands::add_category,
            commands::rename_category,
            commands::delete_category,
            commands::get_settings,
            commands::save_settings,
            commands::get_stats,
            commands::collect_quote,
            commands::reset_quote_count,
            commands::trigger_quote_manually,
            commands::popup_close,
            commands::reload_shortcuts,
            popup::get_popup_quote,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app_handle, event| {
            if let tauri::RunEvent::Exit = event {
                keyboard::shutdown();
            }
        });
}
