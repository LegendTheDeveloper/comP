# comP - AI エージェント向けコンテキストエンジン

**オープンソースで完全ローカル動作するコード分析エンジン。Claude Code・Cursor・Cline 対応。**

comP は Vexp の有料化に伴い開発された、完全無料のオープンソース代替品です。あなたのコードベースを自動でインデックスし、AI エージェントに最適なコンテキストを提供します。全てローカルで処理されます—クラウド送信なし、データ共有なし。

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
git clone https://github.com/comp-dev/comP.git
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
git clone https://github.com/comp-dev/comP.git
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
| **v1.0** | API 安定化、エージェント拡大、コミュニティ統合 |

---

## 謝辞

comP は [Vexp](https://vexp.dev) にインスパイアされており、その強力なコンテキストエンジン機能をオープンソースとしてコミュニティに提供します。

---

## リンク

- **GitHub**: <https://github.com/comp-dev/comP>
- **Issue**: <https://github.com/comp-dev/comP/issues>
- **Discussions**: <https://github.com/comp-dev/comP/discussions>
- **アーキテクチャドキュメント**: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)
- **API リファレンス**: [docs/API.md](docs/API.md)

---

## 質問・お問い合わせ

- 📖 **ドキュメント**: [docs/](docs/)
- 🐛 **バグ報告**: [Issue を作成](https://github.com/comp-dev/comP/issues/new)
- 💬 **フィードバック**: [Discussion を開始](https://github.com/comp-dev/comP/discussions/new)
- 👥 **参加したい**: [CONTRIBUTING.md](CONTRIBUTING.md)
