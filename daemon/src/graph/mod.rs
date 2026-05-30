// Graph module - SQLite-based code graph database
//
// Tables:
// - files: {id, path, hash, last_indexed, language}
// - nodes: {id, file_id, name, kind, line, col, scope}
// - edges: {from_id, to_id, kind}
//
// The graph represents:
// - Nodes: Symbols (functions, classes, variables, types)
// - Edges: Dependencies (function calls, type references, etc.)

use anyhow::Result;
use rusqlite::Connection;
use std::collections::HashMap;
use std::sync::Mutex;

mod schema;

pub use schema::Schema;

/// Graph database interface for storing code structure
pub struct GraphDB {
    // WHY: Use Mutex<Connection> to obtain Send+Sync.
    // Required to run indexing concurrently via tokio::spawn.
    conn: Mutex<Connection>,
}

impl GraphDB {
    /// Create/open SQLite database at workspace
    ///
    /// # Process:
    /// 1. Create .comp/ directory if not exists
    /// 2. Create/open index.db SQLite database
    /// 3. Initialize schema (tables, indexes)
    pub async fn new(workspace_root: &str) -> Result<Self> {
        use std::fs;
        use std::path::Path;

        // Create .comp directory
        let comp_dir = Path::new(workspace_root).join(".comp");
        fs::create_dir_all(&comp_dir)?;

        // Open/create database
        let db_path = comp_dir.join("index.db");
        let conn = Connection::open(db_path)?;
        let db = GraphDB { conn: Mutex::new(conn) };

        // Initialize schema
        db.init_schema()?;

        Ok(db)
    }

    /// Initialize database schema (creates tables and indexes)
    fn init_schema(&self) -> Result<()> {
        // Apply all migrations from schema.rs
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("DB mutex poisoned: {}", e))?;
        Schema::apply_all(&conn)?;
        Ok(())
    }

    /// Insert/update a file in the database
    ///
    /// Returns the file ID for use in subsequent operations
    pub fn upsert_file(&self, path: &str, hash: &str, language: &str) -> Result<i64> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("DB mutex poisoned: {}", e))?;
        conn.execute(
            "INSERT OR REPLACE INTO files (path, hash, language, last_indexed)
             VALUES (?, ?, ?, strftime('%s', 'now'))",
            [path, hash, language],
        )?;

        // Get the inserted/updated file ID
        let mut stmt = conn.prepare(
            "SELECT id FROM files WHERE path = ?"
        )?;
        let file_id: i64 = stmt.query_row([path], |row| row.get(0))?;

        Ok(file_id)
    }

    /// Insert a symbol node
    ///
    /// Returns the node ID for use in dependency tracking
    pub fn insert_node(
        &self,
        file_id: i64,
        name: &str,
        kind: &str,
        line: i32,
        col: i32,
        scope: Option<&str>,
    ) -> Result<i64> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("DB mutex poisoned: {}", e))?;
        conn.execute(
            "INSERT INTO nodes (file_id, name, kind, line, col, scope)
             VALUES (?, ?, ?, ?, ?, ?)",
            rusqlite::params![file_id, name, kind, line, col, scope],
        )?;

        // Get the inserted node ID
        let last_id = conn.last_insert_rowid();
        Ok(last_id)
    }

    /// Insert a dependency edge between nodes
    pub fn insert_edge(&self, from_id: i64, to_id: i64, kind: &str) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("DB mutex poisoned: {}", e))?;
        conn.execute(
            "INSERT OR IGNORE INTO edges (from_id, to_id, kind)
             VALUES (?, ?, ?)",
            rusqlite::params![from_id, to_id, kind],
        )?;
        Ok(())
    }

    /// Get all nodes that depend on a given symbol
    ///
    /// Returns: Vec<(node_id, symbol_name)>
    pub fn get_dependents(&self, node_id: i64) -> Result<Vec<(i64, String)>> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("DB mutex poisoned: {}", e))?;
        let mut stmt = conn.prepare(
            "SELECT edges.to_id, nodes.name FROM edges
             JOIN nodes ON edges.to_id = nodes.id
             WHERE edges.from_id = ?"
        )?;

        let dependents = stmt.query_map([node_id], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?;

        let result = dependents.collect::<Result<Vec<_>, _>>()?;
        Ok(result)
    }

    /// Atomically increment token stats in the metadata table
    ///
    /// WHY: run_pipeline is executed in the MCP daemon process spawned by Claude Code,
    /// while getStats is called from a separate process spawned by the VSCode extension.
    /// Since an in-memory AtomicU64 is not shared between processes, we persist it in the shared SQLite DB.
    pub fn increment_token_stats(&self, tokens_sent: u64, tokens_saved: u64) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("DB mutex poisoned: {}", e))?;
        for key in &["tokens_sent", "tokens_saved", "queries_count"] {
            conn.execute(
                "INSERT OR IGNORE INTO metadata (key, value) VALUES (?, '0')",
                [key],
            )?;
        }
        conn.execute(
            "UPDATE metadata SET value = CAST(CAST(value AS INTEGER) + ? AS TEXT) WHERE key = 'tokens_sent'",
            [tokens_sent as i64],
        )?;
        conn.execute(
            "UPDATE metadata SET value = CAST(CAST(value AS INTEGER) + ? AS TEXT) WHERE key = 'tokens_saved'",
            [tokens_saved as i64],
        )?;
        conn.execute(
            "UPDATE metadata SET value = CAST(CAST(value AS INTEGER) + 1 AS TEXT) WHERE key = 'queries_count'",
            [],
        )?;
        Ok(())
    }

    /// Read token stats from the metadata table
    pub fn get_token_stats(&self) -> Result<(u64, u64, u64)> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("DB mutex poisoned: {}", e))?;
        let tokens_sent = conn.query_row(
            "SELECT CAST(value AS INTEGER) FROM metadata WHERE key = 'tokens_sent'",
            [],
            |row| row.get::<_, i64>(0),
        ).unwrap_or(0) as u64;
        let tokens_saved = conn.query_row(
            "SELECT CAST(value AS INTEGER) FROM metadata WHERE key = 'tokens_saved'",
            [],
            |row| row.get::<_, i64>(0),
        ).unwrap_or(0) as u64;
        let queries_count = conn.query_row(
            "SELECT CAST(value AS INTEGER) FROM metadata WHERE key = 'queries_count'",
            [],
            |row| row.get::<_, i64>(0),
        ).unwrap_or(0) as u64;
        Ok((tokens_sent, tokens_saved, queries_count))
    }

    /// Get total statistics about the index
    pub fn get_stats(&self) -> Result<(i64, i64, i64)> {
        // Returns: (file_count, node_count, edge_count)
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("DB mutex poisoned: {}", e))?;

        let file_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM files",
            [],
            |row| row.get(0)
        )?;

        let node_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM nodes",
            [],
            |row| row.get(0)
        )?;

        let edge_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM edges",
            [],
            |row| row.get(0)
        )?;

        Ok((file_count, node_count, edge_count))
    }

    /// List all indexed files with id, path, language
    ///
    /// WHY: Required for handle_list_indexed_files to return actual data.
    pub fn list_files(&self) -> Result<Vec<(i64, String, String)>> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("DB mutex poisoned: {}", e))?;
        let mut stmt = conn.prepare(
            "SELECT id, path, language FROM files ORDER BY path"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?))
        })?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    /// Count symbols (nodes) per file_id
    pub fn count_symbols_per_file(&self) -> Result<HashMap<i64, i64>> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("DB mutex poisoned: {}", e))?;
        let mut stmt = conn.prepare(
            "SELECT file_id, COUNT(*) FROM nodes GROUP BY file_id"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
        })?;
        let mut map = HashMap::new();
        for row in rows {
            let (file_id, count) = row?;
            map.insert(file_id, count);
        }
        Ok(map)
    }

    /// Search symbols by name (LIKE pattern, case-insensitive)
    ///
    /// WHY: Since SearchEngine's TF-IDF is not yet built, we temporarily return context
    /// by searching symbol names using a LIKE pattern.
    pub fn search_symbols_by_name(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<(String, String, String, i32)>> {
        // Return: (file_path, symbol_name, kind, line)
        let pattern = format!("%{}%", query);
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("DB mutex poisoned: {}", e))?;
        let mut stmt = conn.prepare(
            "SELECT files.path, nodes.name, nodes.kind, nodes.line
             FROM nodes JOIN files ON nodes.file_id = files.id
             WHERE LOWER(nodes.name) LIKE LOWER(?)
             LIMIT ?"
        )?;
        let rows = stmt.query_map(rusqlite::params![pattern, limit as i64], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i32>(3)?,
            ))
        })?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    /// Build symbol_id -> (name, file_path) map for impact analysis
    pub fn get_symbol_map(&self) -> Result<HashMap<i64, (String, String)>> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("DB mutex poisoned: {}", e))?;
        let mut stmt = conn.prepare(
            "SELECT nodes.id, nodes.name, files.path
             FROM nodes JOIN files ON nodes.file_id = files.id"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?;
        let mut map = HashMap::new();
        for row in rows {
            let (id, name, path) = row?;
            map.insert(id, (name, path));
        }
        Ok(map)
    }

    /// Build reverse-dependency map: to_id -> [from_id, ...]
    ///
    /// WHY: Finding "who is affected if symbol X is modified" requires
    /// looking up the caller side (from) reversely.
    pub fn get_reverse_deps(&self) -> Result<HashMap<i64, Vec<i64>>> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("DB mutex poisoned: {}", e))?;
        let mut stmt = conn.prepare("SELECT from_id, to_id FROM edges")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
        })?;
        let mut map: HashMap<i64, Vec<i64>> = HashMap::new();
        for row in rows {
            let (from_id, to_id) = row?;
            map.entry(to_id).or_insert_with(Vec::new).push(from_id);
        }
        Ok(map)
    }

    /// Clear all indexed data (for force re-index)
    pub fn clear_index(&self) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("DB mutex poisoned: {}", e))?;
        conn.execute("DELETE FROM edges", [])?;
        conn.execute("DELETE FROM nodes", [])?;
        conn.execute("DELETE FROM files", [])?;
        Ok(())
    }

    /// Delete a single file and its associated nodes/edges
    ///
    /// WHY: Leaving old entries on file deletion or renaming makes impact analysis
    /// and stats inaccurate. We delete in the order of edges -> nodes -> files
    /// explicitly without relying on CASCADE (assuming SQLite foreign key constraints are not enabled).
    pub fn delete_file(&self, path: &str) -> Result<usize> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("DB mutex poisoned: {}", e))?;

        // Get file_id (no-op with 0 results if it doesn't exist)
        let file_id: Option<i64> = conn
            .query_row("SELECT id FROM files WHERE path = ?", [path], |row| row.get(0))
            .ok();

        let Some(fid) = file_id else { return Ok(0); };

        // 1. Get all node IDs of this file
        let mut stmt = conn.prepare("SELECT id FROM nodes WHERE file_id = ?")?;
        let node_ids: Vec<i64> = stmt
            .query_map([fid], |row| row.get::<_, i64>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        drop(stmt);

        // 2. Delete edges referencing the target nodes (both from and to sides)
        for nid in &node_ids {
            conn.execute(
                "DELETE FROM edges WHERE from_id = ? OR to_id = ?",
                rusqlite::params![nid, nid],
            )?;
        }

        // 3. Delete nodes
        conn.execute("DELETE FROM nodes WHERE file_id = ?", [fid])?;

        // 4. Delete file entry
        let removed = conn.execute("DELETE FROM files WHERE id = ?", [fid])?;

        Ok(removed)
    }

    /// Load all (path → hash) entries from the DB for incremental indexing
    ///
    /// WHY: Passing these to index_workspace lets the indexer skip files whose
    /// content hash hasn't changed since the last session, avoiding a full
    /// re-index on every daemon restart.
    pub fn get_all_file_hashes(&self) -> Result<HashMap<String, String>> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("DB mutex poisoned: {}", e))?;
        let mut stmt = conn.prepare("SELECT path, hash FROM files")?;
        let map = stmt
            .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))?
            .filter_map(|r| r.ok())
            .collect();
        Ok(map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_graphdb_creation() {
        // TODO: Use temp directory for testing
        // let db = GraphDB::new("/tmp/test").await.unwrap();
        // assert graph is initialized
    }
}
