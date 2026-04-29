import { useEffect, useState } from 'react';
import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  Tooltip,
  ResponsiveContainer,
} from 'recharts';
import { api } from '../lib/api';
import type { AppUsage } from '../lib/types';

function fmtDuration(s: number) {
  const h = Math.floor(s / 3600);
  const m = Math.floor((s % 3600) / 60);
  return h > 0 ? `${h}h${m}m` : `${m}m`;
}

export function AppRankView({ days }: { days: 1 | 7 | 30 }) {
  const [data, setData] = useState<AppUsage[]>([]);

  useEffect(() => {
    api.appRanking(days).then(setData);
  }, [days]);

  const chartData = data.map((a) => ({
    app: a.app_name,
    hours: Math.round((a.seconds / 3600) * 100) / 100,
    seconds: a.seconds,
  }));

  return (
    <div>
      <h2 className="text-lg font-semibold mb-4">
        Top apps · {days === 1 ? 'Today' : `Last ${days} days`}
      </h2>
      <div className="bg-white border border-gray-200 rounded p-4 h-96">
        <ResponsiveContainer>
          <BarChart data={chartData} layout="vertical">
            <XAxis type="number" />
            <YAxis dataKey="app" type="category" width={120} />
            <Tooltip
              formatter={(_v, _n, p) =>
                fmtDuration((p.payload as { seconds: number }).seconds)
              }
            />
            <Bar dataKey="hours" fill="#3b82f6" />
          </BarChart>
        </ResponsiveContainer>
      </div>
      {data.length === 0 && (
        <div className="text-center text-gray-500 mt-4 text-sm">
          No app data yet. App names start being captured after this version.
        </div>
      )}
    </div>
  );
}
