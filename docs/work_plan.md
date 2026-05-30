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
- [x] ソースコードコメントの英語化
  - 変更内容: VSCode拡張機能（TypeScript）およびバックエンドデーモン（Rust）のソースコード内に含まれるすべての日本語コメントを簡潔な英語（concise English）に翻訳・置換。`npm run lint:ts`、`npm run test`、`npm run daemon:test`、`test_daemon_rpc.py` を用いた結合検証を含めてすべての動作テストが正常にパスすることを確認。
- [x] 初回起動手順（マニュアル）の整備
  - 変更内容: ワークスペースを初めて開いたときは自動起動せず、手動で「Start comP」ボタンを押す必要がある起動仕様を `README.md` および `README_ja.md` の「クイックスタート」に明記。
- [x] 最新版 VSIX パッケージの作成
  - 変更内容: `npx vsce package` を実行し、英語コメント化および起動マニュアル整備を反映した最新バージョンの拡張機能パッケージ `comp-vscode-0.1.0.vsix`（2.36MB）を作成。

## 残タスク
1. **コミットとプッシュ**
   - 管理ドキュメントの更新分をコミットし、リモート（GitHub）へプッシュ。
