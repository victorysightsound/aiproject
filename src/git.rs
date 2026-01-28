// Git integration - sync recent commits into tracking database

use std::path::Path;
use std::process::Command;

use anyhow::Result;
use rusqlite::Connection;

/// A git commit record
pub struct GitCommit {
    pub hash: String,
    pub short_hash: String,
    pub author: String,
    pub message: String,
    pub committed_at: String,
    pub files_changed: i64,
    pub insertions: i64,
    pub deletions: i64,
}

/// Sync recent git commits into the tracking database.
/// Uses INSERT OR IGNORE to be idempotent.
pub fn sync_recent_commits(conn: &Connection, project_root: &Path, limit: usize) -> Result<()> {
    if !project_root.join(".git").exists() {
        return Ok(());
    }

    // Get recent commits with stats using a delimiter-separated format
    let output = Command::new("git")
        .args([
            "log",
            &format!("-{}", limit),
            "--format=%H%n%h%n%an%n%s%n%ai",
            "--shortstat",
        ])
        .current_dir(project_root)
        .output();

    let output = match output {
        Ok(o) if o.status.success() => o,
        _ => return Ok(()), // Git not available or not a repo, silently skip
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let commits = parse_git_log_output(&stdout);

    for commit in &commits {
        // Insert into git_commits (ignore duplicates by hash)
        conn.execute(
            "INSERT OR IGNORE INTO git_commits (hash, short_hash, author, message, committed_at, files_changed, insertions, deletions)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                commit.hash,
                commit.short_hash,
                commit.author,
                commit.message,
                commit.committed_at,
                commit.files_changed,
                commit.insertions,
                commit.deletions,
            ],
        )?;

        // Also index commit message into FTS for full-text search
        // Use INSERT OR IGNORE pattern via checking if already indexed
        let already_indexed: bool = conn
            .query_row(
                "SELECT 1 FROM tracking_fts WHERE table_name = 'git_commits' AND record_id = (SELECT commit_id FROM git_commits WHERE hash = ?1)",
                [&commit.hash],
                |_| Ok(true),
            )
            .unwrap_or(false);

        if !already_indexed {
            if let Ok(commit_id) = conn.query_row(
                "SELECT commit_id FROM git_commits WHERE hash = ?1",
                [&commit.hash],
                |row| row.get::<_, i64>(0),
            ) {
                let fts_content = format!("{}: {}", commit.short_hash, commit.message);
                let _ = conn.execute(
                    "INSERT INTO tracking_fts (content, table_name, record_id) VALUES (?1, 'git_commits', ?2)",
                    rusqlite::params![fts_content, commit_id],
                );
            }
        }
    }

    Ok(())
}

/// Parse the output of git log with --shortstat
fn parse_git_log_output(output: &str) -> Vec<GitCommit> {
    let mut commits = Vec::new();
    let lines: Vec<&str> = output.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        // Skip empty lines
        if lines[i].trim().is_empty() {
            i += 1;
            continue;
        }

        // We expect: hash, short_hash, author, subject, date (5 lines)
        if i + 4 >= lines.len() {
            break;
        }

        let hash = lines[i].trim().to_string();
        let short_hash = lines[i + 1].trim().to_string();
        let author = lines[i + 2].trim().to_string();
        let message = lines[i + 3].trim().to_string();
        let committed_at_raw = lines[i + 4].trim().to_string();

        // Validate this looks like a commit hash (40 hex chars)
        if hash.len() != 40 || !hash.chars().all(|c| c.is_ascii_hexdigit()) {
            i += 1;
            continue;
        }

        // Parse the date - git outputs "2024-01-15 10:30:00 -0600", we want "2024-01-15 10:30:00"
        let committed_at = committed_at_raw
            .split_whitespace()
            .take(2)
            .collect::<Vec<_>>()
            .join(" ");

        i += 5;

        // Next line might be --shortstat output or empty
        let (files_changed, insertions, deletions) = if i < lines.len() {
            let stat_line = lines[i].trim();
            if stat_line.contains("changed") {
                i += 1;
                parse_shortstat(stat_line)
            } else {
                (0, 0, 0)
            }
        } else {
            (0, 0, 0)
        };

        commits.push(GitCommit {
            hash,
            short_hash,
            author,
            message,
            committed_at,
            files_changed,
            insertions,
            deletions,
        });
    }

    commits
}

/// Parse a --shortstat line like "3 files changed, 10 insertions(+), 2 deletions(-)"
fn parse_shortstat(line: &str) -> (i64, i64, i64) {
    let mut files = 0i64;
    let mut ins = 0i64;
    let mut del = 0i64;

    for part in line.split(',') {
        let part = part.trim();
        let num: i64 = part
            .split_whitespace()
            .next()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        if part.contains("file") {
            files = num;
        } else if part.contains("insertion") {
            ins = num;
        } else if part.contains("deletion") {
            del = num;
        }
    }

    (files, ins, del)
}

/// Get recent commits from the database
pub fn get_recent_commits(conn: &Connection, limit: usize) -> Result<Vec<GitCommit>> {
    let mut stmt = conn.prepare(
        "SELECT hash, short_hash, author, message, committed_at, files_changed, insertions, deletions
         FROM git_commits
         ORDER BY committed_at DESC
         LIMIT ?1",
    )?;

    let commits = stmt
        .query_map([limit as i64], |row| {
            Ok(GitCommit {
                hash: row.get(0)?,
                short_hash: row.get(1)?,
                author: row.get(2)?,
                message: row.get(3)?,
                committed_at: row.get(4)?,
                files_changed: row.get(5)?,
                insertions: row.get(6)?,
                deletions: row.get(7)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(commits)
}

/// Get commits since a given datetime (for session activity)
pub fn get_commits_since(conn: &Connection, since: &str) -> Result<Vec<GitCommit>> {
    let mut stmt = conn.prepare(
        "SELECT hash, short_hash, author, message, committed_at, files_changed, insertions, deletions
         FROM git_commits
         WHERE committed_at >= ?1
         ORDER BY committed_at ASC",
    )?;

    let commits = stmt
        .query_map([since], |row| {
            Ok(GitCommit {
                hash: row.get(0)?,
                short_hash: row.get(1)?,
                author: row.get(2)?,
                message: row.get(3)?,
                committed_at: row.get(4)?,
                files_changed: row.get(5)?,
                insertions: row.get(6)?,
                deletions: row.get(7)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(commits)
}

/// Get count of commits since a given datetime
pub fn get_commit_count_since(conn: &Connection, since: &str) -> Result<i64> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM git_commits WHERE committed_at >= ?1",
        [since],
        |row| row.get(0),
    )?;
    Ok(count)
}

/// Search git commits by message
pub fn search_git_commits(
    conn: &Connection,
    query: &str,
) -> Result<Vec<(i64, String, String, String)>> {
    let pattern = format!("%{}%", query);
    let mut stmt = conn.prepare(
        "SELECT commit_id, short_hash, message, committed_at
         FROM git_commits
         WHERE message LIKE ?1
         ORDER BY committed_at DESC
         LIMIT 20",
    )?;

    let results = stmt
        .query_map([&pattern], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(results)
}
