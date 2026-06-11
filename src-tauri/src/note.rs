use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use tauri::State;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use uuid::Uuid;

use crate::db::AppState;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    pub id: String,
    pub target_type: String,
    pub target_id: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineEntry {
    pub id: String,
    pub kind: String,
    pub occurred_at: String,
    pub title: String,
    pub content: Option<String>,
    pub target_type: Option<String>,
    pub target_id: Option<String>,
    pub related_excerpt_id: Option<String>,
    pub related_topic_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateNoteRequest {
    pub target_type: String,
    pub target_id: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateNoteRequest {
    pub id: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListNotesRequest {
    pub target_type: Option<String>,
    pub target_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListTimelineRequest {
    pub topic_id: Option<String>,
    pub excerpt_id: Option<String>,
}

#[tauri::command]
pub fn create_note(state: State<'_, AppState>, input: CreateNoteRequest) -> Result<Note, String> {
    let target_type = validate_target_type(&input.target_type)?;
    let target_id = normalize_required_text(input.target_id, "targetId")?;
    let content = normalize_required_text(input.content, "content")?;
    let now = now_rfc3339()?;
    let id = Uuid::new_v4().to_string();
    let connection = state
        .db
        .lock()
        .map_err(|_| "database lock was poisoned".to_string())?;

    ensure_target_exists(&connection, &target_type, &target_id)?;

    connection
        .execute(
            "
            INSERT INTO notes (id, target_type, target_id, content, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            ",
            params![id, target_type, target_id, content, now, now],
        )
        .map_err(|error| format!("failed to create note: {error}"))?;

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

    match (input.target_type, input.target_id) {
        (Some(target_type), Some(target_id)) => {
            let target_type = validate_target_type(&target_type)?;
            let mut statement = connection
                .prepare(
                    "
                    SELECT id, target_type, target_id, content, created_at, updated_at
                    FROM notes
                    WHERE target_type = ?1 AND target_id = ?2
                    ORDER BY created_at DESC
                    ",
                )
                .map_err(|error| format!("failed to prepare notes: {error}"))?;
            let rows = statement
                .query_map(params![target_type, target_id], map_note_row)
                .map_err(|error| format!("failed to list notes: {error}"))?;

            rows.collect::<Result<Vec<_>, _>>()
                .map_err(|error| format!("failed to read notes: {error}"))
        }
        _ => {
            let mut statement = connection
                .prepare(
                    "
                    SELECT id, target_type, target_id, content, created_at, updated_at
                    FROM notes
                    ORDER BY created_at DESC
                    ",
                )
                .map_err(|error| format!("failed to prepare notes: {error}"))?;
            let rows = statement
                .query_map([], map_note_row)
                .map_err(|error| format!("failed to list notes: {error}"))?;

            rows.collect::<Result<Vec<_>, _>>()
                .map_err(|error| format!("failed to read notes: {error}"))
        }
    }
}

#[tauri::command]
pub fn update_note(state: State<'_, AppState>, input: UpdateNoteRequest) -> Result<Note, String> {
    let content = normalize_required_text(input.content, "content")?;
    let now = now_rfc3339()?;
    let connection = state
        .db
        .lock()
        .map_err(|_| "database lock was poisoned".to_string())?;

    let changed = connection
        .execute(
            "UPDATE notes SET content = ?2, updated_at = ?3 WHERE id = ?1",
            params![input.id, content, now],
        )
        .map_err(|error| format!("failed to update note: {error}"))?;

    if changed == 0 {
        return Err("note not found".to_string());
    }

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

#[tauri::command]
pub fn list_timeline(
    state: State<'_, AppState>,
    input: Option<ListTimelineRequest>,
) -> Result<Vec<TimelineEntry>, String> {
    let connection = state
        .db
        .lock()
        .map_err(|_| "database lock was poisoned".to_string())?;
    let input = input.unwrap_or_default();
    let mut entries = Vec::new();

    append_excerpt_entries(&connection, &mut entries, input.excerpt_id.as_deref())?;
    append_topic_excerpt_entries(
        &connection,
        &mut entries,
        input.topic_id.as_deref(),
        input.excerpt_id.as_deref(),
    )?;
    append_note_entries(
        &connection,
        &mut entries,
        input.topic_id.as_deref(),
        input.excerpt_id.as_deref(),
    )?;

    entries.sort_by(|left, right| right.occurred_at.cmp(&left.occurred_at));
    Ok(entries)
}

impl Default for ListNotesRequest {
    fn default() -> Self {
        Self {
            target_type: None,
            target_id: None,
        }
    }
}

impl Default for ListTimelineRequest {
    fn default() -> Self {
        Self {
            topic_id: None,
            excerpt_id: None,
        }
    }
}

fn append_excerpt_entries(
    connection: &Connection,
    entries: &mut Vec<TimelineEntry>,
    excerpt_id: Option<&str>,
) -> Result<(), String> {
    if let Some(excerpt_id) = excerpt_id {
        let mut statement = connection
            .prepare(
                "
                SELECT id, quote, reflection, created_at
                FROM excerpts
                WHERE id = ?1
                ",
            )
            .map_err(|error| format!("failed to prepare excerpt timeline: {error}"))?;
        let rows = statement
            .query_map(params![excerpt_id], map_excerpt_timeline_row)
            .map_err(|error| format!("failed to list excerpt timeline: {error}"))?;
        entries.extend(
            rows.collect::<Result<Vec<_>, _>>()
                .map_err(|error| format!("failed to read excerpt timeline: {error}"))?,
        );
    } else {
        let mut statement = connection
            .prepare(
                "
                SELECT id, quote, reflection, created_at
                FROM excerpts
                ",
            )
            .map_err(|error| format!("failed to prepare excerpt timeline: {error}"))?;
        let rows = statement
            .query_map([], map_excerpt_timeline_row)
            .map_err(|error| format!("failed to list excerpt timeline: {error}"))?;
        entries.extend(
            rows.collect::<Result<Vec<_>, _>>()
                .map_err(|error| format!("failed to read excerpt timeline: {error}"))?,
        );
    }

    Ok(())
}

fn append_topic_excerpt_entries(
    connection: &Connection,
    entries: &mut Vec<TimelineEntry>,
    topic_id: Option<&str>,
    excerpt_id: Option<&str>,
) -> Result<(), String> {
    let mut query = "
        SELECT
          topic_excerpts.id,
          topics.title,
          excerpts.quote,
          topic_excerpts.reason,
          topic_excerpts.topic_reflection,
          topic_excerpts.added_at,
          topic_excerpts.excerpt_id,
          topic_excerpts.topic_id
        FROM topic_excerpts
        INNER JOIN topics ON topics.id = topic_excerpts.topic_id
        INNER JOIN excerpts ON excerpts.id = topic_excerpts.excerpt_id
        WHERE 1 = 1
    "
    .to_string();
    let mut params = Vec::new();

    if let Some(topic_id) = topic_id {
        query.push_str(" AND topic_excerpts.topic_id = ?");
        params.push(topic_id.to_string());
    }

    if let Some(excerpt_id) = excerpt_id {
        query.push_str(" AND topic_excerpts.excerpt_id = ?");
        params.push(excerpt_id.to_string());
    }

    let params: Vec<&dyn rusqlite::ToSql> = params
        .iter()
        .map(|value| value as &dyn rusqlite::ToSql)
        .collect();
    let mut statement = connection
        .prepare(&query)
        .map_err(|error| format!("failed to prepare topic timeline: {error}"))?;
    let rows = statement
        .query_map(rusqlite::params_from_iter(params), |row| {
            let id: String = row.get(0)?;
            let topic_title: String = row.get(1)?;
            let quote: String = row.get(2)?;
            let reason: Option<String> = row.get(3)?;
            let topic_reflection: Option<String> = row.get(4)?;
            let added_at: String = row.get(5)?;
            let excerpt_id: String = row.get(6)?;
            let topic_id: String = row.get(7)?;
            let content = topic_reflection.or(reason).or_else(|| Some(quote));

            Ok(TimelineEntry {
                id,
                kind: "topicExcerptAdded".to_string(),
                occurred_at: added_at,
                title: format!("收录到主题：{topic_title}"),
                content,
                target_type: Some("topicExcerpt".to_string()),
                target_id: None,
                related_excerpt_id: Some(excerpt_id),
                related_topic_id: Some(topic_id),
            })
        })
        .map_err(|error| format!("failed to list topic timeline: {error}"))?;

    entries.extend(
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("failed to read topic timeline: {error}"))?,
    );

    Ok(())
}

fn append_note_entries(
    connection: &Connection,
    entries: &mut Vec<TimelineEntry>,
    topic_id: Option<&str>,
    excerpt_id: Option<&str>,
) -> Result<(), String> {
    let mut statement = connection
        .prepare(
            "
            SELECT id, target_type, target_id, content, created_at
            FROM notes
            ORDER BY created_at DESC
            ",
        )
        .map_err(|error| format!("failed to prepare note timeline: {error}"))?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
            ))
        })
        .map_err(|error| format!("failed to list note timeline: {error}"))?;

    for row in rows {
        let (id, target_type, target_id, content, created_at) =
            row.map_err(|error| format!("failed to read note timeline: {error}"))?;
        let related_topic_id = resolve_related_topic_id(connection, &target_type, &target_id)?;
        let related_excerpt_id = resolve_related_excerpt_id(connection, &target_type, &target_id)?;

        if let Some(filter_topic_id) = topic_id {
            if related_topic_id.as_deref() != Some(filter_topic_id) {
                continue;
            }
        }

        if let Some(filter_excerpt_id) = excerpt_id {
            if related_excerpt_id.as_deref() != Some(filter_excerpt_id) {
                continue;
            }
        }

        entries.push(TimelineEntry {
            id,
            kind: "noteCreated".to_string(),
            occurred_at: created_at,
            title: note_title(connection, &target_type, &target_id)?,
            content: Some(content),
            target_type: Some(target_type),
            target_id: Some(target_id),
            related_excerpt_id,
            related_topic_id,
        });
    }

    Ok(())
}

fn map_excerpt_timeline_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<TimelineEntry> {
    let id: String = row.get(0)?;
    let quote: String = row.get(1)?;
    let reflection: Option<String> = row.get(2)?;
    let created_at: String = row.get(3)?;

    Ok(TimelineEntry {
        id: id.clone(),
        kind: "excerptCreated".to_string(),
        occurred_at: created_at,
        title: "新增摘抄".to_string(),
        content: reflection.or_else(|| Some(quote)),
        target_type: Some("excerpt".to_string()),
        target_id: Some(id.clone()),
        related_excerpt_id: Some(id),
        related_topic_id: None,
    })
}

fn get_note_by_id(connection: &Connection, id: &str) -> Result<Note, String> {
    connection
        .query_row(
            "
            SELECT id, target_type, target_id, content, created_at, updated_at
            FROM notes
            WHERE id = ?1
            ",
            params![id],
            map_note_row,
        )
        .map_err(|error| format!("failed to find note: {error}"))
}

fn map_note_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Note> {
    Ok(Note {
        id: row.get(0)?,
        target_type: row.get(1)?,
        target_id: row.get(2)?,
        content: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
}

fn note_title(
    connection: &Connection,
    target_type: &str,
    target_id: &str,
) -> Result<String, String> {
    match target_type {
        "excerpt" => {
            let quote: String = connection
                .query_row(
                    "SELECT quote FROM excerpts WHERE id = ?1",
                    params![target_id],
                    |row| row.get(0),
                )
                .map_err(|error| format!("failed to find note target excerpt: {error}"))?;
            Ok(format!("摘抄笔记：{}", truncate(&quote, 18)))
        }
        "topic" => {
            let title: String = connection
                .query_row(
                    "SELECT title FROM topics WHERE id = ?1",
                    params![target_id],
                    |row| row.get(0),
                )
                .map_err(|error| format!("failed to find note target topic: {error}"))?;
            Ok(format!("主题笔记：{title}"))
        }
        "topicNode" => {
            let title: String = connection
                .query_row(
                    "SELECT title FROM topic_nodes WHERE id = ?1",
                    params![target_id],
                    |row| row.get(0),
                )
                .map_err(|error| format!("failed to find note target topic node: {error}"))?;
            Ok(format!("子主题笔记：{title}"))
        }
        "topicExcerpt" => Ok("主题摘抄笔记".to_string()),
        _ => Err("invalid note target type".to_string()),
    }
}

fn resolve_related_topic_id(
    connection: &Connection,
    target_type: &str,
    target_id: &str,
) -> Result<Option<String>, String> {
    match target_type {
        "topic" => Ok(Some(target_id.to_string())),
        "topicNode" => query_optional_string(
            connection,
            "SELECT topic_id FROM topic_nodes WHERE id = ?1",
            target_id,
        ),
        "topicExcerpt" => query_optional_string(
            connection,
            "SELECT topic_id FROM topic_excerpts WHERE id = ?1",
            target_id,
        ),
        "excerpt" => Ok(None),
        _ => Err("invalid note target type".to_string()),
    }
}

fn resolve_related_excerpt_id(
    connection: &Connection,
    target_type: &str,
    target_id: &str,
) -> Result<Option<String>, String> {
    match target_type {
        "excerpt" => Ok(Some(target_id.to_string())),
        "topicExcerpt" => query_optional_string(
            connection,
            "SELECT excerpt_id FROM topic_excerpts WHERE id = ?1",
            target_id,
        ),
        "topic" | "topicNode" => Ok(None),
        _ => Err("invalid note target type".to_string()),
    }
}

fn query_optional_string(
    connection: &Connection,
    query: &str,
    id: &str,
) -> Result<Option<String>, String> {
    match connection.query_row(query, params![id], |row| row.get(0)) {
        Ok(value) => Ok(Some(value)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(error) => Err(format!("failed to resolve timeline relation: {error}")),
    }
}

fn ensure_target_exists(
    connection: &Connection,
    target_type: &str,
    target_id: &str,
) -> Result<(), String> {
    let table = match target_type {
        "excerpt" => "excerpts",
        "topic" => "topics",
        "topicNode" => "topic_nodes",
        "topicExcerpt" => "topic_excerpts",
        _ => return Err("invalid note target type".to_string()),
    };
    let query = format!("SELECT EXISTS(SELECT 1 FROM {table} WHERE id = ?1)");
    let exists = connection
        .query_row(&query, params![target_id], |row| row.get::<_, bool>(0))
        .map_err(|error| format!("failed to check note target: {error}"))?;

    if exists {
        Ok(())
    } else {
        Err("note target not found".to_string())
    }
}

fn validate_target_type(value: &str) -> Result<String, String> {
    match value {
        "excerpt" | "topic" | "topicNode" | "topicExcerpt" => Ok(value.to_string()),
        _ => Err("targetType must be excerpt, topic, topicNode, or topicExcerpt".to_string()),
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

fn truncate(value: &str, max_chars: usize) -> String {
    let mut chars = value.chars();
    let truncated: String = chars.by_ref().take(max_chars).collect();
    if chars.next().is_some() {
        format!("{truncated}...")
    } else {
        truncated
    }
}

fn now_rfc3339() -> Result<String, String> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|error| format!("failed to format timestamp: {error}"))
}
