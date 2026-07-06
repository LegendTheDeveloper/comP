<p align="center">
  <img src="resources/comp-icon.png" width="128" height="128" alt="comP Logo">
</p>

# comP — AIコーディングアシスタントの「記憶」になるシステム

**オープンソースで完全ローカル動作するコード分析エンジン。Claude Code・Cursor・Cline・Antigravity対応。**

🌐 **[公式ウェブサイト](https://tsucky230.github.io/comP/)**

---

## なぜ comP が必要なのか

### 従来の AI コーディング支援の課題

Claude Code・Cursor・Cline 等の AI コーディングアシスタントは強力ですが、**重大な制限がある**：

**プロジェクト内で質問するたびに、AI は何をするのか？**

1. あなたが「この関数の動作を確認して」と質問する
2. AI は、その関数を理解するために **プロジェクト全体のファイルを読む**
3. データベース接続、設定ファイル、型定義、依存関係…関連するすべてを読む
4. ようやく答える

→ **この「プロジェクト全体を毎回読む」という処理が 3 つの問題を生む**：

| 問題 | 影響 |
| --- | --- |
| **トークン消費が膨大** | $0.10/質問 のコスト。質問 10 回で $1。LLM API の使用料が急増 |
| **応答が遅い** | 5,000 トークン読み込んでから回答。初回応答に 15 秒かかることも |
| **セッション終了で文脈消失** | ウィンドウを閉じたり再起動すると、過去の決定・議論がすべて消える。次のセッションで同じ説明を何度も繰り返す |

**実例：**

```
セッション 1（月曜）:
  Q: JWT トークンの有効期限は何時間？
  A: 現在は 1 時間です（プロジェクト全体を読んで回答）
  
セッション 2（火曜、同じ PC・同じリポジトリなのに）:
  Q: このエラーは JWT の期限が原因？
  A: ...JWT 期限についての基本から説明します...
  → 月曜の会話が **完全に消えている** 👈
```

---

## comP はその問題を解決する

**プロジェクトの「構成図 + インデックス」を自動生成**し、AIが「このファイルはこういう用途でこんな内容」を**一瞬で理解**できる形に変換します。

```
comP導入後：
  質問 → comP が最小限のコンテキスト提供 → AI が高速に答える
  質問 → comP が関連ファイルだけを抽出 → AI が即座に答える
  
効果：$0.006/質問（94% 削減）、セッション横断で履歴を自動復元
```

---

## 実際に何が変わるか

| 項目 | 従来 | comP導入後 |
| --- | --- | --- |
| **入力トークン** | 5,000 tokens/質問 | 300 tokens/質問 |
| **コスト** | $0.10/質問 | $0.006/質問 |
| **初回応答時間** | 15秒 | 3秒 |
| **セッション記憶** | LLM標準機能のみ | 自動永続化・検索可能 |
| **データ外部送信** | クラウドに全送信 | 完全ローカル（社員情報も安全） |

---

## インストール・セットアップ（3ステップ）

### 1. VS Code にインストール

1. VS Code を開く
2. **拡張機能** (`Ctrl+Shift+X`) で **"comP - Code Context Engine"** を検索
3. **インストール** をクリック

### 2. フォルダを開く

VS Code で作業するプロジェクトを開いてください（Git リポジトリ等）。

### 3. comP を起動

- アクティビティバー（左端）の **comP アイコン** をクリック
- サイドバーの **「▶ Start」** ボタンをクリック
- インデックス処理がバックグラウンドで開始（エディターは使用可能）
- **ステータスバー**（VS Code下部）で進捗確認

```text
◈ comP: 12,534 symbols | ✓ Ready
```

---

## AI エージェント別の接続（最初の1回だけ）

comP をインストール後、以下のコマンドで対応エージェント用の設定ファイルが自動生成されます：

```text
Ctrl+Shift+P → "comP: Setup Agents"
```

エージェントを選択すると、接続方法が表示されます：

| エージェント | やること |
| --- | --- |
| **Claude Code** | 生成された `claude mcp add` コマンドを端末にコピー&実行 |
| **GitHub Copilot** | `.vscode/mcp.json` に自動書き込み（何もしなくてOK） |
| **Cursor** | 生成された `cursor_config.json` を `~/.cursor/mcp.json` にコピー |
| **Cline** | Cline の設定ペイン → MCP → comP の設定をペースト |
| **Antigravity** | 設定ファイルに自動書き込み |
| **Aider** | `.aider.conf.yml` に自動書き込み |
| **Windsurf** | 生成された設定を `~/.codeium/windsurf/mcp_config.json` にコピー |
| **Continue.dev** | 生成された設定を `~/.continue/config.py` に追加 |

詳細: [docs/user/MCP_SETUP.md](docs/user/MCP_SETUP.md)

---

## 実際の使い方

### Claude Code（最も簡単）

```markdown
# Claude Code のチャットで：
@comP run_pipeline
authenticate() 関数を変更した場合、影響を受ける関数を全て分析してください
```

comP が自動的に関連ファイルのみを検索・抽出し、Claude が最小限のコンテキストで回答します。

### VS Code のネイティブチャット（@comp）

```markdown
# Copilot Chat で：
@comp #file:src/main.rs を使って、この関数の動作を説明して
```

添付ファイルは comP で自動的に圧縮（コメント削除やスケルトン化）されてから LLM に渡されるため、トークンを無駄にしません。

### 利用可能なコマンド

**Ctrl+Shift+P** で以下が実行可能：

| コマンド | 説明 |
| --- | --- |
| **comP: Setup Agents** | AI エージェント（Claude Code・Cursor等）を接続設定 |
| **comP: Force Re-index** | プロジェクト全体を再スキャン（ファイル追加時など） |
| **comP: Show Impact Graph** | カーソル位置のシンボル変更時の影響範囲を表示 |
| **comP: Copy Active File Compressed** | 開いているファイルをトークン削減形式でクリップボードにコピー |
| **comP: Export Debug Log** | セッション履歴を JSON でエクスポート |

### ステータスバー（VS Code 下部）

```text
◈ comP: 12,534 nodes | ✓ Ready | 60% saved
```

クリックすると統計ダッシュボードが開きます：

- **インデックス進捗**: ファイル数、シンボル数、関係（エッジ）数
- **トークン削減率**: このセッションで削減したトークン数
- **最終実行時刻**: 最後に AI エージェント（Claude Code・Cursor等）が comP を使った時刻

---

## ファイル・ディレクトリの除外設定

大規模プロジェクトで、一部ファイルをインデックス対象から外したい場合：

### `.comp/ignore` ファイルを作成

プロジェクトルートに `.comp/ignore` を作成（`.gitignore` と同じ書き方）：

```gitignore
node_modules/
vendor/
dist/
build/
target/
__pycache__/
*.min.js
```

自動で除外されるディレクトリ：

- `.venv`, `.pytest_cache` など `.` で始まる隠しディレクトリ
- `.gitignore` に一致するパス
- `node_modules`, `venv`, `__pycache__`, `coverage`, `vendor`, `out`

### VS Code の設定から除外

`.comp/ignore` を編集したくない場合、VS Code の設定に追加：

```json
{
  "comp.exclude": ["env", "data", "logs"]
}
```

次回の **Force Re-index** で反映されます。

---

## トークン予算・圧縮レベルの制御

大規模リポジトリの場合、`.comp/config.json` を編集してインデックス動作をカスタマイズできます：

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

| オプション | 説明 |
| --- | --- |
| `max_nodes` | インデックスノード数の上限（超えると警告 or 停止） |
| `on_limit_exceeded` | `"warn"` = 通知して継続 / `"stop"` = 停止 |
| `default_budget_tokens` | `run_pipeline` のトークン予算（自動圧縮レベル選択） |
| `compression_rules` | ファイルタイプ別の圧縮レベル強制（0=フル / 1=コンパクト / 2=スケルトン） |

> **データベースサイズの目安** (`.comp/index.db`): 小規模（~1kファイル）で1-5 MB、中規模（~10kファイル）で20-80 MB、大規模（100k+ファイル）で200 MB～1 GB。シンボルのメタデータのみ保存します。

---

## どうやって動いているのか（技術詳細）

### アーキテクチャ概要

1. **インデックサー（Rust デーモン）**
   - ワークスペースのファイルをスキャン
   - 30+言語対応の構文解析器で関数・変数・型定義等を抽出
   - SQLite に符号化して保存

2. **検索エンジン**
   - BM25（全文検索）+グラフトラバーサル（依存関係追跡）+セマンティックスコアリング
   - クエリを受けて、関連ファイル・シンボルをランク付けして返す

3. **MCP サーバー**
   - Model Context Protocol（業界標準）を通じて AI エージェントに利用可能
   - `run_pipeline`, `get_context`, `get_impact_graph` 等のツールを提供

4. **VS Code 拡張**
   - デーモン起動・停止
   - UI / ダッシュボード表示
   - ユーザーコマンド処理

### データフロー

```text
コードファイル（30+言語）
       ↓
   [構文解析]（tree-sitter）
       ↓
SQLite グラフ DB（.comp/index.db）
       ↓
   [検索エンジン]（BM25 + グラフ）
       ↓
  [MCP サーバー]
       ↓
AI エージェント（Claude Code・Cursor・Cline等）
       ↓
  [コンテキスト圧縮]
       ↓
LLM API
```

### 対応言語（30+）

C, C++, C#, Go, Java, JavaScript, TypeScript, Python, Rust, Ruby, Bash, Kotlin, Swift, PHP, Dart, Elixir, Haskell, Lua, R, Zig, SQL, HTML, CSS, YAML, Scala等。

---

## セキュリティ・プライバシー

- **🔐 完全ローカル実行**: コードとセッション履歴は一切クラウドに送信されません
- **🛡️ 自動除外**: `.comp/` ディレクトリは `.gitignore` に自動登録
- **📋 監査可能**: 全処理がマシン内で完結—外部 API 不要、テレメトリなし
- **🏢 エンタープライズ対応**: 機密情報を含むコードでも安心—オンプレミスで完全に隔離

---

## 必要な環境（開発者向け）

- **OS**: Windows, macOS, Linux
- **VS Code**: 1.85以上
- **Rust**（開発のみ）: 1.70+
- **Node.js**（開発のみ）: 18+

---

## 開発環境のセットアップ

```bash
# リポジトリをクローン
git clone https://github.com/tsucky230/comP.git
cd comP

# 依存関係インストール
npm install

# 拡張機能をビルド
npm run compile

# Rust デーモンをビルド
npm run daemon:build

# F5 キーでテスト実行（VS Code で拡張開発モードを起動）
```

### テスト実行

```bash
# Rust テスト
cargo test --all --manifest-path daemon/Cargo.toml

# TypeScript テスト
npm test

# ウォッチモード
npm run watch
npm run daemon:build -- --watch  # 別ターミナル

# Markdown Lint
npm run lint:md:fix
```

---

## トラブルシューティング

### 「comP がインデックスしない」

1. **ステータスバー**（VS Code 下部）で進捗確認
2. 進捗が止まっている場合 → **Ctrl+Shift+P** → **"comP: Force Re-index"**
3. `.comp/` ディレクトリが存在し `.gitignore` に登録されているか確認

### 「MCP 接続に失敗した」

1. **Ctrl+Shift+P** → **"comP: Setup Agents"** を再実行
2. 使用中のエージェント設定ファイルが正しく生成されているか確認
3. VS Code **出力パネル**（表示 → 出力 → "comP"）でエラーログを確認

### 「インデックスが遅い」

- 大規模リポジトリ（>100k ファイル）は初回インデックスに時間がかかります
- その後は差分インデックス（高速）で処理されます
- システムリソース確認：comP デーモンは通常 <500MB RAM を使用

### 「一部ファイルが認識されない」

- comP は 30+ 言語に対応。対応外のファイル形式は自動的にスキップされます
- Word / PowerPoint / Excel 等の Office ドキュメントと PDF は対応済み（v0.2〜v0.3 で追加）

---

## セッション履歴・永続メモリ（v0.9+の新機能）

comP の最大の特徴：**セッション終了後も文脈を自動復元**（LLM標準機能では不可能）

### 自動保存される情報

- 全チャット履歴（Claude Code・Cursor との会話）
- タスク決定・実装内容
- デーモン再起動後も即座に復帰

### 使用方法

Claude Code のチャットで：

```markdown
@comP session_recall
過去3ヶ月間に JWT 認証について何を決めたのか確認して
```

→ comP が過去の全セッションから関連する会話を検索・抽出して提示

### 仕組み

1. **自動記録**: チャット終了時に全対話を BM25 インデックス化
2. **永続化**: `~/.claude/projects/comP/memory/session/` に保存
3. **検索**: キーワード検索で即座に関連会話を復元

---

## 出力例

### "Generate Context Capsule" 実行時

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

## エージェント互換性

| エージェント | 状況 | 対応 MCP バージョン |
| --- | --- | --- |
| **Claude Code** | ✅ 対応・確認済み | 2024-11-05 |
| **GitHub Copilot** | ✅ 対応・確認済み | 2024-11-05 |
| **Cursor** | ✅ 対応 | 2024-11-05 |
| **Cline** | ✅ 対応 | 2024-11-05 |
| **Windsurf** | ✅ 対応 | 2024-11-05 |
| **Antigravity** | ✅ 対応 | 2024-11-05 |
| **Aider** | ✅ 対応 | 2024-11-05 |
| **Continue.dev** | ✅ 対応 | 2024-11-05 |
| **Gemini** | ❌ 非対応 | — |

MCP 2024-11-05 準拠のクライアントであれば原則対応します。お使いのエージェントで特別な設定が必要な場合は [Issue を作成](https://github.com/tsucky230/comP/issues/new) してください。

---

## 貢献

バグ報告・機能提案・コード貢献を歓迎します。詳細は [CONTRIBUTING.md](CONTRIBUTING.md) をご覧ください。

---

## ライセンス

**MITライセンス** — [LICENSE](LICENSE) 参照

---

## ロードマップ

| バージョン | 機能 | 状況 |
| --- | --- | --- |
| **v0.1** | コアインデックス・基本 MCP・30 言語対応 | ✅ リリース済み |
| **v0.2** | Word / PowerPoint / Excel 対応・BM25 全文検索 | ✅ リリース済み |
| **v0.3** | PDF サポート・高度な影響分析・TF-IDF 検索 | ✅ リリース済み |
| **v0.4** | `run_pipeline` コンテンツモード・`get_git_diff_context` | ✅ リリース済み |
| **v0.5** | コード圧縮コピー・`@comp` チャット参加者・Parquet 対応 | ✅ リリース済み |
| **v0.6** | 動的トークン予算・自動圧縮レベル選択 | ✅ リリース済み |
| **v0.7** | 拡張子別圧縮ルール・Aider 対応 | ✅ リリース済み |
| **v0.8** | 大規模リポジトリ最適化・`.comp/ignore` 対応 | ✅ リリース済み |
| **v0.9** | **セッション履歴・永続メモリ・session_log / session_recall** | ✅ リリース済み |
| **v1.0** | API 安定化・コミュニティ統合 | ⚪ 計画中 |

---

## リンク

- **GitHub**: <https://github.com/tsucky230/comP>
- **Issue**: <https://github.com/tsucky230/comP/issues>
- **Discussions**: <https://github.com/tsucky230/comP/discussions>
- **アーキテクチャドキュメント**: [docs/ARCHITECTURE_ja.md](docs/ARCHITECTURE_ja.md)
- **MCP ツールリファレンス**: [docs/user/MCP_TOOLS.md](docs/user/MCP_TOOLS.md)

---

## 支援する

comP は完全無料・オープンソースです。このプロジェクトが役に立つなら、開発を支援していただけませんか？

- ☕ **[GitHub スポンサー](https://github.com/sponsors/tsucky230)** — 開発を応援
- 💖 **このリポジトリに Star をつける** — 他の人に知らせてください

皆さんのご支援が開発・保守を支えています。ありがとうございます！🙏

---

## よくある質問（FAQ）

### Q: comP を設定したのに、LLM が直にファイルを延々読み始めます

**A:** LLM（Claude Code・Cursor等）が comP を持っていても、指示が不明確だと直接ファイルを読む方が「簡単」だと判断する場合があります。以下の3つを試してください：

#### 1. プロンプトで明示的に指定

直接ファイルを読ませるのではなく、`@comP` を使って comP 経由で質問します：

```markdown
# 良い例：
@comP run_pipeline
このエラーの原因を分析してください

# 悪い例：
src/auth/middleware.ts を見て、エラーの原因を教えて
```

#### 2. エージェント側の指示を強化

comP セットアップ後、`.github/copilot-instructions.md` などに以下を追記して、LLM の優先順位を明確化します：

```markdown
## comP の使用は必須

- **コードのタスク前に必ず実行**: run_pipeline を FIRST に呼ぶ
- **ファイル直読み禁止**: grep/find/Bash を使わずに comP 経由で検索
- **理由**: comP インデックスは full codebase より 94% トークン削減
```

> 💡 Tip: `@comP setup-agents` でセットアップ時に自動生成されるガイドを確認してください

#### 3. セッション記憶を活用

前のセッションの決定を復元するため、新しいセッション開始時に `session_recall` を呼ぶ：

```markdown
# セッション開始時：
@comP session_recall
過去の JWT 認証に関する決定を復元してください
```

セッション記憶があると、LLM は「前回ここで結論が出た」と理解するため、直読みを避けやすくなります。

---

## 質問・バグ報告・フィードバック

- 📖 **ドキュメント**: [docs/](docs/)
- 🐛 **バグ報告**: [Issue を作成](https://github.com/tsucky230/comP/issues/new)
- 💬 **フィードバック**: [Discussion を開始](https://github.com/tsucky230/comP/discussions/new)
- 👥 **プロジェクト参加**: [CONTRIBUTING.md](CONTRIBUTING.md)
