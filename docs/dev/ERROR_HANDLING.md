# 例外処理・エラーハンドリング標準

**参照元**: CLAUDE.md → `@docs/STANDARDS_ERROR_HANDLING.md`
**詳細コード例**: [CONSTITUTION_DETAIL.md#例外処理エラーハンドリング](CONSTITUTION_DETAIL.md)

---

## 1. 基本原則

**例外は握りつぶすな。ログに記録して、適切なレベルで上位へ伝播させる。**

## 2. 必須実装パターン

1. **カスタム例外クラス定義**: ジャンル別（Config/Database/Network…）
2. **ログ + 再raise**: try → except でログ記録 → カスタム例外で再throw
3. **finally でリソース解放**: ファイルクローズ、接続切断
4. **main で例外型ごとに分岐**: 致命的 → exit(1)、回復可能 → 続行

## 3. ベストプラクティス

- **スタックトレース必須**: `exc_info=True` または equivalent
- **エラーコード**: ユーザーに「何が起きたか」を分かりやすく
- **retryable フラグ**: リトライ可能性を明示（オプション）
- **グレースフルシャットダウン**: finally で必ずリソース解放
- **ユーザー向けメッセージとログ用メッセージを分離**

## 4. チェックリスト

- [ ] 全例外を握りつぶしていないか
- [ ] スタックトレースをログ記録
- [ ] カスタム例外で型分岐
- [ ] ユーザー向け/ログ用で異なるメッセージ
- [ ] リソースリーク防止（finally確認）

## 5. 言語別実装

実コード例は [CONSTITUTION_DETAIL.md](CONSTITUTION_DETAIL.md) を参照：

- Python: カスタム Exception クラス + `from e`
- Node.js: Error 継承クラス + try/catch
- Rust: enum エラー型 + `Result<T, E>` + `?` 演算子
