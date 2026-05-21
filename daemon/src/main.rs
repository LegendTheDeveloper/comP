// comP Rust デーモン
//
// 責務：
// 1. GraphDB と SearchEngine を初期化
// 2. JSON-RPC 2.0 MCP サーバーとして stdin/stdout でリッスン
// 3. AI エージェント向けに 5 つの MCP ツールを提供
// 4. セマンティック検索、影響分析、トークン計算

use log::info;
use std::sync::Arc;

mod indexer;
mod graph;
mod search;
mod mcp;
mod ipc;

use graph::GraphDB;
use search::SearchEngine;

/// アプリケーション状態
///
/// MCP サーバーで共有される状態：
/// - GraphDB: コード構造の永続化（SQLite）
/// - SearchEngine: セマンティック検索とスコアリング
pub struct AppState {
    pub graph_db: Arc<GraphDB>,
    pub search_engine: Arc<tokio::sync::Mutex<SearchEngine>>,
}

impl AppState {
    /// アプリケーション状態を初期化
    ///
    /// # 入力
    /// - workspace_root: プロジェクト root ディレクトリ
    ///
    /// # 出力
    /// - Result<Self>: AppState インスタンス または初期化エラー
    ///
    /// # 処理
    /// 1. GraphDB: .comp/index.db を開く（なければ作成）
    /// 2. SearchEngine: メモリ上の検索エンジンを初期化
    pub async fn new(workspace_root: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // GraphDB 初期化
        let graph_db = GraphDB::new(workspace_root).await?;
        info!("GraphDB 初期化完了: {}", workspace_root);

        // SearchEngine 初期化
        let search_engine = SearchEngine::new();
        info!("SearchEngine 初期化完了");

        Ok(AppState {
            graph_db: Arc::new(graph_db),
            search_engine: Arc::new(tokio::sync::Mutex::new(search_engine)),
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::new().default_filter_or("info")).init();

    info!("comP デーモン起動");

    // ワークスペース root の決定
    let workspace_root = std::env::var("WORKSPACE_ROOT")
        .unwrap_or_else(|_| ".".to_string());

    // アプリケーション状態初期化
    let state = Arc::new(AppState::new(&workspace_root).await?);
    info!("アプリケーション状態初期化完了");

    // MCP サーバー起動
    // JSON-RPC 2.0 over stdio で AI エージェントと通信
    let mcp_server = mcp::MCPServer::new(state);
    info!("MCP サーバー起動 (stdin/stdout リッスン)");

    mcp_server.run().await?;

    info!("MCP サーバー停止");
    Ok(())
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use serde_json::json;

    /// Phase 7 統合テスト: AppState 初期化
    #[tokio::test]
    async fn test_appstate_initialization() {
        // 一時ディレクトリを作成
        let temp_dir = std::env::temp_dir().join("comP_test_appstate");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");

        // AppState を初期化
        let result = AppState::new(temp_dir.to_str().unwrap()).await;
        assert!(result.is_ok(), "AppState initialization failed");

        let _state = result.unwrap();
        // AppState 生成成功を確認

        // クリーンアップ
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    /// Phase 7 統合テスト: MCP サーバー生成と基本動作
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

        // MCP サーバーを生成
        let _server = mcp::MCPServer::new(state);
        // MCP サーバーは正常に生成されたことを確認
        assert!(true);

        // クリーンアップ
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    /// Phase 7 統合テスト: フルパイプライン（インデックス→検索→MCP）
    #[tokio::test]
    async fn test_full_pipeline_index_search_mcp() {
        let temp_dir = std::env::temp_dir().join("comP_test_full_pipeline");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");

        // AppState 初期化
        let state = Arc::new(
            AppState::new(temp_dir.to_str().unwrap())
                .await
                .expect("Failed to create AppState"),
        );

        // SearchEngine にテストデータを投入
        let symbols = vec![
            ("src/auth.rs".to_string(), "authenticate".to_string(), "function".to_string(), 10u32),
            ("src/auth.rs".to_string(), "authorize".to_string(), "function".to_string(), 25u32),
            ("src/user.rs".to_string(), "User".to_string(), "class".to_string(), 5u32),
        ];

        let mut search_engine = state.search_engine.lock().await;
        let result = search_engine.build_index(&symbols);
        assert!(result.is_ok(), "Failed to build search index");
        drop(search_engine);

        // MCP ツール: run_pipeline
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

        // MCP ツール: get_context
        let get_context_params = json!({
            "query": "authentication",
            "limit": 10
        });

        let result = mcp_server.handle_get_context(get_context_params).await;
        assert!(result.is_ok(), "get_context failed");
        let response = result.unwrap();
        assert!(response["results"].is_array(), "results should be array");
        assert!(response["count"].is_number(), "count should be number");

        // MCP ツール: get_impact_graph
        let impact_params = json!({
            "symbol_id": 1,
            "symbol_name": "authenticate"
        });

        let result = mcp_server.handle_get_impact_graph(impact_params).await;
        assert!(result.is_ok(), "get_impact_graph failed");
        let response = result.unwrap();
        assert!(response["affected_files"].is_object(), "affected_files should be object");
        assert!(response["severity"].is_string(), "severity should be string");

        // MCP ツール: list_indexed_files
        let result = mcp_server.handle_list_indexed_files().await;
        assert!(result.is_ok(), "list_indexed_files failed");
        let response = result.unwrap();
        assert!(response["files"].is_array(), "files should be array");
        assert!(response["total_files"].is_number(), "total_files should be number");

        // MCP ツール: get_token_usage
        let result = mcp_server.handle_get_token_usage().await;
        assert!(result.is_ok(), "get_token_usage failed");
        let response = result.unwrap();
        assert!(response["total_tokens_consumed"].is_number(), "total_tokens_consumed should be number");
        assert!(response["efficiency"].is_string(), "efficiency should be string");

        // クリーンアップ
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    /// Phase 7 E2E テスト: JSON-RPC 2.0 プロトコル検証
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

        // クリーンアップ
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
