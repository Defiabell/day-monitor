import { useEffect, useState } from 'react';
import { api } from '../lib/api';
import type { TodayStats, MonitorStatus } from '../lib/types';

function fmtDuration(secs: number): string {
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  if (h > 0) return `${h}h ${m}m`;
  return `${m}m`;
}

function statusIndicator(s: MonitorStatus): string {
  switch (s.state) {
    case 'recording':
      return '● Recording';
    case 'paused':
      return '○ Paused';
    case 'error':
      return `⚠ ${s.message ?? 'Error'}`;
    default:
      return s.state;
  }
}

export function Popover() {
  const [stats, setStats] = useState<TodayStats | null>(null);
  const [status, setStatus] = useState<MonitorStatus | null>(null);
  const [error, setError] = useState<string | null>(null);

  const refresh = async () => {
    try {
      const [s, st] = await Promise.all([api.todayStats(), api.status()]);
      setStats(s);
      setStatus(st);
      setError(null);
    } catch (e) {
      setError(String(e));
    }
  };

  useEffect(() => {
    refresh();
    const id = setInterval(refresh, 5000);
    return () => clearInterval(id);
  }, []);

  const togglePause = async () => {
    try {
      const newStatus = await api.togglePause();
      setStatus(newStatus);
    } catch (e) {
      setError(String(e));
    }
  };

  if (error) {
    return (
      <div className="p-3 w-[200px] h-[300px] bg-white text-xs text-red-600">
        {error}
      </div>
    );
  }

  if (!stats || !status) {
    return (
      <div className="p-3 w-[200px] h-[300px] bg-white text-xs text-gray-500">
        Loading…
      </div>
    );
  }

  return (
    <div className="p-3 w-[200px] h-[300px] bg-white text-gray-900 flex flex-col">
      <div className="flex items-center justify-between text-xs">
        <span className="font-medium">{statusIndicator(status)}</span>
        <button
          onClick={() => api.openSettings()}
          className="text-gray-400 hover:text-gray-700"
          title="Settings"
        >
          ⚙
        </button>
      </div>

      <div className="my-2">
        <div className="text-[10px] text-gray-500 uppercase tracking-wide">
          Today
        </div>
        <div className="text-2xl font-bold leading-tight">
          {fmtDuration(stats.total_seconds)}
        </div>
      </div>

      <hr className="border-gray-200" />

      <div className="my-2 flex-1 overflow-hidden space-y-0.5">
        {stats.categories.slice(0, 4).map((c) => (
          <div key={c.category} className="flex justify-between text-[11px]">
            <span className="font-medium">{c.category}</span>
            <span className="text-gray-600">
              {fmtDuration(c.seconds)} ({Math.round(c.percent)}%)
            </span>
          </div>
        ))}
        {stats.categories.length > 4 && (
          <div className="text-[10px] text-gray-400">
            … +{stats.categories.length - 4} more
          </div>
        )}
        {stats.categories.length === 0 && (
          <div className="text-[10px] text-gray-400 italic">No activity yet</div>
        )}
      </div>

      {stats.current_activity && (
        <div className="text-[10px] text-gray-700 mb-2 truncate">
          Now: {stats.current_activity.summary} [
          {stats.current_activity.category}]
        </div>
      )}

      <div className="flex gap-1.5">
        <button
          onClick={togglePause}
          className="flex-1 py-1 text-xs border border-gray-300 rounded hover:bg-gray-50"
        >
          {status.state === 'paused' ? '▶ Resume' : '⏸ Pause'}
        </button>
        <button
          onClick={() => api.openDashboard()}
          className="flex-1 py-1 text-xs border border-gray-300 rounded hover:bg-gray-50"
        >
          Dashboard
        </button>
      </div>
    </div>
  );
}
