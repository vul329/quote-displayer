use std::path::PathBuf;
use std::sync::Mutex;

use crate::models::{AppSettings, QuotesData, Stats};

fn data_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}

fn quotes_path() -> PathBuf {
    data_dir().join("quotes.json")
}

fn settings_path() -> PathBuf {
    data_dir().join("settings.json")
}

fn stats_path() -> PathBuf {
    data_dir().join("stats.json")
}

pub fn load_quotes() -> QuotesData {
    let path = quotes_path();
    if path.exists() {
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    } else {
        QuotesData::default()
    }
}

pub fn save_quotes(data: &QuotesData) {
    if let Ok(json) = serde_json::to_string_pretty(data) {
        let _ = std::fs::write(quotes_path(), json);
    }
}

pub fn load_settings() -> AppSettings {
    let path = settings_path();
    if path.exists() {
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    } else {
        AppSettings::default()
    }
}

pub fn save_settings(settings: &AppSettings) {
    if let Ok(json) = serde_json::to_string_pretty(settings) {
        let _ = std::fs::write(settings_path(), json);
    }
}

pub fn load_stats() -> Stats {
    let path = stats_path();
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    if path.exists() {
        let stats: Option<Stats> = std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok());
        match stats {
            Some(s) if s.today_date == today => s,
            _ => Stats {
                total_displayed: 0,
                today_displayed: 0,
                today_date: today,
            },
        }
    } else {
        Stats {
            total_displayed: 0,
            today_displayed: 0,
            today_date: today,
        }
    }
}

pub fn save_stats(stats: &Stats) {
    if let Ok(json) = serde_json::to_string_pretty(stats) {
        let _ = std::fs::write(stats_path(), json);
    }
}

pub struct AppData {
    pub quotes: Mutex<QuotesData>,
    pub settings: Mutex<AppSettings>,
    pub stats: Mutex<Stats>,
}

impl AppData {
    pub fn new() -> Self {
        Self {
            quotes: Mutex::new(load_quotes()),
            settings: Mutex::new(load_settings()),
            stats: Mutex::new(load_stats()),
        }
    }

    pub fn persist_all(&self) {
        if let Ok(q) = self.quotes.lock() {
            save_quotes(&q);
        }
        if let Ok(s) = self.settings.lock() {
            save_settings(&s);
        }
        if let Ok(st) = self.stats.lock() {
            save_stats(&st);
        }
    }
}
