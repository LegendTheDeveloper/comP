# ワークプラン（作業計画）

## 現在の進捗状況
- [x] TypeScript のユニットテストエラー修正
  - 変更内容: `src/ui/tests/commands.test.ts` 内での `registerCommands` 呼び出しにおける第2引数を、最新の `getDaemonManager`（`() => DaemonManager | null`）に対応するよう `() => mockDaemon` に修正。
- [x] バックエンド（Rust）とフロントエンド（TypeScript/VSCode）の未コミット変更の検証・整理
  - 変更内容: `npm run lint:ts` で TypeScript の静的チェックを実施し、型エラーがないことを確認。また、JSON-RPC を用いてデーモンを直接駆動する Python テストスクリプトを自作し、新規機能（非同期インデックス、BM25検索、累積トークン統計機能）がすべて正常に動作していることを実証。
- [x] 変更内容のコミットとプッシュの実行
  - 変更内容: 15ファイル+新規ドキュメントを含む変更点を意味ごとに論理的な5つのコミットに分割し、ビルド及びテストを実行した上で `git push origin main` を行いました。
- [x] マーケットプレイス登録向けREADMEおよび.vscodeignoreの整備
  - 変更内容: README先頭へのロゴ追加、最新機能（非同期インデックス、トークン統計、BM25検索等）の明記、互換エージェント表へのAntigravity追加、ロードマップのv0.1完了反映を実施。また、VSIXへの不要な開発用・ローカル一時ファイルの混入を防ぐため `.vscodeignore` を修正し、`npx vsce ls` でパッケージが最適化された（2.36MB）ことを検証。

## 残タスク
1. **整備した内容のコミットとプッシュ**
   - 変更された README.md, README_ja.md, .vscodeignore, および管理ドキュメントをコミットし、リモートへプッシュ。
