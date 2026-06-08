<p align="center">
  <img src="resources/comp-icon.png" width="128" height="128" alt="comP Logo">
</p>

# comP - AIエージェント向けコンテキストエンジン

**オープンソースで完全ローカル動作するコード分析エンジン。Claude Code・Cursor・Cline・Antigravity対応。**

🌐 **[公式ウェブサイト](https://tsucky230.github.io/comP/)**

> **AIエージェントは、コードを読み歩くたびに大量のトークンを消費します。**
> comPは、コードベース全体を自動でインデックスし、セマンティックなコードグラフを構築することでこれを解決します。
> AIエージェントはファイルを一つずつ開いて読む必要がなくなり、グラフに問い合わせるだけで必要な情報だけを取得できます。
> 結果、**LLMのトークン消費を最大60〘80%削減。** 全処理は完全ローカル。

Claude Code・Cursor・Cline・Antigravity・GitHub Copilot、その他MCP対応エージェントで利用可能です。

---

## 何ができるのか

- **📑 コードインデックス**: tree-sitterで30+言語のコードを自動解析・グラフ化。**バックグラウンドでの非同期インデックス処理**により、エディター起動時のブロッキングを防ぎます。
- **🎯 スマートコンテキスト**: AIエージェントに最小限のコード情報を提供（トークン60% 削減）。
- **🔍 影響分析**: シンボル変更時に影響を受ける全コードを可視化。
- **📊 トークンカウンター**: コンテキストが正確に何トークンか表示し、セッションおよび蓄積された**トークン削減数と効率（%）**を追跡。
- **🔍 BM25検索**: Markdownファイル向けに、見出し名以外の本文キーワードも補完検索できるフルテキスト検索を搭載。
- **🤝 MCP連携**: Claude Code・Cursor・ClineなどのAIエージェントでModel Context Protocolを介して即座に利用。
- **100% ローカル**: 全処理をPC上で完結。外部サーバー不要。

**対応言語（30以上）**: C, C++, C#, Go, Java, JavaScript, TypeScript, Python, Rust, Ruby, Bash, Kotlin, Swift, PHP, Dart, Elixir, Haskell, Lua, R, Zig, SQL, HTML, CSS, YAML, Scalaなど。

---

## 必要な環境

- **OS**: Windows, macOS, Linux
- **VS Code**: 1.85以上
- **Rust**（開発のみ）: 1.70+
- **Node.js**（開発のみ）: 18+

---

## インストール

### VS Codeマーケットプレイスからインストール

1. VS Codeを開く
2. **拡張機能** (`Ctrl+Shift+X`) を開く
3. **"comP - Code Context Engine"** で検索
4. **インストール** をクリック

### GitHubからインストール（開発向け）

```bash
git clone https://github.com/tsucky230/comP.git
cd comP

# 依存関係インストール
npm install

# 拡張機能をビルド
npm run compile

# Rust デーモンをビルド
npm run daemon:build

# F5 でテスト実行
```

---

## クイックスタート

1. comP拡張機能を **インストール**
2. VS Codeで **フォルダーを開く**（Gitリポジトリなど）
3. デーモンの起動:
   - **はじめてのワークスペースの場合**: アクティビティバー（左端）の **comPアイコン** をクリックしてサイドバーを開き、**「Start comP」ボタン** をクリックして手動でインデックス処理を開始します。
   - **2回目以降（`.comp/` ディレクトリが存在する場合）**: フォルダーを開くと、バックグラウンドで **自動的にインデックス処理が開始** されます。
4. **ステータスバー**（下）でインデックス進捗を確認
5. セットアップコマンドを実行：

```text
Ctrl+Shift+P → "comP: Setup Agents"
```

AIエージェントを選択すると、comPは新しいタブでMarkdownの手順書（および自動設定用のLLMプロンプト）を開きます。また、サイドバーからいつでもエージェントの接続状況（最終アクセス時刻）を確認できます。

---

## エージェント別セットアップ手順

### Claude Code（CLI）

「comP: Setup Agents」を実行すると、ターミナルで実行可能な設定コマンド（およびLLM向けプロンプト）が記載されたMarkdownの手順書が開きます。手順書を見逃した場合は `.comp/config/claude_desktop_config.json` を開き、`command` のパスを使用してください。

**Windows (PowerShell)**:

```powershell
# .comp/config/claude_desktop_config.json の "command" の値を使用してください
claude mcp add comp "C:\path\from\generated\file\comp-daemon.exe" -e COMP_WORKSPACE_ROOT="e:\your\project" -e RUST_LOG=info
```

**macOS / Linux**:

```bash
# .comp/config/claude_desktop_config.json の "command" の値を使用してください
claude mcp add comp "/path/from/generated/file/comp-daemon" -e COMP_WORKSPACE_ROOT="/your/project" -e RUST_LOG=info
```

`claude mcp list` で確認できます。

---

### Cursor

生成されるファイル: `.comp/config/cursor_config.json`

`mcpServers` ブロックを以下にマージしてください：

- **グローバル**（全プロジェクト共通）: `~/.cursor/mcp.json`
- **プロジェクト限定**: ワークスペースルートの `.cursor/mcp.json`

保存後、Cursorを再起動してください。

---

### Cline（VS Code拡張）

生成されるファイル: `.comp/config/cline_config.json`

1. VS Code設定を開く（`Ctrl+,`）
2. `Cline › MCP Servers` を検索
3. **settings.jsonで編集** をクリックし、`mcpServers.comp` ブロックを貼り付け

またはClineパネル → **MCP Servers** タブ → **Add Server** からJSONを貼り付け。

---

### Windsurf

生成されるファイル: `.comp/config/windsurf_config.json`

内容を以下にマージしてください：

```text
~/.codeium/windsurf/mcp_config.json
```

保存後、Windsurfを再起動してください。

---

### GitHub Copilot（VS Code）

comPがワークスペースの `.vscode/mcp.json` に直接書き込みます。**追加手順は不要**です。ワークスペースを開き直すと自動で有効になります。

---

### Antigravity

comPが `~/.gemini/antigravity-ide/mcp_config.json` に直接書き込みます。**追加手順は不要**です。Antigravity IDEを再起動してください。

---

### Continue.dev

生成されるファイル: `.comp/config/continue_config.py`

`mcp_servers` ブロックをContinueの設定ファイルに追加してください：

```text
~/.continue/config.py
```

---

### Aider

comPがワークスペースルートの`.aider.conf.yml`に直接書き込みます。**追加手順は不要**です。Aiderを再起動してください。

---

## エージェント互換性

| エージェント | 状況 | 備考 |
| --- | --- | --- |
| **Claude Code** | ✅ 対応 & 確認済み | 開発者が検証済み |
| **GitHub Copilot** | ✅ 対応 & 確認済み | 開発者が検証済み |
| **Antigravity** | ✅ 対応 & 確認済み | Antigravity IDEの標準エージェント |
| **Cursor** | ✅ 対応済み | MCP 2024-11-05準拠 |
| **Cline** | ✅ 対応済み | MCP 2024-11-05準拠 |
| **Windsurf** | ✅ 対応済み | MCP 2024-11-05準拠 |
| **Aider** | ✅ 対応済み | ワークスペースルートの`.aider.conf.yml`に自動書き込み |
| **Gemini** | ❌ 非対応 | エージェント側のMCPクライアント機能なし |

MCP 2024-11-05準拠のクライアントであれば動作します。お使いのエージェントで特別な設定が必要な場合は [Issueを作成](https://github.com/tsucky230/comP/issues/new) してください。

---

## 使い方

### コマンドパレット

**Ctrl+Shift+P** でアクセス可能：

| コマンド | 説明 |
| --- | --- |
| **comP: Setup Agents** | Claude Code・Cursor・AntigravityなどのAIエージェントを設定 |
| **comP: Force Re-index** | コードベース全体を再インデックス |
| **comP: Generate Context Capsule** | 現在のタスク向け最適コンテキストを抽出 |
| **comP: Show Impact Graph** | カーソル位置のシンボル影響範囲を表示 |
| **comP: Copy Active File Compressed** | 開いているファイルをAST圧縮（コメント除去やスケルトン化）してクリップボードにコピー |
| **comP: Export Debug Log** | `session-memory.json`をエディターで開くか任意パスにエクスポートしてデバッグ情報を確認 |

### ステータスバー

VS Code下部に表示：

```text
◈ comP: 12,534 nodes | ✓ Ready
```

クリックで **統計ダッシュボード** を開く：

- インデックス進捗
- ノード・エッジ数
- トークン削減率（例: 60% 削減）
- セッションおよび累積のトークン統計（送信数、削減数、削減効率 %）

### AIエージェント統合

設定後、Claude CodeなどのAIエージェントがcomPツールを呼び出し可能：

```markdown
# Claude Code のチャットで:
@comP run_pipeline
authenticate() 関数を変更した場合の影響を分析してください
```

### VS Codeチャット参加者（@comp）

VS Codeのチャット（Copilot Chat等）で直接 `@comp` を使って、長大なファイルを自動圧縮しながら質問できます：

```markdown
@comp #file:src/main.rs を使って、この関数の動作を説明して
```

添付されたファイルはcomPのスケルトンモードで自動的に圧縮（コメント削除や関数ボディの `{ ... }` 置き換え）されてLLMに渡るため、トークン消費を最小限に抑えつつ正確な回答を得られます。

---

## 設定

comPの設定はプロジェクトルートの `.comp/` フォルダーに保存されます（デフォルトで `.gitignore` 対象）。

### ファイル・ディレクトリの除外

`.comp/ignore` を作成すると、インデックス対象から除外できます（`.gitignore` と同じ書き方）：

```gitignore
node_modules/
vendor/
dist/
build/
target/
__pycache__/
*.min.js
```

デフォルトで除外されるパターン: `node_modules/`, `.git/`, `dist/`, `build/`, `target/`

### インデックス上限

大規模リポジトリ向けに `.comp/config.json` でインデックス動作を制御できます：

```json
{
  "max_nodes": 100000,
  "on_limit_exceeded": "warn",
  "default_budget_tokens": 8000,
  "compression_rules": {
    "*.md": 0,
    "*.rs": 2,
    "*.ts": 1
  }
}
```

| オプション | 値 | デフォルト | 説明 |
| --- | --- | --- | --- |
| `max_nodes` | 整数 | `200000` | ノード数がこの値を超えたときの閾値 |
| `on_limit_exceeded` | `"warn"` \| `"stop"` | `"warn"` | `warn`: 通知して続行 · `stop`: インデックス停止 |
| `default_budget_tokens` | 整数 | — | `run_pipeline`のトークンバジェット。指定すると圧縮レベルを0→1→2で自動選択 |
| `compression_rules` | オブジェクト | — | 拡張子別の圧縮レベル上書き。globパターン→レベル（0/1/2）。自動バジェット選択より優先 |

> **データベースサイズの目安** (`.comp/index.db`): 小規模（~1kファイル）で1〜5 MB、中規模（~10kファイル）で20〜80 MB、大規模（100k+ファイル）で200 MB〜1 GB。シンボルのメタデータのみ保存—生コンテンツは含みません。
>
> **モノレポの場合**: 作業中のサブフォルダーだけをVS Codeで開いてください。comPは開いているワークスペースフォルダーのみをインデックスします。

---

## 仕組み

### アーキテクチャ

1. **インデックサー（Rustデーモン）**: ワークスペースをスキャン、tree-sitterでパース、SQLiteに保存
2. **検索エンジン**: セマンティック検索+グラフトラバーサル+ BM25検索（補完）で関連コードを検出
3. **MCPサーバー**: Model Context Protocol経由でAIエージェントにツール提供
4. **VS Code拡張**: デーモン管理、UI表示、ユーザーコマンド処理

### データフロー

```text
コードファイル
     ↓
 tree-sitter（30+ 言語解析）
     ↓
 SQLite グラフ DB（.comp/index.db）
     ↓
 セマンティック検索 ＆ BM25
     ↓
 MCP ツール（run_pipeline 等）
     ↓
 AI エージェント（Claude Code・Cursor・Cline・Antigravity）
```

---

## 出力・結果

**"comP: Generate Context Capsule"** 実行時：

```text
📌 メインファイル（全コンテンツ）:
  - src/auth/authenticate.ts (150 行)

📎 関連ファイル（シグネチャのみ）:
  - src/auth/session.ts (14 関数)
  - src/types/user.ts (5 型定義)

📊 コンテキスト概要:
  - 合計トークン: 2,340
  - 削減率: 65%（全ファイル送信比）
  - 推定コスト: $0.04（通常は $0.11）
```

---

## トラブル対応

### 「comPがインデックスしない」

- **ステータスバー**（下）で進捗を確認。
- 止まっている場合: **Ctrl+Shift+P** → "comP: Force Re-index"
- `.comp/` フォルダーが存在し `.gitignore` に登録されているか確認。

### 「MCP接続に失敗した」

- **"comP: Setup Agents"** を再実行。
- 使用中のエージェントに `.comp/mcp-config.json` があるか確認。
- VS Code **出力パネル**（表示 → 出力 → "comP"）でエラー確認。

### 「インデックスが遅い」

- 大規模リポジトリ（>100kファイル）は初回が時間かかる。
- その後は増分処理（高速）。
- システムリソース確認: comPは <500MB RAM。

### 「一部言語が認識されない」

- comPは30+言語をサポート。
- **対応外ファイル** は無視される（インデックス対象外）。
- Word（.docx）はv2で対応予定。

---

## 貢献する

バグ報告・機能提案・コード貢献を歓迎します。詳細は [CONTRIBUTING.md](CONTRIBUTING.md) をご覧ください。

### 開発環境構築

```bash
# クローン・インストール
git clone https://github.com/tsucky230/comP.git
cd comP
npm install

# テスト実行
cargo test --all --manifest-path daemon/Cargo.toml
npm test

# ウォッチモード
npm run watch
npm run daemon:build -- --watch  # 別ターミナル

# Markdown Lint
npm run lint:md:fix
```

---

## ライセンス

**MITライセンス** — [LICENSE](LICENSE) 参照

---

## ロードマップ

| バージョン | 機能 | 状況 |
| --- | --- | --- |
| **v0.1** | コアインデックス、基本MCP、30言語、JSON/XML/Markdown、非同期インデックス、トークン統計。 | ✅ **リリース済み** |
| **v0.2** | Word (.docx)、PowerPoint (.pptx)、Excel (.xlsx) の自動インデックス、BM25全文検索。 | ✅ **リリース済み** |
| **v0.3** | PDF (.pdf) サポート、高度な影響分析（`max_depth`）、TF-IDF検索を `run_pipeline` に接続、マルチパスインデックス、`get_symbol` AST圧縮 | ✅ **リリース済み** |
| **v0.4** | `run_pipeline` コンテンツモード（`include_content`/`compression_level`）、PRレビュー用 `get_git_diff_context` ツール、言語分布対応 `get_project_overview` 拡張 | ✅ **リリース済み** |
| **v0.5** | クリップボードへのコード圧縮コピーコマンド（`copyActiveFileCompressed`）、VS Code Chat Participant APIによる `@comp` チャット参加者機能、Parquet (.parquet) ファイルの自動インデックス化およびBM25全文検索サポート。 | ✅ **リリース済み** |
| **v0.6** | 動的バジェット：`run_pipeline`が`.comp/config.json`の`default_budget_tokens`を読み込み、バジェット内に収まるよう圧縮レベル0→1→2を自動選択。レスポンスに`compression_level_applied` / `budget_adjusted`フラグを追加。 | ✅ **リリース済み** |
| **v0.7** | 拡張子別圧縮ルール（`.comp/config.json`に`compression_rules`追加）。Aiderエージェント対応（`.aider.conf.yml`自動生成）。新コマンド`comP: Export Debug Log`。トークン可視化の状態バグ修正。 | ✅ **リリース済み** |
| **v1.0** | API安定化、エージェント拡大、コミュニティ統合 | ⚪ 計画中 |

---

## リンク

- **GitHub**: <https://github.com/tsucky230/comP>
- **Issue**: <https://github.com/tsucky230/comP/issues>
- **Discussions**: <https://github.com/tsucky230/comP/discussions>
- **アーキテクチャドキュメント**: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)
- **APIリファレンス**: [docs/API.md](docs/API.md)

---

## このプロジェクトへの支援

comPは完全無料・オープンソースです。このプロジェクトが役に立つなら、開発を支援していただけませんか？

- ☕ **[GitHubスポンサー](https://github.com/sponsors/tsucky230)** — 開発を応援（近日公開）
- 💖 **このリポジトリにStarをつける** — 他の人に知らせてください

皆さんのご支援が、開発・保守・新機能追加を支えています。ありがとうございます！🙏

---

## 質問・お問い合わせ

- 📖 **ドキュメント**: [docs/](docs/)
- 🐛 **バグ報告**: [Issueを作成](https://github.com/tsucky230/comP/issues/new)
- 💬 **フィードバック**: [Discussionを開始](https://github.com/tsucky230/comP/discussions/new)
- 👥 **参加したい**: [CONTRIBUTING.md](CONTRIBUTING.md)
