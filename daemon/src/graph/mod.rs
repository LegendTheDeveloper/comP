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

/// Cross-file symbol lookup: `name -> [(node_id, file_id, is_exported)]`.
pub type GlobalSymbolIndex = HashMap<String, Vec<(i64, i64, bool)>>;

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

    /// Register (or update) a repository root and return its id.
    ///
    /// `alias` is the short name that qualifies file paths and scopes queries;
    /// `root_path` is the absolute filesystem root of that repo. Idempotent: an
    /// existing alias keeps its id and has its root_path refreshed.
    pub fn upsert_repo(&self, alias: &str, root_path: &str) -> Result<i64> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("DB mutex poisoned: {}", e))?;
        conn.execute(
            "INSERT INTO repos (alias, root_path) VALUES (?, ?)
             ON CONFLICT(alias) DO UPDATE SET root_path = excluded.root_path",
            rusqlite::params![alias, root_path],
        )?;
        let id: i64 = conn.query_row("SELECT id FROM repos WHERE alias = ?", [alias], |row| row.get(0))?;
        Ok(id)
    }

    /// List all registered repos as (id, alias, root_path), ordered by id.
    ///
    /// Used by the MCP layer to resolve qualified paths ("<alias>/<rel>") back to
    /// absolute filesystem paths and to scope queries by repo alias.
    pub fn list_repos(&self) -> Result<Vec<(i64, String, String)>> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("DB mutex poisoned: {}", e))?;
        let mut stmt = conn.prepare("SELECT id, alias, root_path FROM repos ORDER BY id")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?))
        })?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    /// Insert/update a file in the database
    ///
    /// `char_count` is the UTF-8 byte length of the file content, stored as the
    /// real token-baseline for savings calculations in run_pipeline.
    ///
    /// Returns the file ID for use in subsequent operations
    pub fn upsert_file(&self, path: &str, hash: &str, language: &str, char_count: usize) -> Result<i64> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("DB mutex poisoned: {}", e))?;
        conn.execute(
            "INSERT OR REPLACE INTO files (path, hash, language, last_indexed, char_count)
             VALUES (?, ?, ?, strftime('%s', 'now'), ?)",
            rusqlite::params![path, hash, language, char_count as i64],
        )?;

        // Get the inserted/updated file ID
        let mut stmt = conn.prepare("SELECT id FROM files WHERE path = ?")?;
        let file_id: i64 = stmt.query_row([path], |row| row.get(0))?;

        Ok(file_id)
    }

    /// Insert a symbol node
    ///
    /// Returns the node ID for use in dependency tracking
    #[allow(clippy::too_many_arguments)] // 引数が多いが、全フィールドが必須のため構造体化による分割は過剰
    pub fn insert_node(
        &self,
        file_id: i64,
        name: &str,
        kind: &str,
        line: i32,
        col: i32,
        scope: Option<&str>,
        is_exported: bool,
        signature: Option<&str>,
    ) -> Result<i64> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("DB mutex poisoned: {}", e))?;
        let is_exported_int = if is_exported { 1 } else { 0 };
        conn.execute(
            "INSERT INTO nodes (file_id, name, kind, line, col, scope, is_exported, signature)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![file_id, name, kind, line, col, scope, is_exported_int, signature],
        )?;

        // Get the inserted node ID
        let last_id = conn.last_insert_rowid();
        Ok(last_id)
    }

    /// Store a file and all of its symbols in a single transaction.
    ///
    /// WHY: during indexing this replaces one `upsert_file` + N `insert_node`
    /// calls (each its own autocommit → an fsync/WAL frame commit per statement)
    /// with a single `BEGIN … COMMIT` and a reused prepared statement. Indexing
    /// is write-bound once parsing is parallelised, so batching the writes per
    /// file is the main throughput win. Uses `unchecked_transaction` because the
    /// connection is reached through a shared `&self` behind the Mutex we already
    /// hold — no concurrent transaction can exist on this connection.
    ///
    /// Returns the file ID.
    pub fn store_file_symbols(
        &self,
        path: &str,
        hash: &str,
        language: &str,
        char_count: usize,
        symbols: &[crate::indexer::parser::Symbol],
    ) -> Result<i64> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("DB mutex poisoned: {}", e))?;
        let tx = conn.unchecked_transaction()?;

        tx.execute(
            "INSERT OR REPLACE INTO files (path, hash, language, last_indexed, char_count)
             VALUES (?, ?, ?, strftime('%s', 'now'), ?)",
            rusqlite::params![path, hash, language, char_count as i64],
        )?;
        let file_id: i64 =
            tx.query_row("SELECT id FROM files WHERE path = ?", [path], |row| row.get(0))?;

        {
            let mut stmt = tx.prepare(
                "INSERT INTO nodes (file_id, name, kind, line, col, scope, is_exported, signature)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            )?;
            for s in symbols {
                stmt.execute(rusqlite::params![
                    file_id,
                    s.name,
                    s.kind.as_str(),
                    s.line as i32,
                    s.column as i32,
                    s.scope.as_deref(),
                    if s.is_exported { 1 } else { 0 },
                    s.signature.as_deref(),
                ])?;
            }
        }

        tx.commit()?;
        Ok(file_id)
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

    /// Build a global symbol index for cross-file dependency resolution.
    ///
    /// Returns: `name -> [(node_id, file_id, is_exported)]`. Used by
    /// `DependencyAnalyzer::resolve_global` to link a callee name to its
    /// definition in any indexed file.
    pub fn get_global_symbol_index(&self) -> Result<GlobalSymbolIndex> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("DB mutex poisoned: {}", e))?;
        let mut stmt = conn.prepare("SELECT name, id, file_id, is_exported FROM nodes")?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, i64>(3)? != 0,
            ))
        })?;
        let mut map: GlobalSymbolIndex = HashMap::new();
        for row in rows {
            let (name, id, file_id, is_exported) = row?;
            map.entry(name).or_default().push((id, file_id, is_exported));
        }
        Ok(map)
    }

    /// Delete all edges originating from a file's nodes.
    ///
    /// WHY: On re-index of a changed file, its outbound edges must be rebuilt
    /// from scratch; otherwise stale edges accumulate (FK CASCADE is not enabled).
    pub fn clear_file_edges(&self, file_id: i64) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("DB mutex poisoned: {}", e))?;
        conn.execute(
            "DELETE FROM edges WHERE from_id IN (SELECT id FROM nodes WHERE file_id = ?)",
            [file_id],
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

    /// Files connected to the given files by dependency edges (either direction),
    /// ranked by the number of connecting edges, excluding the input files themselves.
    ///
    /// WHY: run_pipeline returns pivot files by relevance; the blast radius around
    /// them (callers/callees in other files) is what `related_files` reports so an
    /// agent sees which files a change is likely to touch without a second query.
    pub fn get_related_files(&self, file_paths: &[String], limit: usize) -> Result<Vec<(String, usize)>> {
        if file_paths.is_empty() {
            return Ok(Vec::new());
        }
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("DB mutex poisoned: {}", e))?;
        let placeholders = vec!["?"; file_paths.len()].join(",");
        let sql = format!(
            "SELECT path, SUM(cnt) AS total FROM (
                 SELECT f2.path AS path, COUNT(*) AS cnt
                 FROM edges e
                 JOIN nodes n1 ON e.from_id = n1.id
                 JOIN nodes n2 ON e.to_id = n2.id
                 JOIN files f1 ON n1.file_id = f1.id
                 JOIN files f2 ON n2.file_id = f2.id
                 WHERE f1.path IN ({ph}) AND f2.path NOT IN ({ph})
                 GROUP BY f2.path
                 UNION ALL
                 SELECT f1.path AS path, COUNT(*) AS cnt
                 FROM edges e
                 JOIN nodes n1 ON e.from_id = n1.id
                 JOIN nodes n2 ON e.to_id = n2.id
                 JOIN files f1 ON n1.file_id = f1.id
                 JOIN files f2 ON n2.file_id = f2.id
                 WHERE f2.path IN ({ph}) AND f1.path NOT IN ({ph})
                 GROUP BY f1.path
             )
             GROUP BY path
             ORDER BY total DESC, path ASC
             LIMIT ?",
            ph = placeholders
        );

        // The IN-lists appear 4 times, then the LIMIT.
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::with_capacity(file_paths.len() * 4 + 1);
        for _ in 0..4 {
            for p in file_paths {
                params.push(Box::new(p.clone()));
            }
        }
        params.push(Box::new(limit as i64));

        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(
            rusqlite::params_from_iter(params.iter().map(|b| b.as_ref())),
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as usize)),
        )?;
        let result = rows.collect::<Result<Vec<_>, _>>()?;
        Ok(result)
    }

    /// Record a tool call's token consumption in the shared metadata table.
    ///
    /// This is the single write path for all token statistics.  Both the MCP
    /// daemon (Claude Code / Cursor) and the VSCode extension daemon share the
    /// same SQLite file, so writing here makes the numbers visible to both
    /// processes without any in-memory state synchronisation.
    ///
    /// `tokens_saved` is non-zero only for `run_pipeline`, which has a real
    /// full-workspace baseline.  All other tools pass 0 for `tokens_saved`.
    pub fn record_tool_call(&self, tokens_sent: u64, tokens_saved: u64) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("DB mutex poisoned: {}", e))?;
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

    /// Return the total char count across all indexed files divided by 4
    /// (the standard chars-to-tokens approximation).
    ///
    /// This is the honest baseline for run_pipeline savings: how many tokens
    /// an AI would consume if it read every file in the workspace verbatim.
    pub fn get_full_workspace_tokens(&self) -> Result<u64> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("DB mutex poisoned: {}", e))?;
        let total_chars: i64 = conn.query_row(
            "SELECT COALESCE(SUM(char_count), 0) FROM files",
            [],
            |row| row.get(0),
        )?;
        Ok((total_chars as u64) / 4)
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

    /// Get per-repo file/symbol counts for the multi-repo statistics panel.
    ///
    /// Returns (alias, root_path, file_count, node_count) for every registered
    /// repo, ordered by id (workspace root first). Counts are derived from the
    /// "<alias>/<rel>" qualified path prefix rather than a repo_id column on
    /// files, since that prefix is already the qualification scheme; the
    /// "/" delimiter in the LIKE pattern prevents one alias prefix-matching a
    /// different alias that merely starts with the same characters.
    pub fn get_repo_stats(&self) -> Result<Vec<(String, String, i64, i64)>> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("DB mutex poisoned: {}", e))?;
        let mut stmt = conn.prepare(
            "SELECT r.alias, r.root_path,
                    COUNT(DISTINCT f.id) AS file_count,
                    COUNT(n.id) AS node_count
             FROM repos r
             LEFT JOIN files f ON f.path LIKE r.alias || '/%'
             LEFT JOIN nodes n ON n.file_id = f.id
             GROUP BY r.id
             ORDER BY r.id",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, i64>(3)?,
            ))
        })?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
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

    /// Per-file char counts, keyed by file id.
    ///
    /// WHY: run_pipeline token estimates use real file sizes (chars/4) instead of
    /// symbol-count heuristics — a Markdown file with 3 headings and a 200-line
    /// function count the same in symbols but differ 10x in tokens.
    pub fn get_file_char_counts(&self) -> Result<HashMap<i64, i64>> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("DB mutex poisoned: {}", e))?;
        let mut stmt = conn.prepare("SELECT id, char_count FROM files")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
        })?;
        let mut map = HashMap::new();
        for row in rows {
            let (file_id, chars) = row?;
            map.insert(file_id, chars);
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
            map.entry(to_id).or_default().push(from_id);
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

    /// Remove a registered repo and every file/node/edge indexed under its
    /// "<alias>/<rel>" prefix. Same manual edges -> nodes -> files cascade as
    /// `delete_file` (foreign_keys pragma is not enabled), applied in bulk to
    /// every file under the alias, followed by the `repos` row itself.
    ///
    /// Returns the number of files removed. Caller is responsible for deciding
    /// whether `alias` is allowed to be removed (e.g. the workspace root
    /// shouldn't be) and for rebuilding the search index afterward.
    pub fn delete_repo(&self, alias: &str) -> Result<usize> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("DB mutex poisoned: {}", e))?;
        let pattern = format!("{}/%", alias);

        let file_ids: Vec<i64> = {
            let mut stmt = conn.prepare("SELECT id FROM files WHERE path LIKE ?")?;
            let ids = stmt.query_map([&pattern], |row| row.get::<_, i64>(0))?
                .collect::<Result<Vec<_>, _>>()?;
            ids
        };

        for fid in &file_ids {
            let node_ids: Vec<i64> = {
                let mut stmt = conn.prepare("SELECT id FROM nodes WHERE file_id = ?")?;
                let ids = stmt.query_map([fid], |row| row.get::<_, i64>(0))?
                    .collect::<Result<Vec<_>, _>>()?;
                ids
            };
            for nid in &node_ids {
                conn.execute(
                    "DELETE FROM edges WHERE from_id = ? OR to_id = ?",
                    rusqlite::params![nid, nid],
                )?;
            }
            conn.execute("DELETE FROM nodes WHERE file_id = ?", [fid])?;
        }

        let removed = conn.execute("DELETE FROM files WHERE path LIKE ?", [&pattern])?;
        conn.execute("DELETE FROM repos WHERE alias = ?", [alias])?;

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

    /// Get file ID by its relative path
    pub fn get_file_id_by_path(&self, path: &str) -> Result<Option<i64>> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("DB mutex poisoned: {}", e))?;
        let mut stmt = conn.prepare("SELECT id FROM files WHERE path = ?")?;
        let mut rows = stmt.query_map([path], |row| row.get(0))?;
        if let Some(row) = rows.next() {
            Ok(Some(row?))
        } else {
            Ok(None)
        }
    }

    /// Find nodes matching symbol name and optional file_id
    pub fn get_symbols_by_name(&self, name: &str, file_id: Option<i64>) -> Result<Vec<DbNode>> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("DB mutex poisoned: {}", e))?;
        let mut result = Vec::new();
        if let Some(fid) = file_id {
            let mut stmt = conn.prepare(
                "SELECT id, file_id, name, kind, line, col, scope, is_exported, signature 
                 FROM nodes WHERE name = ? AND file_id = ?"
            )?;
            let mapped = stmt.query_map(rusqlite::params![name, fid], |row| {
                Ok(DbNode {
                    id: row.get(0)?,
                    file_id: row.get(1)?,
                    name: row.get(2)?,
                    kind: row.get(3)?,
                    line: row.get(4)?,
                    col: row.get(5)?,
                    scope: row.get(6)?,
                    is_exported: row.get(7).unwrap_or(0),
                    signature: row.get(8)?,
                })
            })?;
            for item in mapped {
                result.push(item?);
            }
        } else {
            let mut stmt = conn.prepare(
                "SELECT id, file_id, name, kind, line, col, scope, is_exported, signature 
                 FROM nodes WHERE name = ?"
            )?;
            let mapped = stmt.query_map(rusqlite::params![name], |row| {
                Ok(DbNode {
                    id: row.get(0)?,
                    file_id: row.get(1)?,
                    name: row.get(2)?,
                    kind: row.get(3)?,
                    line: row.get(4)?,
                    col: row.get(5)?,
                    scope: row.get(6)?,
                    is_exported: row.get(7).unwrap_or(0),
                    signature: row.get(8)?,
                })
            })?;
            for item in mapped {
                result.push(item?);
            }
        }
        Ok(result)
    }

    /// Get all nodes for a specific file, sorted by line
    pub fn get_file_symbols_sorted(&self, file_id: i64) -> Result<Vec<DbNode>> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("DB mutex poisoned: {}", e))?;
        let mut stmt = conn.prepare(
            "SELECT id, file_id, name, kind, line, col, scope, is_exported, signature 
             FROM nodes WHERE file_id = ? ORDER BY line, col"
        )?;
        let mapped = stmt.query_map([file_id], |row| {
            Ok(DbNode {
                id: row.get(0)?,
                file_id: row.get(1)?,
                name: row.get(2)?,
                kind: row.get(3)?,
                line: row.get(4)?,
                col: row.get(5)?,
                scope: row.get(6)?,
                is_exported: row.get(7).unwrap_or(0),
                signature: row.get(8)?,
            })
        })?;
        let mut result = Vec::new();
        for item in mapped {
            result.push(item?);
        }
        Ok(result)
    }

    /// Get nodes that this node depends on (outbound edges)
    pub fn get_node_dependencies_out(&self, node_id: i64) -> Result<Vec<(DbNode, String)>> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("DB mutex poisoned: {}", e))?;
        let mut stmt = conn.prepare(
            "SELECT n.id, n.file_id, n.name, n.kind, n.line, n.col, n.scope, n.is_exported, n.signature, e.kind
             FROM edges e
             JOIN nodes n ON e.to_id = n.id
             WHERE e.from_id = ?"
        )?;
        let mapped = stmt.query_map([node_id], |row| {
            Ok((
                DbNode {
                    id: row.get(0)?,
                    file_id: row.get(1)?,
                    name: row.get(2)?,
                    kind: row.get(3)?,
                    line: row.get(4)?,
                    col: row.get(5)?,
                    scope: row.get(6)?,
                    is_exported: row.get(7).unwrap_or(0),
                    signature: row.get(8)?,
                },
                row.get(9)?,
            ))
        })?;
        let mut result = Vec::new();
        for item in mapped {
            result.push(item?);
        }
        Ok(result)
    }

    /// Get nodes that depend on this node (inbound edges)
    pub fn get_node_dependencies_in(&self, node_id: i64) -> Result<Vec<(DbNode, String)>> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("DB mutex poisoned: {}", e))?;
        let mut stmt = conn.prepare(
            "SELECT n.id, n.file_id, n.name, n.kind, n.line, n.col, n.scope, n.is_exported, n.signature, e.kind
             FROM edges e
             JOIN nodes n ON e.from_id = n.id
             WHERE e.to_id = ?"
        )?;
        let mapped = stmt.query_map([node_id], |row| {
            Ok((
                DbNode {
                    id: row.get(0)?,
                    file_id: row.get(1)?,
                    name: row.get(2)?,
                    kind: row.get(3)?,
                    line: row.get(4)?,
                    col: row.get(5)?,
                    scope: row.get(6)?,
                    is_exported: row.get(7).unwrap_or(0),
                    signature: row.get(8)?,
                },
                row.get(9)?,
            ))
        })?;
        let mut result = Vec::new();
        for item in mapped {
            result.push(item?);
        }
        Ok(result)
    }

    /// Get file path by its ID
    pub fn get_file_path_by_id(&self, file_id: i64) -> Result<String> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("DB mutex poisoned: {}", e))?;
        let path: String = conn.query_row(
            "SELECT path FROM files WHERE id = ?",
            [file_id],
            |row| row.get(0)
        )?;
        Ok(path)
    }

    /// Get all symbols for building the TF-IDF search index
    ///
    /// WHY: SearchEngine.build_index requires all (file_path, name, kind, line) tuples.
    /// Called once after indexing completes to populate the in-memory TF-IDF matrix.
    pub fn get_all_symbols_for_search(&self) -> Result<Vec<(String, String, String, u32)>> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("DB mutex poisoned: {}", e))?;
        let mut stmt = conn.prepare(
            "SELECT files.path, nodes.name, nodes.kind, nodes.line
             FROM nodes JOIN files ON nodes.file_id = files.id"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i32>(3)? as u32,
            ))
        })?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    /// Get all exported symbols, ordered by file path
    pub fn get_exported_symbols_grouped(&self) -> Result<Vec<(String, DbNode)>> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("DB mutex poisoned: {}", e))?;
        let mut stmt = conn.prepare(
            "SELECT f.path, n.id, n.file_id, n.name, n.kind, n.line, n.col, n.scope, n.is_exported, n.signature
             FROM nodes n
             JOIN files f ON n.file_id = f.id
             WHERE n.is_exported = 1
             ORDER BY f.path, n.name"
        )?;
        let mapped = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                DbNode {
                    id: row.get(1)?,
                    file_id: row.get(2)?,
                    name: row.get(3)?,
                    kind: row.get(4)?,
                    line: row.get(5)?,
                    col: row.get(6)?,
                    scope: row.get(7)?,
                    is_exported: row.get(8).unwrap_or(0),
                    signature: row.get(9)?,
                },
            ))
        })?;
        let mut result = Vec::new();
        for item in mapped {
            result.push(item?);
        }
        Ok(result)
    }
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct DbNode {
    pub id: i64,
    pub file_id: i64,
    pub name: String,
    pub kind: String,
    pub line: i32,
    pub col: i32,
    pub scope: Option<String>,
    pub is_exported: i32,
    pub signature: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_graphdb_creation() {
        let temp_dir = std::env::temp_dir().join("comP_test_graphdb_creation");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        let db = GraphDB::new(temp_dir.to_str().unwrap()).await.unwrap();
        let (files, nodes, edges) = db.get_stats().unwrap();
        assert_eq!((files, nodes, edges), (0, 0, 0), "fresh DB must be empty");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[tokio::test]
    async fn test_get_related_files() {
        // a.rs and b.rs are pivots; c.rs is connected to a.rs by two edges
        // (one in each direction) and d.rs is isolated. Expect only c.rs,
        // with the edge count summed across directions.
        let temp_dir = std::env::temp_dir().join("comP_test_get_related_files");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        let db = GraphDB::new(temp_dir.to_str().unwrap()).await.unwrap();
        let fa = db.upsert_file("a.rs", "h1", "rust", 10).unwrap();
        let fb = db.upsert_file("b.rs", "h2", "rust", 10).unwrap();
        let fc = db.upsert_file("c.rs", "h3", "rust", 10).unwrap();
        let fd = db.upsert_file("d.rs", "h4", "rust", 10).unwrap();

        let na = db.insert_node(fa, "alpha", "fn", 1, 0, None, true, None).unwrap();
        let nb = db.insert_node(fb, "beta", "fn", 1, 0, None, true, None).unwrap();
        let nc = db.insert_node(fc, "gamma", "fn", 1, 0, None, true, None).unwrap();
        let _nd = db.insert_node(fd, "delta", "fn", 1, 0, None, true, None).unwrap();

        db.insert_edge(na, nc, "calls").unwrap(); // pivot → related
        db.insert_edge(nc, na, "calls").unwrap(); // related → pivot
        db.insert_edge(na, nb, "calls").unwrap(); // pivot → pivot: must be excluded

        let pivots = vec!["a.rs".to_string(), "b.rs".to_string()];
        let related = db.get_related_files(&pivots, 10).unwrap();
        assert_eq!(related, vec![("c.rs".to_string(), 2)]);

        // Empty input short-circuits.
        assert!(db.get_related_files(&[], 10).unwrap().is_empty());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
