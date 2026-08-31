import { invoke } from '@tauri-apps/api/core';
import { mockApi } from './mockApi';
import type {
  TodayStats,
  MonitorStatus,
  TimelineSegment,
  TrendDay,
  AppUsage,
  AppTrendDay,
  Event,
  Settings,
  CostStats,
} from './types';

// When served as a plain web page (no Tauri runtime), fall back to generated
// demo data so the dashboard can be hosted as a live demo.
const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
export const isDemo = !isTauri;

const realApi = {
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
  appTrends: (days: number, topN = 8) =>
    invoke<AppTrendDay[]>('get_app_trends', { days, topN }),
  events: (date: string, search?: string, category?: string) =>
    invoke<Event[]>('get_events', { date, search, category }),
  categories: () => invoke<string[]>('list_categories'),
  generateAIReport: (date: string, force = false) =>
    invoke<string>('generate_ai_report', { date, force }),
  generateWeeklyReport: (endDate?: string, force = false) =>
    invoke<string>('generate_weekly_report', { endDate, force }),
  exportData: () => invoke<string>('export_data'),
  openDashboard: () => invoke<void>('open_dashboard'),
  openSettings: () => invoke<void>('open_settings'),
  saveApiKey: (key: string) => invoke<void>('save_api_key', { key }),
  apiKeySet: () => invoke<boolean>('get_api_key_set'),
  settings: () => invoke<Settings>('get_settings'),
  saveSettings: (settings: Settings) => invoke<void>('save_settings', { settings }),
  costStats: () => invoke<CostStats>('get_cost_stats'),
};

export const api: typeof realApi = isTauri ? realApi : mockApi;
