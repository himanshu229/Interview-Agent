use chrono::Utc;
use rusqlite::{params, Connection};
use std::path::Path;

use crate::error::AppResult;
use crate::models::{ChatMessage, Conversation, Role};

/// SQLite-backed persistence for conversations and messages.
pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open(path: &Path) -> AppResult<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        let db = Database { conn };
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&self) -> AppResult<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS conversations (
                id         INTEGER PRIMARY KEY AUTOINCREMENT,
                title      TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS messages (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                conversation_id INTEGER NOT NULL,
                role            TEXT NOT NULL,
                content         TEXT NOT NULL,
                source          TEXT NOT NULL DEFAULT 'chat',
                created_at      TEXT NOT NULL,
                FOREIGN KEY (conversation_id)
                    REFERENCES conversations (id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_messages_conversation
                ON messages (conversation_id);
            "#,
        )?;
        Ok(())
    }

    pub fn create_conversation(&self, title: &str) -> AppResult<Conversation> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO conversations (title, created_at, updated_at) VALUES (?1, ?2, ?2)",
            params![title, now],
        )?;
        let id = self.conn.last_insert_rowid();
        Ok(Conversation {
            id,
            title: title.to_string(),
            created_at: now.clone(),
            updated_at: now,
        })
    }

    pub fn list_conversations(&self) -> AppResult<Vec<Conversation>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, created_at, updated_at
             FROM conversations ORDER BY updated_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Conversation {
                id: row.get(0)?,
                title: row.get(1)?,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
            })
        })?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn rename_conversation(&self, id: i64, title: &str) -> AppResult<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE conversations SET title = ?1, updated_at = ?2 WHERE id = ?3",
            params![title, now, id],
        )?;
        Ok(())
    }

    pub fn delete_conversation(&self, id: i64) -> AppResult<()> {
        self.conn
            .execute("DELETE FROM conversations WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn touch_conversation(&self, id: i64) -> AppResult<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE conversations SET updated_at = ?1 WHERE id = ?2",
            params![now, id],
        )?;
        Ok(())
    }

    pub fn insert_message(
        &self,
        conversation_id: i64,
        role: &Role,
        content: &str,
        source: &str,
    ) -> AppResult<ChatMessage> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO messages (conversation_id, role, content, source, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![conversation_id, role.as_str(), content, source, now],
        )?;
        let id = self.conn.last_insert_rowid();
        self.touch_conversation(conversation_id)?;
        Ok(ChatMessage {
            id,
            conversation_id,
            role: role.clone(),
            content: content.to_string(),
            source: source.to_string(),
            created_at: now,
        })
    }

    pub fn get_messages(&self, conversation_id: i64) -> AppResult<Vec<ChatMessage>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, conversation_id, role, content, source, created_at
             FROM messages WHERE conversation_id = ?1 ORDER BY id ASC",
        )?;
        let rows = stmt.query_map(params![conversation_id], |row| {
            let role: String = row.get(2)?;
            Ok(ChatMessage {
                id: row.get(0)?,
                conversation_id: row.get(1)?,
                role: Role::from_str(&role),
                content: row.get(3)?,
                source: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }
}
