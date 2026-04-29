import { useEffect, useState } from 'react';
import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  Tooltip,
  Legend,
  ResponsiveContainer,
  CartesianGrid,
} from 'recharts';
import { api } from '../lib/api';
import type { TrendDay } from '../lib/types';

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

export function TrendsView({ days }: { days: 1 | 7 | 30 }) {
  const [trend, setTrend] = useState<TrendDay[]>([]);
  const range = Math.max(days, 7);

  useEffect(() => {
    api.trends(range).then(setTrend);
  }, [range]);

  const chartData = trend.map((d) => {
    const row: Record<string, string | number> = { date: d.date.slice(5) };
    for (const [cat, secs] of d.by_category) {
      row[cat] = Math.round((secs / 3600) * 100) / 100;
    }
    return row;
  });

  const allCats = Array.from(
    new Set(trend.flatMap((d) => d.by_category.map(([c]) => c))),
  );

  return (
    <div>
      <h2 className="text-lg font-semibold mb-4">
        Trends · Last {range} days (hours)
      </h2>
      <div className="bg-white border border-gray-200 rounded p-4 h-96">
        <ResponsiveContainer>
          <BarChart data={chartData}>
            <CartesianGrid strokeDasharray="3 3" />
            <XAxis dataKey="date" />
            <YAxis />
            <Tooltip />
            <Legend />
            {allCats.map((c) => (
              <Bar
                key={c}
                dataKey={c}
                stackId="a"
                fill={CATEGORY_COLORS[c] ?? '#6b7280'}
              />
            ))}
          </BarChart>
        </ResponsiveContainer>
      </div>
    </div>
  );
}
