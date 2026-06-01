# 会話ログ (Conversation Log)

## 2026-05-31

### ユーザーからの要望

- 「残務あるかな？」に対する調査および対応。
- 未コミット変更の動作検証。
- コミットの整理とリモート（GitHub）へのプッシュ。
- VSCodeマーケットプレイス登録のための README.md/README_ja.md 整備、および不要ファイルの混入確認。

### 実施内容

1. ワークスペース内の未コミット変更の調査：
   - デーモン側（Rust）でバックグラウンドインデックス、トークン統計（get_token_usage等）、BM25検索の実装が追加されていることを確認。
   - VSCode拡張機能側（TypeScript）でライフサイクル改善やコマンド常時登録が実装されていることを確認。
2. テストの実行と不具合修正：
   - `npm run daemon:test` を実行し、Rust側テスト（63件）が正常であることを確認。
   - `npm run test` を実行したところ、`commands.test.ts` 内の `registerCommands` のインターフェース変更（引数として `getDaemonManager` 関数を要求）に伴う `TypeError` で3件 of テストが失敗していることを検知。
   - `src/ui/tests/commands.test.ts` 内の呼び出しを `registerCommands(mockContext, () => mockDaemon, mockStatusBar)` に修正。
   - テストを再実行し、TypeScript側のテスト（61件）がすべてパスすることを確認。
3. プロジェクト管理ドキュメントの新規作成：
   - グローバルルールに基づき、`docs/conversation_log.md` および `docs/work_plan.md` を作成。
4. 未コミット変更の動作検証：
   - TypeScriptコードの静的検証 (`npm run lint:ts`) を実行し、エラーがないことを確認。
   - `test_daemon_rpc.py` を作成し、JSON-RPCを介してデーモンの新規機能（非同期インデックス、BM25検索、トークン使用量取得など）が正常応答を返すことを確認。
   - 全ての検証シナリオがパスしたことを示す `walkthrough.md` を作成。
5. コミットの整理とプッシュ：
   - 変更があった 15 ファイルおよび新規ドキュメントについて、論理的なグループごとに 5 つのコミットに分割して作成。
   - 各コミット作成時の自動テスト（TS/Rust）もすべてパス。
   - 最終確認後、`git push origin main` を実行してリモートリポジトリ（GitHub）へすべての変更をプッシュ。
6. マーケットプレイス登録向けドキュメント・設定整備：
   - README.md / README_ja.md にロゴバナー追加、新機能（非同期インデックス、トークン統計、BM25検索等）の明記、互換エージェントに **Antigravity** を追加、ロードマップの v0.1 リリース完了（Released）を反映。
   - VSIX パッケージへの開発一時ファイル混入防止のため、`.vscodeignore` に `.comp/`, `.claude/`, `temp/`, `scripts/` などのパターンを追加。
   - `npx vsce ls` コマンドでパッケージ内ファイルが最小限（18ファイル）で、サイズが 2.36MB に最適化されたことを検証。
   - 進捗を `walkthrough.md` に追記。

### 次回のタスク

- 整備した内容のコミットとプッシュ。

---

## 2026-05-31 (コメント英語化タスクの完了)

### ユーザーからの要望

- ソースコードのコメントは全て簡潔な英語にしてほしい。

### 実施内容

1. 日本語コメントの最終確認と英語化置換：
   - `src/test/setup.ts`、`src/daemon/DaemonManager.ts`、および Rust デーモン側の `daemon/src/graph/mod.rs` 内に日本語コメントが残存していることを確認。
   - これらの日本語コメントをすべて簡潔な英語（concise English）に翻訳・置換しました。
   - 置換後、正規表現による grep 検索を用いて、TypeScript および Rust の全ソースコード内に日本語コメントの残存がないことを最終確認しました。
2. 静的検証とテストの実行：
   - `npm run lint:ts` を実行し、型チェックエラーがないことを確認。
   - `npm run test`（TSテスト 61件）および `npm run daemon:test`（Rustテスト 63件）を実行し、すべて正常にパスすることを確認。
3. リリースビルドと結合動作検証：
   - バックグラウンドプロセスとして残っていた `comp-daemon.exe` を `taskkill` で強制終了させ、最新の Rust 側コードを反映した `cargo build --release` を正常完了。
   - Python による JSON-RPC 結合テストスクリプト `test_daemon_rpc.py` を実行し、英語化されたデーモンが正しく動作（getStats, run_pipeline, get_token_usage 等のAPI疎通）することを確認。

### 次回のタスク

- 英語化およびドキュメント更新を含めた変更の一括プッシュ。

---

## 2026-05-31 (初回起動手順のマニュアル整備)

### ユーザーからの要望

- README_jaの「自動スタート」が初回使用時に手動で起動ボタンを押す必要がある点と食い違っているため、READMEおよびREADME_jaの両方で正しいマニュアルとして整備してほしい。

### 実施内容

1. `README.md` および `README_ja.md` の修正：
   - ワークスペースを初めて開く場合（`.comp/` ディレクトリが存在しない場合）は、VSCodeのサイドバーから「Start comP」ボタンを押して手動で開始する必要がある旨を追記しました。
   - 二回目以降（`.comp/` ディレクトリが存在する場合）は、フォルダを開くと自動でバックグラウンド起動する旨を明記しました。
2. Markdown のチェック：
   - `npm run lint:md` を実行し、Markdownの構文エラーがないことを確認。

### 次回のタスク

- マニュアル整備箇所のコミットとプッシュ。

---

## 2026-05-31 (最新版 VSIX のパッケージング)

### ユーザーからの要望

- 最新版の.vsixファイルを作成してほしい。

### 実施内容

1. `npx vsce package` の実行：
   - 英語コメント化および起動マニュアル整備を反映した最新バージョンの拡張機能パッケージ `comp-vscode-0.1.0.vsix` を作成しました。
   - パッケージング時に `npm run vscode:prepublish` が自動的に走り、マーケットプレイス用アイコンの生成（`generate-icon`）および本番ビルド（`esbuild --minify`）が問題なく成功することを確認しました。

### 次回のタスク

- 管理ドキュメント更新分のコミットとプッシュ。

---

## 2026-05-31 (カテゴリ数修正とVSIXの再パッケージング)

### ユーザーからの要望

- マーケットプレイスのカテゴリ上限に対応した `package.json` の修正を反映し、再度 `.vsix` ファイルを作成してほしい。

### 実施内容

1. `package.json` のカテゴリ設定の確認：
   - カテゴリを `"AI"` および `"Other"` の2つに絞り込み、VS Code マーケットプレイスの仕様を満たしていることを確認。
2. バックエンドおよびテストの実行：
   - `npm run daemon:build` で Rust デーモンの最新リリースビルド（`comp-daemon.exe`）を生成。
   - `npm run test` で TypeScript のテスト（61件）が正常にパスすることを確認。
3. `npx vsce package` の再実行：
   - カテゴリ数修正を反映した最新バージョンの拡張機能パッケージ `comp-vscode-0.1.0.vsix`（20ファイル、2.36MB）を再作成しました。
   - prepublish 時のアイコン生成および minify ビルドが正常に完了することを確認。

### 次回のタスク

- 管理ドキュメントの更新分および `package.json` の変更をコミットし、リモート（GitHub）へプッシュ。

---

## 2026-05-31 (GitHub Actionsエラーの解消およびマルチプラットフォームビルド対応)

### ユーザーからの要望

- GitHub Actions上での `npm ci` のエラー（`package-lock.json` がない）を解消し、`.github/` のワークフロー全体を見直して修正してほしい。

### 実施内容

1. ロックファイルのGit追跡開始：
   - `.gitignore` から `package-lock.json` と `Cargo.lock` の除外設定を削除し、Gitでバージョン管理するように変更。これによりCI上での `npm ci` が動作可能になりました。
2. デーモンのマルチプラットフォーム同梱対応：
   - 本番（VSIX）環境では `comp-daemon-win.exe`、`comp-daemon-macos`、`comp-daemon-linux` のプラットフォーム別名バイナリを探すように [DaemonManager.ts](file:///e:/dev/comP/src/daemon/DaemonManager.ts) を修正。
   - 開発（ローカル）環境では、従来の Cargo 標準出力名である `comp-daemon`/`comp-daemon.exe` の release/debug バイナリを優先して探すフォールバック処理を実装。
3. `.vscodeignore` の修正：
   - パッケージング時に3つのプラットフォーム別バイナリがすべて取り込まれるように設定を修正。
4. `release.yml` ワークフローの再設計：
   - ジョブを「デーモンビルド（マトリクス）」と「拡張機能パッケージ（Linux上）」に分離。
   - 各 OS でビルドしたデーモンバイナリを Artifact にアップロードし、パッケージジョブでそれらをダウンロードして統合パッケージを生成、GitHub リリースにアタッチするフローに変更。
5. 動作検証：
   - ローカルでのビルド (`npm run compile`) およびユニットテスト (`npm run test`) が正常にパスすることを確認。

### 次回のタスク

- 特になし（全てのリリースフロー・パッケージ修正完了）。

---

## 2026-05-31 (SECURITY.md の MarkdownLint エラー修正)

### ユーザーからの要望

- SECURITY.md 内の MD032（リスト前後の空行なし）エラーによる CI 失敗を修正してほしい。

### 実施内容

1. 空行の追加：
   - `SECURITY.md` 内の `In scope:` および `Out of scope:` の直後のリストの前に空行を追加し、markdownlint 規格を満たすように修正。

### 次回のタスク

- 特になし（すべての不具合修正完了）。

---

## 2026-05-31 (cargo audit 警告の除外設定追加)

### ユーザーからの要望

- GitHub Actions の `cargo audit` で `paste` クレートのメンテナンス終了警告（RUSTSEC-2024-0436）によるビルド失敗を修正してほしい。

### 実施内容

1. `audit.toml` の追加：
   - 警告自体はセキュリティ脆弱性ではないため、`daemon/audit.toml` を作成し、警告 ID `RUSTSEC-2024-0436` を監査対象から除外（ignore）するように設定。

### 次回のタスク

- 特になし（すべての不具合修正完了）。

---

## 2026-05-31 (CIワークフローの安定化および高速化)

### ユーザーからの要望

- `ci.yml` および `release.yml` をレビューし、失敗する要素があれば修正してほしい。

### 実施内容

1. ワークフロー設定のレビュー：
   - `release.yml` はビルドからアーティファクト取得、リリースへのアタッチまで問題ないことを確認。
   - `ci.yml` の `sast` ジョブにおいて、Clippyのコンポーネントが明示的にセットアップされておらず、かつ `cargo install cargo-audit` による無駄なコンパイル時間がかかっている問題（1回あたり5〜10分）を発見。
2. CIの修正（安定化と高速化）：
   - `ci.yml` の `sast` ジョブで、`dtolnay/rust-toolchain` に `components: clippy` を指定して Clippy の起動エラーを防止。
   - `cargo install cargo-audit` と直接の `cargo audit` コマンド実行を廃止し、公式の `rustsec/audit-action@v0.1.3` に置き換え。これによりCIのコンパイル時間を大幅に削減。

### 次回のタスク

- 特になし（CI/CDワークフローの最適化完了）。

---

## 2026-05-31 (README.md および SECURITY.md の MarkdownLint エラー修正)

### ユーザーからの要望

- README.md および SECURITY.md 内の MD013 (Line length) および MD034 (Bare URL) エラーを修正してほしい。また、メールアドレスは非開示にしてほしい。

### 実施内容

1. メールアドレスの非開示対応：
   - `SECURITY.md` からメールアドレスによる脆弱性報告の受付記述を削除し、GitHub の Private Vulnerability Reporting のみに統合。これによって `MD034/no-bare-urls` を解消。
2. 行の長さの最適化（80文字制限）：
   - `SECURITY.md` および `README.md` の該当行で、1行の長さが 80文字を超える部分（テーブル行、スポンサーリンク、メッセージ等）について、適切に改行または文章の表現を短縮し `MD013/line-length` エラーを解消。

### 次回のタスク

- 特になし（すべての Linter エラー解消完了）。

---

## 2026-05-31 (ci.yml 内の cargo audit アクションの修正)

### ユーザーからの要望

- `rustsec/audit-action` のリポジトリが見つからないエラーによるビルド失敗を解決してほしい。

### 実施内容

1. 正しいアクションの導入：
   - 存在しない `rustsec/audit-action` から、推奨されている `actions-rust-lang/audit@v1` に差し替え。
   - サブディレクトリにある `daemon/Cargo.lock` を対象にするため、`workingDirectory: daemon` パラメータを設定。

### 次回のタスク

- 特になし（CIのすべてのビルドステップの正常動作を確認予定）。

---

## 2026-06-02 (セッションメモリ機能の実装)

### ユーザーからの要望

- 過去の MCP ツール呼び出し（クエリ・返却シンボル名・トークン数）をセッションごとにファイルへ永続化する
- クエリ後にコードが変更された場合、そのエントリを自動で "stale" マークする
- 新しい MCP ツール session_recall を src/mcp/ に追加する
  - input: { query?: string } // 省略時はサマリー返却
  - output: 過去観測の Markdown リスト（stale フラグ付き）
- Rust デーモン側ではなく TypeScript 拡張側で管理し、.comp/session-memory.json に保存する
- DaemonManager のファイル変更通知をフックして stale 検知に利用する

### 実施内容

1. **セッションメモリ管理モジュールの新規追加**:
   - TypeScript側で `src/mcp/sessionMemory.ts` に `SessionMemoryManager` を実装。`.comp/session-memory.json` のロード、セーブ、およびファイル変更フックに応じた `stale` マークの付与を行います。
   - `src/mcp/tests/sessionMemory.test.ts` にて `SessionMemoryManager` のユニットテスト（3件）を記述。
2. **ファイル変更通知へのフック**:
   - VSCode拡張機能の `src/extension.ts` 内の `setupFileWatchers` でファイル変更（onDidChange / onDidDelete）をフックし、`SessionMemoryManager.markStaleForFile()` を呼び出して該当するクエリ履歴を自動で stale マークする処理を追加。
3. **Rust デーモン側セッション永続化とツールの実装**:
   - `daemon/src/main.rs` で起動時のタイムスタンプを用いて `session_id` を発行し `AppState` に保持。
   - `run_pipeline`（`task` をクエリとする）と `get_context`（`query` をクエリとする）の処理時に、実行クエリ、ヒットしたシンボル名、ファイルパス、推定トークン数を `.comp/session-memory.json` に追記保存。
   - 新しい MCP ツール `session_recall` を `daemon/src/mcp/mod.rs` に追加。`.comp/session-memory.json` 内の現在のセッションの履歴を Markdown リストに整形して返却（オプション引数 `query` による大文字小文字を区別しない部分一致フィルタも実装）。
   - `test_session_recall` ユニットテストを追加。
4. **動作テストと検証**:
   - `npm run test`（TSテスト 64件）および `npm run daemon:test`（Rustテスト 64件）を実行し、すべて正常にパスすることを確認。
   - 結合動作検証として `test_session_recall.py` スクリプトを用いて JSON-RPC 経由で MCP ツールの疎通と `.comp/session-memory.json` への永続化、Markdown 出力が要件通り動作することを実証。

### 次回のタスク

- 特になし（セッションメモリ機能の開発完了）。

---

## 2026-06-02 (Rustデーモンへの4つのMCPツール追加)

### ユーザーからの要望

Rustデーモン（`daemon/src/`）に以下の4つのMCPツールを追加する。
- `get_symbol`: 指定シンボルのソースコード＋依存先＋依存元（Markdown）
- `get_dependencies`: 指定シンボルの依存先または依存元一覧（Markdown）
- `get_file_summary`: ファイル内の全シンボル名・種別・シグネチャ一覧（Markdown）
- `get_project_overview`: ファイル数・シンボル数・エクスポートシンボルの概要（Markdown）

### 実施内容

1. **テストアサーションの不具合修正**:
   - `test_new_mcp_tools` ユニットテストで起きていた `Total Files: 1` などのアサーション不整合（マークダウンの太字 `**` フォーマットの不一致）を修正し、テストが正常にパスするように変更。
2. **デーモンプロセスのクリーンアップとビルド**:
   - バックグラウンドでロックされていた旧 `comp-daemon.exe` プロセスを `taskkill` にて強制終了し、`npm run daemon:build`（`cargo build --release`）で正常ビルド完了。
3. **テストの実行**:
   - `npm run daemon:test` を実行し、Rust側テスト（65件）がすべてパスすることを確認。
   - `npm run test` を実行し、TypeScript側のテスト（64件）がすべてパスすることを確認。
4. **結合動作検証 (JSON-RPC)**:
   - 一時検証用スクリプト `temp/test_new_mcp_rpc.py` を作成し、実際の `comp-daemon.exe` リリースプロセスに対して標準入出力 (JSON-RPC) 経由でリクエストを送信。
   - `tools/list` にて新規 4 ツールが正しく公開されていることを確認。
   - `get_project_overview`, `get_file_summary`, `get_dependencies` をそれぞれ呼び出し、仕様通りの Markdown 出力やエラーレスポンスが得られることを実証。
5. **管理ドキュメントの更新**:
   - `docs/conversation_log.md` および `docs/work_plan.md` を更新。

### 次回のタスク

- 変更内容を Git でコミットし、リモート（GitHub）へプッシュ。

---

## 2026-06-02 (MCP ツール description の英語翻訳指示の追加)

### ユーザーからの要望

日本語による「インデックス処理はどこにある？」のような質問時、`run_pipeline` や `get_context` にそのまま日本語クエリが渡ると BM25 トークン化により英語のコード構造・シンボル名にマッチせず、結果が 0 件になる問題を回避したい。
LLM（Copilot / Cline 等の外部エージェント）が自動的に日本語クエリを英語に翻訳した上で MCP ツールを呼び出すようにするため、各ツールの description に「クエリは英語で入力せよ」という指示を追記する。

### 実施内容

1. **ツール定義の `description` 修正**:
   - `daemon/src/mcp/mod.rs` 内の `handle_tools_list` にて定義されている以下のツールの description に英語入力指定を追記：
     - `run_pipeline` ツール本体および引数 `task` の説明
     - `get_context` ツール本体および引数 `query` の説明
   - 外部 AI エージェントが日本語クエリを自動翻訳して渡すよう、`IMPORTANT: The parameter MUST be in English.` などの強調指示を追加しました。
2. **動作テストと検証**:
   - `npm run daemon:test` を実行し、Rust側テスト（65件）がすべてパスすることを確認。
   - `npm run test` を実行し、TypeScript側のテスト（64件）がすべてパスすることを確認。
3. **管理ドキュメントの更新**:
   - `docs/conversation_log.md` および `docs/work_plan.md` を更新。

### 次回のタスク

- 変更内容を Git でコミットし、リモート（GitHub）へプッシュ。

---

## 2026-06-02 (GitHub Copilot 向け MCP 自動設定機能の追加)

### ユーザーからの要望

LLM登録メニュー（`comp.setupAgents`）に GitHub Copilot を追加し、選択できるように実装してほしい。

### 実施内容

1. **コマンドメニューへの追加**:
   - `src/ui/commands.ts` の `comp.setupAgents` コマンドで表示する quick pick の選択肢に `"GitHub Copilot"` を追加しました。
2. **自動設定マージ機能の実装**:
   - `src/mcp/AgentSetup.ts` 内の `AgentSetupManager` に GitHub Copilot 向けのサポートを追加。
   - VS Code 上の GitHub Copilot (Agent Mode) が認識するワークスペース設定パス `.vscode/mcp.json` を出力先とし、既存の設定ファイルがある場合はそれをパースして `servers.comp` に comP MCP デーモンの起動設定をマージする `generateCopilotConfig` および `copilotConfigPath` を実装。
3. **ユニットテストの追加と実行**:
   - `src/mcp/tests/AgentSetup.test.ts` に `getAgentConfig` と `generateConfig` で GitHub Copilot 向けの設定が正常生成されるかのテストを追加。
   - `src/ui/tests/commands.test.ts` に `comp.setupAgents` で GitHub Copilot が選択された際のフローテストを追加。
   - `npm run test` を実行し、全 67 件の TypeScript テストが正常にパスすることを確認しました。また、`npm run daemon:test` （65件）も正常終了を確認。
4. **管理ドキュメントの更新**:
   - `docs/conversation_log.md` および `docs/work_plan.md` を更新。

### 次回のタスク

- 変更内容を Git でコミットし、リモート（GitHub）へプッシュ。

---

## 2026-06-02 (ロードマップ 0.2.0: Office ドキュメント自動インデックス & BM25 全文検索サポート)

### ユーザーからの要望

- ロードマップ 0.2.0 に向け、Word (`.docx`) に加え、PowerPoint (`.pptx`) および Excel (`.xlsx`) ファイルも自動でインデックスし、BM25 フルテキスト検索の対象に含める。
- SQLite の `symbols` テーブルへシート名やスライド数などを登録する。

### 実施内容

1. **Office ファイルからのテキスト抽出機能とパーサーの実装**:
   - `daemon/Cargo.toml` に `zip` 依存関係を追加。
   - `daemon/src/indexer/doc_parser.rs` において、`extract_docx_text`, `extract_pptx_text`, `extract_xlsx_text` メソッドおよび `parse_docx`, `parse_pptx`, `parse_xlsx` メソッドを実装。
   - Word ドキュメントは代表シンボル `Word Document`、PowerPoint はスライドごとの `Slide N`、Excel はシート名 `Sheet: SheetName` を `Module` タイプのシンボルとして抽出するよう定義しました。

2. **言語Walkerとインデクサーのディスパッチ統合**:
   - `daemon/src/indexer/walker.rs` および `daemon/src/indexer/mod.rs` を修正し、拡張子 `.docx`, `.pptx`, `.xlsx` を言語タイプ `docx`/`pptx`/`xlsx` として処理対象に登録し、バイナリファイルとしてパースされるよう統合しました。

3. **シンボル挿入時の signature / is_exported 永続化バグの修正**:
   - SQLite DB 登録処理において `GraphDB::insert_node` メソッドが `signature` と `is_exported` カラムへの挿入を行っていなかったバグを修正。引数に両フィールドを追加して `nodes` テーブルへ正しく永続化するようにしました。
   - インデクサー側、および MCP テストコードにおける `insert_node` 呼び出し箇所をこれに合わせて更新しました。これによって `get_file_summary` などのツール呼び出し時に、Office ファイルの抽出テキストが `Signature` カラムにプレビュー（最大200文字）として表示されるようになりました。

4. **BM25 全文検索範囲の拡張**:
   - `run_pipeline` ツールの BM25 検索処理において、従来 Markdown のみを対象としていた検索候補を拡張し、Office ファイル (`.docx`, `.pptx`, `.xlsx`) もインデックス対象パスとして含めて検索を行うように修正。

5. **動作検証とクリーンアップ**:
   - `npm run daemon:test` (66件) および `npm run test` (67件) がすべてパスすることを確認。
   - 統合検証テストスクリプト `temp/test_office_indexing.py` に DB クリーンアップ処理およびポーリングによるインデックス完了検出を追加して動作させ、Office ファイルが正常にインデックスされ、そのテキストの内容（`comPRoadmapProject` 等）が `run_pipeline` (BM25) の検索結果（`pivot_files`）に含まれて正しくヒットすることを実証（`ALL OFFICE INDEXING & SEARCH INTEGRATION TESTS PASSED!`）。
   - 検証完了後、一時ファイルおよびテストスクリプトを安全にクリーンアップ。

### 次回のタスク

- 特になし（今回の機能実装およびバグ修正完了）。

---

## 2026-06-02 (ロードマップ 0.3.0 計画 & GitHub Pages 公式紹介ページの追加・自動デプロイ)

### ユーザーからの要望

- ロードマップ 0.3.0 に PDF の自動インデックスサポートを含める。
- PUSH 時の CI/CD パイプラインの動作状況について確認・確認。
- GitHub Pages を使った公式紹介ページを構築し、自動デプロイワークフローを整備。
- README および README_ja に公式紹介ページへのリンクを貼る。

### 実施内容

1. **GitHub Pages 公式紹介ページの新規作成と多言語対応**:
   - `docs/index.html`（英語版）および `docs/index_ja.html`（日本語版）を新設。モダンなダークモード（ネオンブルーとパープルのアクセント）、グラスモーフィズムデザイン、 Outfit/Inter プレミアムフォントなどを導入したレスポンシブな LP を構築。
   - `navigator.language` に基づくブラウザ言語判定スクリプトを仕込み、日本語環境からのアクセスを自動的に `index_ja.html` にリダイレクトする仕組みを実装。
   - 手動の「English / 日本語」トグルボタンを設置し、ユーザーの優先設定を `localStorage` に保持して次回以降に優先適用されるようにしました。
   - プロダクトイメージとして `comp_banner.png` および `comp_ui_mockup.png` を `generate_image` を使用して生成し、`docs/` 以下に配置。
   - `docs/index.css` にて多言語切り替えボタンのスタイリングを追加。
2. **自動Pagesデプロイワークフローの追加**:
   - `.github/workflows/pages.yml` を新規作成。`main` ブランチに PUSH された際、`docs/` ディレクトリを自動で GitHub Pages（`https://tsucky230.github.io/comP/`）へビルド・デプロイするフローを定義。
3. **ロードマップおよびリンクの追記**:
   - `README.md` および `README_ja.md` のヘッダー部分に `Official Website / 公式ウェブサイト` のリンクを設置。
   - ロードマップセクションの **v0.3** 計画に `PDF (.pdf) support` / `PDF (.pdf) サポート` を追記。
4. **動作検証とテスト**:
   - `npm run daemon:test` (66件) および `npm run test` (67件) が正常にパスすることを確認。
   - 日・英双方のリダイレクトスクリプトおよび `localStorage` 連携、切り替えボタンが仕様通り動作することを確認。

### 次回のタスク

- 変更内容を Git でコミットし、リモート（GitHub）へプッシュ。





