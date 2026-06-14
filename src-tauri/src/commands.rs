use rand::Rng;
use tauri::{Manager, State};

use crate::db::AppData;
use crate::models::*;

/// Internal: pick and return a random quote, incrementing stats.
/// Does NOT update tray tooltip - the caller should do that.
pub fn pick_quote_internal(data: &AppData) -> Option<Quote> {
    let mut quotes = data.quotes.lock().ok()?;
    let settings = data.settings.lock().ok()?;
    let mut stats = data.stats.lock().ok()?;

    let active: Vec<usize> = quotes
        .quotes
        .iter()
        .enumerate()
        .filter(|(_, q)| {
            q.enabled
                && (settings.active_categories.is_empty()
                    || settings.active_categories.contains(&q.category))
        })
        .map(|(i, _)| i)
        .collect();

    if active.is_empty() {
        return None;
    }

    let pick_idx = if settings.weight_mode {
        let total_weight: f64 = active
            .iter()
            .map(|&i| 1.0 / (quotes.quotes[i].show_count as f64 + 1.0))
            .sum();
        let r = rand::thread_rng().gen::<f64>() * total_weight;
        let mut cumulative = 0.0;
        let mut selected = active[0];
        for &i in &active {
            cumulative += 1.0 / (quotes.quotes[i].show_count as f64 + 1.0);
            if r <= cumulative {
                selected = i;
                break;
            }
        }
        selected
    } else {
        active[rand::thread_rng().gen_range(0..active.len())]
    };

    quotes.quotes[pick_idx].show_count += 1;
    stats.total_displayed += 1;
    stats.today_displayed += 1;

    let quote = quotes.quotes[pick_idx].clone();
    drop(quotes);
    drop(settings);
    drop(stats);
    data.persist_all();

    Some(quote)
}

// ── Tauri commands ──────────────────────────────────────────

#[tauri::command]
pub fn get_quote(data: State<AppData>) -> Result<Option<Quote>, String> {
    Ok(pick_quote_internal(data.inner()))
}

#[tauri::command]
pub fn get_all_quotes(data: State<AppData>) -> Result<QuotesData, String> {
    let quotes = data.quotes.lock().map_err(|e| e.to_string())?;
    Ok(quotes.clone())
}

#[tauri::command]
pub fn save_all_quotes(
    data: State<AppData>,
    quotes_data: QuotesData,
) -> Result<(), String> {
    let mut quotes = data.quotes.lock().map_err(|e| e.to_string())?;
    *quotes = quotes_data;
    drop(quotes);
    data.inner().persist_all();
    Ok(())
}

#[tauri::command]
pub fn add_quote(
    data: State<AppData>,
    content: String,
    author: Option<String>,
    category: String,
) -> Result<Quote, String> {
    let mut quotes = data.quotes.lock().map_err(|e| e.to_string())?;
    if !quotes.categories.iter().any(|c| c.name == category) {
        quotes.categories.push(Category { name: category.clone() });
    }
    let new_id = quotes.quotes.iter().map(|q| q.id).max().unwrap_or(0) + 1;
    let quote = Quote {
        id: new_id,
        content,
        author,
        category,
        enabled: true,
        show_count: 0,
        created_at: chrono::Local::now()
            .format("%Y-%m-%d %H:%M:%S")
            .to_string(),
    };
    quotes.quotes.push(quote.clone());
    drop(quotes);
    data.inner().persist_all();
    Ok(quote)
}

#[tauri::command]
pub fn update_quote(data: State<AppData>, quote: Quote) -> Result<(), String> {
    let mut quotes = data.quotes.lock().map_err(|e| e.to_string())?;
    if let Some(existing) = quotes.quotes.iter_mut().find(|q| q.id == quote.id) {
        *existing = quote;
    }
    drop(quotes);
    data.inner().persist_all();
    Ok(())
}

#[tauri::command]
pub fn delete_quote(data: State<AppData>, id: u64) -> Result<(), String> {
    let mut quotes = data.quotes.lock().map_err(|e| e.to_string())?;
    quotes.quotes.retain(|q| q.id != id);
    drop(quotes);
    data.inner().persist_all();
    Ok(())
}

#[tauri::command]
pub fn add_category(data: State<AppData>, name: String) -> Result<(), String> {
    let mut quotes = data.quotes.lock().map_err(|e| e.to_string())?;
    if !quotes.categories.iter().any(|c| c.name == name) {
        quotes.categories.push(Category { name });
    }
    drop(quotes);
    data.inner().persist_all();
    Ok(())
}

#[tauri::command]
pub fn rename_category(
    data: State<AppData>,
    old_name: String,
    new_name: String,
) -> Result<(), String> {
    let mut quotes = data.quotes.lock().map_err(|e| e.to_string())?;
    if let Some(cat) = quotes.categories.iter_mut().find(|c| c.name == old_name) {
        cat.name = new_name.clone();
    }
    for q in quotes.quotes.iter_mut() {
        if q.category == old_name {
            q.category = new_name.clone();
        }
    }
    drop(quotes);
    let mut settings = data.settings.lock().map_err(|e| e.to_string())?;
    settings.active_categories = settings
        .active_categories
        .iter()
        .map(|c| if c == &old_name { &new_name } else { c })
        .cloned()
        .collect();
    drop(settings);
    data.inner().persist_all();
    Ok(())
}

#[tauri::command]
pub fn delete_category(data: State<AppData>, name: String) -> Result<(), String> {
    let mut quotes = data.quotes.lock().map_err(|e| e.to_string())?;
    quotes.categories.retain(|c| c.name != name);
    for q in quotes.quotes.iter_mut() {
        if q.category == name {
            q.category = "未整理".to_string();
        }
    }
    drop(quotes);
    let mut settings = data.settings.lock().map_err(|e| e.to_string())?;
    settings.active_categories.retain(|c| c != &name);
    drop(settings);
    data.inner().persist_all();
    Ok(())
}

#[tauri::command]
pub fn get_settings(data: State<AppData>) -> Result<AppSettings, String> {
    let settings = data.settings.lock().map_err(|e| e.to_string())?;
    Ok(settings.clone())
}

#[tauri::command]
pub fn save_settings(
    data: State<AppData>,
    settings: AppSettings,
) -> Result<(), String> {
    let mut current = data.settings.lock().map_err(|e| e.to_string())?;
    *current = settings;
    drop(current);
    data.inner().persist_all();
    Ok(())
}

/// Keyboard hook reads settings live — this is kept as a no-op for frontend debounce compatibility.
#[tauri::command]
pub fn reload_shortcuts() -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub fn get_stats(data: State<AppData>) -> Result<Stats, String> {
    let stats = data.stats.lock().map_err(|e| e.to_string())?;
    Ok(stats.clone())
}

#[tauri::command]
pub fn collect_quote(data: State<AppData>, content: String) -> Result<(), String> {
    let mut quotes = data.quotes.lock().map_err(|e| e.to_string())?;
    if !quotes.categories.iter().any(|c| c.name == "未整理") {
        quotes.categories.push(Category {
            name: "未整理".to_string(),
        });
    }
    let new_id = quotes.quotes.iter().map(|q| q.id).max().unwrap_or(0) + 1;
    let quote = Quote {
        id: new_id,
        content,
        author: None,
        category: "未整理".to_string(),
        enabled: true,
        show_count: 0,
        created_at: chrono::Local::now()
            .format("%Y-%m-%d %H:%M:%S")
            .to_string(),
    };
    quotes.quotes.push(quote);
    drop(quotes);
    data.inner().persist_all();
    Ok(())
}

#[tauri::command]
pub fn reset_quote_count(data: State<AppData>, id: u64) -> Result<(), String> {
    let mut quotes = data.quotes.lock().map_err(|e| e.to_string())?;
    if let Some(q) = quotes.quotes.iter_mut().find(|q| q.id == id) {
        q.show_count = 0;
    }
    drop(quotes);
    data.inner().persist_all();
    Ok(())
}

#[tauri::command]
pub fn trigger_quote_manually(data: State<AppData>) -> Result<Option<Quote>, String> {
    Ok(pick_quote_internal(data.inner()))
}

#[tauri::command]
pub fn popup_close(window: tauri::WebviewWindow) -> Result<(), String> {
    window.close().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_monitor_count(app: tauri::AppHandle) -> Result<usize, String> {
    let win = app
        .get_webview_window("main")
        .ok_or_else(|| "Main window not found".to_string())?;
    let monitors = win.available_monitors().map_err(|e| e.to_string())?;
    Ok(monitors.len())
}


