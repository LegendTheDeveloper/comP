// SQLite schema definition and migrations

pub struct Schema;

impl Schema {
    /// Pragmas applied on every connection open, before any DDL.
    /// WAL mode allows concurrent reads from the VSCode daemon while the MCP
    /// daemon is writing, without blocking. busy_timeout prevents immediate
    /// SQLITE_BUSY errors under cross-process write contention.
    pub const PRAGMA_INIT: &str = "
        PRAGMA journal_mode=WAL;
        PRAGMA busy_timeout=5000;
    ";

    /// SQL to create all required tables
    ///
    /// Tables:
    /// - files: Indexed source files with hashes
    /// - nodes: Symbols extracted from code (functions, classes, types, etc.)
    /// - edges: Dependencies between symbols
    pub const MIGRATION_001_INIT: &str = r#"
        -- Repos table: one row per indexed repository root (multi-repo support).
        -- alias is the short name used to qualify file paths ("<alias>/<relative>")
        -- and to scope queries via the run_pipeline `repos` parameter.
        CREATE TABLE IF NOT EXISTS repos (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            alias TEXT NOT NULL UNIQUE,
            root_path TEXT NOT NULL UNIQUE
        );

        -- Files table: tracks indexed source files.
        -- path stores the repo-qualified path "<alias>/<relative>", which is
        -- globally unique across repos (so same relative path in two repos no
        -- longer collides on the UNIQUE constraint).
        CREATE TABLE IF NOT EXISTS files (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            path TEXT NOT NULL UNIQUE,
            hash TEXT NOT NULL,
            language TEXT NOT NULL,
            last_indexed INTEGER NOT NULL DEFAULT 0
        );

        -- Symbols/Nodes table: functions, classes, types, variables
        CREATE TABLE IF NOT EXISTS nodes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            file_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            kind TEXT NOT NULL,  -- "function", "class", "type", "variable", etc.
            line INTEGER NOT NULL,
            col INTEGER NOT NULL,
            scope TEXT,  -- parent scope (class name, function name, etc.)
            is_exported INTEGER DEFAULT 0,
            signature TEXT,  -- function/method signature
            FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE
        );

        -- Dependencies/Edges table: relationships between symbols
        CREATE TABLE IF NOT EXISTS edges (
            from_id INTEGER NOT NULL,
            to_id INTEGER NOT NULL,
            kind TEXT NOT NULL,  -- "calls", "references", "extends", "implements", etc.
            PRIMARY KEY (from_id, to_id, kind),
            FOREIGN KEY (from_id) REFERENCES nodes(id) ON DELETE CASCADE,
            FOREIGN KEY (to_id) REFERENCES nodes(id) ON DELETE CASCADE
        );

        -- Metadata table: version, last_indexed_time, etc.
        CREATE TABLE IF NOT EXISTS metadata (
            key TEXT PRIMARY KEY,
            value TEXT
        );

        -- Create indexes for performance
        CREATE INDEX IF NOT EXISTS idx_files_path ON files(path);
        CREATE INDEX IF NOT EXISTS idx_nodes_file_id ON nodes(file_id);
        CREATE INDEX IF NOT EXISTS idx_nodes_kind ON nodes(kind);
        CREATE INDEX IF NOT EXISTS idx_edges_from ON edges(from_id);
        CREATE INDEX IF NOT EXISTS idx_edges_to ON edges(to_id);
    "#;

    /// Apply all migrations to the database
    pub fn apply_all(conn: &rusqlite::Connection) -> anyhow::Result<()> {
        // Pragmas first — WAL mode and busy_timeout
        conn.execute_batch(Self::PRAGMA_INIT)?;

        conn.execute_batch(Self::MIGRATION_001_INIT)?;

        // Migration 002: add char_count column to files for real token baseline
        // Guard: SQLite <3.37 has no ADD COLUMN IF NOT EXISTS, so check manually.
        let has_char_count: bool = conn.query_row(
            "SELECT COUNT(*) FROM pragma_table_info('files') WHERE name='char_count'",
            [],
            |row| row.get::<_, i64>(0),
        ).map_err(|e| anyhow::anyhow!("Migration guard query failed: {}", e))? > 0;
        if !has_char_count {
            conn.execute_batch(
                "ALTER TABLE files ADD COLUMN char_count INTEGER NOT NULL DEFAULT 0;"
            )?;
        }

        // Migration 003: search_history table. Records every run_pipeline /
        // get_context call with its outcome diagnostics so search quality can
        // be reviewed later (sidebar "Recent Searches" + future tuning).
        // Shared DB = single source of truth for both daemon processes.
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS search_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp INTEGER NOT NULL,           -- unix seconds
                tool TEXT NOT NULL,                   -- 'run_pipeline' | 'get_context'
                query TEXT NOT NULL,                  -- task / query string
                keywords TEXT,                        -- JSON array (post-stopword)
                confidence TEXT,                      -- high/medium/low (run_pipeline)
                weak_results INTEGER,                 -- 0/1 (run_pipeline)
                pivot_count INTEGER,                  -- pivots / results returned
                dropped_low_relevance INTEGER,        -- cutoff drops (run_pipeline)
                total_tokens INTEGER,
                duration_ms INTEGER,
                top_pivots TEXT                       -- JSON [{path, score, reasons}]
            );
            CREATE INDEX IF NOT EXISTS idx_search_history_ts ON search_history(timestamp DESC);"
        )?;

        // Initialize metadata keys used by token tracking
        for key in &["tokens_sent", "tokens_saved", "queries_count", "version"] {
            let value = if *key == "version" { "2" } else { "0" };
            conn.execute(
                "INSERT OR IGNORE INTO metadata (key, value) VALUES (?, ?)",
                [key, value],
            )?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_constants_exist() {
        assert!(!Schema::MIGRATION_001_INIT.is_empty());
        assert!(!Schema::PRAGMA_INIT.is_empty());
    }
}
