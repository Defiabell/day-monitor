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
