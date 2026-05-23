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
use serde_json::{json, Value};
use std::sync::Arc;

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
                "run_pipeline" => self.handle_run_pipeline(params).await,
                "get_context" => self.handle_get_context(params).await,
                "get_impact_graph" => self.handle_get_impact_graph(params).await,
                "list_indexed_files" => self.handle_list_indexed_files().await,
                "get_token_usage" => self.handle_get_token_usage().await,
                "getStats" => self.handle_get_stats().await,
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
        // 1. Extract parameters
        let task = params["task"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing 'task' parameter"))?;
        let max_tokens = params["max_tokens"]
            .as_u64()
            .unwrap_or(8000) as usize;

        // For Phase 6 stub: Return structure with realistic data
        // In production, this would:
        // - Call search_engine.search(task, limit)
        // - Collect file metadata from GraphDB
        // - Call get_impact_graph for each pivot file
        // - Count tokens and calculate savings

        let pivot_files = json!([
            {
                "path": "src/auth/authenticate.rs",
                "symbols": 5,
                "tokens": 500
            }
        ]);

        let related_files = json!([
            {
                "path": "src/types/user.rs",
                "symbols": 3,
                "tokens": 200
            }
        ]);

        let total_tokens = 700;
        let full_workspace_tokens = 1800;
        let savings = crate::search::TokenCounter::calculate_savings(full_workspace_tokens, total_tokens);
        let cost = crate::search::TokenCounter::estimate_cost(total_tokens, "sonnet");

        Ok(json!({
            "task": task,
            "pivot_files": pivot_files,
            "related_files": related_files,
            "total_tokens": total_tokens,
            "max_tokens": max_tokens,
            "savings": savings,
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
        // Extract parameters
        let query = params["query"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing 'query' parameter"))?;
        let limit = params["limit"]
            .as_u64()
            .unwrap_or(10) as usize;

        // For Phase 6 stub: Return realistic search results
        // In production, this would:
        // - Call search_engine.search(query, limit)
        // - Filter by kind if provided
        // - Convert SearchResult to JSON
        // - Return sorted by relevance

        let results = json!([
            {
                "file": "src/auth/authenticate.rs",
                "symbol": "authenticate",
                "kind": "function",
                "line": 15,
                "score": 0.95
            },
            {
                "file": "src/auth/authorize.rs",
                "symbol": "authorize_user",
                "kind": "function",
                "line": 42,
                "score": 0.87
            }
        ]);

        Ok(json!({
            "query": query,
            "results": results,
            "count": 2,
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
        // Extract parameters
        let symbol_id = params["symbol_id"]
            .as_i64()
            .ok_or_else(|| anyhow!("Missing 'symbol_id' parameter"))?;
        let symbol_name = params["symbol_name"]
            .as_str()
            .unwrap_or("unknown");

        // For Phase 6 stub: Return realistic impact analysis
        // In production, this would:
        // - Call get_impact_graph(symbol_id) from search engine
        // - Group results by file
        // - Calculate severity (0 = none, 1-5 = low, 6-20 = medium, 20+ = high)
        // - Return impact structure

        let affected_files = json!({
            "src/routes/login.rs": ["handleLogin", "validateCredentials"],
            "src/middleware/auth.rs": ["authMiddleware"]
        });

        let impact_count = 3;
        let severity = match impact_count {
            0 => "none",
            1..=5 => "low",
            6..=20 => "medium",
            _ => "high",
        };

        Ok(json!({
            "symbol_id": symbol_id,
            "symbol": symbol_name,
            "affected_files": affected_files,
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
        // For Phase 6 stub: Return realistic index statistics
        // In production, this would:
        // - Call GraphDB::get_stats()
        // - Query all files from nodes table
        // - Count symbols by language
        // - Return complete statistics

        let files = json!([
            { "path": "src/main.rs", "language": "rust", "symbols": 12 },
            { "path": "src/lib.rs", "language": "rust", "symbols": 8 },
            { "path": "src/auth.ts", "language": "typescript", "symbols": 15 },
        ]);

        let total_files = 10;
        let total_symbols = 50;

        let mut languages = std::collections::HashMap::new();
        languages.insert("rust".to_string(), 20);
        languages.insert("typescript".to_string(), 30);

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
        // For Phase 6 stub: Return realistic token metrics
        // In production, this would:
        // - Query internal token counter
        // - Access query execution logs
        // - Calculate averages and efficiency metrics
        // - Return statistics with timestamp

        let total_tokens_consumed = 45000;
        let queries_executed = 15;
        let average_tokens_per_query = total_tokens_consumed / queries_executed;

        // Timestamp: current time in seconds since UNIX_EPOCH
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        // Efficiency: assuming 60% average token reduction through optimization
        let efficiency = "60%";

        Ok(json!({
            "total_tokens_consumed": total_tokens_consumed,
            "queries_executed": queries_executed,
            "average_tokens_per_query": average_tokens_per_query,
            "timestamp": timestamp,
            "efficiency": efficiency
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

        info!("handle_get_stats: returning stats - files: {}, nodes: {}, edges: {}",
              file_count, node_count, edge_count);

        Ok(json!({
            "total_files": file_count,
            "total_nodes": node_count,
            "total_edges": edge_count
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mcp_server_creation() {
        let state = Arc::new(crate::AppState::new(".").await.expect("Failed to create AppState"));
        let server = MCPServer::new(state);
        // Verify server creation doesn't panic
        assert!(true);
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
        let state = Arc::new(crate::AppState::new(".").await.expect("Failed to create AppState"));
        let server = MCPServer::new(state);

        let result = server.handle_get_token_usage().await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response["total_tokens_consumed"].is_number());
        assert!(response["queries_executed"].is_number());
        assert!(response["average_tokens_per_query"].is_number());
        assert!(response["timestamp"].is_number());
        assert!(response["efficiency"].is_string());
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
}
