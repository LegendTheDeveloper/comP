// MCP（Model Context Protocol）サーバー実装
//
// 責務：JSON-RPC で AI エージェントからのツール呼び出しを処理

use serde_json::{json, Value};

pub struct MCPServer;

#[derive(Debug, Clone)]
pub struct MCPRequest {
    pub method: String,
    pub params: Option<Value>,
    pub id: Option<Value>,
}

#[derive(Debug, Clone)]
pub struct MCPResponse {
    pub result: Option<Value>,
    pub error: Option<Value>,
    pub id: Option<Value>,
}

impl MCPServer {
    /// # リクエスト処理
    ///
    /// # 入力
    /// - request: JSON-RPC リクエスト
    ///
    /// # 出力
    /// - JSON-RPC レスポンス
    pub async fn handle_request(request: MCPRequest) -> Result<MCPResponse, String> {
        match request.method.as_str() {
            "run_pipeline" => Self::handle_run_pipeline(request.params).await,
            "get_context" => Self::handle_get_context(request.params).await,
            "get_impact_graph" => Self::handle_get_impact_graph(request.params).await,
            "list_indexed_files" => Self::handle_list_indexed_files(request.params).await,
            "get_token_usage" => Self::handle_get_token_usage(request.params).await,
            _ => Err(format!("Unknown method: {}", request.method)),
        }
        .map(|result| MCPResponse {
            result: Some(result),
            error: None,
            id: request.id,
        })
        .or_else(|err| {
            Ok(MCPResponse {
                result: None,
                error: Some(json!({ "message": err })),
                id: request.id,
            })
        })
    }

    /// # コンテキスト取得 + 影響分析を 1 呼び出しで実行
    async fn handle_run_pipeline(params: Option<Value>) -> Result<Value, String> {
        // TODO: task 説明を params から抽出
        // TODO: search_context で関連ファイル検索
        // TODO: analyze_impact で影響ファイル抽出
        // TODO: count_tokens で合計トークン計測
        // TODO: capsule フォーマット（pivot files + related + context summary）で返す
        Ok(json!({
            "pivot_files": [],
            "related_files": [],
            "total_tokens": 0,
            "savings_percent": 0,
        }))
    }

    /// # タスク説明から関連ファイルを取得
    async fn handle_get_context(params: Option<Value>) -> Result<Value, String> {
        // TODO: task description を抽出
        // TODO: search_context 実行
        // TODO: ファイル内容とメタデータを返す
        Ok(json!({
            "files": [],
            "total_tokens": 0,
        }))
    }

    /// # シンボル変更時の影響範囲を取得
    async fn handle_get_impact_graph(params: Option<Value>) -> Result<Value, String> {
        // TODO: symbol_name または node_id を抽出
        // TODO: analyze_impact 実行
        // TODO: グラフ構造（nodes, edges）で返す
        Ok(json!({
            "nodes": [],
            "edges": [],
        }))
    }

    /// # インデックス済みファイル一覧
    async fn handle_list_indexed_files(_params: Option<Value>) -> Result<Value, String> {
        // TODO: SQLite グラフから全 file_path を抽出
        // TODO: ファイル数、ノード数、エッジ数の統計を返す
        Ok(json!({
            "files": [],
            "total_files": 0,
            "total_nodes": 0,
            "total_edges": 0,
        }))
    }

    /// # トークン使用統計
    async fn handle_get_token_usage(_params: Option<Value>) -> Result<Value, String> {
        // TODO: 直近のトークン計測記録を返す
        // TODO: タスク別の統計を返す
        Ok(json!({
            "recent_usage": [],
            "total_tokens_today": 0,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_run_pipeline_returns_capsule() {
        let request = MCPRequest {
            method: "run_pipeline".to_string(),
            params: Some(json!({ "task": "fix auth bug" })),
            id: Some(Value::Number(1.into())),
        };
        let response = MCPServer::handle_request(request).await.expect("Request failed");
        assert!(response.error.is_none());
        assert!(response.result.is_some());
    }

    #[tokio::test]
    async fn test_unknown_method_returns_error() {
        let request = MCPRequest {
            method: "invalid_method".to_string(),
            params: None,
            id: Some(Value::Number(1.into())),
        };
        let response = MCPServer::handle_request(request).await.expect("Request failed");
        assert!(response.error.is_some());
    }

    #[tokio::test]
    async fn test_get_context_succeeds() {
        let request = MCPRequest {
            method: "get_context".to_string(),
            params: Some(json!({ "task": "implement caching" })),
            id: Some(Value::Number(2.into())),
        };
        let response = MCPServer::handle_request(request).await.expect("Request failed");
        assert!(response.error.is_none());
    }

    #[tokio::test]
    async fn test_list_indexed_files_succeeds() {
        let request = MCPRequest {
            method: "list_indexed_files".to_string(),
            params: None,
            id: Some(Value::Number(3.into())),
        };
        let response = MCPServer::handle_request(request).await.expect("Request failed");
        assert!(response.error.is_none());
    }
}
