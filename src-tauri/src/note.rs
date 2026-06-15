use rusqlite::{params, params_from_iter, Connection, ToSql};
use serde::{Deserialize, Serialize};
use tauri::State;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use uuid::Uuid;

use crate::db::AppState;
use crate::tag::{list_tags_for_note, replace_note_tags, Tag};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    pub id: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
    pub tags: Vec<Tag>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateNoteRequest {
    pub content: String,
    pub tag_names: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateNoteRequest {
    pub id: String,
    pub content: String,
    pub tag_names: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListNotesRequest {
    pub search: Option<String>,
    pub tag_name: Option<String>,
}

#[tauri::command]
pub fn create_note(state: State<'_, AppState>, input: CreateNoteRequest) -> Result<Note, String> {
    let content = normalize_required_text(input.content, "note")?;
    let tag_names = input.tag_names.unwrap_or_default();
    let now = now_rfc3339()?;
    let id = Uuid::new_v4().to_string();

    let mut connection = state
        .db
        .lock()
        .map_err(|_| "database lock was poisoned".to_string())?;
    let transaction = connection
        .transaction()
        .map_err(|error| format!("failed to start note transaction: {error}"))?;

    transaction
        .execute(
            "
            INSERT INTO notes (id, content, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4)
            ",
            params![id, content, now, now],
        )
        .map_err(|error| format!("failed to create note: {error}"))?;

    replace_note_tags(&transaction, &id, tag_names)?;

    transaction
        .commit()
        .map_err(|error| format!("failed to save note: {error}"))?;

    get_note_by_id(&connection, &id)
}

#[tauri::command]
pub fn list_notes(
    state: State<'_, AppState>,
    input: Option<ListNotesRequest>,
) -> Result<Vec<Note>, String> {
    let connection = state
        .db
        .lock()
        .map_err(|_| "database lock was poisoned".to_string())?;
    let input = input.unwrap_or_default();
    let mut clauses = Vec::new();
    let mut parameter_values = Vec::new();

    if let Some(search) = normalize_optional_text(input.search) {
        clauses.push(
            "(
                lower(COALESCE(notes.content, '')) LIKE lower(?)
                OR EXISTS (
                    SELECT 1
                    FROM note_tags
                    INNER JOIN tags ON tags.id = note_tags.tag_id
                    WHERE note_tags.note_id = notes.id
                      AND lower(tags.name) LIKE lower(?)
                )
            )"
            .to_string(),
        );
        parameter_values.push(format!("%{search}%"));
        parameter_values.push(format!("%{search}%"));
    }

    if let Some(tag_name) = normalize_optional_text(input.tag_name) {
        clauses.push(
            "EXISTS (
                SELECT 1
                FROM note_tags
                INNER JOIN tags ON tags.id = note_tags.tag_id
                WHERE note_tags.note_id = notes.id
                  AND lower(tags.name) = lower(?)
            )"
            .to_string(),
        );
        parameter_values.push(tag_name.trim_start_matches('#').to_string());
    }

    let where_clause = if clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", clauses.join(" AND "))
    };
    let query = format!(
        "
        SELECT id, content, created_at, updated_at
        FROM notes
        {where_clause}
        ORDER BY updated_at DESC, created_at DESC
        "
    );
    let parameters: Vec<&dyn ToSql> = parameter_values
        .iter()
        .map(|value| value as &dyn ToSql)
        .collect();

    let mut statement = connection
        .prepare(&query)
        .map_err(|error| format!("failed to prepare note list: {error}"))?;

    let rows = statement
        .query_map(params_from_iter(parameters), map_note_row)
        .map_err(|error| format!("failed to list notes: {error}"))?;

    let mut notes = rows
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to read notes: {error}"))?;

    for note in &mut notes {
        note.tags = list_tags_for_note(&connection, &note.id)?;
    }

    Ok(notes)
}

impl Default for ListNotesRequest {
    fn default() -> Self {
        Self {
            search: None,
            tag_name: None,
        }
    }
}

#[tauri::command]
pub fn update_note(state: State<'_, AppState>, input: UpdateNoteRequest) -> Result<Note, String> {
    let content = normalize_required_text(input.content, "note")?;
    let now = now_rfc3339()?;

    let mut connection = state
        .db
        .lock()
        .map_err(|_| "database lock was poisoned".to_string())?;
    let transaction = connection
        .transaction()
        .map_err(|error| format!("failed to start note update transaction: {error}"))?;

    let changed = transaction
        .execute(
            "
            UPDATE notes
            SET content = ?2, updated_at = ?3
            WHERE id = ?1
            ",
            params![input.id.as_str(), content, now],
        )
        .map_err(|error| format!("failed to update note: {error}"))?;

    if changed == 0 {
        return Err("note not found".to_string());
    }

    if let Some(tag_names) = input.tag_names {
        replace_note_tags(&transaction, &input.id, tag_names)?;
    }

    transaction
        .commit()
        .map_err(|error| format!("failed to save note update: {error}"))?;

    get_note_by_id(&connection, &input.id)
}

#[tauri::command]
pub fn delete_note(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let connection = state
        .db
        .lock()
        .map_err(|_| "database lock was poisoned".to_string())?;

    let changed = connection
        .execute("DELETE FROM notes WHERE id = ?1", params![id])
        .map_err(|error| format!("failed to delete note: {error}"))?;

    if changed == 0 {
        return Err("note not found".to_string());
    }

    Ok(())
}

pub fn get_note_by_id(connection: &Connection, id: &str) -> Result<Note, String> {
    let mut note = connection
        .query_row(
            "
            SELECT id, content, created_at, updated_at
            FROM notes
            WHERE id = ?1
            ",
            params![id],
            map_note_row,
        )
        .map_err(|error| format!("failed to find note: {error}"))?;

    note.tags = list_tags_for_note(connection, &note.id)?;
    Ok(note)
}

fn map_note_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Note> {
    Ok(Note {
        id: row.get(0)?,
        content: row.get(1)?,
        created_at: row.get(2)?,
        updated_at: row.get(3)?,
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

fn normalize_optional_text(value: Option<String>) -> Option<String> {
    empty_to_none(value)
}

fn now_rfc3339() -> Result<String, String> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|error| format!("failed to format timestamp: {error}"))
}
