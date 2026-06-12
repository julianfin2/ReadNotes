use rusqlite::{params, params_from_iter, Connection, ToSql};
use serde::{Deserialize, Serialize};
use tauri::State;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use uuid::Uuid;

use crate::book::{resolve_excerpt_source, ResolveExcerptSourceInput};
use crate::db::{rebuild_excerpt_search_index, AppState};
use crate::tag::{list_tags_for_excerpt, replace_excerpt_tags, Tag};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Excerpt {
    pub id: String,
    pub quote: String,
    pub reflection: Option<String>,
    pub book_id: Option<String>,
    pub chapter_id: Option<String>,
    pub book_title: Option<String>,
    pub chapter_title: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub tags: Vec<Tag>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateExcerptRequest {
    pub quote: String,
    pub reflection: Option<String>,
    pub book_id: Option<String>,
    pub chapter_id: Option<String>,
    pub book_title: Option<String>,
    pub chapter_title: Option<String>,
    pub tag_names: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateExcerptRequest {
    pub id: String,
    pub quote: String,
    pub reflection: Option<String>,
    pub book_id: Option<String>,
    pub chapter_id: Option<String>,
    pub book_title: Option<String>,
    pub chapter_title: Option<String>,
    pub tag_names: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListExcerptsRequest {
    pub search: Option<String>,
    pub tag_name: Option<String>,
    pub sort_by: Option<String>,
    pub sort_direction: Option<String>,
}

#[tauri::command]
pub fn create_excerpt(
    state: State<'_, AppState>,
    input: CreateExcerptRequest,
) -> Result<Excerpt, String> {
    let quote = normalize_required_text(input.quote.clone(), "quote")?;
    let reflection = empty_to_none(input.reflection.clone());
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
    let (book_id, chapter_id) = resolve_excerpt_source(
        &transaction,
        ResolveExcerptSourceInput {
            book_id: empty_to_none(input.book_id).as_deref(),
            chapter_id: empty_to_none(input.chapter_id).as_deref(),
            book_title: empty_to_none(input.book_title).as_deref(),
            chapter_title: empty_to_none(input.chapter_title).as_deref(),
        },
    )?;

    transaction
        .execute(
            "
            INSERT INTO excerpts (
              id, quote, reflection, book_id, chapter_id, created_at, updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ",
            params![
                id,
                quote,
                reflection.as_deref(),
                book_id.as_deref(),
                chapter_id.as_deref(),
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
pub fn list_excerpts(
    state: State<'_, AppState>,
    input: Option<ListExcerptsRequest>,
) -> Result<Vec<Excerpt>, String> {
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
                excerpts.rowid IN (
                    SELECT rowid FROM excerpt_search WHERE excerpt_search MATCH ?
                )
                OR lower(COALESCE(excerpts.quote, '')) LIKE lower(?)
                OR lower(COALESCE(excerpts.reflection, '')) LIKE lower(?)
                OR lower(COALESCE(books.title, '')) LIKE lower(?)
                OR lower(COALESCE(book_chapters.title, '')) LIKE lower(?)
            )"
            .to_string(),
        );
        parameter_values.push(to_fts_query(&search));
        parameter_values.push(format!("%{search}%"));
        parameter_values.push(format!("%{search}%"));
        parameter_values.push(format!("%{search}%"));
        parameter_values.push(format!("%{search}%"));
    }

    if let Some(tag_name) = normalize_optional_text(input.tag_name) {
        clauses.push(
            "EXISTS (
                SELECT 1
                FROM excerpt_tags
                INNER JOIN tags ON tags.id = excerpt_tags.tag_id
                WHERE excerpt_tags.excerpt_id = excerpts.id
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
    let order_clause =
        build_order_clause(input.sort_by.as_deref(), input.sort_direction.as_deref())?;
    let query = format!(
        "
        SELECT
          excerpts.id,
          excerpts.quote,
          excerpts.reflection,
          excerpts.book_id,
          excerpts.chapter_id,
          books.title AS book_title,
          book_chapters.title AS chapter_title,
          excerpts.created_at,
          excerpts.updated_at
        FROM excerpts
        LEFT JOIN books ON books.id = excerpts.book_id
        LEFT JOIN book_chapters ON book_chapters.id = excerpts.chapter_id
        {where_clause}
        {order_clause}
        "
    );
    let parameters: Vec<&dyn ToSql> = parameter_values
        .iter()
        .map(|value| value as &dyn ToSql)
        .collect();

    let mut statement = connection
        .prepare(&query)
        .map_err(|error| format!("failed to prepare excerpt list: {error}"))?;

    let rows = statement
        .query_map(params_from_iter(parameters), map_excerpt_row)
        .map_err(|error| format!("failed to list excerpts: {error}"))?;

    let mut excerpts = rows
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to read excerpts: {error}"))?;

    for excerpt in &mut excerpts {
        excerpt.tags = list_tags_for_excerpt(&connection, &excerpt.id)?;
    }

    Ok(excerpts)
}

impl Default for ListExcerptsRequest {
    fn default() -> Self {
        Self {
            search: None,
            tag_name: None,
            sort_by: Some("createdAt".to_string()),
            sort_direction: Some("desc".to_string()),
        }
    }
}

#[tauri::command]
pub fn update_excerpt(
    state: State<'_, AppState>,
    input: UpdateExcerptRequest,
) -> Result<Excerpt, String> {
    let mut connection = state
        .db
        .lock()
        .map_err(|_| "database lock was poisoned".to_string())?;

    match update_excerpt_inner(&mut connection, &input) {
        Ok(excerpt) => Ok(excerpt),
        Err(error) if is_malformed_database_error(&error) => {
            rebuild_excerpt_search_index(&connection)?;
            update_excerpt_inner(&mut connection, &input)
        }
        Err(error) => Err(error),
    }
}

fn update_excerpt_inner(
    connection: &mut Connection,
    input: &UpdateExcerptRequest,
) -> Result<Excerpt, String> {
    let quote = normalize_required_text(input.quote.clone(), "quote")?;
    let reflection = empty_to_none(input.reflection.clone());
    let now = now_rfc3339()?;

    let transaction = connection
        .transaction()
        .map_err(|error| format!("failed to start excerpt update transaction: {error}"))?;
    let (book_id, chapter_id) = resolve_excerpt_source(
        &transaction,
        ResolveExcerptSourceInput {
            book_id: empty_to_none(input.book_id.clone()).as_deref(),
            chapter_id: empty_to_none(input.chapter_id.clone()).as_deref(),
            book_title: empty_to_none(input.book_title.clone()).as_deref(),
            chapter_title: empty_to_none(input.chapter_title.clone()).as_deref(),
        },
    )?;

    let changed = transaction
        .execute(
            "
            UPDATE excerpts
            SET
              quote = ?2,
              reflection = ?3,
              book_id = ?4,
              chapter_id = ?5,
              updated_at = ?6
            WHERE id = ?1
            ",
            params![
                input.id.as_str(),
                quote,
                reflection.as_deref(),
                book_id.as_deref(),
                chapter_id.as_deref(),
                now
            ],
        )
        .map_err(|error| format!("failed to update excerpt: {error}"))?;

    if changed == 0 {
        return Err("excerpt not found".to_string());
    }

    if let Some(tag_names) = input.tag_names.clone() {
        replace_excerpt_tags(&transaction, &input.id, tag_names)?;
    }

    transaction
        .commit()
        .map_err(|error| format!("failed to save excerpt update: {error}"))?;

    get_excerpt_by_id(connection, &input.id)
}

fn is_malformed_database_error(error: &str) -> bool {
    error
        .to_lowercase()
        .contains("database disk image is malformed")
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

pub fn get_excerpt_by_id(connection: &Connection, id: &str) -> Result<Excerpt, String> {
    let mut excerpt = connection
        .query_row(
            "
            SELECT
              excerpts.id,
              excerpts.quote,
              excerpts.reflection,
              excerpts.book_id,
              excerpts.chapter_id,
              books.title AS book_title,
              book_chapters.title AS chapter_title,
              excerpts.created_at,
              excerpts.updated_at
            FROM excerpts
            LEFT JOIN books ON books.id = excerpts.book_id
            LEFT JOIN book_chapters ON book_chapters.id = excerpts.chapter_id
            WHERE excerpts.id = ?1
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
        book_id: row.get(3)?,
        chapter_id: row.get(4)?,
        book_title: row.get(5)?,
        chapter_title: row.get(6)?,
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

fn normalize_optional_text(value: Option<String>) -> Option<String> {
    empty_to_none(value)
}

fn build_order_clause(
    sort_by: Option<&str>,
    sort_direction: Option<&str>,
) -> Result<String, String> {
    let column = match sort_by.unwrap_or("createdAt") {
        "createdAt" => "excerpts.created_at",
        "updatedAt" => "excerpts.updated_at",
        _ => return Err("sortBy must be createdAt or updatedAt".to_string()),
    };
    let direction = match sort_direction.unwrap_or("desc") {
        "asc" => "ASC",
        "desc" => "DESC",
        _ => return Err("sortDirection must be asc or desc".to_string()),
    };

    Ok(format!(
        "ORDER BY {column} {direction}, excerpts.created_at DESC"
    ))
}

fn to_fts_query(value: &str) -> String {
    value
        .split_whitespace()
        .map(|term| {
            let escaped = term.replace('"', "\"\"");
            format!("\"{escaped}\"")
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn now_rfc3339() -> Result<String, String> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|error| format!("failed to format timestamp: {error}"))
}
