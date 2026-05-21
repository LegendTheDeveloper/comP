// SQLite schema definition and migrations

pub struct Schema;

impl Schema {
    /// SQL to create all required tables
    /// 
    /// Tables:
    /// - files: Indexed source files with hashes
    /// - nodes: Symbols extracted from code (functions, classes, types, etc.)
    /// - edges: Dependencies between symbols
    pub const MIGRATION_001_INIT: &str = r#"
        -- Files table: tracks indexed source files
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
        conn.execute_batch(Self::MIGRATION_001_INIT)?;
        
        // Initialize metadata
        conn.execute(
            "INSERT OR IGNORE INTO metadata (key, value) VALUES (?, ?)",
            ["version", "1"],
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_constants_exist() {
        assert!(!Schema::MIGRATION_001_INIT.is_empty());
    }
}
