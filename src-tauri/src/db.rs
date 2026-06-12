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
              book_title TEXT,
              chapter_title TEXT,
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
        .map_err(|error| format!("failed to run database migrations: {error}"))?;

    add_column_if_missing(connection, "excerpts", "book_title", "TEXT")?;
    add_column_if_missing(connection, "excerpts", "chapter_title", "TEXT")?;
    migrate_excerpts_without_removed_fields(connection)?;
    backfill_books_from_excerpts(connection)?;

    rebuild_excerpt_search_index(connection)?;

    Ok(())
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
              book_title,
              chapter_title,
              content='excerpts',
              content_rowid='rowid'
            );

            INSERT INTO excerpt_search(rowid, quote, reflection, book_title, chapter_title)
            SELECT rowid, quote, reflection, book_title, chapter_title
            FROM excerpts;

            CREATE TRIGGER excerpts_ai AFTER INSERT ON excerpts BEGIN
              INSERT INTO excerpt_search(rowid, quote, reflection, book_title, chapter_title)
              VALUES (new.rowid, new.quote, new.reflection, new.book_title, new.chapter_title);
            END;

            CREATE TRIGGER excerpts_ad AFTER DELETE ON excerpts BEGIN
              INSERT INTO excerpt_search(
                excerpt_search, rowid, quote, reflection, book_title, chapter_title
              )
              VALUES (
                'delete', old.rowid, old.quote, old.reflection, old.book_title, old.chapter_title
              );
            END;

            CREATE TRIGGER excerpts_au AFTER UPDATE ON excerpts BEGIN
              INSERT INTO excerpt_search(
                excerpt_search, rowid, quote, reflection, book_title, chapter_title
              )
              VALUES (
                'delete', old.rowid, old.quote, old.reflection, old.book_title, old.chapter_title
              );
              INSERT INTO excerpt_search(rowid, quote, reflection, book_title, chapter_title)
              VALUES (new.rowid, new.quote, new.reflection, new.book_title, new.chapter_title);
            END;
            ",
        )
        .map_err(|error| format!("failed to rebuild excerpt search index: {error}"))
}

fn backfill_books_from_excerpts(connection: &Connection) -> Result<(), String> {
    connection
        .execute_batch(
            "
            INSERT OR IGNORE INTO books (id, title, created_at, updated_at)
            SELECT lower(hex(randomblob(4))) || '-' || lower(hex(randomblob(2))) || '-'
              || lower(hex(randomblob(2))) || '-' || lower(hex(randomblob(2))) || '-'
              || lower(hex(randomblob(6))),
              trim(book_title),
              MIN(created_at),
              MAX(updated_at)
            FROM excerpts
            WHERE book_title IS NOT NULL AND trim(book_title) != ''
            GROUP BY lower(trim(book_title));

            INSERT OR IGNORE INTO book_chapters (
              id, book_id, title, sort_order, created_at, updated_at
            )
            SELECT lower(hex(randomblob(4))) || '-' || lower(hex(randomblob(2))) || '-'
              || lower(hex(randomblob(2))) || '-' || lower(hex(randomblob(2))) || '-'
              || lower(hex(randomblob(6))),
              books.id,
              trim(excerpts.chapter_title),
              0,
              MIN(excerpts.created_at),
              MAX(excerpts.updated_at)
            FROM excerpts
            INNER JOIN books ON lower(books.title) = lower(trim(excerpts.book_title))
            WHERE excerpts.book_title IS NOT NULL
              AND trim(excerpts.book_title) != ''
              AND excerpts.chapter_title IS NOT NULL
              AND trim(excerpts.chapter_title) != ''
            GROUP BY books.id, lower(trim(excerpts.chapter_title));
            ",
        )
        .map_err(|error| format!("failed to backfill books from excerpts: {error}"))
}

fn migrate_excerpts_without_removed_fields(connection: &Connection) -> Result<(), String> {
    let columns = table_columns(connection, "excerpts")?;
    let has_removed_columns = ["location", "importance", "status"]
        .iter()
        .any(|column| columns.iter().any(|existing| existing == column));

    if !has_removed_columns {
        return Ok(());
    }

    connection
        .execute_batch(
            "
            PRAGMA foreign_keys = OFF;
            BEGIN;

            DROP TRIGGER IF EXISTS excerpts_ai;
            DROP TRIGGER IF EXISTS excerpts_ad;
            DROP TRIGGER IF EXISTS excerpts_au;
            DROP TABLE IF EXISTS excerpt_search;
            DROP INDEX IF EXISTS idx_excerpts_importance;
            DROP INDEX IF EXISTS idx_excerpts_status;

            CREATE TABLE excerpts_new (
              id TEXT PRIMARY KEY,
              quote TEXT NOT NULL,
              reflection TEXT,
              source_work_id TEXT,
              book_title TEXT,
              chapter_title TEXT,
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL,
              FOREIGN KEY (source_work_id) REFERENCES works(id)
            );

            INSERT INTO excerpts_new (
              id, quote, reflection, source_work_id, book_title, chapter_title,
              created_at, updated_at
            )
            SELECT
              id, quote, reflection, source_work_id, book_title, chapter_title,
              created_at, updated_at
            FROM excerpts;

            DROP TABLE excerpts;
            ALTER TABLE excerpts_new RENAME TO excerpts;

            CREATE INDEX IF NOT EXISTS idx_excerpts_created_at ON excerpts(created_at);
            CREATE INDEX IF NOT EXISTS idx_excerpts_updated_at ON excerpts(updated_at);

            COMMIT;
            PRAGMA foreign_keys = ON;
            ",
        )
        .map_err(|error| format!("failed to migrate excerpt schema: {error}"))
}

fn table_columns(connection: &Connection, table: &str) -> Result<Vec<String>, String> {
    let query = format!("PRAGMA table_info({table})");
    let mut statement = connection
        .prepare(&query)
        .map_err(|error| format!("failed to inspect table {table}: {error}"))?;
    let rows = statement
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|error| format!("failed to read table info for {table}: {error}"))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to collect table columns for {table}: {error}"))
}

fn add_column_if_missing(
    connection: &Connection,
    table: &str,
    column: &str,
    column_type: &str,
) -> Result<(), String> {
    let query = format!("ALTER TABLE {table} ADD COLUMN {column} {column_type}");
    match connection.execute(&query, []) {
        Ok(_) => Ok(()),
        Err(error) if error.to_string().contains("duplicate column name") => Ok(()),
        Err(error) => Err(format!("failed to add column {table}.{column}: {error}")),
    }
}
