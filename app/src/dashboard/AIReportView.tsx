import { useEffect, useState } from 'react';
import { marked } from 'marked';
import { api } from '../lib/api';

type Mode = 'daily' | 'weekly';

export function AIReportView({ date }: { date: string }) {
  const [mode, setMode] = useState<Mode>('daily');
  const [markdown, setMarkdown] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [exists, setExists] = useState(false);

  useEffect(() => {
    setMarkdown(null);
    setError(null);
    setExists(false);
    // Cached report loads instantly; otherwise this generates on first view.
    const load =
      mode === 'daily'
        ? api.generateAIReport(date, false)
        : api.generateWeeklyReport(date, false);
    load
      .then((md) => {
        setMarkdown(md);
        setExists(true);
      })
      .catch(() => {
        // No events for this date/week; user can still click Generate
      });
  }, [date, mode]);

  const generate = async (force = false) => {
    setLoading(true);
    setError(null);
    try {
      const md =
        mode === 'daily'
          ? await api.generateAIReport(date, force)
          : await api.generateWeeklyReport(date, force);
      setMarkdown(md);
      setExists(true);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  const modeBtn = (m: Mode, label: string) => (
    <button
      onClick={() => setMode(m)}
      className={
        'px-3 py-1 text-sm rounded ' +
        (mode === m
          ? 'bg-gray-900 text-white'
          : 'border border-gray-300 hover:bg-gray-100')
      }
    >
      {label}
    </button>
  );

  return (
    <div>
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-3">
          <h2 className="text-lg font-semibold">
            {mode === 'daily' ? `AI 日报 · ${date}` : `AI 周报 · 截至 ${date} 的最近 7 天`}
          </h2>
          <div className="flex gap-1">
            {modeBtn('daily', '日报')}
            {modeBtn('weekly', '周报')}
          </div>
        </div>
        <div className="flex gap-2">
          {!exists && (
            <button
              onClick={() => generate(false)}
              disabled={loading}
              className="px-4 py-1.5 text-sm bg-gray-900 text-white rounded hover:bg-gray-700 disabled:opacity-50"
            >
              {loading ? 'Generating…' : 'Generate Report'}
            </button>
          )}
          {exists && (
            <button
              onClick={() => generate(true)}
              disabled={loading}
              className="px-4 py-1.5 text-sm border border-gray-300 rounded hover:bg-gray-100 disabled:opacity-50"
            >
              {loading ? 'Regenerating…' : '↻ Regenerate'}
            </button>
          )}
        </div>
      </div>

      {error && <div className="text-red-600 text-sm mb-4">{error}</div>}

      {markdown ? (
        <article
          className="bg-white border border-gray-200 rounded p-6 leading-relaxed [&_h1]:text-xl [&_h1]:font-bold [&_h1]:mt-2 [&_h1]:mb-3 [&_h2]:text-base [&_h2]:font-semibold [&_h2]:mt-4 [&_h2]:mb-2 [&_table]:my-3 [&_table]:w-full [&_th]:text-left [&_th]:py-1 [&_th]:border-b [&_td]:py-1 [&_td]:border-b [&_td]:border-gray-100"
          dangerouslySetInnerHTML={{ __html: marked.parse(markdown) as string }}
        />
      ) : !loading && !error ? (
        <div className="text-gray-500">
          {mode === 'daily'
            ? `No report yet for ${date}. Click Generate to create one (uses Claude Haiku, ~1¢).`
            : `该周暂无周报。点击 Generate 基于最近 7 天记录生成（按 ISO 周缓存，周中生成的是截至 ${date} 的部分周报）。`}
        </div>
      ) : null}
    </div>
  );
}
