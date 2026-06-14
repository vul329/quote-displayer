import { useRef, useCallback, useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AppSettings, DisplayPosition, AnimationSpeed } from "../types";

interface SettingsPanelProps {
  settings: AppSettings;
  onSettingsChange: (s: AppSettings) => void;
}

const positions: { label: string; value: DisplayPosition }[] = [
  { label: "右下角", value: "BottomRight" },
  { label: "左下角", value: "BottomLeft" },
  { label: "右上角", value: "TopRight" },
  { label: "置中", value: "Center" },
  { label: "隨機", value: "Random" },
];

const animations: { label: string; value: AnimationSpeed }[] = [
  { label: "關閉", value: "Off" },
  { label: "快速", value: "Fast" },
  { label: "正常", value: "Normal" },
  { label: "慢速", value: "Slow" },
];

export default function SettingsPanel({
  settings,
  onSettingsChange,
}: SettingsPanelProps) {
  const hotkeyTimer = useRef<number | null>(null);
  const [monitorCount, setMonitorCount] = useState(1);

  useEffect(() => {
    invoke<number>("get_monitor_count")
      .then(setMonitorCount)
      .catch(() => setMonitorCount(1));
  }, []);

  const scheduleReload = useCallback(() => {
    if (hotkeyTimer.current) clearTimeout(hotkeyTimer.current);
    hotkeyTimer.current = window.setTimeout(async () => {
      try {
        await invoke("reload_shortcuts");
      } catch (e) {
        console.error("熱鍵註冊失敗:", e);
      }
    }, 500);
  }, []);

  const update = async (partial: Partial<AppSettings>) => {
    const newSettings = { ...settings, ...partial };
    onSettingsChange(newSettings);
    await invoke("save_settings", { settings: newSettings });
    if ("show_hotkey" in partial || "collect_hotkey" in partial) {
      scheduleReload();
    }
  };

  return (
    <div className="settings-panel">
      <h3>其他設定</h3>

      <section className="settings-section">
        <h4>彈窗位置</h4>
        <div className="radio-group">
          {positions.map((p) => (
            <label key={p.value} className="radio-label">
              <input
                type="radio"
                name="position"
                checked={settings.display_position === p.value}
                onChange={() => update({ display_position: p.value })}
              />
              <span>{p.label}</span>
            </label>
          ))}
        </div>
      </section>

      <section className="settings-section">
        <h4>螢幕設定</h4>
        <div className="radio-group monitor-group">
          <label className="radio-label">
            <input
              type="radio"
              name="monitor"
              checked={settings.preferred_screen === null}
              onChange={() => update({ preferred_screen: null })}
            />
            <span>自動（主要螢幕）</span>
          </label>
          {Array.from({ length: monitorCount }, (_, i) => (
            <label key={i} className="radio-label">
              <input
                type="radio"
                name="monitor"
                checked={settings.preferred_screen === i}
                onChange={() => update({ preferred_screen: i })}
              />
              <span>螢幕 {i + 1}</span>
            </label>
          ))}
        </div>
        <span className="input-hint">選取後按快捷鍵測試彈窗位置</span>
      </section>

      <section className="settings-section">
        <h4>彈窗動畫速度</h4>
        <div className="radio-group">
          {animations.map((a) => (
            <label key={a.value} className="radio-label">
              <input
                type="radio"
                name="animation"
                checked={settings.animation_speed === a.value}
                onChange={() => update({ animation_speed: a.value })}
              />
              <span>{a.label}</span>
            </label>
          ))}
        </div>
      </section>

      <section className="settings-section">
        <h4>彈窗顯示時間</h4>
        <input
          type="number"
          min={3}
          max={60}
          value={settings.popup_duration_secs}
          onChange={(e) =>
            update({ popup_duration_secs: parseInt(e.target.value) || 8 })
          }
          className="input-number"
        />
        <span className="input-hint">秒（3～60）</span>
      </section>

      <section className="settings-section">
        <h4>最近不重複則數</h4>
        <input
          type="number"
          min={1}
          max={100}
          value={settings.no_repeat_count}
          onChange={(e) =>
            update({ no_repeat_count: parseInt(e.target.value) || 10 })
          }
          className="input-number"
        />
        <span className="input-hint">則（1～100）</span>
      </section>

      <section className="settings-section">
        <h4>字體設定</h4>
        <div className="font-settings">
          <div>
            <label>字體大小</label>
            <input
              type="number"
              min={14}
              max={48}
              value={settings.font_size}
              onChange={(e) =>
                update({ font_size: parseInt(e.target.value) || 24 })
              }
              className="input-number"
            />
            <span className="input-hint">pt</span>
          </div>
        </div>
      </section>

      <section className="settings-section">
        <h4>全域快捷鍵</h4>
        <div className="hotkey-settings">
          <div>
            <label>顯示佳句</label>
            <input
              type="text"
              value={settings.show_hotkey}
              onChange={(e) => update({ show_hotkey: e.target.value })}
              className="input-text"
            />
          </div>
          <div>
            <label>收藏文字</label>
            <input
              type="text"
              value={settings.collect_hotkey}
              onChange={(e) => update({ collect_hotkey: e.target.value })}
              className="input-text"
            />
          </div>
        </div>
      </section>

      <section className="settings-section">
        <h4>安靜時段</h4>
        <div className="quiet-hours">
          <div>
            <label>開始</label>
            <input
              type="time"
              value={settings.quiet_hours_start || "23:00"}
              onChange={(e) => update({ quiet_hours_start: e.target.value || null })}
            />
          </div>
          <div>
            <label>結束</label>
            <input
              type="time"
              value={settings.quiet_hours_end || "07:00"}
              onChange={(e) => update({ quiet_hours_end: e.target.value || null })}
            />
          </div>
        </div>
      </section>

      <section className="settings-section">
        <h4>每日顯示上限</h4>
        <input
          type="number"
          min={0}
          max={999}
          value={settings.daily_limit || 0}
          onChange={(e) => {
            const v = parseInt(e.target.value);
            update({ daily_limit: v > 0 ? v : null });
          }}
          className="input-number"
        />
        <span className="input-hint">0 = 無上限</span>
      </section>
    </div>
  );
}
