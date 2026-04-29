import { useEffect, useState } from 'react';
import { enable, disable, isEnabled } from '@tauri-apps/plugin-autostart';
import { api } from '../lib/api';
import type { Settings as SettingsT, CostStats } from '../lib/types';

const DEFAULTS: SettingsT = {
  interval_secs: 20,
  max_image_width: 640,
  retention_days: 30,
  dedup_threshold: 12,
  monthly_budget_usd: 0,
  privacy_accepted: false,
  active_hour_start: 0,
  active_hour_end: 24,
  pause_on_battery_below: 0,
};

function fmtUsd(v: number) {
  return `$${v.toFixed(3)}`;
}

export function Settings() {
  const [apiKey, setApiKey] = useState('');
  const [autostart, setAutostart] = useState(false);
  const [legacyPresent, setLegacyPresent] = useState(false);
  const [keyAlreadySet, setKeyAlreadySet] = useState(false);
  const [s, setS] = useState<SettingsT>(DEFAULTS);
  const [cost, setCost] = useState<CostStats | null>(null);
  const [savedFlash, setSavedFlash] = useState(false);

  useEffect(() => {
    isEnabled().then(setAutostart).catch(() => {});
    api.checkLegacyLaunchd().then(setLegacyPresent);
    api.apiKeySet().then(setKeyAlreadySet);
    api.settings().then(setS);
    api.costStats().then(setCost).catch(() => {});
  }, []);

  const save = async () => {
    if (apiKey) {
      await api.saveApiKey(apiKey);
      setKeyAlreadySet(true);
      setApiKey('');
    }
    await api.saveSettings(s);
    setSavedFlash(true);
    setTimeout(() => setSavedFlash(false), 1500);
  };

  const toggleAutostart = async () => {
    try {
      if (autostart) await disable();
      else await enable();
      setAutostart(!autostart);
    } catch (e) {
      console.error(e);
    }
  };

  const removeLegacy = async () => {
    await api.removeLegacyLaunchd();
    setLegacyPresent(false);
  };

  return (
    <div className="bg-white text-gray-900 h-screen overflow-auto">
      <div className="p-6 flex flex-col gap-5 text-sm max-w-2xl">
        <h1 className="text-lg font-semibold">Settings</h1>

        {/* API Key */}
        <section>
          <label className="block text-xs text-gray-500 mb-1">
            Anthropic API Key{' '}
            {keyAlreadySet && (
              <span className="text-green-600">(已配置)</span>
            )}
          </label>
          <input
            type="password"
            placeholder={keyAlreadySet ? '输入新值替换' : 'sk-ant-...'}
            value={apiKey}
            onChange={(e) => setApiKey(e.target.value)}
            className="w-full border border-gray-300 rounded px-2 py-1.5 font-mono text-xs"
          />
          <p className="text-xs text-gray-500 mt-1">
            存储在 <code>~/.day-monitor/.env</code>。
          </p>
        </section>

        {/* Cost stats */}
        {cost && (
          <section className="border border-gray-200 rounded p-3 bg-gray-50">
            <h2 className="text-xs font-semibold text-gray-700 mb-2">
              📊 Token 消耗与费用
            </h2>
            <div className="grid grid-cols-3 gap-3 text-xs">
              <div>
                <div className="text-gray-500">今日</div>
                <div className="font-bold text-base">{fmtUsd(cost.today_usd)}</div>
                <div className="text-gray-500">{cost.today_calls} calls</div>
              </div>
              <div>
                <div className="text-gray-500">本月已花</div>
                <div className="font-bold text-base">{fmtUsd(cost.month_usd)}</div>
                <div className="text-gray-500">{cost.month_calls} calls</div>
              </div>
              <div>
                <div className="text-gray-500">本月预测</div>
                <div className="font-bold text-base">{fmtUsd(cost.projected_month_usd)}</div>
                <div className="text-gray-500">按当前速率</div>
              </div>
            </div>
            <div className="text-[10px] text-gray-400 mt-2">
              基于 Claude Haiku 价格：input ${cost.price_input_per_mtok}/Mtok，
              output ${cost.price_output_per_mtok}/Mtok
            </div>
          </section>
        )}

        {/* Capture interval */}
        <section>
          <label className="block text-xs text-gray-500 mb-1">
            采集间隔: <strong>{s.interval_secs}s</strong>{' '}
            <span className="text-gray-400">
              ({Math.round(60 / s.interval_secs * 60)} 次/小时)
            </span>
          </label>
          <input
            type="range"
            min={10}
            max={120}
            step={5}
            value={s.interval_secs}
            onChange={(e) => setS({ ...s, interval_secs: +e.target.value })}
            className="w-full"
          />
          <div className="flex justify-between text-[10px] text-gray-400">
            <span>10s 高频</span>
            <span>120s 省钱</span>
          </div>
        </section>

        {/* Image width */}
        <section>
          <label className="block text-xs text-gray-500 mb-1">
            截图分辨率: <strong>{s.max_image_width}px</strong>
          </label>
          <select
            value={s.max_image_width}
            onChange={(e) => setS({ ...s, max_image_width: +e.target.value })}
            className="border border-gray-300 rounded px-2 py-1.5 text-sm"
          >
            <option value={320}>320px (最省钱，识别准度低)</option>
            <option value={640}>640px (推荐：~$1/天)</option>
            <option value={1280}>1280px (高准度，~$2/天)</option>
            <option value={0}>原图 (最贵)</option>
          </select>
        </section>

        {/* Dedup threshold */}
        <section>
          <label className="block text-xs text-gray-500 mb-1">
            去重激进度: <strong>{s.dedup_threshold}</strong>
          </label>
          <input
            type="range"
            min={0}
            max={20}
            value={s.dedup_threshold}
            onChange={(e) => setS({ ...s, dedup_threshold: +e.target.value })}
            className="w-full"
          />
          <div className="flex justify-between text-[10px] text-gray-400">
            <span>0 几乎不跳过</span>
            <span>12 推荐</span>
            <span>20 激进省钱</span>
          </div>
        </section>

        {/* Retention */}
        <section>
          <label className="block text-xs text-gray-500 mb-1">
            数据保留天数:{' '}
            <strong>{s.retention_days === 0 ? '永久' : `${s.retention_days} 天`}</strong>
          </label>
          <select
            value={s.retention_days}
            onChange={(e) => setS({ ...s, retention_days: +e.target.value })}
            className="border border-gray-300 rounded px-2 py-1.5 text-sm"
          >
            <option value={7}>7 天</option>
            <option value={30}>30 天</option>
            <option value={90}>90 天</option>
            <option value={365}>1 年</option>
            <option value={0}>永久</option>
          </select>
        </section>

        {/* Budget */}
        <section>
          <label className="block text-xs text-gray-500 mb-1">
            月度预算上限 (USD):{' '}
            <strong>
              {s.monthly_budget_usd === 0 ? '无限制' : `$${s.monthly_budget_usd}`}
            </strong>
          </label>
          <input
            type="number"
            min={0}
            step={1}
            value={s.monthly_budget_usd}
            onChange={(e) => setS({ ...s, monthly_budget_usd: +e.target.value })}
            className="w-full border border-gray-300 rounded px-2 py-1.5"
          />
          <p className="text-[10px] text-gray-400 mt-1">
            超过预算后自动暂停（事件继续记录但不再调 API）。0 = 不限制。
          </p>
        </section>

        {/* Active hours */}
        <section>
          <label className="block text-xs text-gray-500 mb-1">
            采集时段:{' '}
            <strong>
              {s.active_hour_start === s.active_hour_end
                ? '全天'
                : `${s.active_hour_start}:00 – ${s.active_hour_end}:00`}
            </strong>
          </label>
          <div className="flex items-center gap-2">
            <input
              type="number"
              min={0}
              max={24}
              value={s.active_hour_start}
              onChange={(e) =>
                setS({ ...s, active_hour_start: Math.max(0, Math.min(24, +e.target.value)) })
              }
              className="w-16 border border-gray-300 rounded px-2 py-1"
            />
            <span className="text-gray-400">→</span>
            <input
              type="number"
              min={0}
              max={24}
              value={s.active_hour_end}
              onChange={(e) =>
                setS({ ...s, active_hour_end: Math.max(0, Math.min(24, +e.target.value)) })
              }
              className="w-16 border border-gray-300 rounded px-2 py-1"
            />
            <span className="text-[10px] text-gray-400">
              （0–24 小时制；start = end 表示全天）
            </span>
          </div>
        </section>

        {/* Battery */}
        <section>
          <label className="block text-xs text-gray-500 mb-1">
            电池低于此电量时自动暂停（笔记本）:{' '}
            <strong>
              {s.pause_on_battery_below === 0 ? '不限' : `${s.pause_on_battery_below}%`}
            </strong>
          </label>
          <input
            type="range"
            min={0}
            max={50}
            step={5}
            value={s.pause_on_battery_below}
            onChange={(e) => setS({ ...s, pause_on_battery_below: +e.target.value })}
            className="w-full"
          />
          <div className="flex justify-between text-[10px] text-gray-400">
            <span>0% 不暂停</span>
            <span>50%</span>
          </div>
        </section>

        {/* Autostart */}
        <label className="flex items-center gap-2 cursor-pointer">
          <input type="checkbox" checked={autostart} onChange={toggleAutostart} />
          <span>登录时自动启动 Day Monitor</span>
        </label>

        {/* Legacy launchd */}
        {legacyPresent && (
          <div className="border border-yellow-300 bg-yellow-50 rounded p-3">
            <p className="text-xs">
              检测到旧版 <code>com.daymonitor.plist</code>。建议删除以避免重复采集。
            </p>
            <button
              onClick={removeLegacy}
              className="mt-2 px-3 py-1 text-xs bg-yellow-600 text-white rounded hover:bg-yellow-700"
            >
              卸载旧 launchd
            </button>
          </div>
        )}

        <div className="flex justify-end gap-2 pt-4 border-t">
          <button
            onClick={save}
            className="px-4 py-1.5 bg-gray-900 text-white rounded hover:bg-gray-700"
          >
            {savedFlash ? '已保存 ✓' : '保存'}
          </button>
        </div>
      </div>
    </div>
  );
}
