// Documentation database schema
// Based on doc-orchestrator schema with extensions for generation tracking

/// Documentation database schema (separate from tracking.db)
/// Creates {project}_{type}.db in project root
pub const DOCS_SCHEMA: &str = r#"
-- Document metadata
CREATE TABLE IF NOT EXISTS meta (
    key TEXT PRIMARY KEY,
    value TEXT
);

-- Hierarchical sections
CREATE TABLE IF NOT EXISTS sections (
    id INTEGER PRIMARY KEY,
    section_id TEXT UNIQUE NOT NULL,       -- e.g., "1.2.3" or "overview"
    title TEXT NOT NULL,
    parent_id TEXT,                         -- section_id of parent (NULL for root)
    level INTEGER NOT NULL DEFAULT 1,       -- heading level: 1=h1, 2=h2, etc.
    sort_order INTEGER NOT NULL,            -- display order
    content TEXT DEFAULT '',                -- section content (markdown)
    word_count INTEGER DEFAULT 0,
    generated INTEGER DEFAULT 0,            -- 1 if auto-generated, 0 if manual
    source_file TEXT,                       -- file this was generated from (if any)
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT,
    FOREIGN KEY (parent_id) REFERENCES sections(section_id)
);

-- FTS5 for section search
CREATE VIRTUAL TABLE IF NOT EXISTS sections_fts USING fts5(
    title, content,
    content='sections', content_rowid='id'
);

-- Triggers to keep FTS in sync
CREATE TRIGGER IF NOT EXISTS sections_ai AFTER INSERT ON sections BEGIN
    INSERT INTO sections_fts(rowid, title, content)
    VALUES (new.id, new.title, new.content);
END;

CREATE TRIGGER IF NOT EXISTS sections_au AFTER UPDATE ON sections BEGIN
    INSERT INTO sections_fts(sections_fts, rowid, title, content)
    VALUES('delete', old.id, old.title, old.content);
    INSERT INTO sections_fts(rowid, title, content)
    VALUES (new.id, new.title, new.content);
END;

CREATE TRIGGER IF NOT EXISTS sections_ad AFTER DELETE ON sections BEGIN
    INSERT INTO sections_fts(sections_fts, rowid, title, content)
    VALUES('delete', old.id, old.title, old.content);
END;

-- Terminology glossary
CREATE TABLE IF NOT EXISTS terminology (
    id INTEGER PRIMARY KEY,
    canonical TEXT UNIQUE NOT NULL,         -- The correct form: "Ralph Loop"
    variants TEXT NOT NULL DEFAULT '[]',    -- JSON array: ["ralph loop", "RALPH LOOP"]
    definition TEXT,                        -- What the term means
    category TEXT,                          -- Grouping: workflow, architecture, technology
    first_used_in TEXT,                     -- section_id where first introduced
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT,
    FOREIGN KEY (first_used_in) REFERENCES sections(section_id)
);

-- FTS5 for terminology search
CREATE VIRTUAL TABLE IF NOT EXISTS terminology_fts USING fts5(
    canonical, definition,
    content='terminology', content_rowid='id'
);

-- Triggers to keep terminology FTS in sync
CREATE TRIGGER IF NOT EXISTS terminology_ai AFTER INSERT ON terminology BEGIN
    INSERT INTO terminology_fts(rowid, canonical, definition)
    VALUES (new.id, new.canonical, new.definition);
END;

CREATE TRIGGER IF NOT EXISTS terminology_au AFTER UPDATE ON terminology BEGIN
    INSERT INTO terminology_fts(terminology_fts, rowid, canonical, definition)
    VALUES('delete', old.id, old.canonical, old.definition);
    INSERT INTO terminology_fts(rowid, canonical, definition)
    VALUES (new.id, new.canonical, new.definition);
END;

CREATE TRIGGER IF NOT EXISTS terminology_ad AFTER DELETE ON terminology BEGIN
    INSERT INTO terminology_fts(terminology_fts, rowid, canonical, definition)
    VALUES('delete', old.id, old.canonical, old.definition);
END;

-- Cross-references between sections
CREATE TABLE IF NOT EXISTS crossrefs (
    id INTEGER PRIMARY KEY,
    from_section TEXT NOT NULL,             -- source section_id
    to_section TEXT NOT NULL,               -- target section_id
    link_text TEXT,                         -- display text of the link
    validated INTEGER DEFAULT 0,            -- 1 if target exists
    created_at TEXT DEFAULT (datetime('now')),
    FOREIGN KEY (from_section) REFERENCES sections(section_id),
    FOREIGN KEY (to_section) REFERENCES sections(section_id)
);

-- Key concepts and where they're introduced
CREATE TABLE IF NOT EXISTS concepts (
    id INTEGER PRIMARY KEY,
    name TEXT UNIQUE NOT NULL,              -- concept name
    introduced_in TEXT NOT NULL,            -- section_id where first introduced
    description TEXT,                       -- brief description
    created_at TEXT DEFAULT (datetime('now')),
    FOREIGN KEY (introduced_in) REFERENCES sections(section_id)
);

-- Terms to avoid with suggested replacements
CREATE TABLE IF NOT EXISTS banned_terms (
    id INTEGER PRIMARY KEY,
    term TEXT UNIQUE NOT NULL,              -- term to avoid
    replacement TEXT,                       -- suggested canonical term
    reason TEXT,                            -- why it's banned
    created_at TEXT DEFAULT (datetime('now'))
);

-- Documentation editing sessions (separate from proj sessions)
CREATE TABLE IF NOT EXISTS doc_sessions (
    id INTEGER PRIMARY KEY,
    started_at TEXT DEFAULT (datetime('now')),
    ended_at TEXT,
    sections_worked TEXT,                   -- JSON array of section_ids
    notes TEXT
);

-- Consistency check results
CREATE TABLE IF NOT EXISTS check_results (
    id INTEGER PRIMARY KEY,
    check_type TEXT NOT NULL,               -- terminology, crossref, cohesion
    section_id TEXT,                        -- affected section (if applicable)
    severity TEXT DEFAULT 'warning',        -- error, warning, info
    message TEXT NOT NULL,                  -- description of the issue
    suggestion TEXT,                        -- how to fix
    resolved INTEGER DEFAULT 0,             -- 1 if fixed
    created_at TEXT DEFAULT (datetime('now')),
    resolved_at TEXT
);

-- Tracked files for change detection (generation feature)
CREATE TABLE IF NOT EXISTS analyzed_files (
    file_path TEXT PRIMARY KEY,
    content_hash TEXT NOT NULL,             -- SHA256 of file contents
    last_analyzed TEXT DEFAULT (datetime('now'))
);

-- Source code to documentation mapping (generation feature)
CREATE TABLE IF NOT EXISTS source_map (
    id INTEGER PRIMARY KEY,
    section_id TEXT NOT NULL,
    file_path TEXT NOT NULL,
    line_start INTEGER,
    line_end INTEGER,
    symbol_name TEXT,                       -- function/struct/class name
    symbol_type TEXT,                       -- function, struct, module, trait, etc.
    signature TEXT,                         -- full signature if applicable
    FOREIGN KEY (section_id) REFERENCES sections(section_id)
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_sections_parent ON sections(parent_id);
CREATE INDEX IF NOT EXISTS idx_sections_level ON sections(level);
CREATE INDEX IF NOT EXISTS idx_sections_sort ON sections(sort_order);
CREATE INDEX IF NOT EXISTS idx_sections_generated ON sections(generated);
CREATE INDEX IF NOT EXISTS idx_terminology_category ON terminology(category);
CREATE INDEX IF NOT EXISTS idx_crossrefs_from ON crossrefs(from_section);
CREATE INDEX IF NOT EXISTS idx_crossrefs_to ON crossrefs(to_section);
CREATE INDEX IF NOT EXISTS idx_concepts_introduced ON concepts(introduced_in);
CREATE INDEX IF NOT EXISTS idx_check_results_type ON check_results(check_type);
CREATE INDEX IF NOT EXISTS idx_check_results_resolved ON check_results(resolved);
CREATE INDEX IF NOT EXISTS idx_source_map_section ON source_map(section_id);
CREATE INDEX IF NOT EXISTS idx_source_map_file ON source_map(file_path);
"#;

/// Documentation types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DocType {
    Architecture,
    Framework,
    Guide,
    Api,
    Spec,
}

impl DocType {
    pub fn as_str(&self) -> &'static str {
        match self {
            DocType::Architecture => "architecture",
            DocType::Framework => "framework",
            DocType::Guide => "guide",
            DocType::Api => "api",
            DocType::Spec => "spec",
        }
    }

    pub fn from_str(s: &str) -> Option<DocType> {
        match s.to_lowercase().as_str() {
            "architecture" => Some(DocType::Architecture),
            "framework" => Some(DocType::Framework),
            "guide" => Some(DocType::Guide),
            "api" => Some(DocType::Api),
            "spec" => Some(DocType::Spec),
            _ => None,
        }
    }

}

impl std::fmt::Display for DocType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Generate database filename from project name and doc type
pub fn docs_db_filename(project_name: &str, doc_type: DocType) -> String {
    format!("{}_{}.db", project_name, doc_type.as_str())
}

/// Initialize a new documentation database
pub fn init_docs_db(conn: &rusqlite::Connection) -> anyhow::Result<()> {
    conn.execute_batch(DOCS_SCHEMA)?;
    Ok(())
}

/// Set metadata in the docs database
pub fn set_meta(conn: &rusqlite::Connection, key: &str, value: &str) -> anyhow::Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO meta (key, value) VALUES (?1, ?2)",
        [key, value],
    )?;
    Ok(())
}

/// Get metadata from the docs database
pub fn get_meta(conn: &rusqlite::Connection, key: &str) -> anyhow::Result<Option<String>> {
    let result: Result<String, _> = conn.query_row(
        "SELECT value FROM meta WHERE key = ?1",
        [key],
        |row| row.get(0),
    );
    match result {
        Ok(value) => Ok(Some(value)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}
