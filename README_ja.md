<p align="center">
  <img src="resources/comp-icon.png" width="128" height="128" alt="comP Logo">
</p>

# comP - AI エージェント向けコンテキストエンジン

**オープンソースで完全ローカル動作するコード分析エンジン。Claude Code・Cursor・Cline・Antigravity 対応。**

🌐 **[公式ウェブサイト](https://tsucky230.github.io/comP/)**

> **AI エージェントは、コードを読み歩くたびに大量のトークンを消費します。**
> comP は、コードベース全体を自動でインデックスし、セマンティックなコードグラフを構築することでこれを解決します。
> AI エージェントはファイルを一つずつ開いて読む必要がなくなり、グラフに問い合わせるだけで必要な情報だけを取得できます。
> 結果、**LLM のトークン消費を最大60〘80%削減。** 全処理は完全ローカル。

Claude Code・Cursor・Cline・Antigravity・GitHub Copilot 、その他 MCP 対応エージェントで利用可能です。

---

## 何ができるのか

- **📑 コードインデックス**: tree-sitter で 30+ 言語のコードを自動解析・グラフ化。**バックグラウンドでの非同期インデックス処理**により、エディタ起動時のブロッキングを防ぎます。
- **🎯 スマートコンテキスト**: AI エージェントに最小限のコード情報を提供（トークン 60% 削減）。
- **🔍 影響分析**: シンボル変更時に影響を受ける全コードを可視化。
- **📊 トークンカウンター**: コンテキストが正確に何トークンか表示し、セッションおよび蓄積された**トークン削減数と効率（%）**を追跡。
- **🔍 BM25 検索**: Markdown ファイル向けに、見出し名以外の本文キーワードも補完検索できるフルテキスト検索を搭載。
- **🤝 MCP 連携**: Claude Code・Cursor・Cline などの AI エージェントで Model Context Protocol を介して即座に利用。
- **100% ローカル**: 全処理を PC 上で完結—外部サーバー不要。

**対応言語（30 以上）**: C, C++, C#, Go, Java, JavaScript, TypeScript, Python, Rust, Ruby, Bash, Kotlin, Swift, PHP, Dart, Elixir, Haskell, Lua, R, Zig, SQL, HTML, CSS, YAML, Scala など。

---

## 必要な環境

- **OS**: Windows, macOS, Linux
- **VSCode**: 1.85 以上
- **Rust**（開発のみ）: 1.70+
- **Node.js**（開発のみ）: 18+

---

## インストール

### VSCode マーケットプレイスからインストール

1. VSCode を開く
2. **拡張機能** (`Ctrl+Shift+X`) を開く
3. **"comP - Code Context Engine"** で検索
4. **インストール** をクリック

### GitHub からインストール（開発向け）

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

1. comP 拡張機能を **インストール**
2. VSCode で **フォルダを開く**（Git リポジトリなど）
3. デーモンの起動:
   - **初めてのワークスペースの場合**: アクティビティバー（左端）の **comP アイコン** をクリックしてサイドバーを開き、**「Start comP」ボタン** をクリックして手動でインデックス処理を開始します。
   - **2回目以降（`.comp/` ディレクトリが存在する場合）**: フォルダを開くと、バックグラウンドで **自動的にインデックス処理が開始** されます。
4. **ステータスバー**（下）でインデックス進捗を確認
5. セットアップコマンドを実行：

```text
Ctrl+Shift+P → "comP: Setup Agents"
```

AI エージェントを選択すると、comP が `.comp/config/` に設定ファイルを生成します。以下のエージェント別手順に従って有効化してください。

---

## エージェント別セットアップ手順

### Claude Code（CLI）

「comP: Setup Agents」を実行すると `.comp/config/claude_desktop_config.json` が生成されます。このファイルに `command`（デーモンのパス）と `COMP_WORKSPACE_ROOT` が記載されているので、その値を使ってターミナルで登録します：

**Windows (PowerShell)**:

```powershell
# .comp/config/claude_desktop_config.json の "command" と "COMP_WORKSPACE_ROOT" の値に置き換えてください
claude mcp add comp "C:\Users\あなた\AppData\Local\...\comp-daemon.exe" -e COMP_WORKSPACE_ROOT="e:\your\project" -e RUST_LOG=info
```

**macOS / Linux**:

```bash
# .comp/config/claude_desktop_config.json の値に置き換えてください
claude mcp add comp "/path/to/comp-daemon" -e COMP_WORKSPACE_ROOT="/your/project" -e RUST_LOG=info
```

> **パスの確認方法**: `.comp/config/claude_desktop_config.json` を開くと `"command"` キーに正確なパスが記載されています。

登録後は `claude mcp list` で確認できます。

> **macOS/Linux**: `daemon/target/release/comp-daemon`  
> **Windows**: `daemon\target\release\comp-daemon.exe`

---

### Cursor

生成されるファイル: `.comp/config/cursor_config.json`

`mcpServers` ブロックを以下にマージしてください：

- **グローバル**（全プロジェクト共通）: `~/.cursor/mcp.json`
- **プロジェクト限定**: ワークスペースルートの `.cursor/mcp.json`

保存後、Cursor を再起動してください。

---

### Cline（VSCode 拡張）

生成されるファイル: `.comp/config/cline_config.json`

1. VSCode 設定を開く（`Ctrl+,`）
2. `Cline › MCP Servers` を検索
3. **settings.json で編集** をクリックし、`mcpServers.comp` ブロックを貼り付け

または Cline パネル → **MCP Servers** タブ → **Add Server** から JSON を貼り付け。

---

### Windsurf

生成されるファイル: `.comp/config/windsurf_config.json`

内容を以下にマージしてください：

```text
~/.codeium/windsurf/mcp_config.json
```

保存後、Windsurf を再起動してください。

---

### GitHub Copilot（VSCode）

comP がワークスペースの `.vscode/mcp.json` に直接書き込みます。**追加手順は不要**です。ワークスペースを開き直すと自動で有効になります。

---

### Antigravity

comP が `~/.gemini/antigravity-ide/mcp_config.json` に直接書き込みます。**追加手順は不要**です。Antigravity IDE を再起動してください。

---

### Continue.dev

生成されるファイル: `.comp/config/continue_config.py`

`mcp_servers` ブロックを Continue の設定ファイルに追加してください：

```text
~/.continue/config.py
```

---

## エージェント互換性

| エージェント | 状況 | 備考 |
| --- | --- | --- |
| **Claude Code** | ✅ 対応 & 確認済み | 開発者が検証済み |
| **GitHub Copilot** | ✅ 対応 & 確認済み | 開発者が検証済み |
| **Antigravity** | ✅ 対応 & 確認済み | Antigravity IDE の標準エージェント |
| **Cursor** | ✅ 対応済み | MCP 2024-11-05 準拠 |
| **Cline** | ✅ 対応済み | MCP 2024-11-05 準拠 |
| **Windsurf** | ✅ 対応済み | MCP 2024-11-05 準拠 |
| **Gemini** | ❌ 非対応 | エージェント側の MCP クライアント機能なし |

MCP 2024-11-05 準拠のクライアントであれば動作します。お使いのエージェントで特別な設定が必要な場合は [Issue を作成](https://github.com/tsucky230/comP/issues/new) してください。

---

## 使い方

### コマンドパレット

**Ctrl+Shift+P** でアクセス可能：

| コマンド | 説明 |
| --- | --- |
| **comP: Setup Agents** | Claude Code・Cursor・Antigravity などの AI エージェントを設定 |
| **comP: Force Re-index** | コードベース全体を再インデックス |
| **comP: Generate Context Capsule** | 現在のタスク向け最適コンテキストを抽出 |
| **comP: Show Impact Graph** | カーソル位置のシンボル影響範囲を表示 |

### ステータスバー

VSCode 下部に表示：

```text
◈ comP: 12,534 nodes | ✓ Ready
```

クリックで **統計ダッシュボード** を開く：

- インデックス進捗
- ノード・エッジ数
- トークン削減率（例: 60% 削減）
- セッションおよび累積のトークン統計（送信数、削減数、削減効率 %）

### AI エージェント統合

設定後、Claude Code などの AI エージェントが comP ツールを呼び出し可能：

```markdown
# Claude Code のチャットで:
@comP run_pipeline
authenticate() 関数を変更した場合の影響を分析してください
```

---

## 設定

comP の設定はプロジェクトルートの `.comp/` フォルダに保存されます（デフォルトで `.gitignore` 対象）。

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
  "on_limit_exceeded": "warn"
}
```

| オプション | 値 | デフォルト | 説明 |
| --- | --- | --- | --- |
| `max_nodes` | 整数 | `200000` | ノード数がこの値を超えたときの閾値 |
| `on_limit_exceeded` | `"warn"` \| `"stop"` | `"warn"` | `warn`: 通知して続行 · `stop`: インデックス停止 |

> **データベースサイズの目安** (`.comp/index.db`): 小規模（~1k ファイル）で 1〜5 MB、中規模（~10k ファイル）で 20〜80 MB、大規模（100k+ ファイル）で 200 MB〜1 GB。シンボルのメタデータのみ保存—生コンテンツは含みません。
>
> **モノレポの場合**: 作業中のサブフォルダだけを VSCode で開いてください。comP は開いているワークスペースフォルダのみをインデックスします。

---

## 仕組み

### アーキテクチャ

1. **インデックサー（Rust デーモン）**: ワークスペースをスキャン、tree-sitter でパース、SQLite に保存
2. **検索エンジン**: セマンティック検索 + グラフトラバーサル + BM25検索（補完）で関連コードを検出
3. **MCP サーバー**: Model Context Protocol 経由で AI エージェントにツール提供
4. **VSCode 拡張**: デーモン管理、UI 表示、ユーザーコマンド処理

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

### 「comP がインデックスしない」

- **ステータスバー**（下）で進捗を確認。
- 止まっている場合: **Ctrl+Shift+P** → "comP: Force Re-index"
- `.comp/` フォルダが存在し `.gitignore` に登録されているか確認。

### 「MCP 接続に失敗した」

- **"comP: Setup Agents"** を再実行。
- 使用中のエージェントに `.comp/mcp-config.json` があるか確認。
- VSCode **出力パネル**（表示 → 出力 → "comP"）でエラー確認。

### 「インデックスが遅い」

- 大規模リポジトリ（>100k ファイル）は初回が時間かかる。
- その後は増分処理（高速）。
- システムリソース確認: comP は <500MB RAM。

### 「一部言語が認識されない」

- comP は 30+ 言語をサポート。
- **対応外ファイル** は無視される（インデックス対象外）。
- Word（.docx）は v2 で対応予定。

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

**MIT ライセンス** — [LICENSE](LICENSE) 参照

---

## ロードマップ

| バージョン | 機能 | 状況 |
| --- | --- | --- |
| **v0.1** | コアインデックス、基本 MCP、30 言語、JSON/XML/Markdown、非同期インデックス、トークン統計。 | ✅ **リリース済み** |
| **v0.2** | Word (.docx)、PowerPoint (.pptx)、Excel (.xlsx) の自動インデックス、BM25 全文検索。 | ✅ **リリース済み** |
| **v0.3** | PDF (.pdf) サポート、高度な影響分析（`max_depth`）、TF-IDF 検索を `run_pipeline` に接続、マルチパスインデックス、`get_symbol` AST 圧縮 | ✅ **リリース済み** |
| **v0.4** | `run_pipeline` コンテンツモード（`include_content`/`compression_level`）、PR レビュー用 `get_git_diff_context` ツール、言語分布対応 `get_project_overview` 拡張 | ✅ **リリース済み** |
| **v1.0** | API 安定化、エージェント拡大、コミュニティ統合 | ⚪ 計画中 |

---

## リンク

- **GitHub**: <https://github.com/tsucky230/comP>
- **Issue**: <https://github.com/tsucky230/comP/issues>
- **Discussions**: <https://github.com/tsucky230/comP/discussions>
- **アーキテクチャドキュメント**: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)
- **API リファレンス**: [docs/API.md](docs/API.md)

---

## このプロジェクトへの支援

comP は完全無料・オープンソースです。このプロジェクトが役に立つなら、開発を支援していただけませんか？

- ☕ **[GitHub スポンサー](https://github.com/sponsors/tsucky230)** — 開発を応援（近日公開）
- 💖 **このリポジトリに Star をつける** — 他の人に知らせてください

皆さんのご支援が、開発・保守・新機能追加を支えています。ありがとうございます！🙏

---

## 質問・お問い合わせ

- 📖 **ドキュメント**: [docs/](docs/)
- 🐛 **バグ報告**: [Issue を作成](https://github.com/tsucky230/comP/issues/new)
- 💬 **フィードバック**: [Discussion を開始](https://github.com/tsucky230/comP/discussions/new)
- 👥 **参加したい**: [CONTRIBUTING.md](CONTRIBUTING.md)
