use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};

pub struct AppState {
    pub db: Mutex<Connection>,
    pub db_path: Mutex<PathBuf>,
    startup_issue: Mutex<Option<DatabaseStartupIssue>>,
    default_db_path: PathBuf,
    settings_path: PathBuf,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseInfo {
    current_path: String,
    default_path: String,
    using_default: bool,
    startup_issue: Option<DatabaseStartupIssue>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseStartupIssue {
    configured_path: String,
    reason: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppSettings {
    database_path: Option<String>,
}

pub fn open_database(app: &AppHandle) -> Result<AppState, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("failed to resolve app data directory: {error}"))?;

    fs::create_dir_all(&app_data_dir)
        .map_err(|error| format!("failed to create app data directory: {error}"))?;

    let default_db_path = app_data_dir.join("readnotes.sqlite");
    let settings_path = app_data_dir.join("settings.json");
    let settings = read_settings(&settings_path)?;
    let configured_db_path = settings.database_path.map(PathBuf::from);
    let (db_path, connection, startup_issue) = match configured_db_path {
        Some(custom_path) => match open_configured_database(&custom_path) {
            Ok(connection) => (custom_path, connection, None),
            Err(reason) => {
                let connection = open_database_connection(&default_db_path, true)?;
                let issue = DatabaseStartupIssue {
                    configured_path: custom_path.display().to_string(),
                    reason,
                };
                (default_db_path.clone(), connection, Some(issue))
            }
        },
        None => {
            let connection = open_database_connection(&default_db_path, true)?;
            (default_db_path.clone(), connection, None)
        }
    };

    Ok(AppState {
        db: Mutex::new(connection),
        db_path: Mutex::new(db_path),
        startup_issue: Mutex::new(startup_issue),
        default_db_path,
        settings_path,
    })
}

#[tauri::command]
pub fn get_database_info(state: State<'_, AppState>) -> Result<DatabaseInfo, String> {
    let current_path = state
        .db_path
        .lock()
        .map_err(|_| "database path lock was poisoned".to_string())?
        .clone();
    let startup_issue = state
        .startup_issue
        .lock()
        .map_err(|_| "startup issue lock was poisoned".to_string())?
        .clone();

    Ok(database_info(
        &current_path,
        &state.default_db_path,
        startup_issue,
    ))
}

#[tauri::command]
pub fn create_database_at(state: State<'_, AppState>, path: String) -> Result<DatabaseInfo, String> {
    let db_path = normalize_user_database_path(path)?;
    if db_path.exists() {
        return Err("database file already exists".to_string());
    }

    let connection = open_database_connection(&db_path, true)?;
    activate_database(&state, db_path, connection, true)
}

#[tauri::command]
pub fn switch_database(state: State<'_, AppState>, path: String) -> Result<DatabaseInfo, String> {
    let db_path = normalize_user_database_path(path)?;
    if !db_path.exists() {
        return Err("database file does not exist".to_string());
    }
    if !db_path.is_file() {
        return Err("database path must point to a file".to_string());
    }

    let connection = open_database_connection(&db_path, false)?;
    activate_database(&state, db_path, connection, true)
}

#[tauri::command]
pub fn use_default_database(state: State<'_, AppState>) -> Result<DatabaseInfo, String> {
    let db_path = state.default_db_path.clone();
    let connection = open_database_connection(&db_path, true)?;
    activate_database(&state, db_path, connection, false)
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

            CREATE TABLE IF NOT EXISTS notes (
              id TEXT PRIMARY KEY,
              content TEXT NOT NULL,
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL
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

            CREATE TABLE IF NOT EXISTS note_tags (
              note_id TEXT NOT NULL,
              tag_id TEXT NOT NULL,
              PRIMARY KEY (note_id, tag_id),
              FOREIGN KEY (note_id) REFERENCES notes(id) ON DELETE CASCADE,
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

            CREATE TABLE IF NOT EXISTS topic_materials (
              id TEXT PRIMARY KEY,
              topic_id TEXT NOT NULL,
              material_type TEXT NOT NULL,
              material_id TEXT NOT NULL,
              node_id TEXT,
              reason TEXT,
              topic_reflection TEXT,
              sort_order INTEGER NOT NULL DEFAULT 0,
              added_at TEXT NOT NULL,
              updated_at TEXT NOT NULL,
              FOREIGN KEY (topic_id) REFERENCES topics(id) ON DELETE CASCADE,
              FOREIGN KEY (node_id) REFERENCES topic_nodes(id) ON DELETE SET NULL
            );

            CREATE TABLE IF NOT EXISTS drafts (
              entity_type TEXT NOT NULL,
              entity_id TEXT NOT NULL,
              payload TEXT NOT NULL,
              updated_at TEXT NOT NULL,
              PRIMARY KEY (entity_type, entity_id)
            );

            CREATE INDEX IF NOT EXISTS idx_excerpts_created_at ON excerpts(created_at);
            CREATE INDEX IF NOT EXISTS idx_excerpts_updated_at ON excerpts(updated_at);
            CREATE INDEX IF NOT EXISTS idx_excerpts_book_id ON excerpts(book_id);
            CREATE INDEX IF NOT EXISTS idx_excerpts_chapter_id ON excerpts(chapter_id);
            CREATE INDEX IF NOT EXISTS idx_notes_created_at ON notes(created_at);
            CREATE INDEX IF NOT EXISTS idx_notes_updated_at ON notes(updated_at);
            CREATE UNIQUE INDEX IF NOT EXISTS idx_tags_name_nocase ON tags(name COLLATE NOCASE);
            CREATE INDEX IF NOT EXISTS idx_tags_parent_id ON tags(parent_id);
            CREATE UNIQUE INDEX IF NOT EXISTS idx_books_title_nocase ON books(title COLLATE NOCASE);
            CREATE UNIQUE INDEX IF NOT EXISTS idx_book_chapters_book_title_nocase
              ON book_chapters(book_id, title COLLATE NOCASE);
            CREATE INDEX IF NOT EXISTS idx_book_chapters_book_id ON book_chapters(book_id);
            CREATE INDEX IF NOT EXISTS idx_topic_nodes_topic_id ON topic_nodes(topic_id);
            CREATE INDEX IF NOT EXISTS idx_topic_excerpts_topic_id ON topic_excerpts(topic_id);
            CREATE INDEX IF NOT EXISTS idx_topic_excerpts_excerpt_id ON topic_excerpts(excerpt_id);
            CREATE INDEX IF NOT EXISTS idx_topic_materials_topic_id ON topic_materials(topic_id);
            CREATE INDEX IF NOT EXISTS idx_topic_materials_material ON topic_materials(material_type, material_id);
            CREATE INDEX IF NOT EXISTS idx_drafts_updated_at ON drafts(updated_at);

            INSERT OR IGNORE INTO topic_materials (
              id, topic_id, material_type, material_id, node_id, reason, topic_reflection,
              sort_order, added_at, updated_at
            )
            SELECT
              id, topic_id, 'excerpt', excerpt_id, node_id, reason, topic_reflection,
              sort_order, added_at, updated_at
            FROM topic_excerpts;
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

fn open_database_connection(path: &Path, create_parent: bool) -> Result<Connection, String> {
    if create_parent {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("failed to create database directory: {error}"))?;
        }
    }

    let connection =
        Connection::open(path).map_err(|error| format!("failed to open database: {error}"))?;

    connection
        .execute_batch(
            "
            PRAGMA foreign_keys = ON;
            PRAGMA journal_mode = DELETE;
            ",
        )
        .map_err(|error| format!("failed to configure database: {error}"))?;

    initialize_schema(&connection)?;
    Ok(connection)
}

fn open_configured_database(path: &Path) -> Result<Connection, String> {
    if !path.exists() {
        return Err("保存的数据库路径不存在".to_string());
    }

    if !path.is_file() {
        return Err("保存的数据库路径不是文件".to_string());
    }

    open_database_connection(path, false)
        .map_err(|error| format!("保存的数据库文件无法打开或格式不正确：{error}"))
}

fn activate_database(
    state: &AppState,
    db_path: PathBuf,
    connection: Connection,
    persist_custom_path: bool,
) -> Result<DatabaseInfo, String> {
    let settings = AppSettings {
        database_path: if persist_custom_path {
            Some(db_path.display().to_string())
        } else {
            None
        },
    };
    write_settings(&state.settings_path, &settings)?;

    {
        let mut active_connection = state
            .db
            .lock()
            .map_err(|_| "database lock was poisoned".to_string())?;
        *active_connection = connection;
    }

    {
        let mut active_path = state
            .db_path
            .lock()
            .map_err(|_| "database path lock was poisoned".to_string())?;
        *active_path = db_path.clone();
    }

    {
        let mut startup_issue = state
            .startup_issue
            .lock()
            .map_err(|_| "startup issue lock was poisoned".to_string())?;
        *startup_issue = None;
    }

    Ok(database_info(&db_path, &state.default_db_path, None))
}

fn database_info(
    current_path: &Path,
    default_path: &Path,
    startup_issue: Option<DatabaseStartupIssue>,
) -> DatabaseInfo {
    DatabaseInfo {
        current_path: current_path.display().to_string(),
        default_path: default_path.display().to_string(),
        using_default: current_path == default_path,
        startup_issue,
    }
}

fn normalize_user_database_path(path: String) -> Result<PathBuf, String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("database path cannot be empty".to_string());
    }

    let db_path = PathBuf::from(trimmed);
    if !db_path.is_absolute() {
        return Err("database path must be an absolute path".to_string());
    }

    Ok(db_path)
}

fn read_settings(settings_path: &Path) -> Result<AppSettings, String> {
    if !settings_path.exists() {
        return Ok(AppSettings::default());
    }

    let content = fs::read_to_string(settings_path)
        .map_err(|error| format!("failed to read settings: {error}"))?;
    serde_json::from_str(&content).map_err(|error| format!("failed to parse settings: {error}"))
}

fn write_settings(settings_path: &Path, settings: &AppSettings) -> Result<(), String> {
    if let Some(parent) = settings_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create settings directory: {error}"))?;
    }

    let content = serde_json::to_string_pretty(settings)
        .map_err(|error| format!("failed to serialize settings: {error}"))?;
    fs::write(settings_path, content).map_err(|error| format!("failed to write settings: {error}"))
}
