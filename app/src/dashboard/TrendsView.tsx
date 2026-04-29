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
  app: '#94a3b8', // 老数据兼容
  other: '#6b7280',
};

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
          堆叠柱状图：每天的总时长按 14 个分类（coding / meeting / slack / 3d ... ）拆分
        </p>
        <div className="bg-white border border-gray-200 rounded p-4 h-80">
          <ResponsiveContainer>
            <BarChart data={barData}>
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
                <Tooltip />
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
