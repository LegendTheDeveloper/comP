// mod.rs - Model Context Protocol (MCP) Server
//
// Exposes 5 core tools to AI agents (Claude Code, Cursor, Cline, etc.):
// 1. run_pipeline - Full context generation + impact analysis
// 2. get_context - Extract optimized code context
// 3. get_impact_graph - Show code affected by symbol change
// 4. list_indexed_files - Show all indexed files with stats
// 5. get_token_usage - Show token consumption statistics
//
// Protocol: JSON-RPC 2.0 over stdio

use anyhow::{Result, anyhow};
use log::info;
use serde_json::{json, Value};
use std::sync::Arc;
use std::sync::atomic::Ordering;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct SessionCall {
    pub query: String,
    pub symbols: Vec<String>,
    pub files: Vec<String>,
    pub tokens: u64,
    pub stale: bool,
    pub timestamp: u64,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct Session {
    pub id: String,
    pub timestamp: u64,
    pub calls: Vec<SessionCall>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct SessionMemory {
    pub sessions: Vec<Session>,
}

fn get_session_memory_path() -> std::path::PathBuf {
    let root = std::env::var("COMP_WORKSPACE_ROOT")
        .or_else(|_| std::env::var("WORKSPACE_ROOT"))
        .unwrap_or_else(|_| ".".to_string());
    std::path::Path::new(&root).join(".comp").join("session-memory.json")
}

fn record_mcp_call(
    session_id: &str,
    query: String,
    symbols: Vec<String>,
    files: Vec<String>,
    tokens: u64,
) -> Result<()> {
    let path = get_session_memory_path();
    
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut memory: SessionMemory = if path.exists() {
        let file = std::fs::File::open(&path)?;
        let reader = std::io::BufReader::new(file);
        serde_json::from_reader(reader).unwrap_or(SessionMemory { sessions: Vec::new() })
    } else {
        SessionMemory { sessions: Vec::new() }
    };

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    let mut found = false;
    for session in &mut memory.sessions {
        if session.id == session_id {
            session.calls.push(SessionCall {
                query: query.clone(),
                symbols: symbols.clone(),
                files: files.clone(),
                tokens,
                stale: false,
                timestamp: now,
            });
            found = true;
            break;
        }
    }

    if !found {
        memory.sessions.push(Session {
            id: session_id.to_string(),
            timestamp: now,
            calls: vec![SessionCall {
                query,
                symbols,
                files,
                tokens,
                stale: false,
                timestamp: now,
            }],
        });
    }

    let file = std::fs::File::create(&path)?;
    let writer = std::io::BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &memory)?;

    Ok(())
}


/// MCP Server
///
/// Listens on stdin for JSON-RPC 2.0 requests
/// Sends JSON-RPC 2.0 responses to stdout
pub struct MCPServer {
    state: Arc<crate::AppState>,
}

impl MCPServer {
    /// Create a new MCP server
    pub fn new(state: Arc<crate::AppState>) -> Self {
        MCPServer { state }
    }

    /// Run the MCP server
    ///
    /// Listens on stdin for JSON-RPC requests
    /// Writes JSON-RPC responses to stdout
    ///
    /// # Protocol:
    /// Request:
    /// ```json
    /// { "jsonrpc": "2.0", "id": 1, "method": "run_pipeline", "params": { "task": "..." } }
    /// ```
    ///
    /// Response:
    /// ```json
    /// { "jsonrpc": "2.0", "id": 1, "result": { ... } }
    /// ```
    pub async fn run(&self) -> Result<()> {
        use std::io::{self, BufRead, Write};

        let stdin = io::stdin();
        let reader = stdin.lock();
        let mut stdout = io::stdout();

        for line in reader.lines() {
            let line = line?;
            if line.is_empty() {
                continue;
            }

            // Parse JSON-RPC request
            let request: Value = match serde_json::from_str(&line) {
                Ok(v) => v,
                Err(_) => {
                    let error = json!({
                        "jsonrpc": "2.0",
                        "error": {
                            "code": -32700,
                            "message": "Parse error"
                        }
                    });
                    writeln!(stdout, "{}", error)?;
                    continue;
                }
            };

            // Extract method and params
            let method = request["method"].as_str().unwrap_or("");
            let params = request["params"].clone();
            let id = request["id"].clone();

            // Call appropriate handler
            let result = match method {
                "initialize" => self.handle_initialize(params).await,
                "tools/list" => self.handle_tools_list().await,
                "tools/call" => self.handle_tools_call(params).await,
                "run_pipeline" => self.handle_run_pipeline(params).await,
                "get_context" => self.handle_get_context(params).await,
                "get_impact_graph" => self.handle_get_impact_graph(params).await,
                "list_indexed_files" => self.handle_list_indexed_files().await,
                "get_token_usage" => self.handle_get_token_usage().await,
                "getStats" => self.handle_get_stats().await,
                "forceReindex" => self.handle_force_reindex().await,
                "indexFile" => self.handle_index_file(params).await,
                "removeFile" => self.handle_remove_file(params).await,
                "session_recall" => self.handle_session_recall(params).await,
                _ => Err(anyhow!("Unknown method: {}", method)),
            };

            // Build response
            let response = match result {
                Ok(result_value) => json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": result_value
                }),
                Err(e) => json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32603,
                        "message": "Internal error",
                        "data": e.to_string()
                    }
                }),
            };

            // Write response
            writeln!(stdout, "{}", response)?;
            stdout.flush()?;
        }

        Ok(())
    }

    /// Tool 1: run_pipeline
    ///
    /// Full pipeline: task → search → context generation → token counting
    ///
    /// # Request:
    /// ```json
    /// {
    ///   "task": "add user authentication to login endpoint",
    ///   "max_tokens": 8000,
    ///   "include_tests": true
    /// }
    /// ```
    ///
    /// # Response:
    /// ```json
    /// {
    ///   "pivot_files": [
    ///     { "path": "src/auth/authenticate.ts", "symbols": 5, "tokens": 500 }
    ///   ],
    ///   "related_files": [
    ///     { "path": "src/types/user.ts", "symbols": 3, "tokens": 200 }
    ///   ],
    ///   "total_tokens": 700,
    ///   "savings": "62%",
    ///   "estimated_cost": "$0.02"
    /// }
    /// ```
    ///
    /// # Process:
    /// 1. Extract task description from params
    /// 2. Perform semantic search using task as query
    /// 3. Select top results as pivot_files (most relevant)
    /// 4. Gather related files from impact graph
    /// 5. Count tokens across all files
    /// 6. Calculate savings percentage
    /// 7. Estimate API cost based on model
    pub async fn handle_run_pipeline(&self, params: Value) -> Result<Value> {
        let task = params["task"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing 'task' parameter"))?;
        let max_tokens = params["max_tokens"].as_u64().unwrap_or(8000) as usize;

        // 1. Split task into words and query symbol name LIKE for each keyword -> pivot candidates
        // WHY: A LIKE query on the entire task string (e.g. "fix auth bug") would return 0 hits.
        //      We search each word individually and merge with OR to match related files.
        let keywords: Vec<&str> = task
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .filter(|w| w.len() >= 3)
            .collect();

        let mut all_hits: Vec<(String, String, String, i32)> = if keywords.is_empty() {
            self.state.graph_db.search_symbols_by_name(task, 10)?
        } else {
            let mut merged = Vec::new();
            for kw in &keywords {
                merged.extend(self.state.graph_db.search_symbols_by_name(kw, 5)?);
            }
            merged
        };
        // Sort files by first occurrence before applying limits
        all_hits.dedup_by(|a, b| a.0 == b.0 && a.1 == b.1);
        let hits = all_hits;
        let symbol_counts = self.state.graph_db.count_symbols_per_file()?;
        let files_list = self.state.graph_db.list_files()?;
        // WHY: Save language info before consuming. BM25 requires the list of Markdown files.
        let markdown_paths: Vec<String> = files_list
            .iter()
            .filter(|(_, _, lang)| lang == "markdown")
            .map(|(_, path, _)| path.clone())
            .collect();
        let path_to_id: std::collections::HashMap<String, i64> = files_list
            .into_iter()
            .map(|(id, path, _)| (path, id))
            .collect();

        // 2. Deduplicate matched files to build pivot files list
        let mut seen = std::collections::HashSet::new();
        let mut pivot_files = Vec::new();
        let mut pivot_paths: Vec<String> = Vec::new();
        let mut recorded_symbols = Vec::new();
        let mut recorded_files = Vec::new();
        for (file, name, _kind, _line) in hits {
            recorded_symbols.push(name.clone());
            recorded_files.push(file.clone());
            if seen.insert(file.clone()) {
                let sym = path_to_id
                    .get(&file)
                    .and_then(|id| symbol_counts.get(id))
                    .copied()
                    .unwrap_or(0);
                // WHY: sym*20 is an underestimation. An average function has ~50 lines x ~1 token/line, closer to sym*50.
                //      We estimate based on line count approximations since file sizes aren't stored in DB.
                let tokens = (sym as usize).saturating_mul(50);
                pivot_files.push(json!({
                    "path": file.clone(),
                    "symbols": sym,
                    "tokens": tokens
                }));
                pivot_paths.push(file);
            }
        }

        // BM25 full-text search (complements search for Markdown files)
        // WHY: Symbol LIKE queries only match headings, missing body content keywords.
        //      We read Markdown files and score using BM25, then add to pivot files.
        if !markdown_paths.is_empty() && !keywords.is_empty() {
            let workspace_root = std::env::var("COMP_WORKSPACE_ROOT")
                .unwrap_or_else(|_| ".".to_string());
            let bm25_hits = crate::indexer::doc_parser::Bm25Scorer::search_files(
                &workspace_root,
                &markdown_paths,
                &keywords,
                20,
            );
            for (path, _score) in bm25_hits {
                recorded_files.push(path.clone());
                if seen.insert(path.clone()) {
                    let sym = path_to_id
                        .get(&path)
                        .and_then(|id| symbol_counts.get(id))
                        .copied()
                        .unwrap_or(0);
                    let tokens = (sym as usize).saturating_mul(50);
                    pivot_files.push(json!({
                        "path": path.clone(),
                        "symbols": sym,
                        "tokens": tokens
                    }));
                    pivot_paths.push(path);
                }
            }
        }

        let total_tokens: usize = pivot_files
            .iter()
            .filter_map(|v| v["tokens"].as_u64())
            .map(|t| t as usize)
            .sum();

        // 3. Estimate total workspace tokens (for calculating savings ratio)
        let total_symbols: i64 = symbol_counts.values().sum();
        let full_workspace_tokens = (total_symbols as usize).saturating_mul(50).max(total_tokens + 1);

        let savings = crate::search::TokenCounter::calculate_savings(full_workspace_tokens, total_tokens);
        let cost = crate::search::TokenCounter::estimate_cost(total_tokens, "sonnet");

        // Update accumulated token statistics for the session
        let saved_this_call = full_workspace_tokens.saturating_sub(total_tokens) as u64;
        self.state.tokens_sent.fetch_add(total_tokens as u64, Ordering::Relaxed);
        self.state.tokens_saved.fetch_add(saved_this_call, Ordering::Relaxed);
        self.state.queries_count.fetch_add(1, Ordering::Relaxed);
        // Persist to shared DB so the VSCode extension's daemon (separate process) can read these stats
        let _ = self.state.graph_db.increment_token_stats(total_tokens as u64, saved_this_call);

        // Record this call to session memory
        recorded_symbols.sort();
        recorded_symbols.dedup();
        recorded_files.sort();
        recorded_files.dedup();
        let _ = record_mcp_call(
            &self.state.session_id,
            task.to_string(),
            recorded_symbols,
            recorded_files,
            total_tokens as u64,
        );

        Ok(json!({
            "task": task,
            "pivot_files": pivot_files,
            "related_files": [],  // TODO: calculate via impact graph (Phase 5)
            "total_tokens": total_tokens,
            "max_tokens": max_tokens,
            "savings": savings,
            "compression_ratio": savings,
            "full_workspace_tokens": full_workspace_tokens,
            "estimated_cost": cost
        }))
    }

    /// Tool 2: get_context
    ///
    /// Extract relevant code context based on search query
    ///
    /// # Request:
    /// ```json
    /// {
    ///   "query": "authentication functions",
    ///   "limit": 10,
    ///   "kind_filter": "function"
    /// }
    /// ```
    ///
    /// # Response:
    /// ```json
    /// {
    ///   "results": [
    ///     { "file": "src/auth/authenticate.ts", "symbol": "authenticate", "line": 10, "score": 0.95 }
    ///   ],
    ///   "count": 1,
    ///   "query": "authentication functions"
    /// }
    /// ```
    ///
    /// # Process:
    /// 1. Extract query from params
    /// 2. Perform semantic search using TF-IDF
    /// 3. Optionally filter by symbol kind
    /// 4. Return results ranked by relevance score
    pub async fn handle_get_context(&self, params: Value) -> Result<Value> {
        let query = params["query"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing 'query' parameter"))?;
        let limit = params["limit"]
            .as_u64()
            .unwrap_or(10) as usize;
        let kind_filter = params["kind_filter"].as_str();

        // Query GraphDB directly via LIKE (since SearchEngine TF-IDF is not yet built)
        let hits = self.state.graph_db.search_symbols_by_name(query, limit * 2)?;

        let results: Vec<Value> = hits
            .into_iter()
            .filter(|(_, _, kind, _)| kind_filter.is_none_or(|f| f == kind))
            .take(limit)
            .map(|(file, name, kind, line)| {
                json!({
                    "file": file,
                    "symbol": name,
                    "kind": kind,
                    "line": line,
                    "score": 1.0  // Score is fixed for LIKE queries. To be replaced after TF-IDF implementation.
                })
            })
            .collect();

        let count = results.len();

        // Record this call to session memory
        let mut recorded_symbols = Vec::new();
        let mut recorded_files = Vec::new();
        for res in &results {
            if let Some(sym) = res["symbol"].as_str() {
                recorded_symbols.push(sym.to_string());
            }
            if let Some(file) = res["file"].as_str() {
                recorded_files.push(file.to_string());
            }
        }
        recorded_symbols.sort();
        recorded_symbols.dedup();
        recorded_files.sort();
        recorded_files.dedup();
        let estimated_tokens = (count as u64) * 50;
        let _ = record_mcp_call(
            &self.state.session_id,
            query.to_string(),
            recorded_symbols,
            recorded_files,
            estimated_tokens,
        );

        Ok(json!({
            "query": query,
            "results": results,
            "count": count,
            "limit": limit
        }))
    }

    /// Tool 3: get_impact_graph
    ///
    /// Show all code affected by changing a symbol
    ///
    /// # Request:
    /// ```json
    /// {
    ///   "symbol_id": 123,
    ///   "symbol_name": "authenticate"
    /// }
    /// ```
    ///
    /// # Response:
    /// ```json
    /// {
    ///   "symbol": "authenticate",
    ///   "affected_files": {
    ///     "src/routes/login.ts": ["handleLogin", "validateCredentials"],
    ///     "src/middleware/auth.ts": ["authMiddleware"]
    ///   },
    ///   "impact_count": 3,
    ///   "severity": "high"
    /// }
    /// ```
    ///
    /// # Process:
    /// 1. Extract symbol_id from params
    /// 2. Query GraphDB for all symbols that depend on this symbol
    /// 3. Recursively traverse impact graph (BFS)
    /// 4. Group affected symbols by file
    /// 5. Calculate severity based on impact count
    pub async fn handle_get_impact_graph(&self, params: Value) -> Result<Value> {
        let symbol_id = params["symbol_id"]
            .as_i64()
            .ok_or_else(|| anyhow!("Missing 'symbol_id' parameter"))?;
        let symbol_name = params["symbol_name"].as_str().unwrap_or("unknown");

        // Build reverse dependency & symbol maps from GraphDB and invoke SearchEngine BFS
        let reverse_deps = self.state.graph_db.get_reverse_deps()?;
        let symbol_map = self.state.graph_db.get_symbol_map()?;

        let search_engine = self.state.search_engine.lock().await;
        let impact = search_engine.get_impact_graph(symbol_id, &reverse_deps, &symbol_map)?;
        drop(search_engine);

        let mut affected_obj = serde_json::Map::new();
        let mut impact_count: usize = 0;
        for (file, symbols) in &impact {
            impact_count += symbols.len();
            affected_obj.insert(file.clone(), json!(symbols));
        }

        let severity = match impact_count {
            0 => "none",
            1..=5 => "low",
            6..=20 => "medium",
            _ => "high",
        };

        Ok(json!({
            "symbol_id": symbol_id,
            "symbol": symbol_name,
            "affected_files": Value::Object(affected_obj),
            "impact_count": impact_count,
            "severity": severity
        }))
    }

    /// Tool 4: list_indexed_files
    ///
    /// List all indexed files with statistics
    ///
    /// # Response:
    /// ```json
    /// {
    ///   "files": [
    ///     { "path": "src/main.rs", "language": "rust", "symbols": 15 }
    ///   ],
    ///   "total_files": 42,
    ///   "total_symbols": 523,
    ///   "languages": { "rust": 20, "typescript": 22 }
    /// }
    /// ```
    ///
    /// # Process:
    /// 1. Query GraphDB for all files
    /// 2. Count symbols per file
    /// 3. Group by language
    /// 4. Calculate totals and statistics
    pub async fn handle_list_indexed_files(&self) -> Result<Value> {
        let files_raw = self.state.graph_db.list_files()?;
        let symbol_counts = self.state.graph_db.count_symbols_per_file()?;

        let mut total_symbols: i64 = 0;
        let mut languages: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
        let files: Vec<Value> = files_raw
            .into_iter()
            .map(|(id, path, language)| {
                let sym = *symbol_counts.get(&id).unwrap_or(&0);
                total_symbols += sym;
                *languages.entry(language.clone()).or_insert(0) += 1;
                json!({
                    "path": path,
                    "language": language,
                    "symbols": sym
                })
            })
            .collect();
        let total_files = files.len();

        Ok(json!({
            "files": files,
            "total_files": total_files,
            "total_symbols": total_symbols,
            "languages": languages
        }))
    }

    /// Tool 5: get_token_usage
    ///
    /// Show token consumption statistics and metrics
    ///
    /// # Response:
    /// ```json
    /// {
    ///   "total_tokens_consumed": 45000,
    ///   "queries_executed": 15,
    ///   "average_tokens_per_query": 3000,
    ///   "timestamp": 1716226421,
    ///   "efficiency": "75%"
    /// }
    /// ```
    ///
    /// # Process:
    /// 1. Retrieve statistics from internal counter
    /// 2. Calculate average tokens per query
    /// 3. Calculate efficiency (context optimization benefit)
    /// 4. Add current timestamp
    pub async fn handle_get_token_usage(&self) -> Result<Value> {
        let sent = self.state.tokens_sent.load(Ordering::Relaxed);
        let saved = self.state.tokens_saved.load(Ordering::Relaxed);
        let queries = self.state.queries_count.load(Ordering::Relaxed);
        let avg = sent.checked_div(queries).unwrap_or(0);
        let efficiency = (saved * 100).checked_div(sent + saved)
            .map(|e| format!("{}%", e))
            .unwrap_or_else(|| "0%".to_string());
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Ok(json!({
            "total_tokens_consumed": sent,
            "total_tokens_saved": saved,
            "queries_executed": queries,
            "average_tokens_per_query": avg,
            "timestamp": timestamp,
            "efficiency": efficiency
        }))
    }

    /// Force re-index of entire workspace
    ///
    /// WHY: Called from VSCode `comp.forceReindex` command.
    /// Clears and rebuilds the database. Response might be slow since indexing is blocking (beware of timeouts).
    pub async fn handle_force_reindex(&self) -> Result<Value> {
        info!("handle_force_reindex: clearing index");
        self.state.graph_db.clear_index()?;

        let workspace_root = std::env::var("COMP_WORKSPACE_ROOT")
            .or_else(|_| std::env::var("WORKSPACE_ROOT"))
            .unwrap_or_else(|_| ".".to_string());

        info!("handle_force_reindex: rebuilding index for {}", workspace_root);
        let mut indexer = crate::indexer::Indexer::new(&workspace_root);
        let (total, indexed, symbols) = indexer
            .index_workspace(None, &self.state.graph_db)
            .await?;

        let (files, nodes, edges) = self.state.graph_db.get_stats()?;
        info!(
            "handle_force_reindex: complete - {}/{} files, {} symbols, {} nodes, {} edges",
            indexed, total, symbols, nodes, edges
        );

        Ok(json!({
            "total_files": files,
            "total_nodes": nodes,
            "total_edges": edges,
            "indexed_files": indexed,
            "scanned_files": total,
            "symbols_extracted": symbols
        }))
    }

    /// Index a single file (incremental update)
    ///
    /// WHY: Called by VSCode's FileSystemWatcher for each modified file.
    /// The previous implementation was a TODO that did not write to DB, which left stats unreflected.
    pub async fn handle_index_file(&self, params: Value) -> Result<Value> {
        let path_str = params["path"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing 'path' parameter"))?;

        let workspace_root = std::env::var("COMP_WORKSPACE_ROOT")
            .or_else(|_| std::env::var("WORKSPACE_ROOT"))
            .unwrap_or_else(|_| ".".to_string());

        let mut indexer = crate::indexer::Indexer::new(&workspace_root);
        indexer
            .index_file(std::path::Path::new(path_str), &self.state.graph_db)
            .await?;

        Ok(json!({ "status": "ok", "path": path_str }))
    }

    /// Remove a file from the index
    ///
    /// Called by VSCode's onDidDelete. Converts absolute path to workspace-relative and deletes from DB.
    pub async fn handle_remove_file(&self, params: Value) -> Result<Value> {
        let path_str = params["path"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing 'path' parameter"))?;

        let workspace_root = std::env::var("COMP_WORKSPACE_ROOT")
            .or_else(|_| std::env::var("WORKSPACE_ROOT"))
            .unwrap_or_else(|_| ".".to_string());

        // Convert to workspace-relative path if absolute, and normalize \ to / (DB uses / unified)
        let relative_path = std::path::Path::new(path_str)
            .strip_prefix(&workspace_root)
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_else(|_| path_str.replace('\\', "/"));

        let removed = self.state.graph_db.delete_file(&relative_path)?;
        info!("handle_remove_file: deleted {} ({} rows)", relative_path, removed);

        Ok(json!({
            "status": "ok",
            "path": relative_path,
            "removed": removed
        }))
    }

    /// Get index statistics (file count, node count, edge count)
    ///
    /// # Response:
    /// ```json
    /// {
    ///   "total_files": 42,
    ///   "total_nodes": 1250,
    ///   "total_edges": 890
    /// }
    /// ```
    pub async fn handle_get_stats(&self) -> Result<Value> {
        info!("handle_get_stats: called");

        let (file_count, node_count, edge_count) = self.state.graph_db.get_stats()?;
        // Read from shared DB (not in-memory) so VSCode extension's daemon sees stats
        // accumulated by the Claude Code MCP daemon in the same workspace
        let (sent, saved, queries) = self.state.graph_db.get_token_stats().unwrap_or((0, 0, 0));
        let efficiency = (saved * 100).checked_div(sent + saved)
            .map(|e| format!("{}%", e))
            .unwrap_or_else(|| "0%".to_string());
        let avg_tokens_per_query = sent.checked_div(queries).unwrap_or(0);

        info!("handle_get_stats: returning stats - files: {}, nodes: {}, edges: {}",
              file_count, node_count, edge_count);

        Ok(json!({
            "total_files": file_count,
            "total_nodes": node_count,
            "total_edges": edge_count,
            "tokens_sent": sent,
            "tokens_saved": saved,
            "queries_count": queries,
            "efficiency": efficiency,
            "compression_ratio": efficiency,
            "avg_tokens_per_query": avg_tokens_per_query
        }))
    }

    /// MCP initialize handshake — returns server capabilities
    pub async fn handle_initialize(&self, _params: Value) -> Result<Value> {
        Ok(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": { "tools": {} },
            "serverInfo": { "name": "comP", "version": "0.1.0" }
        }))
    }

    /// MCP tools/list — AI-accessible tools with precise descriptions and input schemas
    ///
    /// WHY: Description quality directly controls when AI agents call each tool.
    /// "Call ONLY when..." / "Do NOT call..." constraints prevent accidental invocations
    /// that would pollute the context window mid-implementation.
    pub async fn handle_tools_list(&self) -> Result<Value> {
        Ok(json!({
            "tools": [
                {
                    "name": "run_pipeline",
                    "description": "Call at the START of a new coding task (bug fix, feature, refactor) to retrieve the most relevant files and symbols for that task. Do NOT call mid-implementation or for general questions. Returns pivot files and related symbols ranked by relevance.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "task": {
                                "type": "string",
                                "description": "One sentence describing what you are about to implement or fix. Example: 'fix JWT token expiry bug in auth middleware'"
                            },
                            "max_tokens": {
                                "type": "integer",
                                "description": "Token budget for returned context. Default: 8000"
                            },
                            "include_tests": {
                                "type": "boolean",
                                "description": "Include test files in results. Default: false"
                            }
                        },
                        "required": ["task"]
                    }
                },
                {
                    "name": "get_context",
                    "description": "Search for specific symbols (functions, classes, types) by name or keyword. Use when you know the exact name to look up. Do NOT use for starting a new task — use run_pipeline instead.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "query": {
                                "type": "string",
                                "description": "Symbol name or keyword. Example: 'authenticate' or 'UserRepository'"
                            },
                            "limit": {
                                "type": "integer",
                                "description": "Max results to return. Default: 10"
                            },
                            "kind_filter": {
                                "type": "string",
                                "description": "Filter by symbol kind: 'function', 'class', 'type', or 'variable'"
                            }
                        },
                        "required": ["query"]
                    }
                },
                {
                    "name": "get_impact_graph",
                    "description": "Show which files and symbols are affected when a specific symbol changes. Call before modifying a function or class to understand the blast radius. Requires a numeric symbol_id from a prior run_pipeline or get_context result.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "symbol_id": {
                                "type": "integer",
                                "description": "Numeric ID of the symbol to analyze (obtained from run_pipeline or get_context)"
                            },
                            "symbol_name": {
                                "type": "string",
                                "description": "Human-readable name for display purposes only"
                            }
                        },
                        "required": ["symbol_id"]
                    }
                },
                {
                    "name": "list_indexed_files",
                    "description": "List all indexed files with symbol counts and language breakdown. Use to understand overall codebase structure. Do NOT use to find relevant files for a task — use run_pipeline for that.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {}
                    }
                },
                {
                    "name": "session_recall",
                    "description": "Recall past MCP tool invocations (queries, symbols, tokens) for the current session. Returns a Markdown list with stale status. If query is provided, filters the history by query similarity or matching.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "query": {
                                "type": "string",
                                "description": "Optional search query to filter past invocations"
                            }
                        }
                    }
                }
            ]
        }))
    }

    /// MCP tools/call — dispatches to the appropriate tool handler
    ///
    /// WHY: Standard MCP agents call tools/call instead of the raw method name.
    /// Wraps the result in MCP content format so agents can parse it uniformly.
    pub async fn handle_tools_call(&self, params: Value) -> Result<Value> {
        let name = params["name"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing 'name' in tools/call params"))?;
        let args = params["arguments"].clone();

        let result = match name {
            "run_pipeline" => self.handle_run_pipeline(args).await?,
            "get_context" => self.handle_get_context(args).await?,
            "get_impact_graph" => self.handle_get_impact_graph(args).await?,
            "list_indexed_files" => self.handle_list_indexed_files().await?,
            "get_token_usage" => self.handle_get_token_usage().await?,
            "session_recall" => self.handle_session_recall(args).await?,
            _ => return Err(anyhow!("Unknown tool: {}", name)),
        };

        let text_content = if let Some(s) = result.as_str() {
            s.to_string()
        } else {
            result.to_string()
        };

        Ok(json!({
            "content": [{ "type": "text", "text": text_content }],
            "isError": false
        }))
    }

    /// Tool 6: session_recall
    ///
    /// Recall past MCP tool invocations for the current session.
    pub async fn handle_session_recall(&self, params: Value) -> Result<Value> {
        let query_filter = params["query"].as_str().map(|q| q.to_lowercase());
        let path = get_session_memory_path();

        let mut markdown = String::new();
        markdown.push_str("### Session Recall\n\n");

        if !path.exists() {
            if query_filter.is_some() {
                markdown.push_str("No matching past invocations found for the query.");
            } else {
                markdown.push_str("No past invocations recorded in the current session.");
            }
            return Ok(Value::String(markdown));
        }

        let file = std::fs::File::open(&path)?;
        let reader = std::io::BufReader::new(file);
        let memory: SessionMemory = serde_json::from_reader(reader).unwrap_or(SessionMemory { sessions: Vec::new() });

        let current_session = memory.sessions.iter().find(|s| s.id == self.state.session_id);

        let calls = match current_session {
            Some(s) => &s.calls,
            None => &vec![],
        };

        let mut filtered_count = 0;

        for call in calls {
            if let Some(ref q_filter) = query_filter {
                if !call.query.to_lowercase().contains(q_filter) {
                    continue;
                }
            }

            filtered_count += 1;

            let stale_flag = if call.stale { " [Stale]" } else { "" };
            markdown.push_str(&format!(
                "- **Query**: \"{}\" (Tokens: {}){}\n",
                call.query, call.tokens, stale_flag
            ));
            if !call.symbols.is_empty() {
                markdown.push_str(&format!(
                    "  - **Symbols**: {}\n",
                    call.symbols.iter().map(|s| format!("`{}`", s)).collect::<Vec<_>>().join(", ")
                ));
            }
            if !call.files.is_empty() {
                markdown.push_str(&format!(
                    "  - **Files**: {}\n",
                    call.files.iter().map(|f| format!("`{}`", f)).collect::<Vec<_>>().join(", ")
                ));
            }
        }

        if filtered_count == 0 {
            if query_filter.is_some() {
                markdown.push_str("No matching past invocations found for the query.");
            } else {
                markdown.push_str("No past invocations recorded in the current session.");
            }
        }

        Ok(Value::String(markdown))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mcp_server_creation() {
        let temp_dir = std::env::temp_dir().join("comP_test_mcp_creation");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        let state = Arc::new(crate::AppState::new(temp_dir.to_str().unwrap()).await.expect("Failed to create AppState"));
        let server = MCPServer::new(state);

        // Nodes are 0 at creation since indexing has not run
        let stats = server.handle_get_stats().await.expect("getStats failed");
        assert_eq!(stats["total_files"].as_i64().unwrap(), 0);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_json_rpc_format() {
        // Verify request/response format
        let request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "run_pipeline",
            "params": { "task": "test" }
        });

        assert_eq!(request["jsonrpc"], "2.0");
        assert_eq!(request["method"], "run_pipeline");
    }

    #[tokio::test]
    async fn test_handle_run_pipeline() {
        let state = Arc::new(crate::AppState::new(".").await.expect("Failed to create AppState"));
        let server = MCPServer::new(state);

        let params = json!({
            "task": "add authentication",
            "max_tokens": 8000
        });

        let result = server.handle_run_pipeline(params).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response["pivot_files"].is_array());
        assert!(response["related_files"].is_array());
        assert!(response["total_tokens"].is_number());
        assert!(response["savings"].is_string());
        assert!(response["estimated_cost"].is_string());
    }

    #[tokio::test]
    async fn test_handle_run_pipeline_missing_task() {
        let state = Arc::new(crate::AppState::new(".").await.expect("Failed to create AppState"));
        let server = MCPServer::new(state);

        let params = json!({ "max_tokens": 8000 }); // Missing task

        let result = server.handle_run_pipeline(params).await;
        assert!(result.is_err()); // Should error on missing task
    }

    #[tokio::test]
    async fn test_handle_get_context() {
        let state = Arc::new(crate::AppState::new(".").await.expect("Failed to create AppState"));
        let server = MCPServer::new(state);

        let params = json!({
            "query": "authentication",
            "limit": 10
        });

        let result = server.handle_get_context(params).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response["results"].is_array());
        assert!(response["count"].is_number());
        assert!(response["query"].is_string());
    }

    #[tokio::test]
    async fn test_handle_get_context_missing_query() {
        let state = Arc::new(crate::AppState::new(".").await.expect("Failed to create AppState"));
        let server = MCPServer::new(state);

        let params = json!({ "limit": 10 }); // Missing query

        let result = server.handle_get_context(params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_handle_get_impact_graph() {
        let state = Arc::new(crate::AppState::new(".").await.expect("Failed to create AppState"));
        let server = MCPServer::new(state);

        let params = json!({
            "symbol_id": 123,
            "symbol_name": "authenticate"
        });

        let result = server.handle_get_impact_graph(params).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response["symbol_id"].is_number());
        assert!(response["affected_files"].is_object());
        assert!(response["impact_count"].is_number());
        assert!(response["severity"].is_string());
    }

    #[tokio::test]
    async fn test_handle_get_impact_graph_missing_symbol_id() {
        let state = Arc::new(crate::AppState::new(".").await.expect("Failed to create AppState"));
        let server = MCPServer::new(state);

        let params = json!({ "symbol_name": "authenticate" }); // Missing symbol_id

        let result = server.handle_get_impact_graph(params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_handle_list_indexed_files() {
        let state = Arc::new(crate::AppState::new(".").await.expect("Failed to create AppState"));
        let server = MCPServer::new(state);

        let result = server.handle_list_indexed_files().await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response["files"].is_array());
        assert!(response["total_files"].is_number());
        assert!(response["total_symbols"].is_number());
        assert!(response["languages"].is_object());
    }

    #[tokio::test]
    async fn test_handle_get_token_usage() {
        let temp_dir = std::env::temp_dir().join("comP_test_token_usage");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        let state = Arc::new(crate::AppState::new(temp_dir.to_str().unwrap()).await.expect("Failed to create AppState"));
        let server = MCPServer::new(state);

        let response = server.handle_get_token_usage().await.unwrap();
        assert!(response["timestamp"].is_number());
        assert_eq!(response["total_tokens_consumed"].as_u64().unwrap(), 0);
        assert_eq!(response["queries_executed"].as_u64().unwrap(), 0);
        assert!(response["efficiency"].is_string());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_token_counter_cost_estimation() {
        // Test TokenCounter integration
        let cost = crate::search::TokenCounter::estimate_cost(10000, "sonnet");
        assert!(cost.starts_with("$"));
        assert!(cost.len() > 0);
    }

    #[test]
    fn test_token_counter_savings_calculation() {
        // Test TokenCounter integration
        let savings = crate::search::TokenCounter::calculate_savings(10000, 4000);
        assert!(savings.contains("%"));
    }

    #[tokio::test]
    async fn test_handle_get_stats() {
        let state = Arc::new(crate::AppState::new(".").await.expect("Failed to create AppState"));
        let server = MCPServer::new(state);

        let result = server.handle_get_stats().await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response["total_files"].is_number());
        assert!(response["total_nodes"].is_number());
        assert!(response["total_edges"].is_number());

        // Verify values are non-negative
        assert!(response["total_files"].as_i64().unwrap_or(-1) >= 0);
        assert!(response["total_nodes"].as_i64().unwrap_or(-1) >= 0);
        assert!(response["total_edges"].as_i64().unwrap_or(-1) >= 0);
    }

    #[tokio::test]
    async fn test_session_recall() {
        let temp_dir = std::env::temp_dir().join("comP_test_session_recall");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        std::env::set_var("COMP_WORKSPACE_ROOT", temp_dir.to_str().unwrap());

        let state = Arc::new(crate::AppState::new(temp_dir.to_str().unwrap()).await.expect("Failed to create AppState"));
        let server = MCPServer::new(state.clone());

        let result = server.handle_session_recall(json!({})).await.unwrap();
        assert!(result.as_str().unwrap().contains("No past invocations recorded"));

        record_mcp_call(
            &state.session_id,
            "test task".to_string(),
            vec!["test_symbol".to_string()],
            vec!["src/test.rs".to_string()],
            100,
        ).unwrap();

        let result = server.handle_session_recall(json!({})).await.unwrap();
        let markdown = result.as_str().unwrap();
        assert!(markdown.contains("test task"));
        assert!(markdown.contains("Tokens: 100"));
        assert!(markdown.contains("test_symbol"));

        let result = server.handle_session_recall(json!({ "query": "task" })).await.unwrap();
        assert!(result.as_str().unwrap().contains("test task"));

        let result = server.handle_session_recall(json!({ "query": "nomatch" })).await.unwrap();
        assert!(result.as_str().unwrap().contains("No matching past invocations"));

        std::env::remove_var("COMP_WORKSPACE_ROOT");
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
