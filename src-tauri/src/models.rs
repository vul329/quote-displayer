use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quote {
    pub id: u64,
    pub content: String,
    pub author: Option<String>,
    pub category: String,
    pub enabled: bool,
    pub show_count: u64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ScheduleRule {
    Interval { minutes: u64 },
    RandomRange { min: u64, max: u64 },
    FixedTime { times: Vec<String> },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum DisplayPosition {
    BottomRight,
    BottomLeft,
    TopRight,
    Center,
    Random,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AnimationSpeed {
    Off,
    Fast,
    Normal,
    Slow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    // Display
    pub always_on_top: bool,
    pub dark_mode: bool,
    pub weight_mode: bool,
    pub display_position: DisplayPosition,
    pub animation_speed: AnimationSpeed,
    pub popup_duration_secs: u64,
    pub no_repeat_count: u64,
    pub font_family: Option<String>,
    pub font_size: u32,
    pub preferred_screen: Option<u32>,

    // Schedule
    pub schedule_rules: Vec<ScheduleRule>,
    pub active_categories: Vec<String>,
    pub paused: bool,

    // Hotkeys
    pub show_hotkey: String,
    pub collect_hotkey: String,

    // Auto start
    pub auto_start: bool,

    // Quiet hours
    pub quiet_hours_start: Option<String>,
    pub quiet_hours_end: Option<String>,

    // Daily limit
    pub daily_limit: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotesData {
    pub categories: Vec<Category>,
    pub quotes: Vec<Quote>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stats {
    pub total_displayed: u64,
    pub today_displayed: u64,
    pub today_date: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            always_on_top: true,
            dark_mode: true,
            weight_mode: true,
            display_position: DisplayPosition::BottomRight,
            animation_speed: AnimationSpeed::Normal,
            popup_duration_secs: 8,
            no_repeat_count: 10,
            font_family: None,
            font_size: 24,
            preferred_screen: None,
            schedule_rules: vec![ScheduleRule::RandomRange {
                min: 3,
                max: 5,
            }],
            active_categories: vec![],
            paused: false,
            show_hotkey: "Ctrl+Win+Q".to_string(),
            collect_hotkey: "Ctrl+Win+Shift+Q".to_string(),
            auto_start: false,
            quiet_hours_start: Some("23:00".to_string()),
            quiet_hours_end: Some("07:00".to_string()),
            daily_limit: None,
        }
    }
}

impl Default for QuotesData {
    fn default() -> Self {
        Self {
            categories: vec![Category { name: "未整理".to_string() }],
            quotes: vec![],
        }
    }
}
