use std::sync::Mutex;

use rand::Rng;

use tauri::{WebviewUrl, WebviewWindowBuilder};
use tauri::{AppHandle, Manager, Runtime};

use crate::commands;
use crate::db::AppData;
use crate::models::{DisplayPosition, Quote};
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

const POPUP_W: f64 = 500.0;
const POPUP_H: f64 = 280.0;

/// Calculate popup position based on settings + monitor size.
/// `Monitor::position()` / `Monitor::size()` return PHYSICAL values,
/// but `WebviewWindowBuilder::position()` expects LOGICAL coordinates.
/// We must divide by the DPI scale factor.
fn calc_position<R: Runtime>(
    app: &AppHandle<R>,
    pos: DisplayPosition,
    preferred_screen: Option<u32>,
) -> Option<(f64, f64)> {
    let main_win = app.get_webview_window("main")?;

    // Pick monitor by index, fallback to primary
    let monitors = main_win.available_monitors().ok().unwrap_or_default();
    let mon = preferred_screen
        .and_then(|idx| monitors.get(idx as usize).cloned())
        .or_else(|| main_win.primary_monitor().ok()?);

    let mon = mon?;
    let scale = mon.scale_factor();
    let mp = mon.position();
    let ms = mon.size();
    let (mx, my) = (mp.x as f64 / scale, mp.y as f64 / scale);
    let (mw, mh) = (ms.width as f64 / scale, ms.height as f64 / scale);

    match pos {
        DisplayPosition::Center => {
            Some((mx + (mw - POPUP_W) / 2.0, my + (mh - POPUP_H) / 2.0))
        }
        DisplayPosition::BottomLeft => Some((mx, my + mh - POPUP_H)),
        DisplayPosition::BottomRight => Some((mx + mw - POPUP_W, my + mh - POPUP_H)),
        DisplayPosition::TopRight => Some((mx + mw - POPUP_W, my)),
        DisplayPosition::Random => {
            let mut rng = rand::thread_rng();
            let x = mx + rng.gen_range(0.0..(mw - POPUP_W).max(0.0));
            let y = my + rng.gen_range(0.0..(mh - POPUP_H).max(0.0));
            Some((x, y))
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

    // Read display settings
    let (display_pos, preferred_screen) = {
        let s = data.settings.lock().unwrap();
        (s.display_position, s.preferred_screen)
    };

    // Create the popup webview window at the right position
    let label = format!(
        "popup-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    );

    let mut builder = WebviewWindowBuilder::new(
        app,
        &label,
        WebviewUrl::App("index.html".into()),
    )
    .title("")
    .inner_size(POPUP_W, POPUP_H)
    .decorations(false)
    .resizable(false)
    .always_on_top(true)
    .skip_taskbar(true)
    .transparent(true);

    if let Some((x, y)) = calc_position(app, display_pos, preferred_screen) {
        builder = builder.position(x, y);
    }

    if let Err(e) = builder.build() {
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
