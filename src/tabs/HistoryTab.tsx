import { useCallback, useEffect, useState } from "react";
import type { Conversation } from "@/types";
import {
  deleteConversation,
  listConversations,
  renameConversation,
} from "@/lib/api";

interface HistoryTabProps {
  onOpenConversation: (id: number) => void;
}

export default function HistoryTab({ onOpenConversation }: HistoryTabProps) {
  const [conversations, setConversations] = useState<Conversation[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(async () => {
    setLoading(true);
    try {
      setConversations(await listConversations());
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void load();
  }, [load]);

  const remove = async (id: number) => {
    await deleteConversation(id);
    await load();
  };

  const rename = async (conv: Conversation) => {
    const title = window.prompt("Rename conversation", conv.title);
    if (title && title.trim()) {
      await renameConversation(conv.id, title.trim());
      await load();
    }
  };

  return (
    <div className="history-tab">
      <div className="history-tab__header">
        <h2>Conversation history</h2>
        <button className="btn btn--ghost" onClick={load}>
          ⟳ Refresh
        </button>
      </div>

      {error && <div className="history-tab__error">⚠ {error}</div>}
      {loading ? (
        <p className="muted">Loading…</p>
      ) : conversations.length === 0 ? (
        <div className="empty-state">
          <h2>No conversations yet</h2>
          <p>Your saved chats will appear here.</p>
        </div>
      ) : (
        <ul className="history-list">
          {conversations.map((c) => (
            <li key={c.id} className="history-list__item">
              <button
                className="history-list__open"
                onClick={() => onOpenConversation(c.id)}
              >
                <span className="history-list__title">{c.title}</span>
                <span className="history-list__date">
                  {new Date(c.updatedAt).toLocaleString()}
                </span>
              </button>
              <div className="history-list__actions">
                <button className="icon-btn" title="Rename" onClick={() => rename(c)}>
                  ✎
                </button>
                <button
                  className="icon-btn icon-btn--danger"
                  title="Delete"
                  onClick={() => remove(c.id)}
                >
                  🗑
                </button>
              </div>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
