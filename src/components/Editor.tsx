import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { QuotesData, Quote, Category, AppSettings } from "../types";

interface EditorProps {
  settings: AppSettings;
  onDataChange: () => void;
}

export default function Editor({ settings, onDataChange }: EditorProps) {
  const [data, setData] = useState<QuotesData | null>(null);
  const [selectedCategory, setSelectedCategory] = useState<string>("全部");
  const [editingQuote, setEditingQuote] = useState<Quote | null>(null);
  const [newCategoryName, setNewCategoryName] = useState("");

  const countRef = useRef(0);

  useEffect(() => {
    invoke<QuotesData>("get_all_quotes").then((d) => {
      setData(d);
      countRef.current = d.quotes.length;
    });
    const timer = setInterval(() => {
      invoke<QuotesData>("get_all_quotes").then((d) => {
        if (d.quotes.length !== countRef.current) {
          countRef.current = d.quotes.length;
          setData(d);
          onDataChange();
        }
      });
    }, 2000);
    return () => clearInterval(timer);
  }, []);

  const refresh = useCallback(async () => {
    const d = await invoke<QuotesData>("get_all_quotes");
    setData(d);
    onDataChange();
  }, [onDataChange]);

  if (!data) return <div className="loading">載入中...</div>;

  const filteredQuotes =
    selectedCategory === "全部"
      ? data.quotes
      : data.quotes.filter((q) => q.category === selectedCategory);

  const displayedCategories = ["全部", ...data.categories.map((c) => c.name)];

  const handleAddCategory = async () => {
    const name = newCategoryName.trim();
    if (!name) return;
    const exists = data.categories.some((c) => c.name === name);
    if (exists) return;
    await invoke("add_category", { name });
    setNewCategoryName("");
    refresh();
  };

  const handleRenameCategory = async (oldName: string) => {
    const newName = window.prompt("新的分類名稱：", oldName);
    if (newName && newName.trim() && newName.trim() !== oldName) {
      await invoke("rename_category", {
        oldName,
        newName: newName.trim(),
      });
      if (selectedCategory === oldName) setSelectedCategory(newName.trim());
      refresh();
    }
  };

  const handleDeleteCategory = async (name: string) => {
    if (name === "未整理") return;
    const confirmed = window.confirm(`刪除分類「${name}」？\n該分類下的佳句將移至「未整理」。`);
    if (!confirmed) return;
    await invoke("delete_category", { name });
    if (selectedCategory === name) setSelectedCategory("全部");
    refresh();
  };

  const handleToggleQuote = async (q: Quote) => {
    q.enabled = !q.enabled;
    await invoke("update_quote", { quote: q });
    refresh();
  };

  const handleDeleteQuote = async (id: number) => {
    if (!window.confirm("確定刪除此佳句？")) return;
    await invoke("delete_quote", { id });
    refresh();
  };

  const handleResetCount = async (id: number) => {
    await invoke("reset_quote_count", { id });
    refresh();
  };

  const handleSaveQuote = async () => {
    if (!editingQuote) return;
    await invoke("update_quote", { quote: editingQuote });
    setEditingQuote(null);
    refresh();
  };

  const handleAddQuote = async () => {
    const cat =
      selectedCategory === "全部"
        ? data.categories[0]?.name || "未整理"
        : selectedCategory;
    const content = window.prompt("輸入佳句內容：");
    if (!content) return;
    const author = window.prompt("作者（選填）：");
    await invoke("add_quote", {
      content,
      author: author || null,
      category: cat,
    });
    refresh();
  };

  return (
    <div className="editor-container">
      <div className="editor-sidebar">
        <h3>分類主題</h3>
        <ul className="category-list">
          {displayedCategories.map((cat) => (
            <li
              key={cat}
              className={`category-item ${selectedCategory === cat ? "active" : ""}`}
              onClick={() => setSelectedCategory(cat)}
            >
              {cat}
              {cat !== "全部" && (
                <span className="category-count">
                  {data.quotes.filter((q) => q.category === cat).length}
                </span>
              )}
            </li>
          ))}
        </ul>
        <div className="add-category">
          <input
            type="text"
            placeholder="新分類名稱"
            value={newCategoryName}
            onChange={(e) => setNewCategoryName(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && handleAddCategory()}
          />
          <button onClick={handleAddCategory} className="btn-small">
            +
          </button>
        </div>
      </div>

      <div className="editor-main">
        <div className="editor-toolbar">
          <h3>
            {selectedCategory}
            {selectedCategory !== "全部" && (
              <>
                <button
                  className="btn-text"
                  onClick={() => handleRenameCategory(selectedCategory)}
                  title="重新命名"
                >
                  ✏️
                </button>
                <button
                  className="btn-text"
                  onClick={() => handleDeleteCategory(selectedCategory)}
                  title="刪除分類"
                >
                  🗑️
                </button>
              </>
            )}
          </h3>
          <button onClick={handleAddQuote} className="btn-primary">
            + 新增佳句
          </button>
        </div>

        <div className="quote-list">
          {filteredQuotes.length === 0 && (
            <div className="empty-state">尚無佳句，點擊上方按鈕新增</div>
          )}
          {filteredQuotes.map((q) => (
            <div key={q.id} className={`quote-card ${q.enabled ? "" : "disabled"}`}>
              <div className="quote-content">
                <p className="quote-text">{q.content}</p>
                {q.author && <p className="quote-author">— {q.author}</p>}
                <div className="quote-meta">
                  <span className="tag">{q.category}</span>
                  <span className="show-count">已顯示 {q.show_count} 次</span>
                  <span className="created-at">{q.created_at}</span>
                </div>
              </div>
              <div className="quote-actions">
                <label className="toggle-label">
                  <input
                    type="checkbox"
                    checked={q.enabled}
                    onChange={() => handleToggleQuote(q)}
                  />
                  <span className="toggle-text">啟用</span>
                </label>
                <button
                  className="btn-small"
                  onClick={() => setEditingQuote({ ...q })}
                >
                  編輯
                </button>
                <button
                  className="btn-small"
                  onClick={() => handleResetCount(q.id)}
                  title="重設計數"
                >
                  重置
                </button>
                <button
                  className="btn-small btn-danger"
                  onClick={() => handleDeleteQuote(q.id)}
                >
                  刪除
                </button>
              </div>
            </div>
          ))}
        </div>
      </div>

      {editingQuote && (
        <div className="modal-overlay" onClick={() => setEditingQuote(null)}>
          <div className="modal" onClick={(e) => e.stopPropagation()}>
            <h3>編輯佳句</h3>
            <label>
              內容
              <textarea
                value={editingQuote.content}
                onChange={(e) =>
                  setEditingQuote({ ...editingQuote, content: e.target.value })
                }
                rows={4}
              />
            </label>
            <label>
              作者 / 出處
              <input
                type="text"
                value={editingQuote.author || ""}
                onChange={(e) =>
                  setEditingQuote({
                    ...editingQuote,
                    author: e.target.value || null,
                  })
                }
              />
            </label>
            <label>
              分類
              <select
                value={editingQuote.category}
                onChange={(e) =>
                  setEditingQuote({
                    ...editingQuote,
                    category: e.target.value,
                  })
                }
              >
                {data.categories.map((c) => (
                  <option key={c.name} value={c.name}>
                    {c.name}
                  </option>
                ))}
              </select>
            </label>
            <div className="modal-actions">
              <button className="btn-primary" onClick={handleSaveQuote}>
                儲存
              </button>
              <button onClick={() => setEditingQuote(null)}>取消</button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
