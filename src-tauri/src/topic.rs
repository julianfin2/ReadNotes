use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use tauri::State;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use uuid::Uuid;

use crate::db::AppState;
use crate::excerpt::{get_excerpt_by_id, Excerpt};
use crate::note::{get_note_by_id, Note};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Topic {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub research_question: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TopicNode {
    pub id: String,
    pub topic_id: String,
    pub parent_id: Option<String>,
    pub title: String,
    pub summary: Option<String>,
    pub sort_order: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TopicExcerpt {
    pub id: String,
    pub topic_id: String,
    pub material_type: String,
    pub material_id: String,
    pub excerpt_id: Option<String>,
    pub note_id: Option<String>,
    pub node_id: Option<String>,
    pub reason: Option<String>,
    pub topic_reflection: Option<String>,
    pub sort_order: i64,
    pub added_at: String,
    pub updated_at: String,
    pub excerpt: Option<Excerpt>,
    pub note: Option<Note>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTopicRequest {
    pub title: String,
    pub description: Option<String>,
    pub research_question: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTopicRequest {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub research_question: Option<String>,
    pub status: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTopicNodeRequest {
    pub topic_id: String,
    pub parent_id: Option<String>,
    pub title: String,
    pub summary: Option<String>,
    pub sort_order: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTopicNodeRequest {
    pub id: String,
    pub parent_id: Option<String>,
    pub title: String,
    pub summary: Option<String>,
    pub sort_order: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddMaterialToTopicRequest {
    pub topic_id: String,
    pub material_type: String,
    pub material_id: String,
    pub node_id: Option<String>,
    pub reason: Option<String>,
    pub topic_reflection: Option<String>,
    pub sort_order: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTopicExcerptRequest {
    pub id: String,
    pub node_id: Option<String>,
    pub reason: Option<String>,
    pub topic_reflection: Option<String>,
    pub sort_order: i64,
}

#[tauri::command]
pub fn create_topic(
    state: State<'_, AppState>,
    input: CreateTopicRequest,
) -> Result<Topic, String> {
    let title = normalize_required_text(input.title, "title")?;
    let status = validate_topic_status(input.status.as_deref().unwrap_or("collecting"))?;
    let now = now_rfc3339()?;
    let id = Uuid::new_v4().to_string();
    let connection = state
        .db
        .lock()
        .map_err(|_| "database lock was poisoned".to_string())?;

    connection
        .execute(
            "
            INSERT INTO topics (
              id, title, description, research_question, status, created_at, updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ",
            params![
                id,
                title,
                empty_to_none(input.description),
                empty_to_none(input.research_question),
                status,
                now,
                now
            ],
        )
        .map_err(|error| format!("failed to create topic: {error}"))?;

    get_topic_by_id(&connection, &id)
}

#[tauri::command]
pub fn list_topics(state: State<'_, AppState>) -> Result<Vec<Topic>, String> {
    let connection = state
        .db
        .lock()
        .map_err(|_| "database lock was poisoned".to_string())?;

    let mut statement = connection
        .prepare(
            "
            SELECT id, title, description, research_question, status, created_at, updated_at
            FROM topics
            ORDER BY updated_at DESC
            ",
        )
        .map_err(|error| format!("failed to prepare topics: {error}"))?;

    let rows = statement
        .query_map([], map_topic_row)
        .map_err(|error| format!("failed to list topics: {error}"))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to read topics: {error}"))
}

#[tauri::command]
pub fn update_topic(
    state: State<'_, AppState>,
    input: UpdateTopicRequest,
) -> Result<Topic, String> {
    let title = normalize_required_text(input.title, "title")?;
    let status = validate_topic_status(&input.status)?;
    let now = now_rfc3339()?;
    let connection = state
        .db
        .lock()
        .map_err(|_| "database lock was poisoned".to_string())?;

    let changed = connection
        .execute(
            "
            UPDATE topics
            SET title = ?2, description = ?3, research_question = ?4, status = ?5, updated_at = ?6
            WHERE id = ?1
            ",
            params![
                input.id,
                title,
                empty_to_none(input.description),
                empty_to_none(input.research_question),
                status,
                now
            ],
        )
        .map_err(|error| format!("failed to update topic: {error}"))?;

    if changed == 0 {
        return Err("topic not found".to_string());
    }

    get_topic_by_id(&connection, &input.id)
}

#[tauri::command]
pub fn delete_topic(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let connection = state
        .db
        .lock()
        .map_err(|_| "database lock was poisoned".to_string())?;

    let changed = connection
        .execute("DELETE FROM topics WHERE id = ?1", params![id])
        .map_err(|error| format!("failed to delete topic: {error}"))?;

    if changed == 0 {
        return Err("topic not found".to_string());
    }

    Ok(())
}

#[tauri::command]
pub fn create_topic_node(
    state: State<'_, AppState>,
    input: CreateTopicNodeRequest,
) -> Result<TopicNode, String> {
    let title = normalize_required_text(input.title, "title")?;
    let now = now_rfc3339()?;
    let id = Uuid::new_v4().to_string();
    let connection = state
        .db
        .lock()
        .map_err(|_| "database lock was poisoned".to_string())?;

    ensure_topic_exists(&connection, &input.topic_id)?;
    if let Some(parent_id) = input.parent_id.as_deref() {
        ensure_topic_node_exists(&connection, parent_id)?;
    }

    connection
        .execute(
            "
            INSERT INTO topic_nodes (
              id, topic_id, parent_id, title, summary, sort_order, created_at, updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ",
            params![
                id,
                input.topic_id,
                empty_to_none(input.parent_id),
                title,
                empty_to_none(input.summary),
                input.sort_order.unwrap_or(0),
                now,
                now
            ],
        )
        .map_err(|error| format!("failed to create topic node: {error}"))?;

    get_topic_node_by_id(&connection, &id)
}

#[tauri::command]
pub fn list_topic_nodes(
    state: State<'_, AppState>,
    topic_id: String,
) -> Result<Vec<TopicNode>, String> {
    let connection = state
        .db
        .lock()
        .map_err(|_| "database lock was poisoned".to_string())?;

    let mut statement = connection
        .prepare(
            "
            SELECT id, topic_id, parent_id, title, summary, sort_order, created_at, updated_at
            FROM topic_nodes
            WHERE topic_id = ?1
            ORDER BY sort_order ASC, created_at ASC
            ",
        )
        .map_err(|error| format!("failed to prepare topic nodes: {error}"))?;

    let rows = statement
        .query_map(params![topic_id], map_topic_node_row)
        .map_err(|error| format!("failed to list topic nodes: {error}"))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to read topic nodes: {error}"))
}

#[tauri::command]
pub fn update_topic_node(
    state: State<'_, AppState>,
    input: UpdateTopicNodeRequest,
) -> Result<TopicNode, String> {
    let title = normalize_required_text(input.title, "title")?;
    let now = now_rfc3339()?;
    let connection = state
        .db
        .lock()
        .map_err(|_| "database lock was poisoned".to_string())?;

    if let Some(parent_id) = input.parent_id.as_deref() {
        ensure_topic_node_exists(&connection, parent_id)?;
    }

    let changed = connection
        .execute(
            "
            UPDATE topic_nodes
            SET parent_id = ?2, title = ?3, summary = ?4, sort_order = ?5, updated_at = ?6
            WHERE id = ?1
            ",
            params![
                input.id,
                empty_to_none(input.parent_id),
                title,
                empty_to_none(input.summary),
                input.sort_order,
                now
            ],
        )
        .map_err(|error| format!("failed to update topic node: {error}"))?;

    if changed == 0 {
        return Err("topic node not found".to_string());
    }

    get_topic_node_by_id(&connection, &input.id)
}

#[tauri::command]
pub fn delete_topic_node(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let connection = state
        .db
        .lock()
        .map_err(|_| "database lock was poisoned".to_string())?;

    let changed = connection
        .execute("DELETE FROM topic_nodes WHERE id = ?1", params![id])
        .map_err(|error| format!("failed to delete topic node: {error}"))?;

    if changed == 0 {
        return Err("topic node not found".to_string());
    }

    Ok(())
}

#[tauri::command]
pub fn add_material_to_topic(
    state: State<'_, AppState>,
    input: AddMaterialToTopicRequest,
) -> Result<TopicExcerpt, String> {
    let now = now_rfc3339()?;
    let id = Uuid::new_v4().to_string();
    let connection = state
        .db
        .lock()
        .map_err(|_| "database lock was poisoned".to_string())?;

    ensure_topic_exists(&connection, &input.topic_id)?;
    ensure_material_exists(&connection, &input.material_type, &input.material_id)?;
    if let Some(node_id) = input.node_id.as_deref() {
        ensure_topic_node_exists(&connection, node_id)?;
    }

    connection
        .execute(
            "
            INSERT INTO topic_materials (
              id, topic_id, material_type, material_id, node_id, reason, topic_reflection,
              sort_order, added_at, updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            ",
            params![
                id,
                input.topic_id,
                input.material_type,
                input.material_id,
                empty_to_none(input.node_id),
                empty_to_none(input.reason),
                empty_to_none(input.topic_reflection),
                input.sort_order.unwrap_or(0),
                now,
                now
            ],
        )
        .map_err(|error| format!("failed to add material to topic: {error}"))?;

    get_topic_excerpt_by_id(&connection, &id)
}

#[tauri::command]
pub fn list_topic_materials(
    state: State<'_, AppState>,
    topic_id: String,
) -> Result<Vec<TopicExcerpt>, String> {
    let connection = state
        .db
        .lock()
        .map_err(|_| "database lock was poisoned".to_string())?;

    let mut statement = connection
        .prepare(
            "
            SELECT id, topic_id, material_type, material_id, node_id, reason, topic_reflection,
                   sort_order, added_at, updated_at
            FROM topic_materials
            WHERE topic_id = ?1
            ORDER BY sort_order ASC, added_at DESC
            ",
        )
        .map_err(|error| format!("failed to prepare topic materials: {error}"))?;

    let rows = statement
        .query_map(params![topic_id], map_topic_excerpt_base_row)
        .map_err(|error| format!("failed to list topic materials: {error}"))?;

    let bases = rows
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to read topic materials: {error}"))?;

    bases
        .into_iter()
        .map(|base| hydrate_topic_excerpt(&connection, base))
        .collect()
}

#[tauri::command]
pub fn update_topic_material(
    state: State<'_, AppState>,
    input: UpdateTopicExcerptRequest,
) -> Result<TopicExcerpt, String> {
    let now = now_rfc3339()?;
    let connection = state
        .db
        .lock()
        .map_err(|_| "database lock was poisoned".to_string())?;

    if let Some(node_id) = input.node_id.as_deref() {
        ensure_topic_node_exists(&connection, node_id)?;
    }

    let changed = connection
        .execute(
            "
            UPDATE topic_materials
            SET node_id = ?2, reason = ?3, topic_reflection = ?4, sort_order = ?5, updated_at = ?6
            WHERE id = ?1
            ",
            params![
                input.id,
                empty_to_none(input.node_id),
                empty_to_none(input.reason),
                empty_to_none(input.topic_reflection),
                input.sort_order,
                now
            ],
        )
        .map_err(|error| format!("failed to update topic material: {error}"))?;

    if changed == 0 {
        return Err("topic material not found".to_string());
    }

    get_topic_excerpt_by_id(&connection, &input.id)
}

#[tauri::command]
pub fn remove_material_from_topic(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let connection = state
        .db
        .lock()
        .map_err(|_| "database lock was poisoned".to_string())?;

    let changed = connection
        .execute("DELETE FROM topic_materials WHERE id = ?1", params![id])
        .map_err(|error| format!("failed to remove topic material: {error}"))?;

    if changed == 0 {
        return Err("topic material not found".to_string());
    }

    Ok(())
}

fn get_topic_by_id(connection: &Connection, id: &str) -> Result<Topic, String> {
    connection
        .query_row(
            "
            SELECT id, title, description, research_question, status, created_at, updated_at
            FROM topics
            WHERE id = ?1
            ",
            params![id],
            map_topic_row,
        )
        .map_err(|error| format!("failed to find topic: {error}"))
}

fn get_topic_node_by_id(connection: &Connection, id: &str) -> Result<TopicNode, String> {
    connection
        .query_row(
            "
            SELECT id, topic_id, parent_id, title, summary, sort_order, created_at, updated_at
            FROM topic_nodes
            WHERE id = ?1
            ",
            params![id],
            map_topic_node_row,
        )
        .map_err(|error| format!("failed to find topic node: {error}"))
}

fn get_topic_excerpt_by_id(connection: &Connection, id: &str) -> Result<TopicExcerpt, String> {
    let base = connection
        .query_row(
            "
            SELECT id, topic_id, material_type, material_id, node_id, reason, topic_reflection,
                   sort_order, added_at, updated_at
            FROM topic_materials
            WHERE id = ?1
            ",
            params![id],
            map_topic_excerpt_base_row,
        )
        .map_err(|error| format!("failed to find topic excerpt: {error}"))?;

    hydrate_topic_excerpt(connection, base)
}

struct TopicExcerptBase {
    id: String,
    topic_id: String,
    material_type: String,
    material_id: String,
    node_id: Option<String>,
    reason: Option<String>,
    topic_reflection: Option<String>,
    sort_order: i64,
    added_at: String,
    updated_at: String,
}

fn hydrate_topic_excerpt(
    connection: &Connection,
    base: TopicExcerptBase,
) -> Result<TopicExcerpt, String> {
    let excerpt = if base.material_type == "excerpt" {
        Some(get_excerpt_by_id(connection, &base.material_id)?)
    } else {
        None
    };
    let note = if base.material_type == "note" {
        Some(get_note_by_id(connection, &base.material_id)?)
    } else {
        None
    };

    Ok(TopicExcerpt {
        id: base.id,
        topic_id: base.topic_id,
        material_type: base.material_type.clone(),
        material_id: base.material_id.clone(),
        excerpt_id: (base.material_type == "excerpt").then_some(base.material_id.clone()),
        note_id: (base.material_type == "note").then_some(base.material_id),
        node_id: base.node_id,
        reason: base.reason,
        topic_reflection: base.topic_reflection,
        sort_order: base.sort_order,
        added_at: base.added_at,
        updated_at: base.updated_at,
        excerpt,
        note,
    })
}

fn map_topic_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Topic> {
    Ok(Topic {
        id: row.get(0)?,
        title: row.get(1)?,
        description: row.get(2)?,
        research_question: row.get(3)?,
        status: row.get(4)?,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
    })
}

fn map_topic_node_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<TopicNode> {
    Ok(TopicNode {
        id: row.get(0)?,
        topic_id: row.get(1)?,
        parent_id: row.get(2)?,
        title: row.get(3)?,
        summary: row.get(4)?,
        sort_order: row.get(5)?,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
    })
}

fn map_topic_excerpt_base_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<TopicExcerptBase> {
    Ok(TopicExcerptBase {
        id: row.get(0)?,
        topic_id: row.get(1)?,
        material_type: row.get(2)?,
        material_id: row.get(3)?,
        node_id: row.get(4)?,
        reason: row.get(5)?,
        topic_reflection: row.get(6)?,
        sort_order: row.get(7)?,
        added_at: row.get(8)?,
        updated_at: row.get(9)?,
    })
}

fn ensure_topic_exists(connection: &Connection, id: &str) -> Result<(), String> {
    ensure_exists(connection, "topics", id, "topic not found")
}

fn ensure_topic_node_exists(connection: &Connection, id: &str) -> Result<(), String> {
    ensure_exists(connection, "topic_nodes", id, "topic node not found")
}

fn ensure_excerpt_exists(connection: &Connection, id: &str) -> Result<(), String> {
    ensure_exists(connection, "excerpts", id, "excerpt not found")
}

fn ensure_note_exists(connection: &Connection, id: &str) -> Result<(), String> {
    ensure_exists(connection, "notes", id, "note not found")
}

fn ensure_material_exists(
    connection: &Connection,
    material_type: &str,
    material_id: &str,
) -> Result<(), String> {
    match material_type {
        "excerpt" => ensure_excerpt_exists(connection, material_id),
        "note" => ensure_note_exists(connection, material_id),
        _ => Err("material type must be excerpt or note".to_string()),
    }
}

fn ensure_exists(
    connection: &Connection,
    table: &str,
    id: &str,
    not_found_message: &str,
) -> Result<(), String> {
    let query = format!("SELECT EXISTS(SELECT 1 FROM {table} WHERE id = ?1)");
    let exists = connection
        .query_row(&query, params![id], |row| row.get::<_, bool>(0))
        .map_err(|error| format!("failed to check record existence: {error}"))?;

    if exists {
        Ok(())
    } else {
        Err(not_found_message.to_string())
    }
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

fn validate_topic_status(value: &str) -> Result<String, String> {
    match value {
        "collecting" | "organizing" | "drafting" | "finished" | "paused" => Ok(value.to_string()),
        _ => {
            Err("status must be collecting, organizing, drafting, finished, or paused".to_string())
        }
    }
}

fn now_rfc3339() -> Result<String, String> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|error| format!("failed to format timestamp: {error}"))
}
