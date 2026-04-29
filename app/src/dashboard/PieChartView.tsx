import { useEffect, useState } from 'react';
import {
  PieChart,
  Pie,
  Cell,
  Tooltip,
  Legend,
  ResponsiveContainer,
} from 'recharts';
import { api } from '../lib/api';
import type { CategoryStat } from '../lib/types';

const CATEGORY_COLORS: Record<string, string> = {
  coding: '#3b82f6',
  meeting: '#ef4444',
  slack: '#a855f7',
  wechat: '#22c55e',
  feishu: '#0ea5e9',
  email: '#f59e0b',
  browser: '#06b6d4',
  reading: '#84cc16',
  writing: '#65a30d',
  design: '#ec4899',
  '3d': '#f97316',
  media: '#d946ef',
  data: '#0d9488',
  system: '#94a3b8',
  app: '#94a3b8',
  other: '#6b7280',
};

function fmtDuration(s: number) {
  const h = Math.floor(s / 3600);
  const m = Math.floor((s % 3600) / 60);
  return h > 0 ? `${h}h${m}m` : `${m}m`;
}

export function PieChartView({ days }: { days: 1 | 7 | 30 }) {
  const [data, setData] = useState<CategoryStat[]>([]);
  const [total, setTotal] = useState(0);

  useEffect(() => {
    if (days === 1) {
      api.todayStats().then((s) => {
        setData(s.categories);
        setTotal(s.total_seconds);
      });
    } else {
      api.trends(days).then((trend) => {
        const agg: Record<string, number> = {};
        for (const day of trend) {
          for (const [cat, s] of day.by_category) {
            agg[cat] = (agg[cat] ?? 0) + s;
          }
        }
        const t = Object.values(agg).reduce((a, b) => a + b, 0);
        const stats: CategoryStat[] = Object.entries(agg)
          .map(([category, seconds]) => ({
            category,
            seconds,
            percent: t > 0 ? (seconds * 100) / t : 0,
          }))
          .sort((a, b) => b.seconds - a.seconds);
        setData(stats);
        setTotal(t);
      });
    }
  }, [days]);

  return (
    <div>
      <h2 className="text-lg font-semibold mb-4">
        Category breakdown · {days === 1 ? 'Today' : `Last ${days} days`}
      </h2>
      <div className="bg-white border border-gray-200 rounded p-4">
        <div className="h-80">
          <ResponsiveContainer>
            <PieChart>
              <Pie
                data={data}
                dataKey="seconds"
                nameKey="category"
                innerRadius={70}
                outerRadius={120}
                label={(entry) => {
                  const e = entry as unknown as CategoryStat;
                  return `${e.category} ${Math.round(e.percent)}%`;
                }}
              >
                {data.map((d, i) => (
                  <Cell
                    key={i}
                    fill={CATEGORY_COLORS[d.category] ?? '#6b7280'}
                  />
                ))}
              </Pie>
              <Tooltip formatter={(v) => fmtDuration(v as number)} />
              <Legend />
            </PieChart>
          </ResponsiveContainer>
        </div>
        <div className="text-center text-2xl font-bold mt-2">
          {fmtDuration(total)}
        </div>
      </div>
    </div>
  );
}
