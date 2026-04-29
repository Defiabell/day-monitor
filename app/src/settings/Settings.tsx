import { useEffect, useState } from 'react';
import { enable, disable, isEnabled } from '@tauri-apps/plugin-autostart';
import { api } from '../lib/api';

export function Settings() {
  const [apiKey, setApiKey] = useState('');
  const [autostart, setAutostart] = useState(false);
  const [legacyPresent, setLegacyPresent] = useState(false);
  const [keyAlreadySet, setKeyAlreadySet] = useState(false);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    isEnabled().then(setAutostart).catch(() => {});
    api.checkLegacyLaunchd().then(setLegacyPresent);
    api.apiKeySet().then(setKeyAlreadySet);
  }, []);

  const save = async () => {
    if (apiKey) {
      await api.saveApiKey(apiKey);
      setKeyAlreadySet(true);
      setApiKey('');
    }
    setSaved(true);
    setTimeout(() => setSaved(false), 1500);
  };

  const toggleAutostart = async () => {
    try {
      if (autostart) {
        await disable();
        setAutostart(false);
      } else {
        await enable();
        setAutostart(true);
      }
    } catch (e) {
      console.error(e);
    }
  };

  const removeLegacy = async () => {
    await api.removeLegacyLaunchd();
    setLegacyPresent(false);
  };

  return (
    <div className="p-6 bg-white text-gray-900 h-screen flex flex-col gap-5 text-sm">
      <h1 className="text-lg font-semibold">Settings</h1>

      <div>
        <label className="block text-xs text-gray-500 mb-1">
          Anthropic API Key{' '}
          {keyAlreadySet && (
            <span className="text-green-600">(already configured)</span>
          )}
        </label>
        <input
          type="password"
          placeholder={keyAlreadySet ? 'Enter to replace' : 'sk-ant-...'}
          value={apiKey}
          onChange={(e) => setApiKey(e.target.value)}
          className="w-full border border-gray-300 rounded px-2 py-1.5 font-mono text-xs"
        />
        <p className="text-xs text-gray-500 mt-1">
          Stored at <code>~/.day-monitor/.env</code>
        </p>
      </div>

      <label className="flex items-center gap-2 cursor-pointer">
        <input
          type="checkbox"
          checked={autostart}
          onChange={toggleAutostart}
        />
        <span>Start Day Monitor on login</span>
      </label>

      {legacyPresent && (
        <div className="border border-yellow-300 bg-yellow-50 rounded p-3">
          <p className="text-xs">
            Detected legacy <code>com.daymonitor.plist</code>. Remove to avoid
            duplicate monitors.
          </p>
          <button
            onClick={removeLegacy}
            className="mt-2 px-3 py-1 text-xs bg-yellow-600 text-white rounded hover:bg-yellow-700"
          >
            Remove legacy launchd
          </button>
        </div>
      )}

      <div className="mt-auto flex justify-end gap-2">
        <button
          onClick={save}
          className="px-4 py-1.5 bg-gray-900 text-white rounded hover:bg-gray-700"
        >
          {saved ? 'Saved ✓' : 'Save'}
        </button>
      </div>
    </div>
  );
}
