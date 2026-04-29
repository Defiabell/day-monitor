import { invoke } from '@tauri-apps/api/core';
import type { TodayStats, MonitorStatus } from './types';

export const api = {
  todayStats: () => invoke<TodayStats>('get_today_stats'),
  status: () => invoke<MonitorStatus>('get_status'),
  togglePause: () => invoke<MonitorStatus>('toggle_pause'),
  checkLegacyLaunchd: () => invoke<boolean>('check_legacy_launchd'),
  removeLegacyLaunchd: () => invoke<void>('remove_legacy_launchd'),
  quit: () => invoke<void>('quit_app'),
};
