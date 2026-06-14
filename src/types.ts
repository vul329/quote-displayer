export interface Quote {
  id: number;
  content: string;
  author: string | null;
  category: string;
  enabled: boolean;
  show_count: number;
  created_at: string;
}

export interface Category {
  name: string;
}

export interface QuotesData {
  categories: Category[];
  quotes: Quote[];
}

export type ScheduleRule =
  | { type: "Interval"; minutes: number }
  | { type: "RandomRange"; min: number; max: number }
  | { type: "FixedTime"; times: string[] };

export type DisplayPosition =
  | "BottomRight"
  | "BottomLeft"
  | "TopRight"
  | "Center"
  | "Random";

export type AnimationSpeed = "Off" | "Fast" | "Normal" | "Slow";

export interface AppSettings {
  always_on_top: boolean;
  dark_mode: boolean;
  weight_mode: boolean;
  display_position: DisplayPosition;
  animation_speed: AnimationSpeed;
  popup_duration_secs: number;
  no_repeat_count: number;
  font_family: string | null;
  font_size: number;
  preferred_screen: number | null;
  schedule_rules: ScheduleRule[];
  active_categories: string[];
  paused: boolean;
  show_hotkey: string;
  collect_hotkey: string;
  auto_start: boolean;
  quiet_hours_start: string | null;
  quiet_hours_end: string | null;
  daily_limit: number | null;
}

export interface Stats {
  total_displayed: number;
  today_displayed: number;
  today_date: string;
}
