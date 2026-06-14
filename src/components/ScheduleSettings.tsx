import { invoke } from "@tauri-apps/api/core";
import type { AppSettings, ScheduleRule } from "../types";

interface ScheduleProps {
  settings: AppSettings;
  onSettingsChange: (s: AppSettings) => void;
}

export default function ScheduleSettings({
  settings,
  onSettingsChange,
}: ScheduleProps) {
  const updateRules = async (rules: ScheduleRule[]) => {
    const newSettings = { ...settings, schedule_rules: rules };
    onSettingsChange(newSettings);
    await invoke("save_settings", { settings: newSettings });
  };

  const addInterval = () => {
    const minutes = prompt("間隔分鐘數：", "30");
    if (!minutes) return;
    const m = parseInt(minutes);
    if (isNaN(m) || m < 1) return;
    updateRules([
      ...settings.schedule_rules,
      { type: "Interval", minutes: m },
    ]);
  };

  const addRandomRange = () => {
    const min = prompt("最小間隔（分鐘）：", "3");
    if (!min) return;
    const max = prompt("最大間隔（分鐘）：", "8");
    if (!max) return;
    const minN = parseInt(min);
    const maxN = parseInt(max);
    if (isNaN(minN) || isNaN(maxN) || minN < 1 || maxN < minN) return;
    updateRules([
      ...settings.schedule_rules,
      { type: "RandomRange", min: minN, max: maxN },
    ]);
  };

  const addFixedTime = () => {
    const time = prompt("固定時間（HH:MM）：", "09:00");
    if (!time) return;
    if (!/^\d{2}:\d{2}$/.test(time)) {
      alert("格式錯誤，請輸入 HH:MM");
      return;
    }
    const existing = settings.schedule_rules.find(
      (r): r is ScheduleRule & { type: "FixedTime" } => r.type === "FixedTime"
    );
    if (existing) {
      existing.times = [...existing.times, time].sort();
      updateRules([...settings.schedule_rules]);
    } else {
      updateRules([
        ...settings.schedule_rules,
        { type: "FixedTime", times: [time] },
      ]);
    }
  };

  const removeFixedTime = (ruleIdx: number, timeIdx: number) => {
    const rules = [...settings.schedule_rules];
    const rule = rules[ruleIdx];
    if (rule.type === "FixedTime") {
      rule.times = rule.times.filter((_, i) => i !== timeIdx);
      if (rule.times.length === 0) {
        rules.splice(ruleIdx, 1);
      }
    }
    updateRules(rules);
  };

  const removeRule = (idx: number) => {
    const rules = settings.schedule_rules.filter((_, i) => i !== idx);
    updateRules(rules);
  };

  const renderRule = (rule: ScheduleRule, idx: number) => {
    switch (rule.type) {
      case "Interval":
        return (
          <div key={idx} className="rule-card">
            <span>⏱️ 固定間隔：每 {rule.minutes} 分鐘一則</span>
            <button className="btn-small btn-danger" onClick={() => removeRule(idx)}>
              移除
            </button>
          </div>
        );
      case "RandomRange":
        return (
          <div key={idx} className="rule-card">
            <span>
              🎲 隨機區間：{rule.min} ~ {rule.max} 分鐘
            </span>
            <button className="btn-small btn-danger" onClick={() => removeRule(idx)}>
              移除
            </button>
          </div>
        );
      case "FixedTime":
        return (
          <div key={idx} className="rule-card">
            <div className="rule-header">
              <span>⏰ 固定時間</span>
              <button
                className="btn-small btn-danger"
                onClick={() => removeRule(idx)}
              >
                移除全部
              </button>
            </div>
            <div className="times-list">
              {(rule as ScheduleRule & { times: string[] }).times.map(
                (t, ti) => (
                  <span key={ti} className="time-chip">
                    {t}
                    <button
                      className="chip-remove"
                      onClick={() => removeFixedTime(idx, ti)}
                    >
                      ×
                    </button>
                  </span>
                )
              )}
            </div>
          </div>
        );
    }
  };

  return (
    <div className="schedule-settings">
      <h3>彈窗頻率設定</h3>
      <p className="hint">可複數設定，多種模式可並存。</p>

      <div className="add-rules">
        <button className="btn-secondary" onClick={addInterval}>
          + 固定間隔
        </button>
        <button className="btn-secondary" onClick={addRandomRange}>
          + 隨機區間
        </button>
        <button className="btn-secondary" onClick={addFixedTime}>
          + 固定時間
        </button>
      </div>

      <div className="rules-list">
        {settings.schedule_rules.length === 0 && (
          <div className="empty-state">尚未設定任何排程規則</div>
        )}
        {settings.schedule_rules.map((rule, idx) => renderRule(rule, idx))}
      </div>
    </div>
  );
}
