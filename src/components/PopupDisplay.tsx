import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Quote, AppSettings } from "../types";

export default function PopupDisplay() {
  const [quote, setQuote] = useState<Quote | null>(null);
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [visible, setVisible] = useState(false);
  const timerRef = useRef<number | null>(null);

  const close = () => { invoke("popup_close"); };

  useEffect(() => {
    document.documentElement.style.backgroundColor = "transparent";
    document.body.style.backgroundColor = "transparent";
    invoke<AppSettings>("get_settings").then(setSettings);
    invoke<Quote | null>("get_popup_quote").then((q) => {
      if (q) setQuote(q);
    });
    return () => {
      document.documentElement.style.backgroundColor = "";
      document.body.style.backgroundColor = "";
    };
  }, []);

  useEffect(() => {
    if (!quote || !settings) return;
    const enterTimer = setTimeout(() => setVisible(true), 50);
    const duration = settings.popup_duration_secs * 1000;
    timerRef.current = window.setTimeout(close, duration);
    return () => {
      clearTimeout(enterTimer);
      if (timerRef.current) clearTimeout(timerRef.current);
    };
  }, [quote, settings]);

  if (!quote || !settings) return null;

  const isDark = settings.dark_mode;
  const showCount = quote.show_count;

  return (
    <div className={`popup-overlay ${isDark ? "dark" : "light"} ${visible ? "visible" : ""}`} onMouseDown={close}>
      <span className="popup-close" onMouseDown={(e) => { e.stopPropagation(); close(); }}>✕</span>
      <div className="popup-card" onMouseDown={(e) => e.stopPropagation()}>
        <div className="popup-quote-text">{quote.content}</div>
        {quote.author && (
          <div className="popup-author">— {quote.author}</div>
        )}
        <div className="popup-footer">
          <span className="popup-tag">{quote.category}</span>
          <span className="popup-count">#{showCount}</span>
        </div>
      </div>
    </div>
  );
}
