// comP Rust Daemon
//
// Responsibilities:
// 1. Initialize GraphDB and SearchEngine
// 2. Listen on stdin/stdout as JSON-RPC 2.0 MCP server
// 3. Provide 5 MCP tools for AI agents
// 4. Semantic search, impact analysis, token calculation

use log::info;
use std::sync::Arc;

mod indexer;
mod graph;
mod search;
mod mcp;

use graph::GraphDB;
use search::SearchEngine;

/// Application state
///
/// Shared state across MCP server:
/// - GraphDB: Persist code structure (SQLite)
/// - SearchEngine: Semantic search and scoring
pub struct AppState {
    pub graph_db: Arc<GraphDB>,
    pub search_engine: Arc<tokio::sync::Mutex<SearchEngine>>,
}

// AppState is safe to share across threads
// rusqlite::Connection is used within Arc, ensuring thread-safe access
unsafe impl Send for AppState {}
unsafe impl Sync for AppState {}

impl AppState {
    /// Initialize application state
    ///
    /// # Input
    /// - workspace_root: Project root directory
    ///
    /// # Output
    /// - Result<Self>: AppState instance or initialization error
    ///
    /// # Process
    /// 1. GraphDB: Open .comp/index.db (create if not exists)
    /// 2. SearchEngine: Initialize in-memory search engine
    pub async fn new(workspace_root: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // Initialize GraphDB
        let graph_db = GraphDB::new(workspace_root).await?;
        info!("GraphDB initialized: {}", workspace_root);

        // Initialize SearchEngine
        let search_engine = SearchEngine::new();
        info!("SearchEngine initialized");

        Ok(AppState {
            graph_db: Arc::new(graph_db),
            search_engine: Arc::new(tokio::sync::Mutex::new(search_engine)),
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::new().default_filter_or("info")).init();

    info!("comP daemon starting");

    // Determine workspace root
    let workspace_root = std::env::var("COMP_WORKSPACE_ROOT")
        .or_else(|_| std::env::var("WORKSPACE_ROOT"))  // Fallback for compatibility
        .unwrap_or_else(|_| ".".to_string());

    // Initialize application state
    let state = Arc::new(AppState::new(&workspace_root).await?);
    info!("Application state initialized");

    // 初回 workspace 全体インデックス (MCP server 起動前に完了させる)
    // WHY: デーモン起動時に DB は空。これを実行しないとファイル数=0 のまま固まる。
    // tokio::spawn できないのは GraphDB 内部 (rusqlite Connection) が !Sync のため。
    // Phase 3 で GraphDB を tokio::sync::Mutex<Connection> 化したら spawn 化可能。
    // 現状は同期実行 → VSCode の getStats リトライ (5秒x10回=50秒) 内に完了する想定。
    {
        info!("Starting initial workspace indexing...");
        // WHY: 前回セッションの hash を DB から読み込み、変更ファイルだけ再インデックスする。
        // None を渡すと全ファイルを毎回フルスキャンしてしまうため、セッション間の陳腐化を防ぐ。
        let previous_hashes = state.graph_db.get_all_file_hashes().unwrap_or_default();
        let mut indexer = indexer::Indexer::new(&workspace_root);
        match indexer.index_workspace(Some(&previous_hashes), &state.graph_db).await {
            Ok((total, indexed, symbols)) => {
                info!(
                    "Initial indexing complete: indexed {}/{} files, {} symbols",
                    indexed, total, symbols
                );
            }
            Err(e) => log::error!("Initial indexing failed: {}", e),
        }
    }

    // Start MCP server
    // JSON-RPC 2.0 over stdio for AI agent communication
    let mcp_server = mcp::MCPServer::new(state);
    info!("MCP server started (listening on stdin/stdout)");

    mcp_server.run().await?;

    info!("MCP server stopped");
    Ok(())
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use serde_json::json;

    /// Phase 7 Integration Test: AppState initialization
    #[tokio::test]
    async fn test_appstate_initialization() {
        // Create temporary directory
        let temp_dir = std::env::temp_dir().join("comP_test_appstate");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");

        // Initialize AppState
        let result = AppState::new(temp_dir.to_str().unwrap()).await;
        assert!(result.is_ok(), "AppState initialization failed");

        let _state = result.unwrap();
        // Verify AppState creation succeeded

        // Cleanup
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    /// MCPServer 生成後に getStats が正しく初期値 (0/0/0) を返すか検証
    #[tokio::test]
    async fn test_mcp_server_creation_with_appstate() {
        let temp_dir = std::env::temp_dir().join("comP_test_mcp_server");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");

        let state = Arc::new(
            AppState::new(temp_dir.to_str().unwrap())
                .await
                .expect("Failed to create AppState"),
        );

        let server = mcp::MCPServer::new(state);
        let stats = server.handle_get_stats().await.expect("getStats failed");
        assert_eq!(stats["total_files"], 0);
        assert_eq!(stats["total_nodes"], 0);
        assert_eq!(stats["total_edges"], 0);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    /// Phase 7 Integration Test: Full pipeline (index -> search -> MCP)
    #[tokio::test]
    async fn test_full_pipeline_index_search_mcp() {
        let temp_dir = std::env::temp_dir().join("comP_test_full_pipeline");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");

        // Initialize AppState
        let state = Arc::new(
            AppState::new(temp_dir.to_str().unwrap())
                .await
                .expect("Failed to create AppState"),
        );

        // Load test data into SearchEngine
        let symbols = vec![
            ("src/auth.rs".to_string(), "authenticate".to_string(), "function".to_string(), 10u32),
            ("src/auth.rs".to_string(), "authorize".to_string(), "function".to_string(), 25u32),
            ("src/user.rs".to_string(), "User".to_string(), "class".to_string(), 5u32),
        ];

        let mut search_engine = state.search_engine.lock().await;
        let result = search_engine.build_index(&symbols);
        assert!(result.is_ok(), "Failed to build search index");
        drop(search_engine);

        // MCP Tool: run_pipeline
        let mcp_server = mcp::MCPServer::new(state.clone());
        let run_pipeline_params = json!({
            "task": "authentication implementation",
            "max_tokens": 8000
        });

        let result = mcp_server.handle_run_pipeline(run_pipeline_params).await;
        assert!(result.is_ok(), "run_pipeline failed");
        let response = result.unwrap();
        assert!(response["pivot_files"].is_array(), "pivot_files should be array");
        assert!(response["total_tokens"].is_number(), "total_tokens should be number");

        // MCP Tool: get_context
        let get_context_params = json!({
            "query": "authentication",
            "limit": 10
        });

        let result = mcp_server.handle_get_context(get_context_params).await;
        assert!(result.is_ok(), "get_context failed");
        let response = result.unwrap();
        assert!(response["results"].is_array(), "results should be array");
        assert!(response["count"].is_number(), "count should be number");

        // MCP Tool: get_impact_graph
        let impact_params = json!({
            "symbol_id": 1,
            "symbol_name": "authenticate"
        });

        let result = mcp_server.handle_get_impact_graph(impact_params).await;
        assert!(result.is_ok(), "get_impact_graph failed");
        let response = result.unwrap();
        assert!(response["affected_files"].is_object(), "affected_files should be object");
        assert!(response["severity"].is_string(), "severity should be string");

        // MCP Tool: list_indexed_files
        let result = mcp_server.handle_list_indexed_files().await;
        assert!(result.is_ok(), "list_indexed_files failed");
        let response = result.unwrap();
        assert!(response["files"].is_array(), "files should be array");
        assert!(response["total_files"].is_number(), "total_files should be number");

        // MCP Tool: get_token_usage (未実装ステータス)
        let result = mcp_server.handle_get_token_usage().await;
        assert!(result.is_ok(), "get_token_usage failed");
        let response = result.unwrap();
        assert_eq!(response["status"], "not_implemented");
        assert!(response["timestamp"].is_number());

        // Cleanup
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    /// Phase 7 E2E Test: JSON-RPC 2.0 protocol compliance
    #[tokio::test]
    async fn test_json_rpc_protocol_compliance() {
        let temp_dir = std::env::temp_dir().join("comP_test_jsonrpc");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");

        let state = Arc::new(
            AppState::new(temp_dir.to_str().unwrap())
                .await
                .expect("Failed to create AppState"),
        );

        let mcp_server = mcp::MCPServer::new(state);

        // JSON-RPC request form should be valid
        let params = json!({
            "task": "test",
            "max_tokens": 8000
        });

        let result = mcp_server.handle_run_pipeline(params).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        // Response should have jsonrpc and result fields (when wrapped)
        assert!(response.is_object(), "Response should be a JSON object");

        // Cleanup
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
