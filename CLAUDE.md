# comP プロジェクト憲法

## 言語

全ての会話は**日本語で実施**します。

---

## コーディング・プロセス

### 1. スケルトンコードの作成

コーディング開始時は、必ず以下の要素を含むスケルトンコードを最初に作成します：

- **クラス定義**: 目的、責務、公開インターフェース

- **関数定義**: パラメータ、戻り値、処理意図

- **分岐ロジック**: if/else, switch の各分岐の意図をコメント記述

- **変数定義**: 型アノテーション、用途、制約条件

### 2. コメント駆動設計

```text
✓ 記述すべきコメント：
  - WHY：なぜその分岐が必要なのか
  - 入出力の制約、前提条件
  - 非自明なアルゴリズムロジック
  - 境界値の扱い方

✗ 記述しないコメント：
  - 処理の説明（コード自体が説明する場合）
  - 関数・変数の一覧
  - 「used by X」「added for Y」などの責務外の記述

```

### 3. 単体テスト駆動開発（TDD）

1. スケルトン＆コメントを確認
2. **単体テスト優先作成**：コメント記述の意図をテストで検証
3. テストが通るようにコード実装
4. リファクタリング

**テスト実装の原則：**

- 正常系、異常系、境界値を必ずカバー

- 分岐ロジックの全パターンをテスト

- モック/スタブの使用は最小限（統合テストは実物データを使用）

### 4. カバレッジ要件

- **目標**: 80% 以上の行カバレッジ

- **優先度**: 制御フロー（分岐）カバレッジ > 行カバレッジ

- ビジネスロジックの境界値は 100% に近づける

---

## 成果物の完全性保証

コード修正時は、以下 **すべて** を更新します：

### 必須更新項目

| 対象 | 確認項目 |
| --- | --- |
| **コード** | 実装そのもの、型安全性、パフォーマンス |
| **テスト** | 既存テストの有効性確認、新規テスト追加 |
| **ドキュメント** | API仕様、使用例、パラメータ説明 |
| **スケルトン＆コメント** | 設計との一貫性確認 |
| **関連ファイル** | 参照元、依存ファイルの整合性 |

**❌ 禁止事項：**

- コードだけの修正で終わる

- テストを更新しずに仕様変更

- ドキュメントが古いまま残す

- スケルトンコメントと実装の乖離

---

## ファイル・ディレクトリ構成

```plaintext
e:\dev\comP\
├── CLAUDE.md                    # このファイル（プロジェクト憲法）
├── src\                         # 実装コード
│   └── [モジュール名]\
│       ├── [機能].rs/ts/etc
│       └── tests\
│           └── [機能]_test.rs/ts
├── docs\                        # ドキュメント
│   ├── API.md
│   ├── ARCHITECTURE.md
│   └── DESIGN.md
└── coverage\                    # カバレッジレポート

```

---

## コメント記述テンプレート

### 関数/メソッド

```rust
// [機能名]
//
// # 入力
// - param1: 説明、型、制約
// - param2: 説明、型、制約
//
// # 出力
// - 戻り値の説明、型、null/error の場合の扱い
//
// # 前提条件
// - [前提条件があれば記述]
fn function_name(param1: Type, param2: Type) -> Result<T, E> {
    // [実装]
}

```

### 分岐ロジック

```rust
// [分岐の意図]: なぜこの条件判定が必要か
if condition {
    // [この分岐で何をするのか、なぜか]
} else {
    // [else 分岐の意図]
}

```

### 複雑なアルゴリズム

```rust
// [アルゴリズムの概要と境界値の扱い]
// 例：「バイナリサーチを使用。left=0, right=n-1 は境界値なので注意」

```

---

## チェックリスト（実装完了前に確認）

- [ ] スケルトンコード作成＆全コメント記述済み

- [ ] 単体テスト（正常系、異常系、境界値）実装完了

- [ ] テスト実行、全テスト 🟢

- [ ] カバレッジ 80% 以上確認

- [ ] コード実装完了

- [ ] 既存テスト影響確認（他のテストが壊れていないか）

- [ ] ドキュメント更新（API、使用例、制約条件）

- [ ] 関連ファイル確認（参照元の更新）

- [ ] スケルトンコメント ↔ 実装 の一貫性確認

- [ ] コードレビュー用の PR メッセージ作成

---

## コンテキスト永続化・トークン節約戦略

### 中間ファイル管理（コンテキスト枯渇対策）

コンテキストが枯渇するリスクに備え、以下を **必ず実施**します：

#### 作業中間ファイル配置

```plaintext
temp/
├── log/                          # 会話ログ・チェックポイント
│   ├── session_YYYYMMDD_HHmm.md  # セッション生ログ
│   ├── context_checkpoint.json   # コンテキストスナップショット
│   └── token_usage.txt           # トークン使用統計
├── skeleton/                      # スケルトン・コメント済みコード
│   └── [機能名]_skeleton.rs/ts
├── tests/                         # テスト中間ファイル
│   └── [機能名]_test_draft.rs/ts
├── design/                        # 設計ドキュメント草稿
│   └── [機能名]_design.md
└── checklist/                     # チェックリスト進捗
    └── [機能名]_checklist.md

```

#### セッション保持の実装

1. **タスク開始時**：スケルトン、テスト骨格を `temp/skeleton/` に保存
2. **進行中**：チェックリスト、設計ノートを `temp/design/` に逐次保存
3. **セッション切れ時**：最新状態を `temp/context_checkpoint.json` に集約
4. **セッション復帰時**：チェックポイントから即座に復帰

#### コンテキストログ保持

- 全ての会話を `temp/log/session_*.md` に記録
  - 質問、決定事項、修正履歴、失敗パターン

- セッション終了時、本ログを適切に削除（ユーザー指示まで保持）

- 重要な決定は `CLAUDE.md` 修正ログに反映

### トークン節約戦略

#### 節約基本原則

- **重複説明の排除**：一度説明したコンセプトは `temp/design/` への参照で代替

- **スケルトンコード先行**：実装コード全文読込の前に設計・テストで意図確認

- **ファイルグループ化**：関連ファイルは一度に Read（複数回の Read は避ける）

- **修正差分保持**：全ファイル再読込でなく、Edit での差分記録を活用

#### 大規模タスクの分割

- **タスク分割基準**：1セッション ~15K トークン 以下に収める

- **チェックポイント**：各タスク完了時に `temp/checklist/` で進捗記録

- **セッション継続**：新規セッションで `context_checkpoint.json` から復帰

#### RTK活用（トークン圧縮）

- `rtk gain` で使用統計を定期確認

- `rtk discover` で未活用の最適化機会を探索

- 頻出コマンドの自動化を hooks 経由で設定

---

## Markdown品質・Lint要件

### Markdown Lint実行

生成する **すべての Markdown ファイル** に対して、以下を実施します：

- **Lint ツール**: `markdownlint` (Node.js) または `mdl`

- **実行タイミング**: 生成直後、コミット前に必ず実行

- **許可される Warning**:
  - ドメイン固有の慣例（プロジェクト特有の書き方）

- **禁止される Warning**:
  - 不正な見出しレベル（H1の複数使用）
  - テーブル不整形
  - 空行不足
  - リスト形式エラー

**チェックコマンド例**:

```bash
npx markdownlint "**/*.md"
mdl docs/

```

---

## Python環境管理（uv 必須）

Python を使用する場合は、以下 **必須**:

### 仮想環境構成

```bash
uv venv                    # 仮想環境作成（.venv）
uv pip install -r requirements.txt  # 依存関係インストール

```

### 依存ファイル

- `pyproject.toml` : プロジェクト設定（推奨）

- `requirements.txt` : インストール対象パッケージ（自動生成）

**Why**: `uv` は従来の `pip` + `venv` より高速で再現性が高い

**チェック項目**:

- [ ] `uv` がインストール済み

- [ ] `.venv` が `.gitignore` に記述済み

- [ ] `requirements.txt` が Git で管理

- [ ] 本体ドキュメントに `uv venv` 起動手順記載

---

## OSS作法・メタデータファイル生成

### 必須ファイル

以下を **プロジェクト初期化時に必ず生成** します：

| ファイル | 説明 | 参考 |
| --- | --- | --- |
| `README.md` | 英語、初心者向け詳細ガイド | [Keep a Changelog](https://keepachangelog.com/) |
| `README_ja.md` | 同内容の日本語版 | 同上 |
| `LICENSE` | ライセンス宣言（推奨: MIT/Apache2.0） | [SPDX](https://spdx.org/licenses/) |
| `CHANGELOG.md` | 変更履歴（バージョン管理） | [Semantic Versioning](https://semver.org/) |
| `SBOM.json` / `SBOM.xml` | ソフトウェア部品表（CycloneDX形式） | [CycloneDX](https://cyclonedx.org/) |
| `.gitignore` | Git 除外ルール | GitHub template |
| `CONTRIBUTING.md` | 貢献ガイド | Open Source |

### README.md（英語）の構成

以下の構造で **初心者でも理解できる** ドキュメントを作成：

```markdown

# [プロジェクト名]

## What It Does（何が出来るのか）

- 3行以内で機能を説明

- 具体的なユースケース例

- デモ画像/GIF があると効果的

## Prerequisites（必要な環境）

- OS（Windows/Mac/Linux）

- 言語バージョン（Python 3.9+, Node.js 18+ など）

## Installation（インストール方法）

- ステップバイステップガイド

- 実際に動くコマンド例

- よくあるエラー対応

## Quick Start（クイックスタート）

- 実際に動く最小限の例

- 5分以内で結果を確認できる内容

## Usage（使用方法）

- よくあるユースケース 3～5個

- コマンド例 + 実行結果

- 設定ファイルサンプル（あれば）

## Output（効果・得られる結果）

- 実行後、何が得られるのか

- パフォーマンス数値（あれば）

## Troubleshooting（トラブル対応）

- よくあるエラーと解決方法

- FAQ

## License

- SPDX identifier（MIT など）

- 著作権表示

## Contributing

- `CONTRIBUTING.md` へのリンク

```

### README_ja.md（日本語）

上記と同じ構成で、日本語版を作成：

```markdown

# [プロジェクト名]

## できること

## 必要な環境

## インストール

## クイックスタート

## 使い方

## 効果

## トラブル対応

## ライセンス

## 貢献方法

```

### SBOM（ソフトウェア部品表）生成

Python の場合：

```bash
pip install cyclonedx-bom
cyclonedx-py -o SBOM.json requirements.txt

```

Node.js の場合：

```bash
npm install -g @cyclonedx/npm
cyclonedx-npm -o SBOM.json

```

---

## CI/CD・セキュリティテスト要件

### PUSH時の必須チェック

全てのプッシュ（Pull Request を含む）で以下を自動実行します：

#### 1. 単体テスト

```bash

# Python の場合
pytest --cov=src --cov-report=term-missing --cov-report=html

# Node.js の場合
npm test -- --coverage

# Rust の場合
cargo test --all

```

要件:

- カバレッジ 80% 以上（[[coding-standards-comp]] 参照）

- テスト失敗時は PUSH 不可

- カバレッジレポートを成果物として保持

#### 2. SAST（静的セキュリティテスト）

実装基準:

**Python**: `bandit` + `semgrep`

```bash
bandit -r src/
semgrep --config p/security-audit src/

```

**Node.js**: `npm audit` + `semgrep`

```bash
npm audit --production
semgrep --config p/security-audit src/

```

**Rust**: `cargo-audit`

```bash
cargo audit

```

脆弱性レベル:

- Critical/High: PUSH ブロック（修正まで PUSH 不可）

- Medium: 警告のみ（記録して進行可）

#### 3. DAST（動的セキュリティテスト）

サーバー起動が必要な場合のみ実装：

```bash

# 簡易DAST：基本的なHTTP セキュリティヘッダ確認
curl -I https://localhost:8000 | grep -i "strict-transport-security\|x-frame-options\|content-security-policy"

```

確認項目:

- HTTPS の使用（本番環境）

- セキュリティヘッダ設定（CSP, HSTS, X-Frame-Options）

- 認証/認可の動作確認（あれば）

### GitHub Actions ワークフロー設定

`.github/workflows/ci.yml` 雛形：

```yaml
name: CI/CD Pipeline

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main, develop]
  release:
    types: [published]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      # テスト実行
      - name: Run Tests
        run: |
          # Python の場合の例
          pip install -r requirements.txt pytest pytest-cov
          pytest --cov=src --cov-report=html

      # SAST: 依存関係チェック
      - name: Dependency Security Check
        run: |
          pip install bandit semgrep
          bandit -r src/ || true
          semgrep --config p/security-audit src/ || true

      # テスト結果アップロード
      - name: Upload Coverage Reports
        uses: codecov/codecov-action@v3
        with:
          files: ./htmlcov/index.html

  sbom:
    needs: test
    runs-on: ubuntu-latest
    # タグ付けリリース時のみ実行
    if: startsWith(github.ref, 'refs/tags/')
    steps:
      - uses: actions/checkout@v4

      # SBOM 生成
      - name: Generate SBOM
        run: |
          # Python の場合
          pip install cyclonedx-bom
          pip install -r requirements.txt
          cyclonedx-py -o SBOM.json requirements.txt

          # または Node.js の場合
          # npm install -g @cyclonedx/npm
          # cyclonedx-npm -o SBOM.json

      # SBOM を GitHub Release にアップロード
      - name: Upload SBOM to Release
        uses: softprops/action-gh-release@v1
        with:
          files: SBOM.json
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}

      # SBOM をリポジトリにコミット（オプション）
      - name: Commit SBOM
        run: |
          git config user.name "github-actions[bot]"
          git config user.email "github-actions[bot]@users.noreply.github.com"
          git add SBOM.json
          git commit -m "docs: update SBOM for ${{ github.ref }}" || true
          git push

```

### ローカル検証コマンド

PUSH 前にローカルで実行するコマンド集：

```bash

# テスト + カバレッジ確認
pytest --cov=src --cov-report=term-missing

# SAST チェック（Python）
bandit -r src/
semgrep --config p/security-audit src/

# SBOM 生成（開発時）
cyclonedx-py -o SBOM.json requirements.txt

```

チェックリスト項目:

- [ ] 単体テスト: 全テスト 🟢 カバレッジ 80% 以上

- [ ] SAST: Critical/High なし

- [ ] DAST（あれば）: セキュリティヘッダ確認

- [ ] 本番 PUSH 前: GitHub Actions 結果確認

---

## ロギング・監視の仕組み

### ログレベルと用途

全てのプロジェクトで以下のログレベルを統一採用：

| レベル | 用途 | 例 |
| --- | --- | --- |
| DEBUG | 開発時のデバッグ情報 | 変数値、関数呼び出し |
| INFO | 重要な処理フロー | 起動完了、処理開始/終了 |
| WARNING | 注意が必要な状況 | 非推奨API使用、リソース枯渇兆候 |
| ERROR | エラー発生（処理継続可） | 例外キャッチ、リトライ可能エラー |
| CRITICAL | 致命的エラー（停止推奨） | データベース接続失敗、認証失敗 |

### 言語別実装

#### Python（logging モジュール）

```python
import logging
import sys

# ロガー設定
logger = logging.getLogger(__name__)
logger.setLevel(logging.DEBUG)

# コンソールハンドラ
console_handler = logging.StreamHandler(sys.stdout)
console_handler.setLevel(logging.INFO)

# ファイルハンドラ
file_handler = logging.FileHandler('app.log')
file_handler.setLevel(logging.DEBUG)

# ログフォーマット
formatter = logging.Formatter(
    '[%(asctime)s] [%(levelname)s] [%(name)s:%(lineno)d] %(message)s',
    datefmt='%Y-%m-%d %H:%M:%S'
)
console_handler.setFormatter(formatter)
file_handler.setFormatter(formatter)

logger.addHandler(console_handler)
logger.addHandler(file_handler)

# 使用例
logger.debug('デバッグ情報')
logger.info('処理開始')
logger.warning('警告メッセージ')
logger.error('エラー発生', exc_info=True)
logger.critical('致命的エラー')

```

#### Node.js（winston）

```javascript
const winston = require('winston');

const logger = winston.createLogger({
  level: 'info',
  format: winston.format.combine(
    winston.format.timestamp({ format: 'YYYY-MM-DD HH:mm:ss' }),
    winston.format.printf(({ timestamp, level, message, ...meta }) => {
      const metaStr = Object.keys(meta).length ? JSON.stringify(meta) : '';
      return `[${timestamp}] [${level}] ${message} ${metaStr}`;
    })
  ),
  transports: [
    new winston.transports.File({ filename: 'app.log' }),
    new winston.transports.Console()
  ]
});

// 使用例
logger.info('処理開始');
logger.error('エラー発生', { error: err.message });

```

#### Rust（log + env_logger）

```rust
use log::{debug, info, warn, error};

fn main() {
    env_logger::Builder::from_default_env()
        .format_timestamp_millis()
        .init();

    info!("アプリケーション起動");
    debug!("デバッグ情報");
    warn!("警告メッセージ");
    error!("エラー発生");
}

```

### ログ出力先

- **開発環境**：コンソール + ローカルログファイル（`logs/app.log`）

- **本番環境**：ファイル出力 + 外部ログ集約サービス（Datadog, CloudWatch など）

### ロギングのベストプラクティス

- **変数値はログに含める**：「エラーが発生しました」ではなく「user_id=123 でエラー発生」

- **スタックトレース必須**：例外はスタックトレース付きでログ記録

- **機密情報は除外**：パスワード、API キー、個人情報はマスク

- **適切なレベル使用**：すべてを INFO/ERROR に集約しない

---

## 例外処理・エラーハンドリング

### 基本原則

**例外は握りつぶすな。ログに記録して、適切なレベルで上位へ伝播させる。**

### 実装パターン（言語別）

#### Python

```python
import logging

logger = logging.getLogger(__name__)

# カスタム例外の定義
class ApplicationError(Exception):
    """アプリケーション固有のエラー基底クラス"""
    pass

class ConfigurationError(ApplicationError):
    """設定ファイル読込失敗"""
    pass

class DatabaseError(ApplicationError):
    """データベース操作失敗"""
    pass

def load_config(config_path):
    """設定ファイルを読み込む

    # 入力
    - config_path: 設定ファイルパス

    # 出力
    - 設定辞書

    # 例外
    - FileNotFoundError: ファイルが見つからない
    - ConfigurationError: ファイル解析失敗
    """
    try:
        with open(config_path, 'r') as f:
            config = json.load(f)
        logger.info(f'設定ファイル読み込み完了: {config_path}')
        return config
    except FileNotFoundError as e:
        logger.error(f'設定ファイルが見つかりません: {config_path}', exc_info=True)
        raise ConfigurationError(f'設定ファイルが見つかりません: {config_path}') from e
    except json.JSONDecodeError as e:
        logger.error(f'設定ファイルの解析に失敗しました: {config_path}', exc_info=True)
        raise ConfigurationError(f'設定ファイルの解析に失敗しました: {config_path}') from e
    except Exception as e:
        logger.error(f'予期しないエラーが発生しました', exc_info=True)
        raise ApplicationError(f'設定ファイル読込に失敗しました') from e

def main():
    try:
        config = load_config('config.json')
        # 処理実行
    except ConfigurationError as e:
        logger.critical(f'致命的エラー: 設定ファイルが読み込めません')
        sys.exit(1)
    except ApplicationError as e:
        logger.error(f'アプリケーションエラー: {str(e)}')
        sys.exit(1)
    except Exception as e:
        logger.critical(f'予期しないエラーが発生しました', exc_info=True)
        sys.exit(1)
    finally:
        logger.info('処理終了')

```

#### Node.js

```javascript
const logger = require('./logger');

// カスタム例外クラス
class ApplicationError extends Error {
  constructor(message, code = 'INTERNAL_ERROR', statusCode = 500) {
    super(message);
    this.code = code;
    this.statusCode = statusCode;
  }
}

class ConfigurationError extends ApplicationError {
  constructor(message) {
    super(message, 'CONFIG_ERROR', 500);
  }
}

async function loadConfig(configPath) {
  try {
    const fs = require('fs').promises;
    const content = await fs.readFile(configPath, 'utf8');
    const config = JSON.parse(content);
    logger.info(`設定ファイル読み込み完了: ${configPath}`);
    return config;
  } catch (error) {
    if (error.code === 'ENOENT') {
      logger.error(`設定ファイルが見つかりません: ${configPath}`, { error });
      throw new ConfigurationError(`設定ファイルが見つかりません: ${configPath}`);
    } else if (error instanceof SyntaxError) {
      logger.error(`設定ファイルの解析に失敗しました: ${configPath}`, { error });
      throw new ConfigurationError(`設定ファイルの解析に失敗しました: ${configPath}`);
    } else {
      logger.error(`予期しないエラーが発生しました`, { error });
      throw new ApplicationError(`設定ファイル読込に失敗しました`);
    }
  }
}

async function main() {
  try {
    const config = await loadConfig('config.json');
    // 処理実行
  } catch (error) {
    if (error instanceof ConfigurationError) {
      logger.critical(`致命的エラー: 設定ファイルが読み込めません`);
      process.exit(1);
    } else if (error instanceof ApplicationError) {
      logger.error(`アプリケーションエラー: ${error.message}`);
      process.exit(1);
    } else {
      logger.critical(`予期しないエラーが発生しました`, { error });
      process.exit(1);
    }
  } finally {
    logger.info('処理終了');
  }
}

main();

```

#### Rust

```rust
use log::{error, info, warn};
use std::fs;

#[derive(Debug)]
enum AppError {
    ConfigNotFound(String),
    ConfigParseError(String),
    DatabaseError(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AppError::ConfigNotFound(msg) => write!(f, "設定ファイルが見つかりません: {}", msg),
            AppError::ConfigParseError(msg) => write!(f, "設定ファイルの解析に失敗: {}", msg),
            AppError::DatabaseError(msg) => write!(f, "データベースエラー: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

fn load_config(config_path: &str) -> Result<String, AppError> {
    // ファイル読込
    let content = fs::read_to_string(config_path)
        .map_err(|e| {
            error!("設定ファイルが見つかりません: {} ({})", config_path, e);
            AppError::ConfigNotFound(config_path.to_string())
        })?;

    // JSON解析
    serde_json::from_str::<serde_json::Value>(&content)
        .map_err(|e| {
            error!("設定ファイルの解析に失敗: {} ({})", config_path, e);
            AppError::ConfigParseError(e.to_string())
        })?;

    info!("設定ファイル読み込み完了: {}", config_path);
    Ok(content)
}

fn main() {
    env_logger::init();

    match load_config("config.json") {
        Ok(config) => {
            info!("処理開始");
            // 処理実行
            info!("処理終了");
        }
        Err(AppError::ConfigNotFound(_) | AppError::ConfigParseError(_)) => {
            error!("致命的エラー: 設定ファイルが読み込めません");
            std::process::exit(1);
        }
        Err(e) => {
            error!("予期しないエラーが発生しました: {}", e);
            std::process::exit(1);
        }
    }
}

```

### エラーハンドリングのベストプラクティス

- **スタックトレース必須**：`exc_info=True` または equivalent で必ず記録

- **カスタム例外定義**：ジャンル別に例外を分類（Config, Database, Network など）

- **エラーコード**：ユーザーに「何が起きたのか」を分かりやすく伝える

- **リトライ可能性を明示**：エラーオブジェクトに retryable フラグを持たせる（オプション）

- **グレースフルシャットダウン**：finally ブロックでリソース解放（ファイルクローズ、接続切断など）

### エラーハンドリングのチェックリスト

- [ ] 全ての例外を握りつぶしていないか確認

- [ ] エラー時にスタックトレースをログに記録

- [ ] カスタム例外を定義して、例外型で分岐処理

- [ ] ユーザー向け/ログ用で異なるメッセージを用意

- [ ] リソースリークが発生していないか（finally ブロック確認）

---

## Markdown Lint セットアップ（プロジェクト基本設定）

### インストール（初回のみ）

```bash
npm install --save-dev markdownlint-cli2
```

### Lint チェック実行

```bash
# チェックのみ（エラー表示）
npm run lint:md

# チェック + 自動修正
npm run lint:md:fix
```

### 設定ファイル

- `.markdownlintrc.json`：Lint ルール設定
  - MD003（見出しスタイル）：無効化
  - MD013（行長）：無効化
  - MD024（重複見出し）：siblings_only = 兄弟見出しのみチェック
  - MD034（リンク前後）：無効化

### ローカル開発環境での自動チェック

VSCode で `markdownlint` 拡張機能をインストール：

- Ctrl+Shift+X で「markdownlint」を検索
- David Anson 作の `vscode-markdownlint` をインストール
- ファイル保存時に自動チェック、問題パネルにエラー表示

### CI/CD で Lint を実行

GitHub Actions で自動チェック：

```yaml
# .github/workflows/lint.yml
name: Markdown Lint

on: [push, pull_request]

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v3
      - run: npm install --save-dev markdownlint-cli2
      - run: npm run lint:md
```

---

## スキル化の許可

本憲法に基づくプロセス効率化のため、以下のスキル化を **許可**します：

- ✅ テンプレート生成自動化（スケルトン、テスト骨格）

- ✅ カバレッジレポート生成・分析の自動化

- ✅ チェックリスト管理スキル

- ✅ ドキュメント同期検証スキル

- ✅ コンテキスト永続化・復帰スキル（中間ファイル管理）

- ✅ トークン節約分析スキル

---

## 修正・改善ログ

| 日付 | 修正内容 |
| --- | --- |
| 2026-05-21 | 初版作成 |
| 2026-05-21 | コンテキスト永続化・トークン節約戦略を追加 |
| 2026-05-21 | Markdown品質・Lint、Python(uv)、OSS作法を追加 |
| 2026-05-21 | CI/CD・セキュリティテスト（SAST/DAST、SBOM自動生成）を追加 |
| 2026-05-21 | ロギング・監視、例外処理・エラーハンドリングを追加 |
