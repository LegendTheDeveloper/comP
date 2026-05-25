# MCP Tools Reference

comP exposes 5 tools via the Model Context Protocol (JSON-RPC 2.0 over stdio).

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
{ "symbol": "request", "file": "src/daemon/DaemonManager.ts" }
```

---

### `list_indexed_files`

List all indexed files with symbol counts and detected language.

```json
{}
```

---

### `get_stats`

Return total file, node, and edge counts (index health check).

```json
{}
```
