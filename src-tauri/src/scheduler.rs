use std::sync::Mutex;
use std::time::Duration;

use chrono::Timelike;
use tauri::{AppHandle, Manager};

use crate::db::AppData;
use crate::models::ScheduleRule;
use crate::popup;

pub struct LastTriggerTime(pub Mutex<i64>);

impl LastTriggerTime {
    pub fn new() -> Self {
        Self(Mutex::new(chrono::Local::now().timestamp()))
    }

    pub fn update(&self) {
        if let Ok(mut t) = self.0.lock() {
            *t = chrono::Local::now().timestamp();
        }
    }
}

pub struct QuoteScheduler {
    app: AppHandle,
}

impl QuoteScheduler {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }

    pub fn start(&self) {
        let app = self.app.clone();
        std::thread::spawn(move || {
            let mut last_fixed_checked_minute: i64 = -1;

            loop {
                std::thread::sleep(Duration::from_secs(15));

                let data = match app.try_state::<AppData>() {
                    Some(d) => d,
                    None => continue,
                };
                let settings = match data.settings.lock() {
                    Ok(s) => s,
                    Err(_) => continue,
                };

                if settings.paused {
                    drop(settings);
                    continue;
                }

                // Check quiet hours
                if let (Some(ref start), Some(ref end)) =
                    (&settings.quiet_hours_start, &settings.quiet_hours_end)
                {
                    let now = chrono::Local::now();
                    let start_parts: Vec<u32> =
                        start.split(':').filter_map(|s| s.parse().ok()).collect();
                    let end_parts: Vec<u32> =
                        end.split(':').filter_map(|s| s.parse().ok()).collect();

                    if start_parts.len() == 2 && end_parts.len() == 2 {
                        let start_min = start_parts[0] as i64 * 60 + start_parts[1] as i64;
                        let end_min = end_parts[0] as i64 * 60 + end_parts[1] as i64;
                        let now_min = now.hour() as i64 * 60 + now.minute() as i64;

                        let in_quiet = if start_min <= end_min {
                            now_min >= start_min && now_min < end_min
                        } else {
                            now_min >= start_min || now_min < end_min
                        };

                        if in_quiet {
                            drop(settings);
                            continue;
                        }
                    }
                }

                // Check daily limit
                if let Some(limit) = settings.daily_limit {
                    if let Ok(stats) = data.stats.lock() {
                        if stats.today_displayed >= limit {
                            drop(settings);
                            drop(stats);
                            continue;
                        }
                    }
                }

                // Evaluate schedule rules
                let now = chrono::Local::now();
                let now_minute = now.hour() as i64 * 60 + now.minute() as i64;
                let mut should_show = false;

                // Get last trigger time
                let last_ts = app
                    .try_state::<LastTriggerTime>()
                    .and_then(|lt| lt.0.lock().ok().map(|t| *t))
                    .unwrap_or(0);

                for rule in &settings.schedule_rules {
                    match rule {
                        ScheduleRule::Interval { minutes } => {
                            let elapsed = chrono::Local::now().timestamp() - last_ts;
                            if elapsed >= (*minutes as i64 * 60) {
                                should_show = true;
                                break;
                            }
                        }
                        ScheduleRule::RandomRange { min, max } => {
                            let elapsed = chrono::Local::now().timestamp() - last_ts;
                            if elapsed >= (*min as i64 * 60) {
                                let max_secs = (*max.max(min)) as i64 * 60;
                                let progress = elapsed as f64 / max_secs as f64;
                                if progress >= 1.0 || rand::random::<f64>() < progress.min(1.0) {
                                    should_show = true;
                                    break;
                                }
                            }
                        }
                        ScheduleRule::FixedTime { times } => {
                            for t in times {
                                let parts: Vec<&str> = t.split(':').collect();
                                if parts.len() == 2 {
                                    if let (Ok(h), Ok(m)) =
                                        (parts[0].parse::<u32>(), parts[1].parse::<u32>())
                                    {
                                        let rule_minute = h as i64 * 60 + m as i64;
                                        if now_minute != last_fixed_checked_minute
                                            && now_minute == rule_minute
                                        {
                                            should_show = true;
                                            last_fixed_checked_minute = now_minute;
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if should_show {
                        break;
                    }
                }

                drop(settings);

                if should_show {
                    if let Some(lt) = app.try_state::<LastTriggerTime>() {
                        lt.update();
                    }
                    popup::show_quote_popup(&app);
                }
            }
        });
    }
}
