// Documentation database operations
// Handles creation, opening, and basic CRUD for docs databases

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::Utc;
use rusqlite::Connection;

use crate::schema_docs::{self, DocType};

/// Open or create a documentation database
pub fn open_docs_db(path: &Path) -> Result<Connection> {
    let conn = Connection::open(path)
        .with_context(|| format!("Failed to open docs database at {:?}", path))?;

    // Enable foreign keys
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;

    Ok(conn)
}

/// Create a new documentation database with schema
pub fn create_docs_db(
    path: &Path,
    project_name: &str,
    doc_type: DocType,
) -> Result<Connection> {
    let conn = open_docs_db(path)?;

    // Initialize schema
    schema_docs::init_docs_db(&conn)?;

    // Set metadata
    let now = Utc::now().format("%Y-%m-%dT%H:%M:%S%.6f").to_string();
    schema_docs::set_meta(&conn, "project_name", project_name)?;
    schema_docs::set_meta(&conn, "doc_type", doc_type.as_str())?;
    schema_docs::set_meta(&conn, "created_at", &now)?;
    schema_docs::set_meta(&conn, "version", "1.0.0")?;

    Ok(conn)
}

/// Find docs database in project directory
pub fn find_docs_db(project_root: &Path) -> Option<PathBuf> {
    // Look for *_architecture.db, *_framework.db, *_guide.db, *_api.db, *_spec.db
    let patterns = ["architecture", "framework", "guide", "api", "spec"];

    if let Ok(entries) = std::fs::read_dir(project_root) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.ends_with(".db") {
                    for pattern in &patterns {
                        if name.ends_with(&format!("_{}.db", pattern)) {
                            // Verify this is a valid docs database with expected schema
                            if let Ok(conn) = open_docs_db(&path) {
                                if is_valid_docs_db(&conn) {
                                    return Some(path);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

/// Get docs database info
#[derive(Debug)]
pub struct DocsDbInfo {
    pub path: PathBuf,
    pub project_name: String,
    pub doc_type: String,
    pub created_at: Option<String>,
    pub version: Option<String>,
    pub section_count: i64,
    pub term_count: i64,
    pub imported_from: Option<String>,
}

/// Check if database has the expected schema (meta or metadata table exists, plus sections)
pub fn is_valid_docs_db(conn: &Connection) -> bool {
    // Check for either 'meta' (new schema) or 'metadata' (old doc-orchestrator schema)
    // Also verify sections table exists
    let has_meta = conn.query_row(
        "SELECT 1 FROM sqlite_master WHERE type='table' AND name='meta'",
        [],
        |_| Ok(()),
    ).is_ok();

    let has_metadata = conn.query_row(
        "SELECT 1 FROM sqlite_master WHERE type='table' AND name='metadata'",
        [],
        |_| Ok(()),
    ).is_ok();

    let has_sections = conn.query_row(
        "SELECT 1 FROM sqlite_master WHERE type='table' AND name='sections'",
        [],
        |_| Ok(()),
    ).is_ok();

    (has_meta || has_metadata) && has_sections
}

/// Check if database uses 'meta' table (new schema) or 'metadata' table (old schema)
fn has_meta_table(conn: &Connection) -> bool {
    conn.query_row(
        "SELECT 1 FROM sqlite_master WHERE type='table' AND name='meta'",
        [],
        |_| Ok(()),
    ).is_ok()
}

/// Get metadata value, trying both new 'meta' and old 'metadata' table
fn get_meta_compat(conn: &Connection, key: &str) -> Result<Option<String>> {
    if has_meta_table(conn) {
        schema_docs::get_meta(conn, key)
    } else {
        // Try old 'metadata' table
        let result: Result<String, _> = conn.query_row(
            "SELECT value FROM metadata WHERE key = ?1",
            [key],
            |row| row.get(0),
        );
        match result {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}

/// Get info about a docs database
pub fn get_docs_info(conn: &Connection) -> Result<DocsDbInfo> {
    // First check if this is a valid docs database
    if !is_valid_docs_db(conn) {
        anyhow::bail!("Database does not have expected schema.");
    }

    let project_name = get_meta_compat(conn, "project_name")?
        .or_else(|| get_meta_compat(conn, "title").ok().flatten())
        .unwrap_or_else(|| "unknown".to_string());
    let doc_type = get_meta_compat(conn, "doc_type")?
        .unwrap_or_else(|| "unknown".to_string());
    let created_at = get_meta_compat(conn, "created_at")?;
    let version = get_meta_compat(conn, "version")?;
    let imported_from = get_meta_compat(conn, "imported_from")?
        .or_else(|| get_meta_compat(conn, "imported_at").ok().flatten());

    let section_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sections",
        [],
        |row| row.get(0),
    )?;

    // terminology table might not exist in older schemas
    let term_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM terminology",
        [],
        |row| row.get(0),
    ).unwrap_or(0);

    Ok(DocsDbInfo {
        path: PathBuf::new(), // Caller should set this
        project_name,
        doc_type,
        created_at,
        version,
        section_count,
        term_count,
        imported_from,
    })
}

/// Section data structure
#[derive(Debug, Clone)]
pub struct Section {
    pub id: i64,
    pub section_id: String,
    pub title: String,
    pub parent_id: Option<String>,
    pub level: i32,
    pub sort_order: i32,
    pub content: String,
    pub word_count: i32,
    pub generated: bool,
    pub source_file: Option<String>,
}

/// Insert a new section
pub fn insert_section(
    conn: &Connection,
    section_id: &str,
    title: &str,
    parent_id: Option<&str>,
    level: i32,
    sort_order: i32,
    content: &str,
    generated: bool,
    source_file: Option<&str>,
) -> Result<i64> {
    let word_count = content.split_whitespace().count() as i32;
    let now = Utc::now().format("%Y-%m-%dT%H:%M:%S%.6f").to_string();

    conn.execute(
        r#"INSERT INTO sections
           (section_id, title, parent_id, level, sort_order, content, word_count, generated, source_file, created_at, updated_at)
           VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?10)"#,
        rusqlite::params![
            section_id,
            title,
            parent_id,
            level,
            sort_order,
            content,
            word_count,
            generated as i32,
            source_file,
            now,
        ],
    )?;

    Ok(conn.last_insert_rowid())
}

/// Update an existing section's content
pub fn update_section(
    conn: &Connection,
    section_id: &str,
    title: &str,
    content: &str,
    source_file: Option<&str>,
) -> Result<()> {
    let word_count = content.split_whitespace().count() as i32;
    let now = Utc::now().format("%Y-%m-%dT%H:%M:%S%.6f").to_string();

    conn.execute(
        r#"UPDATE sections
           SET title = ?1, content = ?2, word_count = ?3, source_file = ?4, updated_at = ?5
           WHERE section_id = ?6"#,
        rusqlite::params![title, content, word_count, source_file, now, section_id],
    )?;

    Ok(())
}

/// Delete all generated sections (for refresh)
pub fn delete_generated_sections(conn: &Connection) -> Result<usize> {
    let count = conn.execute("DELETE FROM sections WHERE generated = 1", [])?;
    Ok(count)
}

/// Get count of generated vs manual sections
pub fn get_section_counts(conn: &Connection) -> Result<(i64, i64)> {
    let generated: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sections WHERE generated = 1",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let manual: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sections WHERE generated = 0 OR generated IS NULL",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    Ok((generated, manual))
}

/// Check if sections table has the 'generated' column (new schema)
fn has_generated_column(conn: &Connection) -> bool {
    conn.query_row(
        "SELECT generated FROM sections LIMIT 1",
        [],
        |_| Ok(()),
    )
    .is_ok()
}

/// Get all sections ordered by sort_order
pub fn get_all_sections(conn: &Connection) -> Result<Vec<Section>> {
    // Check if we have the new schema with generated/source_file columns
    if has_generated_column(conn) {
        let mut stmt = conn.prepare(
            "SELECT id, section_id, title, parent_id, level, sort_order, content, word_count, generated, source_file
             FROM sections ORDER BY sort_order"
        )?;

        let sections = stmt.query_map([], |row| {
            Ok(Section {
                id: row.get(0)?,
                section_id: row.get(1)?,
                title: row.get(2)?,
                parent_id: row.get(3)?,
                level: row.get(4)?,
                sort_order: row.get(5)?,
                content: row.get(6)?,
                word_count: row.get(7)?,
                generated: row.get::<_, i32>(8)? != 0,
                source_file: row.get(9)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(sections)
    } else {
        // Older schema without generated/source_file
        let mut stmt = conn.prepare(
            "SELECT id, section_id, title, parent_id, level, sort_order, content, word_count
             FROM sections ORDER BY sort_order"
        )?;

        let sections = stmt.query_map([], |row| {
            Ok(Section {
                id: row.get(0)?,
                section_id: row.get(1)?,
                title: row.get(2)?,
                parent_id: row.get(3)?,
                level: row.get(4)?,
                sort_order: row.get(5)?,
                content: row.get(6)?,
                word_count: row.get(7)?,
                generated: false,
                source_file: None,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(sections)
    }
}

/// Terminology entry
#[derive(Debug, Clone)]
pub struct TermEntry {
    pub id: i64,
    pub canonical: String,
    pub variants: Vec<String>,
    pub definition: Option<String>,
    pub category: Option<String>,
    pub first_used_in: Option<String>,
}

/// Insert a terminology entry
pub fn insert_term(
    conn: &Connection,
    canonical: &str,
    definition: Option<&str>,
    category: Option<&str>,
    variants: &[&str],
) -> Result<i64> {
    let variants_json = serde_json::to_string(variants)?;

    conn.execute(
        r#"INSERT INTO terminology (canonical, definition, category, variants)
           VALUES (?1, ?2, ?3, ?4)"#,
        rusqlite::params![canonical, definition, category, variants_json],
    )?;

    Ok(conn.last_insert_rowid())
}

/// Get all terminology entries
pub fn get_all_terms(conn: &Connection) -> Result<Vec<TermEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, canonical, variants, definition, category, first_used_in FROM terminology ORDER BY canonical"
    )?;

    let terms = stmt.query_map([], |row| {
        let variants_json: String = row.get(2)?;
        let variants: Vec<String> = serde_json::from_str(&variants_json).unwrap_or_default();

        Ok(TermEntry {
            id: row.get(0)?,
            canonical: row.get(1)?,
            variants,
            definition: row.get(3)?,
            category: row.get(4)?,
            first_used_in: row.get(5)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;

    Ok(terms)
}

/// Search sections using FTS5
pub fn search_sections(conn: &Connection, query: &str) -> Result<Vec<Section>> {
    if has_generated_column(conn) {
        let mut stmt = conn.prepare(
            r#"SELECT s.id, s.section_id, s.title, s.parent_id, s.level, s.sort_order,
                      s.content, s.word_count, s.generated, s.source_file
               FROM sections s
               JOIN sections_fts fts ON s.id = fts.rowid
               WHERE sections_fts MATCH ?1
               ORDER BY rank"#
        )?;

        let sections = stmt.query_map([query], |row| {
            Ok(Section {
                id: row.get(0)?,
                section_id: row.get(1)?,
                title: row.get(2)?,
                parent_id: row.get(3)?,
                level: row.get(4)?,
                sort_order: row.get(5)?,
                content: row.get(6)?,
                word_count: row.get(7)?,
                generated: row.get::<_, i32>(8)? != 0,
                source_file: row.get(9)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(sections)
    } else {
        let mut stmt = conn.prepare(
            r#"SELECT s.id, s.section_id, s.title, s.parent_id, s.level, s.sort_order,
                      s.content, s.word_count
               FROM sections s
               JOIN sections_fts fts ON s.id = fts.rowid
               WHERE sections_fts MATCH ?1
               ORDER BY rank"#
        )?;

        let sections = stmt.query_map([query], |row| {
            Ok(Section {
                id: row.get(0)?,
                section_id: row.get(1)?,
                title: row.get(2)?,
                parent_id: row.get(3)?,
                level: row.get(4)?,
                sort_order: row.get(5)?,
                content: row.get(6)?,
                word_count: row.get(7)?,
                generated: false,
                source_file: None,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(sections)
    }
}

/// Track an analyzed file for change detection
pub fn track_analyzed_file(conn: &Connection, file_path: &str, content_hash: &str) -> Result<()> {
    let now = Utc::now().format("%Y-%m-%dT%H:%M:%S%.6f").to_string();

    conn.execute(
        "INSERT OR REPLACE INTO analyzed_files (file_path, content_hash, last_analyzed) VALUES (?1, ?2, ?3)",
        [file_path, content_hash, &now],
    )?;

    Ok(())
}

/// Get hash of previously analyzed file
pub fn get_analyzed_file_hash(conn: &Connection, file_path: &str) -> Result<Option<String>> {
    let result: Result<String, _> = conn.query_row(
        "SELECT content_hash FROM analyzed_files WHERE file_path = ?1",
        [file_path],
        |row| row.get(0),
    );

    match result {
        Ok(hash) => Ok(Some(hash)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// Compute SHA256 hash of file contents
pub fn hash_file(path: &Path) -> Result<String> {
    use sha2::{Sha256, Digest};

    let contents = std::fs::read(path)
        .with_context(|| format!("Failed to read file: {:?}", path))?;

    let mut hasher = Sha256::new();
    hasher.update(&contents);
    let result = hasher.finalize();

    Ok(format!("{:x}", result))
}
