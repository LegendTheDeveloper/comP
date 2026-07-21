# MCP Tools Reference

comP exposes tools via the Model Context Protocol (JSON-RPC 2.0 over stdio).

## Setup

Run `comP: Setup Agent MCP` from the VS Code Command Palette to auto-configure
Claude Code, Cursor, Cline, Windsurf, or Continue.

## Tools

### `run_pipeline`

Primary tool. Splits a task description into keywords, searches the indexed symbol
graph, and returns ranked context files.

```json
{ "task": "fix JWT validation bug", "max_tokens": 8000 }
```

Parameters:

- `task` (string, required): natural language description of the task
- `max_tokens` (number, optional, default 8000): result budget
- `min_score_ratio` (number, optional, default 0.30): drop pivots scoring below this fraction of the top score
- `max_pivots` (integer, optional, default 20): maximum pivot files returned (capped at 5 when `weak_results` is true)
- `max_file_budget_share` (number, optional, default 0.25): max share of the budget one pivot may consume; oversized files get extra compression, then hard truncation (`truncated: true`)
- `doc_token_cap` (integer, optional, default 1500): additional absolute token cap for doc pivots (markdown/office/pdf)
- `include_content` (boolean, optional): if true, each pivot_file entry includes a `content` field with the file contents
- `compression_level` (0/1/2, optional, default 0) — content compression applied when `include_content` is true:
  - `0` — full source (no change)
  - `1` — compact: comments and blank lines removed (~20-35% smaller)
  - `2` — skeleton: function/class bodies replaced with `{ ... }` (~50-70% smaller)

Response fields (v0.6+):

- `compression_level_applied` (number) — actual compression level used after auto-budget selection
- `budget_adjusted` (boolean) — `true` if compression level was raised to fit within `default_budget_tokens`
- `compression_rules_applied` (boolean) — `true` if any per-extension rules from `compression_rules` were applied

Response fields (v0.9.2+):

- `related_files` (array) — files one dependency hop away from the pivot files (callers/callees in other files), ranked by connecting-edge count, up to 10 entries:

  ```json
  [{ "path": "src/auth/middleware.rs", "edge_count": 4 }]
  ```

- Token estimates per pivot file are based on the real indexed file size (`chars / 4`), no longer on symbol-count heuristics

Response fields (v0.9.4+, unified relevance scoring):

- `pivot_files[].score` (number): unified relevance score combining symbol match quality, TF-IDF and BM25 (plus a git-diff boost). Normalized per query: comparable within one response, never across calls
- `pivot_files[].match_reasons` (array of strings): which engines matched, e.g. `["symbol:survey+feedback", "tfidf", "bm25", "git_diff"]`
- `pivot_files[].truncated` (boolean, optional): the file exceeded its per-file token cap even at maximum compression and was hard-capped
- `confidence` ("high" / "medium" / "low"): overall result strength, judged from raw engine signals
- `weak_results` (boolean): true when no engine found confident evidence; the pivot list is capped at 5 and agents should fall back to their own search
- `dropped_low_relevance` (number): candidates discarded by the relevance cutoff (score below `min_score_ratio` of the top score, or below the absolute floor)

`.comp/config.json` keys for the cutoff (all optional): `min_score_abs` (default 0.05, config-only), `min_score_ratio`, `max_pivots`, `max_file_budget_share`, `doc_token_cap`. Params override config.

Note: git-diff files are score-boosted and never dropped by the cutoff, but no longer jump unconditionally ahead of strong matches.

Search history (v0.9.5+): every `run_pipeline` / `get_context` call is recorded in the shared index DB (`search_history` table, newest 500 kept) with its query, filtered keywords, confidence, weak_results, pivot/dropped counts, tokens, duration, and top-8 pivots with scores. Retrieve via the `getSearchHistory` JSON-RPC method (`{ "limit": 50 }`, capped at 200); the VS Code sidebar shows it as the "Recent Searches" panel. Intended for reviewing search quality and tuning the relevance scoring.

Keyword coverage and workspace noise (v0.9.6+):

- Noise keywords: the tokens of the registered repo aliases (e.g. game, launcher, cloud for GameLauncherCloud-*) plus the optional `noise_keywords` config array are skipped in the symbol-LIKE and filename channels; they still count in TF-IDF and BM25. Matching the product's own name proves nothing.
- `uncovered_keywords` (response): defining (rarest) task keywords that found no code evidence. If the feature's name is listed, the feature likely does not exist yet (greenfield) and the pivots are integration points.
- `weak_reason` (response): set when all defining keywords are uncovered; confidence drops to "low" and `weak_results` triggers. A single uncovered rarest keyword caps confidence at "medium".
- Generated `X.Designer.cs` twins are dropped from pivots when their base `X.cs` is also a candidate.
- `.sql` files now participate in the BM25 doc channel (schema files, RLS policies).
- `search_history.kw_info` records per-keyword df/weight/best-quality plus the uncovered list for offline tuning.

---

### `get_context`

Search symbols by query string. Returns ranked matches with file paths and line numbers.

```json
{ "query": "DaemonManager", "limit": 10 }
```

---

### `get_impact_graph`

Show all files affected by changes to a symbol (blast radius analysis).

```json
{ "symbol": "request", "file": "src/daemon/DaemonManager.ts", "max_depth": 3 }
```

Parameters:

- `symbol` (string, required) — symbol name to analyze
- `file` (string, optional) — narrow to a specific file when the symbol appears in multiple files
- `max_depth` (number, optional, default 0) — BFS hop limit; 0 means unlimited transitive traversal

---

### `list_indexed_files`

List all indexed files with symbol counts and detected language.

```json
{}
```

---

### `get_symbol`

Return full source of a specific symbol with optional compression.

```json
{ "symbol": "authenticate", "file": "src/auth.rs", "compression_level": 1 }
```

Parameters:

- `symbol` (string, required) — exact symbol name
- `file` (string, optional) — narrow to a specific file
- `compression_level` (number, optional, default 0):
  - `0` — full source (no change)
  - `1` — compact: comments and blank lines removed
  - `2` — skeleton: function/class bodies replaced with `{ ... }`

---

### `get_stats`

Return total file, node, and edge counts (index health check).

```json
{}
```

Response fields (v0.9.2+):

- `daemon_version` (string) — version of the running daemon binary. Compare against the installed release to detect a stale daemon that kept running across an upgrade (on Windows the running exe stays locked, so rebuilds do not take effect until the daemon restarts).

---

### `get_git_diff_context`

Get context for files changed in a git diff. Runs `git diff --name-only <base_ref>` and maps each changed file to its indexed symbols.

```json
{ "base_ref": "main" }
```

Parameters:

- `base_ref` (string, optional, default `HEAD~1`) — git ref to diff against. Use `main` or `master` for branch comparisons.

Returns a Markdown table of changed files with language, symbol count, and whether each file is indexed.

---

### `session_log`

ユーザーの依頼と対応結果を `.comp/history/log-YYYY-MM.jsonl` に永続記録します。
書き込み直後に BM25 インデックスへ即時反映されるため、次回以降の `run_pipeline` 検索で過去のやりとりが自然に参照されます。

セッション切れ・デーモン再起動後も残る「作業ログ」として、重要タスクの完了時に呼んでください。

```json
{
  "request": "session_log MCPツールを追加する",
  "outcome": "daemon/src/mcp/mod.rs に handle_session_log を実装し JSONL 追記＋即時インデックスを完了",
  "files": ["daemon/src/mcp/mod.rs", "daemon/src/indexer/walker.rs"]
}
```

パラメータ:

- `request` (string, 必須) — ユーザーの依頼テキスト（最大 600 文字）
- `outcome` (string, 省略可) — 対応結果の要約（最大 400 文字）
- `files` (string[], 省略可) — 変更したファイルパスの一覧

レスポンス例:

```json
{ "status": "ok", "path": ".comp/history/log-2026-06.jsonl", "timestamp": 1751023456789 }
```

---

### `session_recall`

過去のやりとりをセッション横断で検索・返却します。デーモン再起動をまたいだ **全セッション** を対象とします。

`.comp/session-memory.json`（run_pipeline / get_context の自動記録）と `.comp/history/*.jsonl`（session_log の明示記録・Stop hook 自動記録）を統合し、新しい順で返します。

```json
{ "query": "session_log", "limit": 10 }
```

パラメータ:

- `query` (string, 省略可) — request・outcome 両フィールドへの部分一致フィルタ
- `limit` (number, 省略可, デフォルト 20) — 返却件数の上限

レスポンス形式（Markdown テキスト）:

```
### Session Recall

- `2026-06-27 01:30` **Query**: "session_log MCPツールを追加する" (Tokens: 4200)
  - **Outcome**: daemon/src/mcp/mod.rs に handle_session_log を実装し JSONL 追記＋即時インデックスを完了
  - **Symbols**: `SessionCall`, `format_epoch_ms`（該当する場合）
  - **Files**: `daemon/src/mcp/mod.rs`, `daemon/src/indexer/walker.rs`（該当する場合）
```

各項目（Outcome・Symbols・Files）は、データが存在する場合のみ表示されます。

**v0.9.2+**: Symbols・Files は各エントリ **先頭 5 件まで** 表示し、超過分は `… (+N more)` と件数のみ示します（run_pipeline 自動記録は数十件のシンボルを含むことがあり、全列挙すると recall 自体がトークンを浪費するため）。

**推奨**: 新しいセッション開始時や作業再開時に `session_recall` を呼び、前回の依頼と対応を確認してください。
