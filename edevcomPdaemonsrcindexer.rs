// インデックエンジン
//
// 責務：tree-sitter でコードをパース、シンボル・関係を抽出してグラフ DB に保存

use std::path::PathBuf;

pub struct Indexer;

impl Indexer {
    /// # インデックス開始
    ///
    /// # 入力
    /// - workspace_path: インデックス対象のワークスペースパス
    ///
    /// # 出力
    /// - インデックス完了したノード数
    pub fn index_workspace(workspace_path: &PathBuf) -> Result<usize, String> {
        // TODO: walker で files を列挙
        // TODO: parser で tree-sitter パース
        // TODO: graph に nodes/edges を書き込み
        Ok(0)
    }

    /// # ファイル再インデックス（増分）
    ///
    /// # 入力
    /// - file_path: 対象ファイル
    /// - file_hash: ファイル内容ハッシュ（変更検出）
    ///
    /// # 出力
    /// - インデックス済みシンボル数
    pub fn reindex_file(file_path: &str, file_hash: &str) -> Result<usize, String> {
        // TODO: 既存ハッシュと比較
        // TODO: 異なればパース・更新
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_workspace_returns_node_count() {
        // TODO: テンポラリディレクトリで workspace インデックス
        // TODO: node count > 0 を確認
    }

    #[test]
    fn test_reindex_file_detects_changes() {
        // TODO: ファイル内容を変更
        // TODO: reindex_file が変更検出することを確認
    }
}
