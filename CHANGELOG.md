# Changelog

All notable changes to comP are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/) and this project adheres to [Semantic Versioning](https://semver.org/).

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
