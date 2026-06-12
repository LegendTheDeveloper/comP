# Changelog

All notable changes to comP are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/) and this project adheres to [Semantic Versioning](https://semver.org/).

---

## [0.8.1] - 2026-06-13

### Fixed

- **`.venv` 等の大量混入バグ**: ディレクトリ走査を `walkdir` から `ignore` クレートへ置き換え。`filter_entry` で除外ディレクトリのサブツリーごと枝刈りするようにし、`.venv/Lib/site-packages/...` が再帰的にインデックスされ forceReindex がタイムアウトする問題を解消（別リポジトリで約 4,400 → 123 ファイルに減少）
- **部分一致による誤除外**: `path.contains("build")` 方式をパスセグメント完全一致に変更。`src/builder.rs` や `targets.rs` が誤って除外される問題を修正
- **FileSystemWatcher / `index_file` の除外漏れ**: daemon 側に単一ファイル用スキップガード、拡張側 watcher に早期 return を追加し、`.venv` 配下変更による不要な再インデックスを抑止

### Added

- **`.comp/ignore`**: gitignore 構文の補助除外ファイルに対応（`ignore` クレートの `add_ignore`）
- **`comp.exclude` 設定**: VS Code 設定で除外ディレクトリ名を指定可能に。`.comp/config.json` の `exclude` 配列へ同期され、daemon の除外リストへ反映
- **自動制限**: 5 MiB 超のファイルをスキップ、2,000 ファイル超で上位ディレクトリ内訳付きの警告ログを出力

### Changed

- **`workspace_root` を daemon state へ一元化**: 各ハンドラが `COMP_WORKSPACE_ROOT` 環境変数を都度読む方式から、起動時に `AppState` が保持する値を使う方式に変更し、起動時 root との乖離を排除
- **ドキュメント同期**: CONFIGURATION / GETTING_STARTED に Python プロジェクト向け除外手順・`comp.exclude`・自動制限を追記。`.gitignore` を尊重する旨の誤記を実態（`.comp/ignore`）に修正

---

## [0.8.0] - 2026-06-10

### Added

- **git diff スコアブースト**: `run_pipeline` が `git diff HEAD` の変更ファイルを候補の先頭に昇格。`pivot_file` エントリに `git_diff: true` マーカーと `coverage.git_diff_boosted` カウントを追加。git 未使用環境ではサイレントにフォールバック
- **サイドバー Re-index ボタン**: SidebarPanel に ↺ ボタンを追加。既存の `comp.forceReindex` コマンドに接続。デーモン停止中は無効化

### Changed

- Rust テスト 6 件（git-diff エッジケース全網羅）、TS テスト 2 件（Re-index ボタン表示・メッセージディスパッチ）を追加

---

## [0.7.1] - 2026-06-08

### Changed

- **ドキュメント全同期**: Aider 対応・Export Debug Log・compression_rules・run_pipeline レスポンスフィールド・MCP_SETUP Aider セクションを v0.7.0 に合わせ更新
- **run_pipeline 優先ルール強化**: ツール説明に「常に最初に呼ぶ」「Read/Bash/grep 代替禁止」を明記
- **Setup Agents 出力にエージェント別憲法ガイドを追加**: CLAUDE.md / .cursor/rules / .clinerules 等へ自動追記するプロンプトを生成
- `.mcp.json` を `.gitignore` に追加（マシン固有の絶対パスを含むため）

---

## [0.7.0] - 2026-06-07

### Added

- **拡張子別圧縮ルール(#7)**: `.comp/config.json`に`compression_rules`フィールドを追加。`{ "*.md": 0, "*.rs": 2 }`のようなパターンでファイルごとの圧縮レベルを指定可能。`run_pipeline`レスポンスに`compression_rules_applied`フラグを追加
- **Aiderエージェント対応(#8)**: `comp setupAgents`でAiderを選択可能に。`.aider.conf.yml`に`mcp-servers`ブロックを生成。既存の設定ファイルがある場合はマージ警告を追加
- **デバッグログエクスポート(#10)**: `comP: Export Debug Log`コマンドを追加。`session-memory.json`をエディターで開くか、任意のパスにエクスポート可能

### Fixed

- **トークン可視化の状態不整合 (#5)**:
  - `startDaemonStack()`完了時にStatusBarが`efficiency`なしで"Ready"を表示していた問題を修正。起動直後に`getStats`を呼んでトークン統計を即時反映
  - `forceReindex`後の`updateStats`で`efficiency`が渡されず表示がリセットされていたバグを修正

### Why

v0.7.xクイックウィン群。圧縮ルールによりMarkdownドキュメントを常に非圧縮（level 0）に保ちつつコードを強圧縮するなど、プロジェクト固有の最適化が可能になる。

---

## [0.5.4] - 2026-06-05

### Added

- **Multi-agent MCP setup**: `comp setupAgents` command now generates configuration templates for Antigravity, GitHub Copilot, Cursor, Cline, and Continue.dev in addition to Claude Code
- **Antigravity IDE support**: comP now officially supports Antigravity with automatic MCP server registration via `mcp-servers-manifest.json`
- **MCP Setup Guide**: New [docs/user/MCP_SETUP.md](docs/user/MCP_SETUP.md) with per-agent configuration instructions including troubleshooting
- **Multi-agent configuration docs**: Updated [docs/user/CONFIGURATION.md](docs/user/CONFIGURATION.md) with multi-workspace setup and simultaneous agent usage guidelines
- **MCP server development guide**: Added [CONTRIBUTING.md](CONTRIBUTING.md) section for MCP tool development with testing checklist and examples

### Changed

- **GitHub Copilot instructions**: Added [src/templates/copilot-instructions.md](src/templates/copilot-instructions.md) with best practices for using comP with Copilot Chat
- **README.md**: Added reference to MCP_SETUP.md for detailed multi-agent configuration

### Why

Feedback from Antigravity users showed that comP's MCP server wasn't discoverable without explicit setup documentation. This release makes comP a "first-class citizen" MCP server that agents can detect and register automatically, with clear setup instructions for all major AI platforms.

---

## [0.5.3] - 2026-06-04

### Added

- **Markdown heading signature**: `parse_markdown()` now captures the first body line after each heading as `signature`, improving BM25 search precision and `get_file_summary` previews
- **`run_pipeline` coverage field**: Response now includes `coverage.indexed_doc_files`, `coverage.bm25_hits`, and `coverage.pivot_file_types` — gives agents a verifiable signal that Markdown and document files were searched, preventing false "not indexed" assumptions

### Changed

- **`run_pipeline` tool description**: Updated to explicitly include documentation tasks (writing/editing Markdown, updating docs), preventing agents from skipping the tool for non-code tasks

---

## [0.4.0] - 2026-06-03

### Added

- **`run_pipeline` content mode**: New `include_content` (bool) and `compression_level` (0/1/2) params — returns compressed file content directly in pivot_file entries, eliminating a second round-trip
- **`get_git_diff_context` tool**: New MCP tool — runs `git diff --name-only <base_ref>` and maps changed files to indexed symbols; useful for PR review and change impact analysis
- **Enhanced `get_project_overview`**: Now includes language distribution (files per language) and top-10 files by symbol count before the full file table

---

## [0.3.0] - 2026-06-02

### Added

- **PDF support**: lopdf-based text extraction; PDFs indexed as page-level symbols with BM25 full-text search
- **Advanced impact analysis**: `max_depth` parameter for `get_impact_graph` — limits BFS hop count (0 = unlimited)
- **TF-IDF search wired to `run_pipeline`**: After indexing, `SearchEngine.build_index()` is called; `run_pipeline` now merges LIKE + TF-IDF results for better recall
- **Multi-path support**: `additional_paths` array in `.comp/config.json` — index monorepo sub-directories or sibling projects into the same graph DB
- **AST-based compression** (`get_symbol` `compression_level` param):
  - Level 0: full source (no-op)
  - Level 1 (compact): comments and blank lines removed via tree-sitter
  - Level 2 (skeleton): function/class bodies replaced with `{ ... }`
- **Slim Markdown output for `get_symbol`**: more concise format with one-liner dependency summaries

---

## [0.2.1] - 2026-05-31

### Fixed

- CI release workflow: fixed vsce publish option and suppressed Node 20 deprecation warnings
- Resolved invalid secrets reference in release.yml

---

## [0.2.0] - 2026-05-28

### Added

- Word (.docx), PowerPoint (.pptx), and Excel (.xlsx) automatic indexing
- BM25 full-text search for Markdown and Office documents
- New MCP tools: `get_symbol`, `get_dependencies`, `get_file_summary`, `get_project_overview`, `session_recall`
- Token compression roadmap preparation

---

## [0.1.0] - 2026-05-21

### Added

#### Core Daemon Features (Phases 3-7)

- **GraphDB Module**: SQLite-based code graph database
  - Persistent storage of symbols and dependencies
  - SHA256-based file change detection
  - Incremental indexing support
  - Full schema with performance indexes

- **Code Parser Integration**: Language-aware symbol extraction
  - tree-sitter support for 30+ languages
  - JSON/XML/Markdown document parsing
  - Dependency analysis with regex patterns
  - Symbol-to-node-ID mapping

- **Search Engine**: TF-IDF semantic search
  - Tokenization for camelCase, snake_case, SCREAMING_CASE
  - Cosine similarity ranking (0.0-1.0)
  - BFS-based impact graph traversal
  - Fuzzy symbol matching

- **MCP Tools** (JSON-RPC 2.0):
  1. `run_pipeline`: Full context generation with token counting
  2. `get_context`: Query-based semantic search
  3. `get_impact_graph`: Change impact analysis
  4. `list_indexed_files`: Index statistics
  5. `get_token_usage`: Token consumption metrics

- **AppState Integration**: Unified state management
  - GraphDB + SearchEngine initialization
  - Mutex-protected concurrent access
  - Automatic workspace detection

- **Testing**: 66 unit tests + 4 integration tests (97% success rate)

#### Build & Release

- **SBOM.json**: CycloneDX 1.4 format dependency tracking
- **GitHub Release Workflow**: Automated VSIX + SBOM upload on tag push
- **Release Notes**: Feature summary with installation instructions

### Changed

- N/A (initial release)

### Fixed

- Fixed regex patterns with proper quote escaping
- Fixed SQLite DEFAULT clause for timestamp columns
- Resolved async/await issues in test suite
- Compilation warnings addressed

### Deprecated

- N/A

### Removed

- N/A

### Security

- All 12 Rust dependencies use MIT or Apache 2.0 licenses
- No external network connectivity required
- SBOM.json provides full license and vulnerability tracking
- Data stays local in .comp/index.db within workspace

---

## Versioning Policy

### Major.Minor.Patch (MAJOR.MINOR.PATCH)

- **MAJOR**: Breaking changes to API or MCP tools
- **MINOR**: New features that are backward compatible
- **PATCH**: Bug fixes and maintenance

Example: `0.1.0` → `0.2.0` (feature) → `0.2.1` (bugfix)

---

## Future Versions (Planned)

### v0.2.0

- Word (.docx) document support
- Advanced impact analysis with transitive dependencies
- Custom context generation templates

### v0.3.0

- Embedding-based semantic search
- Cross-repository indexing
- Real-time symbol navigation

### v1.0.0

- Stable API guarantee
- Extended agent support
- Community integrations

---

## How to Report Changes

When submitting a PR:

1. Update `CHANGELOG.md` under **[Unreleased]** section
2. Choose the appropriate section: Added, Changed, Fixed, Deprecated, Removed, Security
3. Use clear, concise language describing the change
4. Reference issue number (e.g., "Resolves #123")

Example:

```markdown
### Added

- New `get_symbols` MCP tool for listing all exported symbols (Resolves #45)
- Support for Kotlin language via tree-sitter-kotlin

### Fixed

- MCP connection timeout on large repositories (Fixes #38)
```

---

## Release Checklist

Before releasing a new version:

1. [ ] All tests pass locally
2. [ ] Update version in `package.json`
3. [ ] Update CHANGELOG.md with release date and version
4. [ ] Verify all features/fixes are documented
5. [ ] Run Markdown linting: `npm run lint:md:fix`
6. [ ] Commit and create annotated tag: `git tag -a vX.Y.Z -m "Release vX.Y.Z"`
7. [ ] Push commits and tag: `git push origin main && git push origin vX.Y.Z`
8. [ ] Verify GitHub Actions completed successfully
9. [ ] Review GitHub Release with VSIX and SBOM artifacts
