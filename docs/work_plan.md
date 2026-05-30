# ワークプラン（作業計画）

## 現在の進捗状況
- [x] TypeScript のユニットテストエラー修正
  - 変更内容: `src/ui/tests/commands.test.ts` 内での `registerCommands` 呼び出しにおける第2引数を、最新 of `getDaemonManager`（`() => DaemonManager | null`）に対応するよう `() => mockDaemon` に修正。
- [x] バックエンド（Rust）とフロントエンド（TypeScript/VSCode）の未コミット変更の検証・整理
  - 変更内容: `npm run lint:ts` で TypeScript の静的チェックを実施し、型エラーがないことを確認。また、JSON-RPC を用いてデーモンを直接駆動する Python テストスクリプトを自作し、新規機能（非同期インデックス、BM25検索、累積トークン統計機能）がすべて正常に動作していることを実証。
- [x] 変更内容のコミットとプッシュの実行
  - 変更内容: 15ファイル+新規ドキュメントを含む変更点を意味ごとに論理的な5つのコミットに分割し、ビルド及びテストを実行した上で `git push origin main` を行いました。

## 残タスク
- なし。すべての変更が正常に検証され、リモートリポジトリにプッシュされました。
