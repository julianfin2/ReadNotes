use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use tauri::State;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use uuid::Uuid;

use crate::db::AppState;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Book {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
    pub chapter_count: i64,
    pub excerpt_count: i64,
    pub chapters: Vec<BookChapter>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookChapter {
    pub id: String,
    pub book_id: String,
    pub title: String,
    pub sort_order: i64,
    pub created_at: String,
    pub updated_at: String,
    pub excerpt_count: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateBookRequest {
    pub title: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateBookRequest {
    pub id: String,
    pub title: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateBookChapterRequest {
    pub book_id: String,
    pub title: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateBookChapterRequest {
    pub id: String,
    pub title: String,
}

pub struct ResolveExcerptSourceInput<'a> {
    pub book_id: Option<&'a str>,
    pub chapter_id: Option<&'a str>,
    pub book_title: Option<&'a str>,
    pub chapter_title: Option<&'a str>,
}

#[tauri::command]
pub fn list_books(state: State<'_, AppState>) -> Result<Vec<Book>, String> {
    let connection = state
        .db
        .lock()
        .map_err(|_| "database lock was poisoned".to_string())?;

    list_all_books(&connection)
}

#[tauri::command]
pub fn create_book(state: State<'_, AppState>, input: CreateBookRequest) -> Result<Book, String> {
    let mut connection = state
        .db
        .lock()
        .map_err(|_| "database lock was poisoned".to_string())?;

    let transaction = connection
        .transaction()
        .map_err(|error| format!("failed to start book transaction: {error}"))?;
    let book_id = find_or_create_book(&transaction, &input.title)?;
    transaction
        .commit()
        .map_err(|error| format!("failed to save book: {error}"))?;

    get_book_by_id(&connection, &book_id)
}

#[tauri::command]
pub fn update_book(state: State<'_, AppState>, input: UpdateBookRequest) -> Result<Book, String> {
    let title = normalize_required_text(input.title, "book title")?;
    let now = now_rfc3339()?;
    let connection = state
        .db
        .lock()
        .map_err(|_| "database lock was poisoned".to_string())?;

    let duplicate: Option<String> = connection
        .query_row(
            "SELECT id FROM books WHERE lower(title) = lower(?1) AND id != ?2",
            params![title, input.id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| format!("failed to check duplicate book: {error}"))?;
    if duplicate.is_some() {
        return Err("book title already exists".to_string());
    }

    let changed = connection
        .execute(
            "UPDATE books SET title = ?2, updated_at = ?3 WHERE id = ?1",
            params![input.id, title, now],
        )
        .map_err(|error| format!("failed to update book: {error}"))?;
    if changed == 0 {
        return Err("book not found".to_string());
    }

    get_book_by_id(&connection, &input.id)
}

#[tauri::command]
pub fn delete_book(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let connection = state
        .db
        .lock()
        .map_err(|_| "database lock was poisoned".to_string())?;

    let changed = connection
        .execute("DELETE FROM books WHERE id = ?1", params![id])
        .map_err(|error| format!("failed to delete book: {error}"))?;
    if changed == 0 {
        return Err("book not found".to_string());
    }

    Ok(())
}

#[tauri::command]
pub fn create_book_chapter(
    state: State<'_, AppState>,
    input: CreateBookChapterRequest,
) -> Result<BookChapter, String> {
    let mut connection = state
        .db
        .lock()
        .map_err(|_| "database lock was poisoned".to_string())?;

    let transaction = connection
        .transaction()
        .map_err(|error| format!("failed to start chapter transaction: {error}"))?;
    let chapter_id = find_or_create_chapter(&transaction, &input.book_id, &input.title)?;
    transaction
        .commit()
        .map_err(|error| format!("failed to save chapter: {error}"))?;

    get_chapter_by_id(&connection, &chapter_id)
}

#[tauri::command]
pub fn update_book_chapter(
    state: State<'_, AppState>,
    input: UpdateBookChapterRequest,
) -> Result<BookChapter, String> {
    let title = normalize_required_text(input.title, "chapter title")?;
    let now = now_rfc3339()?;
    let connection = state
        .db
        .lock()
        .map_err(|_| "database lock was poisoned".to_string())?;

    let book_id: String = connection
        .query_row(
            "SELECT book_id FROM book_chapters WHERE id = ?1",
            params![input.id],
            |row| row.get(0),
        )
        .map_err(|error| format!("failed to find chapter: {error}"))?;

    let duplicate: Option<String> = connection
        .query_row(
            "
            SELECT id FROM book_chapters
            WHERE book_id = ?1 AND lower(title) = lower(?2) AND id != ?3
            ",
            params![book_id, title, input.id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| format!("failed to check duplicate chapter: {error}"))?;
    if duplicate.is_some() {
        return Err("chapter title already exists".to_string());
    }

    connection
        .execute(
            "UPDATE book_chapters SET title = ?2, updated_at = ?3 WHERE id = ?1",
            params![input.id, title, now],
        )
        .map_err(|error| format!("failed to update chapter: {error}"))?;

    get_chapter_by_id(&connection, &input.id)
}

#[tauri::command]
pub fn delete_book_chapter(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let connection = state
        .db
        .lock()
        .map_err(|_| "database lock was poisoned".to_string())?;

    let changed = connection
        .execute("DELETE FROM book_chapters WHERE id = ?1", params![id])
        .map_err(|error| format!("failed to delete chapter: {error}"))?;
    if changed == 0 {
        return Err("chapter not found".to_string());
    }

    Ok(())
}

pub fn resolve_excerpt_source(
    connection: &Connection,
    input: ResolveExcerptSourceInput<'_>,
) -> Result<(Option<String>, Option<String>), String> {
    let book_id = match normalize_optional_text(input.book_id) {
        Some(book_id) => Some(require_book(connection, &book_id)?),
        None => match normalize_optional_text(input.book_title) {
            Some(book_title) => Some(find_or_create_book(connection, &book_title)?),
            None => None,
        },
    };

    let chapter_id = match (
        book_id.as_deref(),
        normalize_optional_text(input.chapter_id),
    ) {
        (Some(book_id), Some(chapter_id)) => {
            Some(require_chapter(connection, book_id, &chapter_id)?)
        }
        _ => match (
            book_id.as_deref(),
            normalize_optional_text(input.chapter_title),
        ) {
            (Some(book_id), Some(chapter_title)) => {
                Some(find_or_create_chapter(connection, book_id, &chapter_title)?)
            }
            _ => None,
        },
    };

    Ok((book_id, chapter_id))
}

fn list_all_books(connection: &Connection) -> Result<Vec<Book>, String> {
    let mut statement = connection
        .prepare(
            "
            SELECT
              books.id,
              books.title,
              books.created_at,
              books.updated_at,
              COUNT(DISTINCT book_chapters.id) AS chapter_count,
              COUNT(DISTINCT excerpts.id) AS excerpt_count
            FROM books
            LEFT JOIN book_chapters ON book_chapters.book_id = books.id
            LEFT JOIN excerpts ON excerpts.book_id = books.id
            GROUP BY books.id
            ORDER BY lower(books.title) ASC
            ",
        )
        .map_err(|error| format!("failed to prepare book list: {error}"))?;

    let rows = statement
        .query_map([], map_book_row)
        .map_err(|error| format!("failed to list books: {error}"))?;
    let mut books = rows
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to read books: {error}"))?;

    for book in &mut books {
        book.chapters = list_chapters_for_book(connection, &book.id)?;
    }

    Ok(books)
}

fn list_chapters_for_book(
    connection: &Connection,
    book_id: &str,
) -> Result<Vec<BookChapter>, String> {
    let mut statement = connection
        .prepare(
            "
            SELECT
              book_chapters.id,
              book_chapters.book_id,
              book_chapters.title,
              book_chapters.sort_order,
              book_chapters.created_at,
              book_chapters.updated_at,
              COUNT(excerpts.id) AS excerpt_count
            FROM book_chapters
            LEFT JOIN excerpts ON excerpts.chapter_id = book_chapters.id
            WHERE book_chapters.book_id = ?1
            GROUP BY book_chapters.id
            ORDER BY book_chapters.sort_order ASC, lower(book_chapters.title) ASC
            ",
        )
        .map_err(|error| format!("failed to prepare chapter list: {error}"))?;
    let rows = statement
        .query_map(params![book_id], map_chapter_row)
        .map_err(|error| format!("failed to list chapters: {error}"))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to read chapters: {error}"))
}

fn get_book_by_id(connection: &Connection, id: &str) -> Result<Book, String> {
    let mut book = connection
        .query_row(
            "
            SELECT
              books.id,
              books.title,
              books.created_at,
              books.updated_at,
              COUNT(DISTINCT book_chapters.id) AS chapter_count,
              COUNT(DISTINCT excerpts.id) AS excerpt_count
            FROM books
            LEFT JOIN book_chapters ON book_chapters.book_id = books.id
            LEFT JOIN excerpts ON excerpts.book_id = books.id
            WHERE books.id = ?1
            GROUP BY books.id
            ",
            params![id],
            map_book_row,
        )
        .map_err(|error| format!("failed to find book: {error}"))?;
    book.chapters = list_chapters_for_book(connection, &book.id)?;
    Ok(book)
}

fn get_chapter_by_id(connection: &Connection, id: &str) -> Result<BookChapter, String> {
    connection
        .query_row(
            "
            SELECT
              book_chapters.id,
              book_chapters.book_id,
              book_chapters.title,
              book_chapters.sort_order,
              book_chapters.created_at,
              book_chapters.updated_at,
              COUNT(excerpts.id) AS excerpt_count
            FROM book_chapters
            LEFT JOIN excerpts ON excerpts.chapter_id = book_chapters.id
            WHERE book_chapters.id = ?1
            GROUP BY book_chapters.id
            ",
            params![id],
            map_chapter_row,
        )
        .map_err(|error| format!("failed to find chapter: {error}"))
}

fn find_or_create_book(connection: &Connection, title: &str) -> Result<String, String> {
    let title = normalize_required_text(title.to_string(), "book title")?;
    if let Some(id) = connection
        .query_row(
            "SELECT id FROM books WHERE lower(title) = lower(?1)",
            params![title],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| format!("failed to find book by title: {error}"))?
    {
        return Ok(id);
    }

    let id = Uuid::new_v4().to_string();
    let now = now_rfc3339()?;
    connection
        .execute(
            "INSERT INTO books (id, title, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
            params![id, title, now, now],
        )
        .map_err(|error| format!("failed to create book: {error}"))?;

    Ok(id)
}

fn require_book(connection: &Connection, id: &str) -> Result<String, String> {
    let exists = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM books WHERE id = ?1)",
            params![id],
            |row| row.get::<_, bool>(0),
        )
        .map_err(|error| format!("failed to check book: {error}"))?;
    if exists {
        Ok(id.to_string())
    } else {
        Err("book not found".to_string())
    }
}

fn require_chapter(connection: &Connection, book_id: &str, id: &str) -> Result<String, String> {
    let exists = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM book_chapters WHERE id = ?1 AND book_id = ?2)",
            params![id, book_id],
            |row| row.get::<_, bool>(0),
        )
        .map_err(|error| format!("failed to check chapter: {error}"))?;
    if exists {
        Ok(id.to_string())
    } else {
        Err("chapter not found for selected book".to_string())
    }
}

fn find_or_create_chapter(
    connection: &Connection,
    book_id: &str,
    title: &str,
) -> Result<String, String> {
    let title = normalize_required_text(title.to_string(), "chapter title")?;
    if let Some(id) = connection
        .query_row(
            "
            SELECT id FROM book_chapters
            WHERE book_id = ?1 AND lower(title) = lower(?2)
            ",
            params![book_id, title],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| format!("failed to find chapter by title: {error}"))?
    {
        return Ok(id);
    }

    let sort_order = connection
        .query_row(
            "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM book_chapters WHERE book_id = ?1",
            params![book_id],
            |row| row.get::<_, i64>(0),
        )
        .map_err(|error| format!("failed to calculate chapter order: {error}"))?;
    let id = Uuid::new_v4().to_string();
    let now = now_rfc3339()?;
    connection
        .execute(
            "
            INSERT INTO book_chapters (
              id, book_id, title, sort_order, created_at, updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            ",
            params![id, book_id, title, sort_order, now, now],
        )
        .map_err(|error| format!("failed to create chapter: {error}"))?;

    Ok(id)
}

fn map_book_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Book> {
    Ok(Book {
        id: row.get(0)?,
        title: row.get(1)?,
        created_at: row.get(2)?,
        updated_at: row.get(3)?,
        chapter_count: row.get(4)?,
        excerpt_count: row.get(5)?,
        chapters: Vec::new(),
    })
}

fn map_chapter_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<BookChapter> {
    Ok(BookChapter {
        id: row.get(0)?,
        book_id: row.get(1)?,
        title: row.get(2)?,
        sort_order: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
        excerpt_count: row.get(6)?,
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

fn normalize_optional_text(value: Option<&str>) -> Option<String> {
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
