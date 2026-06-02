# MCP Tools Reference

comP exposes tools via the Model Context Protocol (JSON-RPC 2.0 over stdio).

## Setup

Run `comP: Setup Agent MCP` from the VSCode Command Palette to auto-configure
Claude Code, Cursor, Cline, Windsurf, or Continue.

## Tools

### `run_pipeline`

Primary tool. Splits a task description into keywords, searches the indexed symbol
graph, and returns ranked context files.

```json
{ "task": "fix JWT validation bug", "max_tokens": 8000 }
```

Parameters:

- `task` (string, required) — natural language description of the task
- `max_tokens` (number, optional, default 8000) — result budget
- `include_tests` (boolean, optional) — include test files in results
- `include_content` (boolean, optional) — if true, each pivot_file entry includes a `content` field with the file contents
- `compression_level` (0/1/2, optional, default 0) — content compression applied when `include_content` is true:
  - `0` — full source (no change)
  - `1` — compact: comments and blank lines removed (~20-35% smaller)
  - `2` — skeleton: function/class bodies replaced with `{ ... }` (~50-70% smaller)

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

---

### `get_git_diff_context`

Get context for files changed in a git diff. Runs `git diff --name-only <base_ref>` and maps each changed file to its indexed symbols.

```json
{ "base_ref": "main" }
```

Parameters:

- `base_ref` (string, optional, default `HEAD~1`) — git ref to diff against. Use `main` or `master` for branch comparisons.

Returns a Markdown table of changed files with language, symbol count, and whether each file is indexed.
