import { useEffect, useState } from 'react';
import { api } from '../lib/api';
import type { Event } from '../lib/types';

function fmtDuration(s: number) {
  const h = Math.floor(s / 3600);
  const m = Math.floor((s % 3600) / 60);
  if (h > 0) return `${h}h${m}m`;
  if (m > 0) return `${m}m`;
  return `${s}s`;
}

export function EventsView({ date }: { date: string }) {
  const [events, setEvents] = useState<Event[]>([]);
  const [search, setSearch] = useState('');
  const [category, setCategory] = useState('all');
  const [categories, setCategories] = useState<string[]>([]);

  useEffect(() => {
    api.categories().then(setCategories);
  }, []);

  useEffect(() => {
    api.events(date, search, category).then(setEvents);
  }, [date, search, category]);

  return (
    <div>
      <h2 className="text-lg font-semibold mb-4">Events · {date}</h2>
      <div className="flex gap-2 mb-3">
        <input
          type="text"
          placeholder="Search summary…"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          className="flex-1 border border-gray-300 rounded px-3 py-1.5 text-sm"
        />
        <select
          value={category}
          onChange={(e) => setCategory(e.target.value)}
          className="border border-gray-300 rounded px-2 py-1.5 text-sm"
        >
          <option value="all">All categories</option>
          {categories.map((c) => (
            <option key={c} value={c}>
              {c}
            </option>
          ))}
        </select>
      </div>
      <div className="bg-white border border-gray-200 rounded overflow-hidden">
        <table className="w-full text-sm">
          <thead>
            <tr className="text-left text-xs text-gray-500 bg-gray-50">
              <th className="py-2 px-3">Time</th>
              <th className="px-3">Duration</th>
              <th className="px-3">Category</th>
              <th className="px-3">App</th>
              <th className="px-3">Summary</th>
            </tr>
          </thead>
          <tbody>
            {events.map((e) => (
              <tr
                key={e.id}
                className="border-t border-gray-100 hover:bg-gray-50"
              >
                <td className="py-1.5 px-3 font-mono text-xs">
                  {e.timestamp.slice(11, 16)}
                </td>
                <td className="px-3 text-xs">{fmtDuration(e.duration_s)}</td>
                <td className="px-3 text-xs">{e.category}</td>
                <td className="px-3 text-xs">{e.app_name ?? ''}</td>
                <td className="px-3">{e.summary}</td>
              </tr>
            ))}
          </tbody>
        </table>
        {events.length === 0 && (
          <div className="p-6 text-center text-gray-500 text-sm">
            No events match.
          </div>
        )}
      </div>
      <div className="mt-2 text-xs text-gray-500">{events.length} events</div>
    </div>
  );
}
