// SQL schemas - Full implementation in Task #7

/// Tracking database schema (embedded as const)
pub const TRACKING_SCHEMA: &str = r#"
-- Project metadata
CREATE TABLE IF NOT EXISTS project_meta (
    key TEXT PRIMARY KEY,
    value TEXT,
    updated_at TEXT DEFAULT (datetime('now'))
);

-- Sessions
CREATE TABLE IF NOT EXISTS sessions (
    session_id INTEGER PRIMARY KEY AUTOINCREMENT,
    started_at TEXT DEFAULT (datetime('now')),
    ended_at TEXT,
    agent TEXT,
    summary TEXT,
    files_touched TEXT,
    status TEXT DEFAULT 'active',
    full_context_shown INTEGER DEFAULT 0
);

-- Decisions
CREATE TABLE IF NOT EXISTS decisions (
    decision_id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER,
    created_at TEXT DEFAULT (datetime('now')),
    topic TEXT NOT NULL,
    decision TEXT NOT NULL,
    rationale TEXT,
    alternatives TEXT,
    status TEXT DEFAULT 'active',
    superseded_by INTEGER,
    FOREIGN KEY (session_id) REFERENCES sessions(session_id),
    FOREIGN KEY (superseded_by) REFERENCES decisions(decision_id)
);

-- Tasks
CREATE TABLE IF NOT EXISTS tasks (
    task_id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER,
    created_at TEXT DEFAULT (datetime('now')),
    completed_at TEXT,
    description TEXT NOT NULL,
    status TEXT DEFAULT 'pending',
    priority TEXT DEFAULT 'normal',
    blocked_by TEXT,
    parent_task_id INTEGER,
    notes TEXT,
    FOREIGN KEY (session_id) REFERENCES sessions(session_id),
    FOREIGN KEY (parent_task_id) REFERENCES tasks(task_id)
);

-- Blockers
CREATE TABLE IF NOT EXISTS blockers (
    blocker_id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER,
    created_at TEXT DEFAULT (datetime('now')),
    resolved_at TEXT,
    description TEXT NOT NULL,
    status TEXT DEFAULT 'active',
    resolution TEXT,
    related_task_id INTEGER,
    FOREIGN KEY (session_id) REFERENCES sessions(session_id),
    FOREIGN KEY (related_task_id) REFERENCES tasks(task_id)
);

-- Context notes
CREATE TABLE IF NOT EXISTS context_notes (
    note_id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER,
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now')),
    category TEXT NOT NULL,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    status TEXT DEFAULT 'active',
    FOREIGN KEY (session_id) REFERENCES sessions(session_id)
);

-- Milestones
CREATE TABLE IF NOT EXISTS milestones (
    milestone_id INTEGER PRIMARY KEY AUTOINCREMENT,
    created_at TEXT DEFAULT (datetime('now')),
    target_date TEXT,
    achieved_at TEXT,
    name TEXT NOT NULL,
    description TEXT,
    status TEXT DEFAULT 'pending'
);

-- Questions
CREATE TABLE IF NOT EXISTS questions (
    question_id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER,
    created_at TEXT DEFAULT (datetime('now')),
    answered_at TEXT,
    question TEXT NOT NULL,
    context TEXT,
    answer TEXT,
    status TEXT DEFAULT 'open',
    FOREIGN KEY (session_id) REFERENCES sessions(session_id)
);

-- Session references
CREATE TABLE IF NOT EXISTS session_references (
    reference_id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER,
    created_at TEXT DEFAULT (datetime('now')),
    title TEXT NOT NULL,
    url TEXT,
    type TEXT,
    notes TEXT,
    relevance TEXT,
    FOREIGN KEY (session_id) REFERENCES sessions(session_id)
);

-- Activity log
CREATE TABLE IF NOT EXISTS activity_log (
    log_id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER,
    timestamp TEXT DEFAULT (datetime('now')),
    action_type TEXT NOT NULL,
    action_id INTEGER,
    summary TEXT NOT NULL,
    FOREIGN KEY (session_id) REFERENCES sessions(session_id)
);

-- Context snapshots for delta tracking
CREATE TABLE IF NOT EXISTS context_snapshots (
    snapshot_id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER,
    created_at TEXT DEFAULT (datetime('now')),
    snapshot_type TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    item_counts TEXT,
    FOREIGN KEY (session_id) REFERENCES sessions(session_id)
);

-- Compressed sessions
CREATE TABLE IF NOT EXISTS compressed_sessions (
    compression_id INTEGER PRIMARY KEY AUTOINCREMENT,
    created_at TEXT DEFAULT (datetime('now')),
    session_ids TEXT NOT NULL,
    date_range_start TEXT,
    date_range_end TEXT,
    compressed_summary TEXT NOT NULL,
    original_token_estimate INTEGER,
    compressed_token_estimate INTEGER
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_sessions_status ON sessions(status);
CREATE INDEX IF NOT EXISTS idx_sessions_started ON sessions(started_at);
CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);
CREATE INDEX IF NOT EXISTS idx_decisions_topic ON decisions(topic);
CREATE INDEX IF NOT EXISTS idx_context_notes_category ON context_notes(category);
CREATE INDEX IF NOT EXISTS idx_activity_log_session ON activity_log(session_id);
CREATE INDEX IF NOT EXISTS idx_activity_log_timestamp ON activity_log(timestamp);
CREATE INDEX IF NOT EXISTS idx_context_snapshots_session ON context_snapshots(session_id);
"#;

/// FTS5 virtual table for full-text search
pub const FTS_SCHEMA: &str = r#"
CREATE VIRTUAL TABLE IF NOT EXISTS tracking_fts USING fts5(
    content,
    table_name,
    record_id,
    content='',
    tokenize='porter'
);
"#;

/// Initialize database with schema
pub fn init_tracking_schema(conn: &rusqlite::Connection) -> anyhow::Result<()> {
    conn.execute_batch(TRACKING_SCHEMA)?;
    conn.execute_batch(FTS_SCHEMA)?;

    // Set schema version
    conn.execute(
        "INSERT OR REPLACE INTO project_meta (key, value) VALUES ('schema_version', ?1)",
        [crate::SCHEMA_VERSION],
    )?;

    Ok(())
}
