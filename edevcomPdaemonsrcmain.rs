// comP Rust デーモン
//
// 責務：
// 1. IPC ソケット待受（VSCode 拡張と通信）
// 2. tree-sitter でコードをパース、SQLite グラフ DB に保存
// 3. MCP サーバーとしてツール実装（run_pipeline, get_context 等）
// 4. セマンティック検索、影響分析

use log::info;
use std::path::PathBuf;
use tokio::net::UnixListener;

mod indexer;
mod graph;
mod search;
mod mcp;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::new().default_filter_or("info")).init();

    info!("comP デーモン起動");

    // スケルトン：IPC待受開始
    // TODO: Windows Named Pipe / Unix Socket 対応
    // TODO: デーモンプロセス管理

    Ok(())
}

// # indexer: tree-sitter パーサー、ファイルウォーカー、グラフ書き込み
// - walker.rs: ファイル列挙、増分検出
// - parser.rs: 言語別 tree-sitter グラマー管理
// - doc_parser.rs: JSON/XML/Markdown 独自パーサー
// - mod.rs: これらの調整役

// # graph: SQLite グラフ DB アクセス
// - schema.rs: テーブル定義（nodes, edges, file_hash）
// - mod.rs: CRUD 操作

// # search: セマンティック検索、影響分析
// - mod.rs: tf-idf ベース全文検索
// - token.rs: tiktoken-rs トークン計測

// # mcp: MCP サーバー実装
// - tools.rs: ツール 5 本（run_pipeline, get_context, 等）
// - mod.rs: JSON-RPC サーバー（stdio / TCP）
