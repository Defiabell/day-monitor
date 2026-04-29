import { useEffect, useState } from 'react';
import { api } from '../lib/api';
import type { TimelineSegment } from '../lib/types';

const CATEGORY_COLORS: Record<string, string> = {
  coding: '#3b82f6',
  meeting: '#ef4444',
  slack: '#a855f7',
  wechat: '#22c55e',
  feishu: '#0ea5e9',
  email: '#f59e0b',
  browser: '#06b6d4',
  reading: '#84cc16',
  design: '#ec4899',
  app: '#94a3b8',
  other: '#6b7280',
};

function fmtDuration(s: number) {
  const h = Math.floor(s / 3600);
  const m = Math.floor((s % 3600) / 60);
  return h > 0 ? `${h}h${m}m` : `${m}m`;
}

function timeToMinutes(hhmm: string): number {
  const [h, m] = hhmm.split(':').map(Number);
  return (h || 0) * 60 + (m || 0);
}

export function TimelineView({ date }: { date: string }) {
  const [segments, setSegments] = useState<TimelineSegment[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    setLoading(true);
    api
      .timeline(date)
      .then((s) => {
        setSegments(s);
        setLoading(false);
      })
      .catch(() => setLoading(false));
  }, [date]);

  if (loading) return <div className="text-gray-500">Loading…</div>;
  if (segments.length === 0)
    return <div className="text-gray-500">No activity for {date}</div>;

  const startMin = Math.max(
    0,
    Math.min(...segments.map((s) => timeToMinutes(s.start))) - 30,
  );
  const endMin = Math.min(
    24 * 60,
    Math.max(...segments.map((s) => timeToMinutes(s.end))) + 30,
  );
  const totalMin = endMin - startMin;

  return (
    <div>
      <h2 className="text-lg font-semibold mb-4">{date} Timeline</h2>
      <div className="relative bg-white border border-gray-200 rounded p-4">
        <div className="relative h-12 bg-gray-100 rounded overflow-hidden">
          {segments.map((seg, i) => {
            const left =
              ((timeToMinutes(seg.start) - startMin) / totalMin) * 100;
            const width = (seg.duration_s / 60 / totalMin) * 100;
            return (
              <div
                key={i}
                className="absolute top-0 bottom-0"
                style={{
                  left: `${left}%`,
                  width: `${Math.max(width, 0.3)}%`,
                  background: CATEGORY_COLORS[seg.category] ?? '#6b7280',
                }}
                title={`${seg.start}–${seg.end} ${seg.category}: ${seg.summary} (${fmtDuration(seg.duration_s)})`}
              />
            );
          })}
        </div>
        <div className="flex justify-between text-xs text-gray-500 mt-1">
          <span>
            {Math.floor(startMin / 60).toString().padStart(2, '0')}:
            {(startMin % 60).toString().padStart(2, '0')}
          </span>
          <span>
            {Math.floor(endMin / 60).toString().padStart(2, '0')}:
            {(endMin % 60).toString().padStart(2, '0')}
          </span>
        </div>
      </div>

      <table className="mt-6 w-full text-sm">
        <thead>
          <tr className="text-left text-xs text-gray-500 border-b">
            <th className="py-2">Start–End</th>
            <th>Duration</th>
            <th>Category</th>
            <th>Activity</th>
          </tr>
        </thead>
        <tbody>
          {segments.map((s, i) => (
            <tr key={i} className="border-b border-gray-100">
              <td className="py-2 font-mono">
                {s.start}–{s.end}
              </td>
              <td>{fmtDuration(s.duration_s)}</td>
              <td>
                <span
                  className="inline-block px-2 py-0.5 rounded text-xs text-white"
                  style={{
                    background: CATEGORY_COLORS[s.category] ?? '#6b7280',
                  }}
                >
                  {s.category}
                </span>
              </td>
              <td>{s.summary}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
