use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use tauri::State;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use uuid::Uuid;

use crate::db::AppState;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub color: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TagWithCount {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub color: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub excerpt_count: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTagRequest {
    pub name: String,
    pub parent_id: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTagRequest {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetExcerptTagsRequest {
    pub excerpt_id: String,
    pub tag_names: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetNoteTagsRequest {
    pub note_id: String,
    pub tag_names: Vec<String>,
}

#[tauri::command]
pub fn create_tag(state: State<'_, AppState>, input: CreateTagRequest) -> Result<Tag, String> {
    let connection = state
        .db
        .lock()
        .map_err(|_| "database lock was poisoned".to_string())?;

    create_tag_record(
        &connection,
        input.name,
        empty_to_none(input.parent_id),
        empty_to_none(input.color),
    )
}

#[tauri::command]
pub fn list_tags(state: State<'_, AppState>) -> Result<Vec<Tag>, String> {
    let connection = state
        .db
        .lock()
        .map_err(|_| "database lock was poisoned".to_string())?;

    list_all_tags(&connection)
}

#[tauri::command]
pub fn list_tags_with_counts(state: State<'_, AppState>) -> Result<Vec<TagWithCount>, String> {
    let connection = state
        .db
        .lock()
        .map_err(|_| "database lock was poisoned".to_string())?;

    let mut statement = connection
        .prepare(
            "
            SELECT
              tags.id,
              tags.name,
              tags.parent_id,
              tags.color,
              tags.created_at,
              tags.updated_at,
              COUNT(DISTINCT excerpt_tags.excerpt_id) + COUNT(DISTINCT note_tags.note_id) AS excerpt_count
            FROM tags
            LEFT JOIN excerpt_tags ON excerpt_tags.tag_id = tags.id
            LEFT JOIN note_tags ON note_tags.tag_id = tags.id
            GROUP BY tags.id
            ORDER BY lower(tags.name) ASC
            ",
        )
        .map_err(|error| format!("failed to prepare tags with counts: {error}"))?;

    let rows = statement
        .query_map([], map_tag_with_count_row)
        .map_err(|error| format!("failed to list tags with counts: {error}"))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to read tags with counts: {error}"))
}

#[tauri::command]
pub fn update_tag(state: State<'_, AppState>, input: UpdateTagRequest) -> Result<Tag, String> {
    let name = normalize_tag_name(input.name)?;
    let now = now_rfc3339()?;
    let connection = state
        .db
        .lock()
        .map_err(|_| "database lock was poisoned".to_string())?;

    let duplicate_id = connection
        .query_row(
            "SELECT id FROM tags WHERE lower(name) = lower(?1) AND id != ?2",
            params![name, input.id],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(|error| format!("failed to check duplicate tag: {error}"))?;

    if duplicate_id.is_some() {
        return Err("tag name already exists".to_string());
    }

    let changed = connection
        .execute(
            "
            UPDATE tags
            SET name = ?2, parent_id = ?3, color = ?4, updated_at = ?5
            WHERE id = ?1
            ",
            params![
                input.id,
                name,
                empty_to_none(input.parent_id),
                empty_to_none(input.color),
                now
            ],
        )
        .map_err(|error| format!("failed to update tag: {error}"))?;

    if changed == 0 {
        return Err("tag not found".to_string());
    }

    get_tag_by_id(&connection, &input.id)
}

#[tauri::command]
pub fn delete_tag(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let connection = state
        .db
        .lock()
        .map_err(|_| "database lock was poisoned".to_string())?;

    let changed = connection
        .execute("DELETE FROM tags WHERE id = ?1", params![id])
        .map_err(|error| format!("failed to delete tag: {error}"))?;

    if changed == 0 {
        return Err("tag not found".to_string());
    }

    Ok(())
}

#[tauri::command]
pub fn set_excerpt_tags(
    state: State<'_, AppState>,
    input: SetExcerptTagsRequest,
) -> Result<Vec<Tag>, String> {
    let mut connection = state
        .db
        .lock()
        .map_err(|_| "database lock was poisoned".to_string())?;

    let transaction = connection
        .transaction()
        .map_err(|error| format!("failed to start tag transaction: {error}"))?;

    replace_excerpt_tags(&transaction, &input.excerpt_id, input.tag_names)?;

    transaction
        .commit()
        .map_err(|error| format!("failed to save excerpt tags: {error}"))?;

    list_tags_for_excerpt(&connection, &input.excerpt_id)
}

#[tauri::command]
pub fn set_note_tags(
    state: State<'_, AppState>,
    input: SetNoteTagsRequest,
) -> Result<Vec<Tag>, String> {
    let mut connection = state
        .db
        .lock()
        .map_err(|_| "database lock was poisoned".to_string())?;

    let transaction = connection
        .transaction()
        .map_err(|error| format!("failed to start note tag transaction: {error}"))?;

    replace_note_tags(&transaction, &input.note_id, input.tag_names)?;

    transaction
        .commit()
        .map_err(|error| format!("failed to save note tags: {error}"))?;

    list_tags_for_note(&connection, &input.note_id)
}

#[tauri::command]
pub fn list_excerpt_tags(
    state: State<'_, AppState>,
    excerpt_id: String,
) -> Result<Vec<Tag>, String> {
    let connection = state
        .db
        .lock()
        .map_err(|_| "database lock was poisoned".to_string())?;

    list_tags_for_excerpt(&connection, &excerpt_id)
}

pub fn replace_excerpt_tags(
    connection: &Connection,
    excerpt_id: &str,
    tag_names: Vec<String>,
) -> Result<(), String> {
    let exists = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM excerpts WHERE id = ?1)",
            params![excerpt_id],
            |row| row.get::<_, bool>(0),
        )
        .map_err(|error| format!("failed to check excerpt: {error}"))?;

    if !exists {
        return Err("excerpt not found".to_string());
    }

    connection
        .execute(
            "DELETE FROM excerpt_tags WHERE excerpt_id = ?1",
            params![excerpt_id],
        )
        .map_err(|error| format!("failed to clear excerpt tags: {error}"))?;

    let tags = find_or_create_tags_by_names(connection, tag_names)?;
    for tag in tags {
        connection
            .execute(
                "
                INSERT OR IGNORE INTO excerpt_tags (excerpt_id, tag_id)
                VALUES (?1, ?2)
                ",
                params![excerpt_id, tag.id],
            )
            .map_err(|error| format!("failed to attach tag: {error}"))?;
    }

    Ok(())
}

pub fn find_or_create_tags_by_names(
    connection: &Connection,
    tag_names: Vec<String>,
) -> Result<Vec<Tag>, String> {
    let mut tags = Vec::new();
    let mut normalized_names = Vec::new();

    for tag_name in tag_names {
        let normalized = normalize_tag_name(tag_name)?;
        if !normalized_names
            .iter()
            .any(|name: &String| name.eq_ignore_ascii_case(&normalized))
        {
            normalized_names.push(normalized);
        }
    }

    for name in normalized_names {
        tags.push(find_or_create_tag_by_name(connection, &name)?);
    }

    Ok(tags)
}

pub fn list_tags_for_excerpt(
    connection: &Connection,
    excerpt_id: &str,
) -> Result<Vec<Tag>, String> {
    let mut statement = connection
        .prepare(
            "
            SELECT tags.id, tags.name, tags.parent_id, tags.color, tags.created_at, tags.updated_at
            FROM tags
            INNER JOIN excerpt_tags ON excerpt_tags.tag_id = tags.id
            WHERE excerpt_tags.excerpt_id = ?1
            ORDER BY lower(tags.name) ASC
            ",
        )
        .map_err(|error| format!("failed to prepare excerpt tags: {error}"))?;

    let rows = statement
        .query_map(params![excerpt_id], map_tag_row)
        .map_err(|error| format!("failed to list excerpt tags: {error}"))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to read excerpt tags: {error}"))
}

pub fn replace_note_tags(
    connection: &Connection,
    note_id: &str,
    tag_names: Vec<String>,
) -> Result<(), String> {
    let exists = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM notes WHERE id = ?1)",
            params![note_id],
            |row| row.get::<_, bool>(0),
        )
        .map_err(|error| format!("failed to check note: {error}"))?;

    if !exists {
        return Err("note not found".to_string());
    }

    connection
        .execute("DELETE FROM note_tags WHERE note_id = ?1", params![note_id])
        .map_err(|error| format!("failed to clear note tags: {error}"))?;

    let tags = find_or_create_tags_by_names(connection, tag_names)?;
    for tag in tags {
        connection
            .execute(
                "
                INSERT OR IGNORE INTO note_tags (note_id, tag_id)
                VALUES (?1, ?2)
                ",
                params![note_id, tag.id],
            )
            .map_err(|error| format!("failed to attach note tag: {error}"))?;
    }

    Ok(())
}

pub fn list_tags_for_note(connection: &Connection, note_id: &str) -> Result<Vec<Tag>, String> {
    let mut statement = connection
        .prepare(
            "
            SELECT tags.id, tags.name, tags.parent_id, tags.color, tags.created_at, tags.updated_at
            FROM tags
            INNER JOIN note_tags ON note_tags.tag_id = tags.id
            WHERE note_tags.note_id = ?1
            ORDER BY lower(tags.name) ASC
            ",
        )
        .map_err(|error| format!("failed to prepare note tags: {error}"))?;

    let rows = statement
        .query_map(params![note_id], map_tag_row)
        .map_err(|error| format!("failed to list note tags: {error}"))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to read note tags: {error}"))
}

fn list_all_tags(connection: &Connection) -> Result<Vec<Tag>, String> {
    let mut statement = connection
        .prepare(
            "
            SELECT id, name, parent_id, color, created_at, updated_at
            FROM tags
            ORDER BY lower(name) ASC
            ",
        )
        .map_err(|error| format!("failed to prepare tags: {error}"))?;

    let rows = statement
        .query_map([], map_tag_row)
        .map_err(|error| format!("failed to list tags: {error}"))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to read tags: {error}"))
}

fn find_or_create_tag_by_name(connection: &Connection, name: &str) -> Result<Tag, String> {
    if let Some(tag) = get_tag_by_name(connection, name)? {
        return Ok(tag);
    }

    create_tag_record(connection, name.to_string(), None, None)
}

fn create_tag_record(
    connection: &Connection,
    name: String,
    parent_id: Option<String>,
    color: Option<String>,
) -> Result<Tag, String> {
    let name = normalize_tag_name(name)?;
    if get_tag_by_name(connection, &name)?.is_some() {
        return Err("tag name already exists".to_string());
    }

    let id = Uuid::new_v4().to_string();
    let now = now_rfc3339()?;

    connection
        .execute(
            "
            INSERT INTO tags (id, name, parent_id, color, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            ",
            params![id, name, parent_id, color, now, now],
        )
        .map_err(|error| format!("failed to create tag: {error}"))?;

    get_tag_by_id(connection, &id)
}

fn get_tag_by_id(connection: &Connection, id: &str) -> Result<Tag, String> {
    connection
        .query_row(
            "
            SELECT id, name, parent_id, color, created_at, updated_at
            FROM tags
            WHERE id = ?1
            ",
            params![id],
            map_tag_row,
        )
        .map_err(|error| format!("failed to find tag: {error}"))
}

fn get_tag_by_name(connection: &Connection, name: &str) -> Result<Option<Tag>, String> {
    connection
        .query_row(
            "
            SELECT id, name, parent_id, color, created_at, updated_at
            FROM tags
            WHERE lower(name) = lower(?1)
            ",
            params![name],
            map_tag_row,
        )
        .optional()
        .map_err(|error| format!("failed to find tag by name: {error}"))
}

fn map_tag_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Tag> {
    Ok(Tag {
        id: row.get(0)?,
        name: row.get(1)?,
        parent_id: row.get(2)?,
        color: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
}

fn map_tag_with_count_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<TagWithCount> {
    Ok(TagWithCount {
        id: row.get(0)?,
        name: row.get(1)?,
        parent_id: row.get(2)?,
        color: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
        excerpt_count: row.get(6)?,
    })
}

fn normalize_tag_name(value: String) -> Result<String, String> {
    let normalized = value.trim().trim_start_matches('#').trim().to_string();
    if normalized.is_empty() {
        Err("tag name cannot be empty".to_string())
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

fn now_rfc3339() -> Result<String, String> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|error| format!("failed to format timestamp: {error}"))
}
