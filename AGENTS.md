# comP — AI Agent Instructions

## MANDATORY: use comP MCP pipeline — do NOT grep or glob the codebase

For every task — bug fixes, features, refactors, debugging:
**call `run_pipeline` FIRST**. It searches the indexed codebase and returns
the most relevant files and symbols for your task.

Do NOT use grep, glob, Bash find, or cat to search/explore the codebase.
comP returns pre-indexed, graph-ranked context that is more relevant and
uses fewer tokens than manual searching.
Only use Read when you need exact raw content to edit a specific line.

## Primary Tool

- `run_pipeline` — **USE THIS FOR EVERYTHING**. Splits your task into keywords,
  searches the symbol graph, and returns ranked pivot files.

  Examples:
  - `run_pipeline({ "task": "fix JWT validation bug" })`
  - `run_pipeline({ "task": "add user authentication", "max_tokens": 12000 })`
  - `run_pipeline({ "task": "sidebar panel webview", "max_pivots": 10 })`

  Each pivot carries a `score` (relevance, normalized per query) and
  `match_reasons`. If the response has `weak_results: true`, the index found
  nothing confident: fall back to your own search.

## Other MCP tools (use only when run_pipeline is insufficient)

- `get_context` — search symbols by query string, returns ranked results
- `get_impact_graph` — show files affected by a symbol change (blast radius)
- `list_indexed_files` — list all indexed files with symbol counts and language
- `get_stats` — show total file/node/edge counts (health check)

## Workflow

1. `run_pipeline({ "task": "..." })` — ALWAYS FIRST
2. Need to see what's indexed? Use `list_indexed_files`
3. Editing a specific file? Use Read only for exact line content
4. Need blast radius before refactor? Use `get_impact_graph`

## Parameters

- `max_tokens`: increase result budget (default: 8000)
- `min_score_ratio`: relevance cutoff as a fraction of the top score (default: 0.30)
- `max_pivots`: cap on returned pivot files (default: 20)
- `max_file_budget_share`: max budget share per pivot (default: 0.25)
- `doc_token_cap`: absolute token cap for doc pivots (default: 1500)
