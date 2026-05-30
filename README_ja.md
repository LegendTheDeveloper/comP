# comP - AI エージェント向けコンテキストエンジン

**オープンソースで完全ローカル動作するコード分析エンジン。Claude Code・Cursor・Cline 対応。**

comP は Claude Code・Cursor・Cline などの AI エージェントが、コードベースを効率的に理解・分析できるよう支援します。プロジェクトを自動でインデックスし、セマンティックなコードグラフを構築し、トークン数を算出します—全てローカルで完結します。

---

## 何ができるのか

- **📑 コードインデックス**: tree-sitter で 30+ 言語のコードを自動解析・グラフ化
- **🎯 スマートコンテキスト**: AI エージェントに最小限のコード情報を提供（トークン 60% 削減）
- **🔍 影響分析**: シンボル変更時に影響を受ける全コードを可視化
- **📊 トークンカウンター**: コンテキストが正確に何トークンか表示
- **🤝 MCP 連携**: Claude Code・Cursor・Cline など AI エージェントで即座に利用
- **100% ローカル**: 全処理を PC 上で完結—外部サーバー不要

**対応言語（30 以上）**: C, C++, C#, Go, Java, JavaScript, TypeScript, Python, Rust, Ruby, Bash, Kotlin, Swift, PHP, Dart, Elixir, Haskell, Lua, R, Zig, SQL, HTML, CSS, YAML, Scala など。

---

## 必要な環境

- **OS**: Windows, macOS, Linux
- **VSCode**: 1.85 以上
- **Rust**（開発のみ）: 1.70+
- **Node.js**（開発のみ）: 18+

---

## インストール

### VSCode マーケットプレイス（近日公開）

1. VSCode を開く
2. **拡張機能** (Ctrl+Shift+X) を開く
3. **"comP - Code Context Engine"** で検索
4. **インストール** をクリック

### GitHub（開発版）

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
3. comP が自動的に **インデックス処理を開始**
4. **ステータスバー**（下）でインデックス進捗を確認
5. comP コマンドを実行：

```bash
Ctrl+Shift+P → "comP: Setup Agents"
```

AI エージェント（Claude Code・Cursor・Cline など）を選択し、comP が自動で MCP を設定します。

---

## エージェント互換性

| エージェント | 状況 | 備考 |
| --- | --- | --- |
| **Claude Code** | ✅ 確認済み | 開発者が検証 |
| **GitHub Copilot** | ✅ 確認済み | 開発者が検証 |
| Cursor | ⚪ 動作するはず | MCP 2024-11-05 準拠・未検証 |
| Cline | ⚪ 動作するはず | MCP 2024-11-05 準拠・未検証 |
| Windsurf | ⚪ 動作するはず | MCP 2024-11-05 準拠・未検証 |
| Gemini | ❌ 非対応 | MCP 非対応（REST API のみ） |

MCP 2024-11-05 準拠のクライアントであれば動作するはずです。お使いのエージェントで設定が必要な場合は [Issue を作成](https://github.com/tsucky230/comP/issues/new) してください。

---

## 使い方

### コマンドパレット

**Ctrl+Shift+P** でアクセス可能：

| コマンド | 説明 |
| --- | --- |
| **comP: Setup Agents** | Claude Code・Cursor など AI エージェントを設定 |
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

### AI エージェント統合

設定後、Claude Code など AI エージェントが comP ツールを呼び出し可能：

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
2. **検索エンジン**: セマンティック検索 + グラフトラバーサルで関連コード検出
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
 セマンティック検索
     ↓
 MCP ツール（run_pipeline 等）
     ↓
 AI エージェント（Claude Code・Cursor・Cline）
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

- **ステータスバー**（下）で進捗を確認
- 止まっている場合: **Ctrl+Shift+P** → "comP: Force Re-index"
- `.comp/` フォルダが存在し `.gitignore` に登録されているか確認

### 「MCP 接続に失敗した」

- **"comP: Setup Agents"** を再実行
- 使用中のエージェント（Claude Code 等）に `.comp/mcp-config.json` があるか確認
- VSCode **出力パネル**（表示 → 出力 → "comP"）でエラー確認

### 「インデックスが遅い」

- 大規模リポジトリ（>100k ファイル）は初回が時間かかる
- その後は増分処理（高速）
- システムリソース確認: comP は <500MB RAM

### 「一部言語が認識されない」

- comP は 30+ 言語をサポート
- **対応外ファイル** は無視される（インデックス対象外）
- Word（.docx）は v2 で対応予定

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

| バージョン | 機能 |
| --- | --- |
| **v0.1** | コアインデックス、基本 MCP、30 言語、JSON/XML/Markdown |
| **v0.2** | Word サポート、高度な影響分析 |
| **v0.3** | 埋め込みベース検索、複数リポジトリ対応 |
| **v0.4** | プロジェクトドキュメント自動生成（インデックスから CLAUDE.md / README 生成）、Git 差分対応コンテキスト（PR レビュー・影響範囲検知） |
| **v1.0** | API 安定化、エージェント拡大、コミュニティ統合 |

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
