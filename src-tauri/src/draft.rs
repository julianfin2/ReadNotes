use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use tauri::State;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use crate::db::AppState;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Draft {
    pub entity_type: String,
    pub entity_id: String,
    pub payload: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveDraftRequest {
    pub entity_type: String,
    pub entity_id: String,
    pub payload: String,
}

#[tauri::command]
pub fn get_draft(
    state: State<'_, AppState>,
    entity_type: String,
    entity_id: String,
) -> Result<Option<Draft>, String> {
    let entity_type = normalize_required_text(entity_type, "entityType")?;
    let entity_id = normalize_required_text(entity_id, "entityId")?;
    let connection = state
        .db
        .lock()
        .map_err(|_| "database lock was poisoned".to_string())?;

    connection
        .query_row(
            "
            SELECT entity_type, entity_id, payload, updated_at
            FROM drafts
            WHERE entity_type = ?1 AND entity_id = ?2
            ",
            params![entity_type, entity_id],
            |row| {
                Ok(Draft {
                    entity_type: row.get(0)?,
                    entity_id: row.get(1)?,
                    payload: row.get(2)?,
                    updated_at: row.get(3)?,
                })
            },
        )
        .optional()
        .map_err(|error| format!("failed to get draft: {error}"))
}

#[tauri::command]
pub fn save_draft(state: State<'_, AppState>, input: SaveDraftRequest) -> Result<Draft, String> {
    let entity_type = normalize_required_text(input.entity_type, "entityType")?;
    let entity_id = normalize_required_text(input.entity_id, "entityId")?;
    let payload = normalize_required_text(input.payload, "payload")?;
    let updated_at = now_rfc3339()?;
    let connection = state
        .db
        .lock()
        .map_err(|_| "database lock was poisoned".to_string())?;

    connection
        .execute(
            "
            INSERT INTO drafts (entity_type, entity_id, payload, updated_at)
            VALUES (?1, ?2, ?3, ?4)
            ON CONFLICT(entity_type, entity_id) DO UPDATE SET
              payload = excluded.payload,
              updated_at = excluded.updated_at
            ",
            params![entity_type, entity_id, payload, updated_at],
        )
        .map_err(|error| format!("failed to save draft: {error}"))?;

    Ok(Draft {
        entity_type,
        entity_id,
        payload,
        updated_at,
    })
}

#[tauri::command]
pub fn delete_draft(
    state: State<'_, AppState>,
    entity_type: String,
    entity_id: String,
) -> Result<(), String> {
    let entity_type = normalize_required_text(entity_type, "entityType")?;
    let entity_id = normalize_required_text(entity_id, "entityId")?;
    let connection = state
        .db
        .lock()
        .map_err(|_| "database lock was poisoned".to_string())?;

    connection
        .execute(
            "DELETE FROM drafts WHERE entity_type = ?1 AND entity_id = ?2",
            params![entity_type, entity_id],
        )
        .map_err(|error| format!("failed to delete draft: {error}"))?;

    Ok(())
}

fn normalize_required_text(value: String, field: &str) -> Result<String, String> {
    let normalized = value.trim().to_string();
    if normalized.is_empty() {
        Err(format!("{field} cannot be empty"))
    } else {
        Ok(normalized)
    }
}

fn now_rfc3339() -> Result<String, String> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|error| format!("failed to format time: {error}"))
}
