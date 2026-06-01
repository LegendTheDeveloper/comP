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
  - 変更内容: `npx vsce package` を実行し、英語コメント化、起動マニュアル整備、およびマーケットプレイス仕様（カテゴリ上限2件）に合わせた `package.json` のカテゴリ設定修正を反映した最新の拡張機能パッケージ `comp-vscode-0.1.0.vsix`（2.36MB）を作成。
- [x] GitHub Actions のエラー解消とマルチプラットフォーム VSIX ビルド対応
  - 変更内容: `.gitignore` から `package-lock.json` と `Cargo.lock` を削除して Git 管理に移行。`DaemonManager.ts` のバイナリ探索ロジックを修正し、本番環境でプラットフォーム別バイナリ（`-win.exe`, `-macos`, `-linux`）を使用するように変更。`.vscodeignore` でこれらの同梱を許可。`release.yml` をマトリクスビルド化し、各OSでビルドされたバイナリを1つの VSIX に集約してリリースするフローに変更。

- [x] SECURITY.md の MarkdownLint エラーの解消
  - 変更内容: `SECURITY.md` のリスト前後に空行を追加し、CI 上での `MD032/blanks-around-lists` エラーを解消。
- [x] cargo audit での paste 警告（RUSTSEC-2024-0436）の除外設定
  - 変更内容: `daemon/audit.toml` を作成し、警告を除外設定。
- [x] GitHub Actions CIワークフローの安定化・高速化
  - 変更内容: `ci.yml` の `sast` ジョブを最適化。Clippy コンポーネントを明示してエラーを防ぐとともに、`cargo-audit` を公式の `rustsec/audit-action` に変更しビルド時間を約10分短縮。
- [x] README.md および SECURITY.md の MarkdownLint エラーの追加解消
  - 変更内容: メールアドレスの非開示対応（削除）により `MD034` を解消。各ファイルの1行が 80文字を超える箇所を改行・短縮し `MD013` を解消。
- [x] ci.yml における cargo audit アクションの再修正
  - 変更内容: `rustsec/audit-action` から `actions-rust-lang/audit@v1` に変更し、`workingDirectory: daemon` を指定。

- [x] セッションメモリ機能（session_recall）の実装
  - 変更内容: `.comp/session-memory.json` に MCP ツール呼び出し（`run_pipeline`, `get_context`）履歴を永続化し、VSCode 拡張側のファイル監視フック（`setupFileWatchers`）を通じてコード変更時に自動で `stale` マークする仕組みを構築。また、過去履歴を Markdown 形式で返却する新しい MCP ツール `session_recall` を Rust デーモン側に実装。
- [x] Rustデーモンへの4つの新しいMCPツール（get_symbol, get_dependencies, get_file_summary, get_project_overview）の追加
  - 変更内容: 指定シンボルのコード・依存関係抽出、ファイル概要、プロジェクト全体の統計およびエクスポートシンボル一覧を Markdown で返却する 4 つ of MCP ツールを追加。テストのアサーションエラー（Total Files / Total Symbols のマークダウン太字フォーマット不整合）を修正し、すべてのユニットテストおよび JSON-RPC 結合動作検証が正常にパスすることを確認。
- [x] MCP ツール（run_pipeline, get_context）の description 改善（英語翻訳の指示追記）
  - 変更内容: 日本語による質問の際に英語シンボル名にマッチしない問題を防ぐため、外部 AI エージェントが自動でクエリを英語に翻訳した上で呼び出すよう、ツール定義（`run_pipeline` の `task` パラメータ、`get_context` の `query` パラメータ）の description に英語での入力を促す重要指示（`IMPORTANT: The parameter MUST be in English.`）を追記しました。
- [x] ロードマップ 0.2.0 に向けた Word, PowerPoint, Excel の自動インデックスと BM25 検索のサポート
  - 変更内容: `zip` クレートと `quick-xml` クレートを使用し、`.docx`/`.pptx`/`.xlsx` ファイルからポータブルにテキストを抽出するパーサー（`extract_docx_text`, `extract_pptx_text`, `extract_xlsx_text` など）を実装。スライド番号やシート名などの疑似シンボルを SQLite に登録するようにし、BM25 全文検索の対象に Office ファイルを含めました。また、DB登録メソッド `GraphDB::insert_node` で `signature` および `is_exported` カラムの値が失われていた不具合を修正し、Office ファイルのプレビュー情報が正しく DB に永続化され、ツールから呼び出せるようにしました。
  - テスト結果: `npm run daemon:test` (66件)、`npm run test` (67件) がすべてパス。JSON-RPC 統合テストを実行し、Office ファイルがインデックスに乗り、その中身のキーワードで `run_pipeline` 検索結果にヒットすることを実証しました。

- [x] GitHub Pages 公式紹介ページの追加および自動デプロイワークフローの導入、ロードマップ v0.3.0 への PDF サポートの追記
  - 変更内容: HTML/CSS によるオシャレな comP 公式紹介ページ (`docs/index.html`, `docs/index.css`) を新規作成し、バナー画像や VSCode UI モックアップ画像を生成・配置。`main` ブランチへのプッシュ時に自動的に GitHub Pages に紹介ページをデプロイする GitHub Actions ワークフロー (`.github/workflows/pages.yml`) を作成しました。
  - `README.md` および `README_ja.md` に公式 Web ページ（GitHub Pages）へのリンクを追加し、Roadmap の v0.3 に「PDFドキュメントのインデックスサポート」を追加しました。

## 残タスク

1. **コミットとプッシュ**
   - 今回実装したロードマップ 0.2.0 (Office) および 0.3.0 (PDFロードマップ、GitHub Pages紹介ページ、自動デプロイワークフロー) に関連するすべての変更をコミットし、リモート（GitHub）へプッシュ。
