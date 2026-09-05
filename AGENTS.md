# comP — AI Agent Instructions

## Start every task with `run_pipeline`

For every task — bug fixes, features, refactors, debugging — call `run_pipeline`
first, before grepping or reading around. It ranks the indexed files of every
registered repo for the task and says where inside them the task's words hit.
Then read what it pointed at; search by hand only when it tells you to
(`weak_results: true`).

- `run_pipeline({ "task": "fix JWT validation bug" })`
- `run_pipeline({ "task": "add user authentication", "repos": ["Backend"] })`
- `run_pipeline({ "task": "sidebar panel webview", "max_pivots": 10 })`

The `task` must be in English: keywords are matched against symbol names.

## Reading the answer

Each entry of `pivot_files` carries:

- `path` — repo-qualified as `<repo>/<relative>`.
- `score` — normalized per query; comparable only within one response.
- `match_reasons` — which engines matched (`symbol:settings+keybinds`, `filename`, `tfidf`, `bm25`, `git_diff`).
- `matched_symbols` — `[{ name, kind, line }]`: the symbols the task's words hit.
  Read those lines with your file reader instead of the whole file.
- `vendor: true` — third-party code (asset-store packages, NuGet, node_modules).
  Demoted, still returned when it is the best answer.

At the top level:

- `working_tree_files` — files dirty in git that matched nothing. Mid-edit context, not answers.
- `confidence` — `high` only when the top three pivots cover most of the task
  (`coverage.top_coverage`) and the first pivot is first-party.
- `weak_results: true` — the index found nothing it trusts: fall back to your own
  search (`weak_reason` explains why).
- `uncovered_keywords` — defining task words that matched nothing in code. If your
  feature's name is there, it probably does not exist yet and the pivots are
  integration points, not the feature.
- `dropped_low_relevance`, `dropped_vendor` — what was cut and why.

## Other tools (when `run_pipeline` is not enough)

- `get_context` — symbols by name or keyword (first-party rows first).
- `get_symbol` — a symbol's source slice plus its dependencies.
- `get_file_summary` — every symbol of one file with line and signature (paged, 300 rows).
- `get_impact_graph` — files affected by a symbol change.
- `list_indexed_files` — paged listing (300 entries), filter by `prefix` or `language`.
- `get_project_overview` — one-screen summary: repos, languages, largest folders and files.
- `session_recall` — what was searched before, across sessions.

## Parameters worth knowing

- `max_pivots` (20), `min_score_ratio` (0.30), `repos` (all).
- `vendor_score_factor` (0.5): set `1.0` when the task is about a third-party package itself.
- `vendor_pivot_share` (0.25): max share of the pivots third-party files may take while first-party code competes.
- `include_content: true` packs compressed file content into `max_tokens` (8000); without it the answer is metadata only and nothing is dropped for budget.

## Workspace configuration (`.comp/config.json`, all optional)

- `skip_extensions` — extra extensions never indexed (Unity asset formats and `.meta` are built in).
- `vendor_paths` / `.comp/vendor` (gitignore syntax) and `first_party_paths` — override the third-party detection.
- `vendor_auto`, `vendor_active_min_commits` (6), `vendor_auto_max_commits` (3), `vendor_auto_min_files` (20), `vendor_activity_months` (18) — the git-activity heuristic.
- `noise_keywords`, `min_score_abs` and the cutoff knobs: see `docs/user/CONFIGURATION.md`.

## Session continuity

Sessions persist across daemon restarts. When resuming work, call `session_recall()`
(`{ "query": "keyword" }` to filter, `{ "limit": 5 }` for the last N) and continue in
that context.
