use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use rusqlite::Connection;
use tauri::{AppHandle, Manager};

pub struct AppState {
    pub db: Mutex<Connection>,
    pub db_path: PathBuf,
}

pub fn open_database(app: &AppHandle) -> Result<AppState, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("failed to resolve app data directory: {error}"))?;

    fs::create_dir_all(&app_data_dir)
        .map_err(|error| format!("failed to create app data directory: {error}"))?;

    let db_path = app_data_dir.join("readnotes.sqlite");
    let connection =
        Connection::open(&db_path).map_err(|error| format!("failed to open database: {error}"))?;

    connection
        .execute_batch(
            "
            PRAGMA foreign_keys = ON;
            PRAGMA journal_mode = WAL;
            PRAGMA user_version = 1;
            ",
        )
        .map_err(|error| format!("failed to configure database: {error}"))?;

    run_migrations(&connection)?;

    Ok(AppState {
        db: Mutex::new(connection),
        db_path,
    })
}

fn run_migrations(connection: &Connection) -> Result<(), String> {
    connection
        .execute_batch(
            "
            CREATE TABLE IF NOT EXISTS works (
              id TEXT PRIMARY KEY,
              title TEXT NOT NULL,
              author TEXT,
              type TEXT,
              created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS excerpts (
              id TEXT PRIMARY KEY,
              quote TEXT NOT NULL,
              reflection TEXT,
              source_work_id TEXT,
              location TEXT,
              importance INTEGER NOT NULL DEFAULT 3,
              status TEXT NOT NULL DEFAULT 'inbox',
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL,
              FOREIGN KEY (source_work_id) REFERENCES works(id)
            );

            CREATE TABLE IF NOT EXISTS tags (
              id TEXT PRIMARY KEY,
              name TEXT NOT NULL,
              parent_id TEXT,
              color TEXT,
              created_at TEXT NOT NULL,
              FOREIGN KEY (parent_id) REFERENCES tags(id)
            );

            CREATE TABLE IF NOT EXISTS excerpt_tags (
              excerpt_id TEXT NOT NULL,
              tag_id TEXT NOT NULL,
              PRIMARY KEY (excerpt_id, tag_id),
              FOREIGN KEY (excerpt_id) REFERENCES excerpts(id) ON DELETE CASCADE,
              FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS topics (
              id TEXT PRIMARY KEY,
              title TEXT NOT NULL,
              description TEXT,
              research_question TEXT,
              status TEXT NOT NULL DEFAULT 'collecting',
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS topic_nodes (
              id TEXT PRIMARY KEY,
              topic_id TEXT NOT NULL,
              parent_id TEXT,
              title TEXT NOT NULL,
              summary TEXT,
              sort_order INTEGER NOT NULL DEFAULT 0,
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL,
              FOREIGN KEY (topic_id) REFERENCES topics(id) ON DELETE CASCADE,
              FOREIGN KEY (parent_id) REFERENCES topic_nodes(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS topic_excerpts (
              id TEXT PRIMARY KEY,
              topic_id TEXT NOT NULL,
              excerpt_id TEXT NOT NULL,
              node_id TEXT,
              reason TEXT,
              topic_reflection TEXT,
              sort_order INTEGER NOT NULL DEFAULT 0,
              added_at TEXT NOT NULL,
              updated_at TEXT NOT NULL,
              FOREIGN KEY (topic_id) REFERENCES topics(id) ON DELETE CASCADE,
              FOREIGN KEY (excerpt_id) REFERENCES excerpts(id) ON DELETE CASCADE,
              FOREIGN KEY (node_id) REFERENCES topic_nodes(id) ON DELETE SET NULL
            );

            CREATE TABLE IF NOT EXISTS notes (
              id TEXT PRIMARY KEY,
              target_type TEXT NOT NULL,
              target_id TEXT NOT NULL,
              content TEXT NOT NULL,
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_excerpts_created_at ON excerpts(created_at);
            CREATE INDEX IF NOT EXISTS idx_excerpts_updated_at ON excerpts(updated_at);
            CREATE INDEX IF NOT EXISTS idx_excerpts_status ON excerpts(status);
            CREATE UNIQUE INDEX IF NOT EXISTS idx_tags_name_nocase ON tags(name COLLATE NOCASE);
            CREATE INDEX IF NOT EXISTS idx_tags_parent_id ON tags(parent_id);
            CREATE INDEX IF NOT EXISTS idx_topic_nodes_topic_id ON topic_nodes(topic_id);
            CREATE INDEX IF NOT EXISTS idx_topic_excerpts_topic_id ON topic_excerpts(topic_id);
            CREATE INDEX IF NOT EXISTS idx_topic_excerpts_excerpt_id ON topic_excerpts(excerpt_id);
            CREATE INDEX IF NOT EXISTS idx_notes_target ON notes(target_type, target_id);
            ",
        )
        .map_err(|error| format!("failed to run database migrations: {error}"))?;

    Ok(())
}
