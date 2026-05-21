// グラフ DB（SQLite）
//
// 責務：nodes（シンボル）、edges（関係）を SQLite に保存・検索

use std::path::PathBuf;

pub struct GraphDB;

impl GraphDB {
    /// # DB 初期化
    ///
    /// # 入力
    /// - db_path: SQLite DB ファイルパス
    ///
    /// # 出力
    /// - DB コネクション（または初期化成功）
    pub fn init(db_path: &PathBuf) -> Result<Self, String> {
        // TODO: .comp/index.db 作成・接続
        // TODO: nodes, edges, file_hash テーブル作成
        Ok(Self)
    }

    /// # シンボル追加
    ///
    /// # 入力
    /// - node_id: シンボル一意 ID
    /// - symbol_name: 関数名、クラス名など
    /// - file_path: ファイルパス
    /// - line: 行番号
    /// - kind: 関数/クラス/変数など
    ///
    /// # 出力
    /// - 保存成功
    pub fn add_node(
        &self,
        node_id: &str,
        symbol_name: &str,
        file_path: &str,
        line: u32,
        kind: &str,
    ) -> Result<(), String> {
        // TODO: INSERT OR REPLACE INTO nodes
        Ok(())
    }

    /// # 依存関係追加
    ///
    /// # 入力
    /// - from_id: シンボル ID（呼び出し元）
    /// - to_id: シンボル ID（呼び出し先）
    ///
    /// # 出力
    /// - 保存成功
    pub fn add_edge(&self, from_id: &str, to_id: &str) -> Result<(), String> {
        // TODO: INSERT OR REPLACE INTO edges
        Ok(())
    }

    /// # シンボル検索
    ///
    /// # 入力
    /// - symbol_name: 検索キー
    ///
    /// # 出力
    /// - マッチしたシンボル情報のベクタ
    pub fn search_symbol(&self, symbol_name: &str) -> Result<Vec<SymbolInfo>, String> {
        // TODO: SELECT * FROM nodes WHERE symbol_name LIKE '%...%'
        Ok(vec![])
    }

    /// # 影響分析（逆参照）
    ///
    /// # 入力
    /// - node_id: シンボル ID
    ///
    /// # 出力
    /// - このシンボルに依存するノード ID のリスト
    pub fn get_dependents(&self, node_id: &str) -> Result<Vec<String>, String> {
        // TODO: SELECT from_id FROM edges WHERE to_id = node_id
        Ok(vec![])
    }
}

#[derive(Debug, Clone)]
pub struct SymbolInfo {
    pub node_id: String,
    pub symbol_name: String,
    pub file_path: String,
    pub line: u32,
    pub kind: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn create_test_db() -> GraphDB {
        let temp_dir = env::temp_dir();
        let db_path = temp_dir.join("test_comp.db");
        let _ = std::fs::remove_file(&db_path);
        GraphDB::init(&db_path).expect("Failed to create test DB")
    }

    #[test]
    fn test_add_node_succeeds() {
        let db = create_test_db();
        let result = db.add_node(
            "node1",
            "authenticate",
            "src/auth/auth.ts",
            42,
            "function",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_add_edge_succeeds() {
        let db = create_test_db();
        db.add_node("node1", "authenticate", "src/auth/auth.ts", 42, "function")
            .unwrap();
        db.add_node("node2", "validateToken", "src/auth/validate.ts", 10, "function")
            .unwrap();
        let result = db.add_edge("node1", "node2");
        assert!(result.is_ok());
    }

    #[test]
    fn test_search_symbol_returns_results() {
        let db = create_test_db();
        db.add_node("node1", "authenticate", "src/auth/auth.ts", 42, "function")
            .unwrap();
        let results = db.search_symbol("authenticate").expect("Search failed");
        assert!(results.len() > 0);
    }

    #[test]
    fn test_get_dependents_finds_reverse_refs() {
        let db = create_test_db();
        db.add_node("node1", "authenticate", "src/auth/auth.ts", 42, "function")
            .unwrap();
        db.add_node("node2", "validateToken", "src/auth/validate.ts", 10, "function")
            .unwrap();
        db.add_edge("node2", "node1").unwrap();
        let dependents = db.get_dependents("node1").expect("Failed to get dependents");
        assert!(dependents.contains(&"node2".to_string()));
    }
}
