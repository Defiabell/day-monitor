export interface CategoryStat {
  category: string;
  seconds: number;
  percent: number;
}

export interface Event {
  id: number;
  timestamp: string;
  summary: string;
  category: string;
  app_name: string | null;
  duration_s: number;
}

export interface TodayStats {
  total_seconds: number;
  categories: CategoryStat[];
  current_activity: Event | null;
}

export interface MonitorStatus {
  state: 'recording' | 'paused' | 'error';
  message: string | null;
  pid: number | null;
  last_error: string | null;
  skip_reason: string | null;
}

export interface TimelineSegment {
  start: string;
  end: string;
  category: string;
  summary: string;
  duration_s: number;
}

export interface TrendDay {
  date: string;
  by_category: [string, number][];
}

export interface AppUsage {
  app_name: string;
  seconds: number;
  event_count: number;
}

export interface AppTrendDay {
  date: string;
  by_app: [string, number][];
}

export interface Settings {
  interval_secs: number;
  max_image_width: number;
  retention_days: number;
  dedup_threshold: number;
  monthly_budget_usd: number;
  privacy_accepted: boolean;
  active_hour_start: number;
  active_hour_end: number;
  pause_on_battery_below: number;
}

export interface CostStats {
  today_usd: number;
  today_calls: number;
  today_input_tokens: number;
  today_output_tokens: number;
  month_usd: number;
  month_calls: number;
  projected_month_usd: number;
  price_input_per_mtok: number;
  price_output_per_mtok: number;
}
