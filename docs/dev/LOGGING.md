# ロギング標準

**参照元**: CLAUDE.md → `@docs/STANDARDS_LOGGING.md`
**詳細コード例**: [CONSTITUTION_DETAIL.md#ロギング監視の仕組み](CONSTITUTION_DETAIL.md)

---

## 1. ログレベル定義（全プロジェクト統一）

| レベル | 用途 | 例 |
| --- | --- | --- |
| DEBUG | 開発時のデバッグ情報 | 変数値、関数呼び出し |
| INFO | 重要な処理フロー | 起動完了、処理開始/終了 |
| WARNING | 注意が必要 | 非推奨API使用、リソース枯渇兆候 |
| ERROR | エラー（処理継続可） | 例外キャッチ、リトライ可能エラー |
| CRITICAL | 致命的エラー | DB接続失敗、認証失敗 |

## 2. 出力先

- **開発環境**: コンソール + `logs/app.log`
- **本番環境**: ファイル + 外部集約サービス（Datadog/CloudWatch）

## 3. フォーマット（推奨）

```text
[YYYY-MM-DD HH:MM:SS] [LEVEL] [module:line] message {context}
```

## 4. ベストプラクティス

- **変数値を含める**: ❌「エラーが発生」→ ✅「user_id=123 でエラー」
- **スタックトレース必須**: 例外は exc_info=True で記録
- **機密情報マスク**: パスワード、APIキー、個人情報は除外
- **適切なレベル**: すべてを INFO/ERROR に集約しない

## 5. 言語別実装

実コード例は [CONSTITUTION_DETAIL.md](CONSTITUTION_DETAIL.md) の「ロギング・監視の仕組み」セクションを参照：

- Python: `logging` モジュール
- Node.js: `winston`
- Rust: `log + env_logger`
