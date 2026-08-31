// Demo data source used when the app runs outside Tauri (e.g. the hosted
// GitHub Pages demo). Generates a deterministic, believable "engineer's day"
// per date so every visitor sees the same rich sample data.
import type {
  TodayStats,
  MonitorStatus,
  TimelineSegment,
  TrendDay,
  AppUsage,
  AppTrendDay,
  Event,
  Settings,
  CostStats,
  CategoryStat,
} from './types';

// ---------- deterministic PRNG ----------

function hashStr(s: string): number {
  let h = 2166136261;
  for (let i = 0; i < s.length; i++) {
    h ^= s.charCodeAt(i);
    h = Math.imul(h, 16777619);
  }
  return h >>> 0;
}

function mulberry32(seed: number): () => number {
  let a = seed;
  return () => {
    a |= 0;
    a = (a + 0x6d2b79f5) | 0;
    let t = Math.imul(a ^ (a >>> 15), 1 | a);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

// ---------- vocabulary (mirrors analyze.rs VALID_CATEGORIES, v2 taxonomy) ----------

const SUMMARIES: Record<string, string[]> = {
  coding: [
    '在 Cursor 写 React 组件',
    '终端里跑测试',
    '重构订阅模块',
    '写 TypeScript 类型',
    '调试接口报错',
    '写单元测试',
  ],
  'ai-coding': ['Claude Code 在跑任务', '看 AI 代理改代码', '等 Codex 输出结果'],
  'code-review': ['看 GitHub PR diff', 'review 同事的改动'],
  meeting: ['Zoom 视频例会', '屏幕共享讨论方案', '和同事过需求'],
  communicating: [
    '回复 Slack 消息',
    '回微信消息',
    '看飞书文档评论',
    '处理邮件收件箱',
  ],
  browsing: ['查 Stack Overflow', '看 GitHub 仓库', '浏览技术博客'],
  reading: ['读技术文档', '看 Notion 笔记', '读 API 文档'],
  writing: ['写设计文档', '在 Obsidian 记笔记', '写周报'],
  design: ['在 Figma 看设计稿'],
  data: ['看 Grafana 面板', '写 SQL 查询数据'],
  entertainment: ['刷 B 站视频', '逛社交媒体'],
  app: ['整理 Finder 文件'],
};

const APPS: Record<string, string[]> = {
  coding: ['Cursor', 'iTerm2', 'VS Code'],
  'ai-coding': ['iTerm2', 'Cursor'],
  'code-review': ['Chrome'],
  meeting: ['Zoom'],
  communicating: ['Slack', 'WeChat', 'Lark', 'Mail'],
  browsing: ['Chrome'],
  reading: ['Chrome', 'Notion'],
  writing: ['Obsidian', 'Notion'],
  design: ['Figma'],
  data: ['Chrome', 'DataGrip'],
  entertainment: ['Chrome'],
  app: ['Finder'],
};

interface Block {
  from: number; // minutes since midnight
  to: number;
  cats: string[]; // weighted pool
}

function weekdayBlocks(rng: () => number): Block[] {
  const blocks: Block[] = [
    { from: 9 * 60 + 20 + Math.floor(rng() * 20), to: 10 * 60, cats: ['communicating', 'communicating', 'reading'] },
    { from: 10 * 60, to: 10 * 60 + 30, cats: ['meeting'] },
    { from: 10 * 60 + 32, to: 12 * 60 + 30, cats: ['coding', 'coding', 'ai-coding', 'browsing', 'reading'] },
    { from: 13 * 60 + 30 + Math.floor(rng() * 15), to: 15 * 60 + 30, cats: ['coding', 'ai-coding', 'code-review', 'communicating'] },
  ];
  if (rng() < 0.5) {
    blocks.push({ from: 15 * 60 + 32, to: 16 * 60 + 15, cats: ['meeting'] });
    blocks.push({ from: 16 * 60 + 17, to: 18 * 60 + 30, cats: ['coding', 'ai-coding', 'data', 'browsing', 'design'] });
  } else {
    blocks.push({ from: 15 * 60 + 32, to: 18 * 60 + 30, cats: ['coding', 'coding', 'ai-coding', 'data', 'browsing'] });
  }
  blocks.push({ from: 18 * 60 + 32, to: 19 * 60 + 15 + Math.floor(rng() * 30), cats: ['writing', 'communicating', 'entertainment'] });
  return blocks;
}

function weekendBlocks(rng: () => number): Block[] {
  const start = 20 * 60 + Math.floor(rng() * 40);
  return [{ from: start, to: start + 90 + Math.floor(rng() * 60), cats: ['coding', 'browsing', 'reading', 'entertainment'] }];
}

function pad(n: number): string {
  return n.toString().padStart(2, '0');
}

function minToHHMM(min: number): string {
  return `${pad(Math.floor(min / 60))}:${pad(min % 60)}`;
}

function pick<T>(rng: () => number, arr: T[]): T {
  return arr[Math.floor(rng() * arr.length)];
}

interface DayData {
  segments: TimelineSegment[];
  events: Event[];
}

const dayCache = new Map<string, DayData>();

function generateDay(date: string): DayData {
  const cached = dayCache.get(date);
  if (cached) return cached;

  const rng = mulberry32(hashStr(date));
  const dow = new Date(`${date}T12:00:00`).getDay();
  const isWeekend = dow === 0 || dow === 6;
  const blocks = isWeekend ? weekendBlocks(rng) : weekdayBlocks(rng);

  const segments: TimelineSegment[] = [];
  const events: Event[] = [];
  let eventId = hashStr(date) % 100000;

  for (const block of blocks) {
    let cursor = block.from;
    while (cursor < block.to - 4) {
      const len = Math.min(8 + Math.floor(rng() * 28), block.to - cursor);
      const category = pick(rng, block.cats);
      const summary = pick(rng, SUMMARIES[category] ?? ['屏幕活动']);
      const app = pick(rng, APPS[category] ?? ['Finder']);
      const start = minToHHMM(cursor);
      const end = minToHHMM(cursor + len);
      segments.push({ start, end, category, summary, duration_s: len * 60 });
      events.push({
        id: eventId++,
        timestamp: `${date} ${start}:${pad(Math.floor(rng() * 60))}`,
        summary,
        category,
        app_name: app,
        duration_s: len * 60,
      });
      cursor += len + (rng() < 0.25 ? 2 + Math.floor(rng() * 4) : 0);
    }
  }

  const data = { segments, events };
  dayCache.set(date, data);
  return data;
}

// Same "today" convention as Dashboard's todayStr() (UTC calendar date), so the
// default picker date and the generated data always agree for demo visitors.
function utcDateStr(offsetDays = 0): string {
  const d = new Date();
  d.setUTCDate(d.getUTCDate() - offsetDays);
  return d.toISOString().slice(0, 10);
}

function categoryStats(segments: TimelineSegment[]): CategoryStat[] {
  const byCat = new Map<string, number>();
  for (const s of segments) {
    byCat.set(s.category, (byCat.get(s.category) ?? 0) + s.duration_s);
  }
  const total = [...byCat.values()].reduce((a, b) => a + b, 0) || 1;
  return [...byCat.entries()]
    .map(([category, seconds]) => ({ category, seconds, percent: (seconds / total) * 100 }))
    .sort((a, b) => b.seconds - a.seconds);
}

function sleep(ms: number): Promise<void> {
  return new Promise((r) => setTimeout(r, ms));
}

async function delay<T>(value: T): Promise<T> {
  await sleep(80 + Math.random() * 150);
  return value;
}

function fmtHours(seconds: number): string {
  const h = Math.floor(seconds / 3600);
  const m = Math.round((seconds % 3600) / 60);
  return h > 0 ? `${h} 小时 ${m} 分钟` : `${m} 分钟`;
}

function buildReport(date: string): string {
  const { segments, events } = generateDay(date);
  if (segments.length === 0) return `# ${date} 工作日报\n\n当天没有记录到活动。`;
  const stats = categoryStats(segments);
  const total = segments.reduce((a, s) => a + s.duration_s, 0);
  const coding = stats.find((s) => s.category === 'coding')?.seconds ?? 0;
  const meeting = stats.find((s) => s.category === 'meeting')?.seconds ?? 0;
  const comm = stats
    .filter((s) => s.category === 'communicating')
    .reduce((a, s) => a + s.seconds, 0);

  let longest = segments[0];
  for (const s of segments) {
    if (s.category === 'coding' && s.duration_s > (longest?.duration_s ?? 0)) longest = s;
  }

  const rows = stats
    .slice(0, 5)
    .map((s) => `| ${s.category} | ${fmtHours(s.seconds)} | ${s.percent.toFixed(0)}% |`)
    .join('\n');

  return [
    `# ${date} 工作日报`,
    '',
    '## 总览',
    `今日累计记录 **${fmtHours(total)}**，共 ${events.length} 个活动片段。`,
    '',
    '| 类别 | 时长 | 占比 |',
    '| --- | --- | --- |',
    rows,
    '',
    '## 深度工作',
    `全天编码合计 **${fmtHours(coding)}**，最长的一段专注发生在 ${longest.start}–${longest.end}（${longest.summary}）。`,
    '',
    '## 会议与沟通',
    `会议 ${fmtHours(meeting)}；Slack、微信、邮件等碎片沟通合计 ${fmtHours(comm)}。`,
    '',
    '## AI 协作',
    `AI 代理编码 ${fmtHours(stats.find((s) => s.category === 'ai-coding')?.seconds ?? 0)}，与人工编码穿插进行。`,
    '',
    '## 建议',
    '- 上午 10:30–12:30 是今天最高效的编码时段，重要任务尽量安排在这个窗口。',
    '- 沟通消息集中在午后，可以尝试批量处理而不是随到随回。',
    '',
    '> 演示数据由本地生成——真实应用中，这份日报由 Claude 基于你一天的活动记录撰写。',
  ].join('\n');
}

// ---------- mock state ----------

let paused = false;

const demoSettings: Settings = {
  interval_secs: 20,
  max_image_width: 1024,
  retention_days: 30,
  dedup_threshold: 0.92,
  monthly_budget_usd: 0,
  privacy_accepted: true,
  active_hour_start: 9,
  active_hour_end: 23,
  pause_on_battery_below: 20,
  blacklist_apps: ['1Password', 'Keychain Access'],
  show_tray_stats: true,
  api_base_url: '',
  model: '',
  price_input_per_mtok: 0,
  price_output_per_mtok: 0,
};

function buildWeeklyReport(endDate: string): string {
  const end = new Date(`${endDate}T12:00:00Z`);
  const days: string[] = [];
  for (let i = 6; i >= 0; i--) {
    const d = new Date(end);
    d.setUTCDate(d.getUTCDate() - i);
    days.push(d.toISOString().slice(0, 10));
  }
  const lines = days.map((date) => {
    const { segments } = generateDay(date);
    const total = segments.reduce((a, s) => a + s.duration_s, 0);
    const top = categoryStats(segments)[0];
    return `| ${date} | ${fmtHours(total)} | ${top ? top.category : '无记录'} |`;
  });
  return [
    `# 工作周报（${days[0]} ~ ${days[6]}）`,
    '',
    '## 本周概览',
    '本周节奏平稳，编码为主线，会议与沟通穿插其间。',
    '',
    '## 每日一行',
    '| 日期 | 时长 | 主要内容 |',
    '| --- | --- | --- |',
    ...lines,
    '',
    '> 演示数据由本地生成——真实应用中，这份周报由 Claude 基于 7 天记录撰写。',
  ].join('\n');
}

function status(): MonitorStatus {
  return {
    state: paused ? 'paused' : 'recording',
    message: null,
    pid: 41234,
    last_error: null,
    skip_reason: null,
  };
}

export const mockApi = {
  todayStats: (): Promise<TodayStats> => {
    const { segments, events } = generateDay(utcDateStr());
    return delay({
      total_seconds: segments.reduce((a, s) => a + s.duration_s, 0),
      categories: categoryStats(segments),
      current_activity: events[events.length - 1] ?? null,
    });
  },
  status: () => delay(status()),
  togglePause: () => {
    paused = !paused;
    return delay(status());
  },
  checkLegacyLaunchd: () => delay(false),
  removeLegacyLaunchd: () => delay(undefined),
  quit: () => delay(undefined),
  timeline: (date?: string) => delay(generateDay(date ?? utcDateStr()).segments),
  trends: (days: number): Promise<TrendDay[]> =>
    delay(
      Array.from({ length: days }, (_, i) => {
        const date = utcDateStr(days - 1 - i);
        const byCat = new Map<string, number>();
        for (const s of generateDay(date).segments) {
          byCat.set(s.category, (byCat.get(s.category) ?? 0) + s.duration_s);
        }
        return { date, by_category: [...byCat.entries()] };
      }),
    ),
  appRanking: (days: number): Promise<AppUsage[]> => {
    const byApp = new Map<string, { seconds: number; count: number }>();
    for (let i = 0; i < days; i++) {
      for (const e of generateDay(utcDateStr(i)).events) {
        const key = e.app_name ?? 'Unknown';
        const cur = byApp.get(key) ?? { seconds: 0, count: 0 };
        cur.seconds += e.duration_s;
        cur.count += 1;
        byApp.set(key, cur);
      }
    }
    return delay(
      [...byApp.entries()]
        .map(([app_name, v]) => ({ app_name, seconds: v.seconds, event_count: v.count }))
        .sort((a, b) => b.seconds - a.seconds),
    );
  },
  appTrends: (days: number, topN = 8): Promise<AppTrendDay[]> =>
    delay(
      Array.from({ length: days }, (_, i) => {
        const date = utcDateStr(days - 1 - i);
        const byApp = new Map<string, number>();
        for (const e of generateDay(date).events) {
          byApp.set(e.app_name ?? 'Unknown', (byApp.get(e.app_name ?? 'Unknown') ?? 0) + e.duration_s);
        }
        return {
          date,
          by_app: [...byApp.entries()].sort((a, b) => b[1] - a[1]).slice(0, topN),
        };
      }),
    ),
  events: (date: string, search?: string, category?: string): Promise<Event[]> => {
    let rows = generateDay(date).events;
    if (category) rows = rows.filter((e) => e.category === category);
    if (search) {
      const q = search.toLowerCase();
      rows = rows.filter(
        (e) => e.summary.toLowerCase().includes(q) || (e.app_name ?? '').toLowerCase().includes(q),
      );
    }
    return delay([...rows].reverse());
  },
  categories: () => delay(Object.keys(SUMMARIES)),
  generateAIReport: async (date: string, _force = false): Promise<string> => {
    await sleep(700); // pretend to think
    return buildReport(date);
  },
  generateWeeklyReport: async (endDate?: string, _force = false): Promise<string> => {
    await sleep(900);
    return buildWeeklyReport(endDate ?? utcDateStr());
  },
  exportData: () => delay('~/Downloads/day-monitor-export-demo.json'),
  openDashboard: (): Promise<void> => {
    window.location.href = './dashboard.html';
    return Promise.resolve();
  },
  openSettings: (): Promise<void> => {
    window.location.href = './settings.html';
    return Promise.resolve();
  },
  saveApiKey: (_key: string) => delay(undefined),
  apiKeySet: () => delay(true),
  settings: () => delay({ ...demoSettings }),
  saveSettings: (_settings: Settings) => delay(undefined),
  costStats: (): Promise<CostStats> =>
    delay({
      today_usd: 0.11,
      today_calls: 924,
      today_input_tokens: 1_282_400,
      today_output_tokens: 46_180,
      month_usd: 2.37,
      month_calls: 19_866,
      projected_month_usd: 3.4,
      price_input_per_mtok: 0.08,
      price_output_per_mtok: 0.4,
    }),
};
