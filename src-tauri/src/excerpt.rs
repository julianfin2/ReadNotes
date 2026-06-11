use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use tauri::State;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use uuid::Uuid;

use crate::db::AppState;
use crate::tag::{list_tags_for_excerpt, replace_excerpt_tags, Tag};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Excerpt {
    pub id: String,
    pub quote: String,
    pub reflection: Option<String>,
    pub source_work_id: Option<String>,
    pub location: Option<String>,
    pub importance: i64,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub tags: Vec<Tag>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateExcerptRequest {
    pub quote: String,
    pub reflection: Option<String>,
    pub source_work_id: Option<String>,
    pub location: Option<String>,
    pub importance: Option<i64>,
    pub status: Option<String>,
    pub tag_names: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateExcerptRequest {
    pub id: String,
    pub quote: String,
    pub reflection: Option<String>,
    pub source_work_id: Option<String>,
    pub location: Option<String>,
    pub importance: i64,
    pub status: String,
}

#[tauri::command]
pub fn create_excerpt(
    state: State<'_, AppState>,
    input: CreateExcerptRequest,
) -> Result<Excerpt, String> {
    let quote = normalize_required_text(input.quote, "quote")?;
    let importance = validate_importance(input.importance.unwrap_or(3))?;
    let status = validate_status(input.status.as_deref().unwrap_or("inbox"))?;
    let tag_names = input.tag_names.unwrap_or_default();
    let now = now_rfc3339()?;
    let id = Uuid::new_v4().to_string();

    let mut connection = state
        .db
        .lock()
        .map_err(|_| "database lock was poisoned".to_string())?;

    let transaction = connection
        .transaction()
        .map_err(|error| format!("failed to start excerpt transaction: {error}"))?;

    transaction
        .execute(
            "
            INSERT INTO excerpts (
              id, quote, reflection, source_work_id, location,
              importance, status, created_at, updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            ",
            params![
                id,
                quote,
                empty_to_none(input.reflection),
                empty_to_none(input.source_work_id),
                empty_to_none(input.location),
                importance,
                status,
                now,
                now
            ],
        )
        .map_err(|error| format!("failed to create excerpt: {error}"))?;

    replace_excerpt_tags(&transaction, &id, tag_names)?;

    transaction
        .commit()
        .map_err(|error| format!("failed to save excerpt: {error}"))?;

    get_excerpt_by_id(&connection, &id)
}

#[tauri::command]
pub fn list_excerpts(state: State<'_, AppState>) -> Result<Vec<Excerpt>, String> {
    let connection = state
        .db
        .lock()
        .map_err(|_| "database lock was poisoned".to_string())?;

    let mut statement = connection
        .prepare(
            "
            SELECT
              id, quote, reflection, source_work_id, location,
              importance, status, created_at, updated_at
            FROM excerpts
            ORDER BY created_at DESC
            ",
        )
        .map_err(|error| format!("failed to prepare excerpt list: {error}"))?;

    let rows = statement
        .query_map([], map_excerpt_row)
        .map_err(|error| format!("failed to list excerpts: {error}"))?;

    let mut excerpts = rows
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to read excerpts: {error}"))?;

    for excerpt in &mut excerpts {
        excerpt.tags = list_tags_for_excerpt(&connection, &excerpt.id)?;
    }

    Ok(excerpts)
}

#[tauri::command]
pub fn update_excerpt(
    state: State<'_, AppState>,
    input: UpdateExcerptRequest,
) -> Result<Excerpt, String> {
    let quote = normalize_required_text(input.quote, "quote")?;
    let importance = validate_importance(input.importance)?;
    let status = validate_status(&input.status)?;
    let now = now_rfc3339()?;

    let connection = state
        .db
        .lock()
        .map_err(|_| "database lock was poisoned".to_string())?;

    let changed = connection
        .execute(
            "
            UPDATE excerpts
            SET
              quote = ?2,
              reflection = ?3,
              source_work_id = ?4,
              location = ?5,
              importance = ?6,
              status = ?7,
              updated_at = ?8
            WHERE id = ?1
            ",
            params![
                input.id,
                quote,
                empty_to_none(input.reflection),
                empty_to_none(input.source_work_id),
                empty_to_none(input.location),
                importance,
                status,
                now
            ],
        )
        .map_err(|error| format!("failed to update excerpt: {error}"))?;

    if changed == 0 {
        return Err("excerpt not found".to_string());
    }

    get_excerpt_by_id(&connection, &input.id)
}

#[tauri::command]
pub fn archive_excerpt(state: State<'_, AppState>, id: String) -> Result<Excerpt, String> {
    set_excerpt_status(state, id, "archived")
}

#[tauri::command]
pub fn delete_excerpt(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let connection = state
        .db
        .lock()
        .map_err(|_| "database lock was poisoned".to_string())?;

    let changed = connection
        .execute("DELETE FROM excerpts WHERE id = ?1", params![id])
        .map_err(|error| format!("failed to delete excerpt: {error}"))?;

    if changed == 0 {
        return Err("excerpt not found".to_string());
    }

    Ok(())
}

#[tauri::command]
pub fn get_database_path(state: State<'_, AppState>) -> String {
    state.db_path.display().to_string()
}

fn set_excerpt_status(
    state: State<'_, AppState>,
    id: String,
    status: &str,
) -> Result<Excerpt, String> {
    let now = now_rfc3339()?;
    let connection = state
        .db
        .lock()
        .map_err(|_| "database lock was poisoned".to_string())?;

    let changed = connection
        .execute(
            "UPDATE excerpts SET status = ?2, updated_at = ?3 WHERE id = ?1",
            params![id, status, now],
        )
        .map_err(|error| format!("failed to update excerpt status: {error}"))?;

    if changed == 0 {
        return Err("excerpt not found".to_string());
    }

    get_excerpt_by_id(&connection, &id)
}

pub fn get_excerpt_by_id(connection: &Connection, id: &str) -> Result<Excerpt, String> {
    let mut excerpt = connection
        .query_row(
            "
            SELECT
              id, quote, reflection, source_work_id, location,
              importance, status, created_at, updated_at
            FROM excerpts
            WHERE id = ?1
            ",
            params![id],
            map_excerpt_row,
        )
        .map_err(|error| format!("failed to find excerpt: {error}"))?;

    excerpt.tags = list_tags_for_excerpt(connection, &excerpt.id)?;
    Ok(excerpt)
}

fn map_excerpt_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Excerpt> {
    Ok(Excerpt {
        id: row.get(0)?,
        quote: row.get(1)?,
        reflection: row.get(2)?,
        source_work_id: row.get(3)?,
        location: row.get(4)?,
        importance: row.get(5)?,
        status: row.get(6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
        tags: Vec::new(),
    })
}

fn normalize_required_text(value: String, field: &str) -> Result<String, String> {
    let normalized = value.trim().to_string();
    if normalized.is_empty() {
        Err(format!("{field} cannot be empty"))
    } else {
        Ok(normalized)
    }
}

fn empty_to_none(value: Option<String>) -> Option<String> {
    value.and_then(|text| {
        let trimmed = text.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

fn validate_importance(value: i64) -> Result<i64, String> {
    if (1..=5).contains(&value) {
        Ok(value)
    } else {
        Err("importance must be between 1 and 5".to_string())
    }
}

fn validate_status(value: &str) -> Result<String, String> {
    match value {
        "inbox" | "processed" | "archived" => Ok(value.to_string()),
        _ => Err("status must be inbox, processed, or archived".to_string()),
    }
}

fn now_rfc3339() -> Result<String, String> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|error| format!("failed to format timestamp: {error}"))
}
