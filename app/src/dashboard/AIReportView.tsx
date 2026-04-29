import { useEffect, useState } from 'react';
import { marked } from 'marked';
import { api } from '../lib/api';

export function AIReportView({ date }: { date: string }) {
  const [markdown, setMarkdown] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [exists, setExists] = useState(false);

  useEffect(() => {
    setMarkdown(null);
    setError(null);
    setExists(false);
    // Try cached first (force=false)
    api
      .generateAIReport(date, false)
      .then((md) => {
        setMarkdown(md);
        setExists(true);
      })
      .catch(() => {
        // No cache or no events; user must click Generate
      });
  }, [date]);

  const generate = async (force = false) => {
    setLoading(true);
    setError(null);
    try {
      const md = await api.generateAIReport(date, force);
      setMarkdown(md);
      setExists(true);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  return (
    <div>
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-lg font-semibold">AI Report · {date}</h2>
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
          No report yet for {date}. Click Generate to create one (uses Claude
          Haiku, ~1¢).
        </div>
      ) : null}
    </div>
  );
}
