// Single source of truth for category colors (v2 activity-centric taxonomy).
// Legacy keys are kept so old cached data and the demo still render.
export const CATEGORY_COLORS: Record<string, string> = {
  coding: '#3b82f6',
  'ai-coding': '#8b5cf6',
  'code-review': '#6366f1',
  meeting: '#ef4444',
  communicating: '#a855f7',
  writing: '#65a30d',
  reading: '#84cc16',
  design: '#ec4899',
  data: '#0d9488',
  '3d': '#f97316',
  browsing: '#06b6d4',
  entertainment: '#d946ef',
  app: '#94a3b8',
  other: '#6b7280',
  // legacy (pre-v2) values
  slack: '#a855f7',
  wechat: '#22c55e',
  feishu: '#0ea5e9',
  email: '#f59e0b',
  browser: '#06b6d4',
  media: '#d946ef',
  system: '#94a3b8',
};

export function categoryColor(category: string): string {
  return CATEGORY_COLORS[category] ?? '#6b7280';
}
