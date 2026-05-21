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

mod schema;

pub use schema::Schema;

/// Graph database interface for storing code structure
pub struct GraphDB {
    conn: Connection,
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
        let db = GraphDB { conn };

        // Initialize schema
        db.init_schema()?;

        Ok(db)
    }

    /// Initialize database schema (creates tables and indexes)
    fn init_schema(&self) -> Result<()> {
        // Apply all migrations from schema.rs
        Schema::apply_all(&self.conn)?;
        Ok(())
    }

    /// Insert/update a file in the database
    ///
    /// Returns the file ID for use in subsequent operations
    pub fn upsert_file(&self, path: &str, hash: &str, language: &str) -> Result<i64> {
        self.conn.execute(
            "INSERT OR REPLACE INTO files (path, hash, language, last_indexed)
             VALUES (?, ?, ?, strftime('%s', 'now'))",
            [path, hash, language],
        )?;

        // Get the inserted/updated file ID
        let mut stmt = self.conn.prepare(
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
        self.conn.execute(
            "INSERT INTO nodes (file_id, name, kind, line, col, scope)
             VALUES (?, ?, ?, ?, ?, ?)",
            rusqlite::params![file_id, name, kind, line, col, scope],
        )?;

        // Get the inserted node ID
        let last_id = self.conn.last_insert_rowid();
        Ok(last_id)
    }

    /// Insert a dependency edge between nodes
    pub fn insert_edge(&self, from_id: i64, to_id: i64, kind: &str) -> Result<()> {
        self.conn.execute(
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
        let mut stmt = self.conn.prepare(
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

    /// Get total statistics about the index
    pub fn get_stats(&self) -> Result<(i64, i64, i64)> {
        // Returns: (file_count, node_count, edge_count)

        let file_count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM files",
            [],
            |row| row.get(0)
        )?;

        let node_count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM nodes",
            [],
            |row| row.get(0)
        )?;

        let edge_count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM edges",
            [],
            |row| row.get(0)
        )?;

        Ok((file_count, node_count, edge_count))
    }

    /// Clear all indexed data (for force re-index)
    pub fn clear_index(&self) -> Result<()> {
        self.conn.execute("DELETE FROM edges", [])?;
        self.conn.execute("DELETE FROM nodes", [])?;
        self.conn.execute("DELETE FROM files", [])?;
        Ok(())
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
