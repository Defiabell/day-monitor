import { invoke } from '@tauri-apps/api/core';
import type {
  TodayStats,
  MonitorStatus,
  TimelineSegment,
  TrendDay,
  AppUsage,
  Event,
} from './types';

export const api = {
  todayStats: () => invoke<TodayStats>('get_today_stats'),
  status: () => invoke<MonitorStatus>('get_status'),
  togglePause: () => invoke<MonitorStatus>('toggle_pause'),
  checkLegacyLaunchd: () => invoke<boolean>('check_legacy_launchd'),
  removeLegacyLaunchd: () => invoke<void>('remove_legacy_launchd'),
  quit: () => invoke<void>('quit_app'),
  timeline: (date?: string) =>
    invoke<TimelineSegment[]>('get_timeline', { date }),
  trends: (days: number) => invoke<TrendDay[]>('get_trends', { days }),
  appRanking: (days: number) => invoke<AppUsage[]>('get_app_ranking', { days }),
  events: (date: string, search?: string, category?: string) =>
    invoke<Event[]>('get_events', { date, search, category }),
  categories: () => invoke<string[]>('list_categories'),
  generateAIReport: (date: string, force = false) =>
    invoke<string>('generate_ai_report', { date, force }),
  openDashboard: () => invoke<void>('open_dashboard'),
  openSettings: () => invoke<void>('open_settings'),
  saveApiKey: (key: string) => invoke<void>('save_api_key', { key }),
  apiKeySet: () => invoke<boolean>('get_api_key_set'),
};
