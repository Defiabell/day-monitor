import { useState } from 'react';
import { TimelineView } from './TimelineView';
import { PieChartView } from './PieChartView';
import { TrendsView } from './TrendsView';
import { AppRankView } from './AppRankView';
import { EventsView } from './EventsView';
import { AIReportView } from './AIReportView';

type View = 'timeline' | 'pie' | 'trends' | 'apps' | 'events' | 'report';

const VIEWS: { id: View; label: string; icon: string }[] = [
  { id: 'timeline', label: 'Timeline', icon: '📊' },
  { id: 'pie', label: 'Categories', icon: '🥧' },
  { id: 'trends', label: 'Trends', icon: '📈' },
  { id: 'apps', label: 'Apps', icon: '📱' },
  { id: 'events', label: 'Events', icon: '📋' },
  { id: 'report', label: 'AI Report', icon: '✨' },
];

function todayStr(): string {
  const d = new Date();
  return d.toISOString().slice(0, 10);
}

export function Dashboard() {
  const [view, setView] = useState<View>('timeline');
  const [date, setDate] = useState(todayStr());
  const [days, setDays] = useState<1 | 7 | 30>(1);

  return (
    <div className="flex h-screen bg-gray-50 text-gray-900 font-sans">
      <aside className="w-44 bg-white border-r border-gray-200 flex flex-col">
        <div className="p-4 text-base font-semibold border-b border-gray-200">
          ☕ Day Monitor
        </div>
        <nav className="flex-1 p-2 space-y-1">
          {VIEWS.map((v) => (
            <button
              key={v.id}
              onClick={() => setView(v.id)}
              className={`w-full text-left px-3 py-2 rounded text-sm flex items-center gap-2 ${
                view === v.id
                  ? 'bg-gray-100 font-medium'
                  : 'hover:bg-gray-50'
              }`}
            >
              <span>{v.icon}</span> {v.label}
            </button>
          ))}
        </nav>
      </aside>

      <div className="flex-1 flex flex-col overflow-hidden">
        <header className="flex items-center gap-3 px-6 py-3 border-b border-gray-200 bg-white">
          <div className="flex gap-1">
            {([1, 7, 30] as const).map((d) => (
              <button
                key={d}
                onClick={() => {
                  setDays(d);
                  // 点 Today 时把右上角日期也重置成今天
                  if (d === 1) setDate(todayStr());
                }}
                className={`px-3 py-1 text-xs rounded border ${
                  days === d
                    ? 'bg-gray-900 text-white border-gray-900'
                    : 'border-gray-300 hover:bg-gray-100'
                }`}
              >
                {d === 1 ? 'Today' : `${d}d`}
              </button>
            ))}
          </div>
          <input
            type="date"
            value={date}
            onChange={(e) => setDate(e.target.value)}
            className="ml-auto text-sm border border-gray-300 rounded px-2 py-1"
          />
        </header>

        <main className="flex-1 overflow-auto p-6">
          {view === 'timeline' && <TimelineView date={date} />}
          {view === 'pie' && <PieChartView days={days} />}
          {view === 'trends' && <TrendsView days={days} />}
          {view === 'apps' && <AppRankView days={days} />}
          {view === 'events' && <EventsView date={date} />}
          {view === 'report' && <AIReportView date={date} />}
        </main>
      </div>
    </div>
  );
}
