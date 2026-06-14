use std::sync::Mutex;

use tauri::{WebviewUrl, WebviewWindowBuilder};
use tauri::{AppHandle, Manager, Runtime};

use crate::commands;
use crate::db::AppData;
use crate::models::Quote;
use crate::tray;

/// Stores the most recently triggered quote for the popup to display.
pub struct ActiveQuote(pub Mutex<Option<Quote>>);

impl ActiveQuote {
    pub fn new() -> Self {
        Self(Mutex::new(None))
    }

    pub fn set(&self, quote: Quote) {
        if let Ok(mut q) = self.0.lock() {
            *q = Some(quote);
        }
    }

    pub fn take(&self) -> Option<Quote> {
        if let Ok(mut q) = self.0.lock() {
            q.take()
        } else {
            None
        }
    }
}

/// Create a popup window and show a random quote.
/// Called from the scheduler or global hotkey/tray.
pub fn show_quote_popup<R: Runtime>(app: &AppHandle<R>) {
    let data = app.state::<AppData>();

    // Check if paused
    if let Ok(settings) = data.settings.lock() {
        if settings.paused {
            return;
        }
    }

    // Pick a quote (this also increments counters)
    let quote = match commands::pick_quote_internal(data.inner()) {
        Some(q) => q,
        None => return,
    };

    // Store for the popup window to retrieve
    if let Some(active) = app.try_state::<ActiveQuote>() {
        active.set(quote);
    }

    // Update tray tooltip
    tray::trigger_tooltip_update(app);

    // Create the popup webview window
    let label = format!(
        "popup-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    );

    let win = WebviewWindowBuilder::new(app, &label, WebviewUrl::App("index.html".into()))
        .title("")
        .inner_size(500.0, 280.0)
        .decorations(false)
        .resizable(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .transparent(true)
        .center()
        .build();

    if let Err(e) = win {
        eprintln!("Failed to create popup window: {}", e);
    }
}

/// Tauri command for the popup to retrieve the quote.
/// The popup window's React app calls this on load.
#[tauri::command]
pub fn get_popup_quote(
    app: tauri::AppHandle,
    data: tauri::State<AppData>,
) -> Result<Option<Quote>, String> {
    // First try to get the quote from ActiveQuote (set by show_quote_popup)
    if let Some(active) = app.try_state::<ActiveQuote>() {
        if let Some(quote) = active.take() {
            return Ok(Some(quote));
        }
    }
    // Fallback: pick a fresh quote directly
    Ok(commands::pick_quote_internal(data.inner()))
}
