import { useEffect, useState } from 'react';
import {
  BarChart,
  Bar,
  LineChart,
  Line,
  XAxis,
  YAxis,
  Tooltip,
  Legend,
  ResponsiveContainer,
  CartesianGrid,
} from 'recharts';
import { api } from '../lib/api';
import type { TrendDay, AppTrendDay } from '../lib/types';
import { categoryColor } from '../lib/categories';

// Sort tooltip rows by hours descending so the biggest time sinks read first
const byHoursDesc = (item: { value?: unknown }) => -Number(item.value ?? 0);

// Distinct color palette for apps (lines), sorted for visual contrast
const APP_COLORS = [
  '#2563eb', '#dc2626', '#16a34a', '#9333ea', '#ea580c',
  '#0891b2', '#ca8a04', '#be185d', '#15803d', '#7c3aed',
];

export function TrendsView({ days }: { days: 1 | 7 | 30 }) {
  const [trend, setTrend] = useState<TrendDay[]>([]);
  const [appTrend, setAppTrend] = useState<AppTrendDay[]>([]);
  const range = Math.max(days, 7);

  useEffect(() => {
    api.trends(range).then(setTrend);
    api.appTrends(range, 8).then(setAppTrend);
  }, [range]);

  // Bar chart data: per-day per-category hours
  const barData = trend.map((d) => {
    const row: Record<string, string | number> = { date: d.date.slice(5) };
    for (const [cat, secs] of d.by_category) {
      row[cat] = Math.round((secs / 3600) * 100) / 100;
    }
    return row;
  });
  const allCats = Array.from(
    new Set(trend.flatMap((d) => d.by_category.map(([c]) => c))),
  );

  // Line chart data: per-day per-app hours, top apps only
  const allApps = Array.from(
    new Set(appTrend.flatMap((d) => d.by_app.map(([a]) => a))),
  );
  const lineData = appTrend.map((d) => {
    const row: Record<string, string | number> = { date: d.date.slice(5) };
    const dayMap = Object.fromEntries(d.by_app);
    for (const app of allApps) {
      row[app] = Math.round(((dayMap[app] ?? 0) / 3600) * 100) / 100;
    }
    return row;
  });

  return (
    <div className="space-y-6">
      <section>
        <h2 className="text-lg font-semibold mb-1">
          按分类（category）· 最近 {range} 天（小时）
        </h2>
        <p className="text-xs text-gray-500 mb-3">
          堆叠柱状图：每天的总时长按活动分类（coding / ai-coding / meeting / communicating ... ）拆分
        </p>
        <div className="bg-white border border-gray-200 rounded p-4 h-80">
          <ResponsiveContainer>
            <BarChart data={barData}>
              <CartesianGrid strokeDasharray="3 3" />
              <XAxis dataKey="date" />
              <YAxis />
              <Tooltip itemSorter={byHoursDesc} />
              <Legend />
              {allCats.map((c) => (
                <Bar
                  key={c}
                  dataKey={c}
                  stackId="a"
                  fill={categoryColor(c)}
                />
              ))}
            </BarChart>
          </ResponsiveContainer>
        </div>
      </section>

      <section>
        <h2 className="text-lg font-semibold mb-1">
          按应用（app）· Top 8 · 最近 {range} 天（小时）
        </h2>
        <p className="text-xs text-gray-500 mb-3">
          线形图：维度是具体应用名（VS Code / Chrome / Slack ... ），不是分类。
          只画使用时长最久的 8 个 app；屏幕被识别为黑屏 / 无应用名时不计入。
        </p>
        <div className="bg-white border border-gray-200 rounded p-4 h-80">
          {allApps.length === 0 ? (
            <div className="flex items-center justify-center h-full text-gray-500 text-sm">
              No app data yet — line chart will populate as data accumulates.
            </div>
          ) : (
            <ResponsiveContainer>
              <LineChart data={lineData}>
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis dataKey="date" />
                <YAxis />
                <Tooltip itemSorter={byHoursDesc} />
                <Legend />
                {allApps.map((app, i) => (
                  <Line
                    key={app}
                    type="monotone"
                    dataKey={app}
                    stroke={APP_COLORS[i % APP_COLORS.length]}
                    strokeWidth={2}
                    dot={{ r: 3 }}
                    activeDot={{ r: 5 }}
                  />
                ))}
              </LineChart>
            </ResponsiveContainer>
          )}
        </div>
      </section>
    </div>
  );
}
