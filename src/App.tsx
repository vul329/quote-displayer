import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import Editor from "./components/Editor";
import ThemeSelector from "./components/ThemeSelector";
import ScheduleSettings from "./components/ScheduleSettings";
import SettingsPanel from "./components/SettingsPanel";
import PopupDisplay from "./components/PopupDisplay";
import type { AppSettings, Stats } from "./types";

type View = "editor" | "select-theme" | "schedule" | "settings";

function App() {
  const [isPopup, setIsPopup] = useState(false);
  const [view, setView] = useState<View>("editor");
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [stats, setStats] = useState<Stats | null>(null);

  const loadData = useCallback(async () => {
    try {
      const s = await invoke<AppSettings>("get_settings");
      setSettings(s);
      const st = await invoke<Stats>("get_stats");
      setStats(st);
    } catch (e) {
      console.error(e);
    }
  }, []);

  useEffect(() => {
    const win = getCurrentWindow();
    if (win.label.startsWith("popup-")) {
      setIsPopup(true);
      return;
    }
    loadData();

    const unlistenNav = listen<string>("navigate", (e) => {
      setView(e.payload as View);
    });
    const unlistenDark = listen<boolean>("darkmode-changed", (e) => {
      setSettings((prev) => (prev ? { ...prev, dark_mode: e.payload } : prev));
    });
    const unlistenOntop = listen<boolean>("ontop-changed", (e) => {
      setSettings((prev) =>
        prev ? { ...prev, always_on_top: e.payload } : prev
      );
    });
    const unlistenWeight = listen<boolean>("weight-changed", (e) => {
      setSettings((prev) => (prev ? { ...prev, weight_mode: e.payload } : prev));
    });
    const unlistenCollect = listen("quote-collected", loadData);

    return () => {
      unlistenNav.then((f) => f());
      unlistenDark.then((f) => f());
      unlistenOntop.then((f) => f());
      unlistenWeight.then((f) => f());
      unlistenCollect.then((f) => f());
    };
  }, [loadData]);

  const handleClose = async () => {
    const win = getCurrentWindow();
    await win.hide();
  };

  if (isPopup) {
    return <PopupDisplay />;
  }

  if (!settings) return <div className="loading">載入中...</div>;

  const isDark = settings.dark_mode;

  return (
    <div className={`app ${isDark ? "dark" : "light"}`}>
      <header className="app-header">
        <div className="app-title">
          <h1>佳句隨機顯示器</h1>
          {stats && (
            <span className="stats-badge">
              共 {stats.total_displayed} 次 | 今日 {stats.today_displayed} 次
            </span>
          )}
        </div>
        <div className="header-actions">
          <span className={`status-dot ${settings.paused ? "paused" : "active"}`} />
          <span className="status-text">
            {settings.paused ? "已暫停" : "啟用中"}
          </span>
          <button className="btn-close" onClick={handleClose} title="縮小至系統列">
            _
          </button>
        </div>
      </header>

      <nav className="app-nav">
        <button
          className={`nav-btn ${view === "editor" ? "active" : ""}`}
          onClick={() => setView("editor")}
        >
          編輯佳句
        </button>
        <button
          className={`nav-btn ${view === "select-theme" ? "active" : ""}`}
          onClick={() => setView("select-theme")}
        >
          選擇主題
        </button>
        <button
          className={`nav-btn ${view === "schedule" ? "active" : ""}`}
          onClick={() => setView("schedule")}
        >
          排程設定
        </button>
        <button
          className={`nav-btn ${view === "settings" ? "active" : ""}`}
          onClick={() => setView("settings")}
        >
          其他設定
        </button>
      </nav>

      <main className="app-main">
        {view === "editor" && (
          <Editor settings={settings} onDataChange={loadData} />
        )}
        {view === "select-theme" && (
          <ThemeSelector settings={settings} onSettingsChange={setSettings} />
        )}
        {view === "schedule" && (
          <ScheduleSettings settings={settings} onSettingsChange={setSettings} />
        )}
        {view === "settings" && (
          <SettingsPanel settings={settings} onSettingsChange={setSettings} />
        )}
      </main>
    </div>
  );
}

export default App;
