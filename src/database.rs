// Database module - Full implementation in Task #4

use anyhow::Result;
use rusqlite::Connection;
use std::path::Path;

/// Opens or creates a SQLite database with standard settings
pub fn open_database(path: &Path) -> Result<Connection> {
    let conn = Connection::open(path)?;

    // Enable foreign keys
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;

    // Use WAL mode for better concurrency
    conn.execute_batch("PRAGMA journal_mode = WAL;")?;

    Ok(conn)
}

/// Creates a backup of the database
pub fn backup_database(source: &Path, dest: &Path) -> Result<()> {
    std::fs::copy(source, dest)?;
    Ok(())
}

/// Gets the schema version from the database
pub fn get_schema_version(conn: &Connection) -> Result<Option<String>> {
    let result: Result<String, _> = conn.query_row(
        "SELECT value FROM project_meta WHERE key = 'schema_version'",
        [],
        |row| row.get(0),
    );

    match result {
        Ok(version) => Ok(Some(version)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// Sets the schema version in the database
pub fn set_schema_version(conn: &Connection, version: &str) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO project_meta (key, value) VALUES ('schema_version', ?1)",
        [version],
    )?;
    Ok(())
}
