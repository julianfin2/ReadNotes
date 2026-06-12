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
            PRAGMA journal_mode = DELETE;
            ",
        )
        .map_err(|error| format!("failed to configure database: {error}"))?;

    initialize_schema(&connection)?;

    Ok(AppState {
        db: Mutex::new(connection),
        db_path,
    })
}

fn initialize_schema(connection: &Connection) -> Result<(), String> {
    create_schema(connection)?;
    ensure_excerpt_search_index(connection)?;

    Ok(())
}

fn create_schema(connection: &Connection) -> Result<(), String> {
    connection
        .execute_batch(
            "
            CREATE TABLE IF NOT EXISTS books (
              id TEXT PRIMARY KEY,
              title TEXT NOT NULL,
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS book_chapters (
              id TEXT PRIMARY KEY,
              book_id TEXT NOT NULL,
              title TEXT NOT NULL,
              sort_order INTEGER NOT NULL DEFAULT 0,
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL,
              FOREIGN KEY (book_id) REFERENCES books(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS excerpts (
              id TEXT PRIMARY KEY,
              quote TEXT NOT NULL,
              reflection TEXT,
              book_id TEXT,
              chapter_id TEXT,
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL,
              FOREIGN KEY (book_id) REFERENCES books(id) ON DELETE SET NULL,
              FOREIGN KEY (chapter_id) REFERENCES book_chapters(id) ON DELETE SET NULL
            );

            CREATE TABLE IF NOT EXISTS tags (
              id TEXT PRIMARY KEY,
              name TEXT NOT NULL,
              parent_id TEXT,
              color TEXT,
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL,
              FOREIGN KEY (parent_id) REFERENCES tags(id) ON DELETE SET NULL
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

            CREATE INDEX IF NOT EXISTS idx_excerpts_created_at ON excerpts(created_at);
            CREATE INDEX IF NOT EXISTS idx_excerpts_updated_at ON excerpts(updated_at);
            CREATE INDEX IF NOT EXISTS idx_excerpts_book_id ON excerpts(book_id);
            CREATE INDEX IF NOT EXISTS idx_excerpts_chapter_id ON excerpts(chapter_id);
            CREATE UNIQUE INDEX IF NOT EXISTS idx_tags_name_nocase ON tags(name COLLATE NOCASE);
            CREATE INDEX IF NOT EXISTS idx_tags_parent_id ON tags(parent_id);
            CREATE UNIQUE INDEX IF NOT EXISTS idx_books_title_nocase ON books(title COLLATE NOCASE);
            CREATE UNIQUE INDEX IF NOT EXISTS idx_book_chapters_book_title_nocase
              ON book_chapters(book_id, title COLLATE NOCASE);
            CREATE INDEX IF NOT EXISTS idx_book_chapters_book_id ON book_chapters(book_id);
            CREATE INDEX IF NOT EXISTS idx_topic_nodes_topic_id ON topic_nodes(topic_id);
            CREATE INDEX IF NOT EXISTS idx_topic_excerpts_topic_id ON topic_excerpts(topic_id);
            CREATE INDEX IF NOT EXISTS idx_topic_excerpts_excerpt_id ON topic_excerpts(excerpt_id);
            ",
        )
        .map_err(|error| format!("failed to create database schema: {error}"))
}

fn ensure_excerpt_search_index(connection: &Connection) -> Result<(), String> {
    connection
        .execute_batch(
            "
            CREATE VIRTUAL TABLE IF NOT EXISTS excerpt_search USING fts5(
              quote,
              reflection,
              content='excerpts',
              content_rowid='rowid'
            );

            INSERT INTO excerpt_search(rowid, quote, reflection)
            SELECT rowid, quote, reflection
            FROM excerpts
            WHERE rowid NOT IN (SELECT rowid FROM excerpt_search);

            DROP TRIGGER IF EXISTS excerpts_ai;
            DROP TRIGGER IF EXISTS excerpts_ad;
            DROP TRIGGER IF EXISTS excerpts_au;

            CREATE TRIGGER excerpts_ai AFTER INSERT ON excerpts BEGIN
              INSERT INTO excerpt_search(rowid, quote, reflection)
              VALUES (new.rowid, new.quote, new.reflection);
            END;

            CREATE TRIGGER excerpts_ad AFTER DELETE ON excerpts BEGIN
              INSERT INTO excerpt_search(excerpt_search, rowid, quote, reflection)
              VALUES ('delete', old.rowid, old.quote, old.reflection);
            END;

            CREATE TRIGGER excerpts_au AFTER UPDATE ON excerpts BEGIN
              INSERT INTO excerpt_search(excerpt_search, rowid, quote, reflection)
              VALUES ('delete', old.rowid, old.quote, old.reflection);
              INSERT INTO excerpt_search(rowid, quote, reflection)
              VALUES (new.rowid, new.quote, new.reflection);
            END;
            ",
        )
        .map_err(|error| format!("failed to ensure excerpt search index: {error}"))
}

pub fn rebuild_excerpt_search_index(connection: &Connection) -> Result<(), String> {
    connection
        .execute_batch(
            "
            DROP TRIGGER IF EXISTS excerpts_ai;
            DROP TRIGGER IF EXISTS excerpts_ad;
            DROP TRIGGER IF EXISTS excerpts_au;
            DROP TABLE IF EXISTS excerpt_search;

            CREATE VIRTUAL TABLE excerpt_search USING fts5(
              quote,
              reflection,
              content='excerpts',
              content_rowid='rowid'
            );

            INSERT INTO excerpt_search(rowid, quote, reflection)
            SELECT rowid, quote, reflection
            FROM excerpts;

            CREATE TRIGGER excerpts_ai AFTER INSERT ON excerpts BEGIN
              INSERT INTO excerpt_search(rowid, quote, reflection)
              VALUES (new.rowid, new.quote, new.reflection);
            END;

            CREATE TRIGGER excerpts_ad AFTER DELETE ON excerpts BEGIN
              INSERT INTO excerpt_search(excerpt_search, rowid, quote, reflection)
              VALUES ('delete', old.rowid, old.quote, old.reflection);
            END;

            CREATE TRIGGER excerpts_au AFTER UPDATE ON excerpts BEGIN
              INSERT INTO excerpt_search(excerpt_search, rowid, quote, reflection)
              VALUES ('delete', old.rowid, old.quote, old.reflection);
              INSERT INTO excerpt_search(rowid, quote, reflection)
              VALUES (new.rowid, new.quote, new.reflection);
            END;
            ",
        )
        .map_err(|error| format!("failed to rebuild excerpt search index: {error}"))
}
