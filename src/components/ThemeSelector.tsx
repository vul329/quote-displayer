import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AppSettings, Category } from "../types";

interface ThemeSelectorProps {
  settings: AppSettings;
  onSettingsChange: (s: AppSettings) => void;
}

export default function ThemeSelector({
  settings,
  onSettingsChange,
}: ThemeSelectorProps) {
  const [categories, setCategories] = useState<Category[]>([]);

  useEffect(() => {
    invoke<{ categories: Category[]; quotes: unknown[] }>("get_all_quotes").then(
      (d) => setCategories(d.categories)
    );
  }, []);

  const toggleCategory = async (name: string) => {
    const current = settings.active_categories;
    const next = current.includes(name)
      ? current.filter((c) => c !== name)
      : [...current, name];
    const newSettings = { ...settings, active_categories: next };
    onSettingsChange(newSettings);
    await invoke("save_settings", { settings: newSettings });
  };

  return (
    <div className="theme-selector">
      <h3>選擇要輪播的主題</h3>
      <p className="hint">
        勾選的主題才會出現在隨機輪播中。未勾選者將被跳過。
      </p>
      <div className="theme-list">
        {categories.map((cat) => (
          <label key={cat.name} className="theme-item">
            <input
              type="checkbox"
              checked={settings.active_categories.includes(cat.name)}
              onChange={() => toggleCategory(cat.name)}
            />
            <span>{cat.name}</span>
          </label>
        ))}
      </div>
      {categories.length === 0 && (
        <div className="empty-state">尚無分類，請先在編輯佳句中新增</div>
      )}
    </div>
  );
}
