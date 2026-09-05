# Configuration

All settings are under `comp.*` in VS Code settings (`Ctrl+,`).

| Setting | Type | Default | Description |
| --- | --- | --- | --- |
| `comp.maxTokens` | number | `8000` | Maximum tokens for `run_pipeline` context capsule |
| `comp.enableCodeLens` | boolean | `true` | Show dependency counts as CodeLens above symbols |
| `comp.autoIndex` | boolean | `true` | Automatically index files on workspace open |
| `comp.exclude` | string[] | `[]` | Additional directory names to exclude from indexing. Synced to `.comp/config.json` on activation. Changes take effect after Force Re-index. |

## Indexing scope and search quality (`.comp/config.json`, v0.9.7)

All keys are optional. `.comp/` is per repo: an `additional_paths` repo reads its own `.comp/config.json`, `.comp/ignore` and `.comp/vendor`.

| Key | Default | Description |
| --- | --- | --- |
| `skip_extensions` | `[]` | Extensions never indexed, added to the built-in list (Unity `.meta`, `.prefab`, `.unity`, `.asset`, `.mat`, `.anim`, `.controller`, ...). Files of unknown extension and NuGet documentation XML (`<doc><assembly>`) are never indexed either; shaders, `.txt`, `.ini`, installer and build scripts are stored without symbols so they stay findable by name. |
| `vendor_paths` | `[]` | gitignore-style patterns forced to third-party. The lines of `.comp/vendor` are appended. |
| `first_party_paths` | `[]` | Patterns forced to first-party (beats everything else). |
| `vendor_auto` | `true` | Treat a folder with `vendor_auto_min_files` or more indexed files and at most `vendor_auto_max_commits` distinct commits in the activity window as a dropped-in package. Needs 50+ commits in the repo. |
| `vendor_active_min_commits` | `6` | A folder with this many distinct commits in the window is first-party even under a built-in vendor pattern (bought assets the project patches). |
| `vendor_auto_max_commits` | `3` | See `vendor_auto`. |
| `vendor_auto_min_files` | `20` | See `vendor_auto`. |
| `vendor_activity_months` | `18` | How far back `git log` looks. Cached in the index DB for `vendor_activity_ttl_hours` (24). |
| `vendor_score_factor` | `0.5` | Multiplier for third-party files in `run_pipeline` (also a request parameter). |
| `vendor_pivot_share` | `0.25` | Max share of the pivots third-party files may take while first-party candidates compete (also a request parameter). |
| `doc_pivot_share` | `0.15` | Max share of the pivots doc files (markdown, sql, pdf, office) may take while code candidates compete (also a request parameter). |
| `doc_score_factor` | `0.7` | Multiplier for doc files while at least three code candidates compete; a doc's only channel is BM25, whose best hit is always normalized to the full weight (also a request parameter). |
| `noise_keywords` | `[]` | Keywords skipped in the LIKE and filename channels, merged with the tokens of the repo aliases. |
| `min_score_abs`, `min_score_ratio`, `max_pivots`, `max_file_budget_share`, `doc_token_cap` | 0.05, 0.30, 20, 0.25, 1500 | Relevance cutoff and per-file caps (all but `min_score_abs` are also request parameters). |

Built-in vendor patterns: `Assets/Plugins/`, `Assets/Standard Assets/`, `Assets/TextMesh Pro/`, `Assets/PostProcessing/`, `Assets/Photon/`, `/Packages/`, `/Library/`, `packages/`, `bin/`, `obj/`, `node_modules/`, `vendor/`, `dist/`, `wwwroot/lib/`, `ThirdParty/`, `third_party/`, `3rdParty/`, `External/`, `Externals/`, `phpmailer/`, `*.min.js`. `get_stats.vendor_folders` lists which folders were demoted and by which rule (`config`, `builtin`, `git-inactive`).

## Workspace vs User settings

Settings can be applied at user level (`~/.config/Code/User/settings.json`) or
per-workspace (`.vscode/settings.json`). Workspace settings take precedence.

## Multi-Agent Configuration

comP works with multiple AI agents simultaneously. Each agent gets its own configuration:

### VS Code Integrated Agents

For agents running inside VS Code (Copilot, Cline), configure MCP servers in `.vscode/mcp.json`:

```json
{
  "servers": {
    "comp": {
      "command": "comp-daemon",
      "args": [],
      "env": {
        "COMP_WORKSPACE_ROOT": "."
      }
    }
  }
}
```

### External Agents (Claude Code, Cursor, Antigravity)

External agents use their own MCP configuration files. Run:

```
Ctrl+Shift+P → "comP: Setup Agents"
```

This generates agent-specific configs in `.comp/config/`:

- `claude_desktop_config.json` (Claude Code)
- `cursor_config.json` (Cursor)
- `cline_config.json` (Cline)
- `antigravity-settings.json` (Antigravity)

Copy these configs to each agent's configuration directory (see [docs/user/MCP_SETUP.md](./MCP_SETUP.md) for per-agent paths).

### Using Multiple Agents in One Workspace

You can use Claude Code + Cursor + Copilot simultaneously:

1. **Setup Claude Code**: Copy `claude_desktop_config.json` to `~/.claude/claude_desktop_config.json`
2. **Setup Cursor**: Copy `cursor_config.json` to `~/.cursor/mcp.json` or `.cursor/mcp.json`
3. **Setup Copilot**: Already configured in `.vscode/mcp.json` (automatic)

All three agents will use the same `.comp/index.db` for shared indexing.

---

## Multi-path indexing (monorepo / multi-root)

Create `.comp/config.json` in the workspace root to index additional directories
into the same graph database:

```json
{
  "additional_paths": [
    "../shared-lib",
    "/absolute/path/to/another-project"
  ]
}
```

---

## Compression Rules

Control compression level per file extension in `.comp/config.json`:

```json
{
  "default_budget_tokens": 8000,
  "compression_rules": {
    "*.md": 0,
    "*.rs": 2,
    "*.ts": 1
  }
}
```

| Option | Values | Description |
| --- | --- | --- |
| `default_budget_tokens` | integer | Token budget for `run_pipeline`. When set, compression level is auto-selected (0→1→2) to fit within budget. |
| `compression_rules` | object | Glob pattern → compression level (0/1/2). Overrides auto-budget selection per file. |

Compression levels:

- `0` — full source (no change)
- `1` — compact: comments and blank lines removed (~20-35% smaller)
- `2` — skeleton: function/class bodies replaced with `{ ... }` (~50-70% smaller)

All paths are indexed into the primary workspace's `.comp/index.db`.
Relative paths are resolved from the workspace root.

---

## Agent Setup Configuration

### Auto-generation of LLM Constitution Files

When you run `comP: Setup Agents`, comP automatically creates or updates local constitution files
(`.claude/CLAUDE.md`, `CLAUDE.md`, and agent-specific files like `.github/copilot-instructions.md`)
with **Session Continuity** instructions that prompt the LLM to call `session_recall()` when resuming work.

To **disable** this auto-generation (if you prefer to manage constitution files manually),
add this to `.comp/config.json`:

```json
{
  "autoGenerateConstitution": false
}
```

| Setting | Type | Default | Description |
| --- | --- | --- | --- |
| `autoGenerateConstitution` | boolean | `true` | When `true`, Setup Agents auto-creates/updates CLAUDE.md files with session_recall instructions. When `false`, skips auto-generation (manual control). |

**Note**: Setting this to `false` does NOT prevent MCP configuration files (e.g., `claude_desktop_config.json`)
from being generated — only the LLM instruction files are affected.

---

## Excluding files from indexing

comP respects `.gitignore` (and nested `.gitignore` files throughout the workspace).
Files and directories matching gitignore patterns are never indexed or re-indexed.

Hidden directories (names starting with `.`) are also excluded automatically,
so `.venv`, `.pytest_cache`, `.mypy_cache`, etc. require no additional configuration.

### Excluding directories via VS Code settings

Use the `comp.exclude` setting to specify directory names that should never be indexed:

```json
// .vscode/settings.json
{
  "comp.exclude": ["env", "data", "dist"]
}
```

The extension syncs this list to `.comp/config.json` on activation. The daemon reads it each time
an indexer is created, so both initial indexing and **Force Re-index** pick up the changes.

> **Note**: Values in `comp.exclude` are matched against directory **name segments** (not paths),
> so `"env"` excludes any directory named `env` at any depth.

### Excluding via config.json directly

You can also write the `exclude` array directly to `.comp/config.json`:

```json
{
  "exclude": ["env", "data"]
}
```

Changes to `.comp/config.json` take effect on the next **Force Re-index** (`Ctrl+Shift+P` →
`comP: Force Re-index Workspace`).

### Excluding via .comp/ignore

To exclude additional paths that are **not** already covered by `.gitignore`, create
`.comp/ignore` in the workspace root using standard gitignore syntax:

```gitignore
# .comp/ignore
venv/
__pycache__/
legacy_data/
*.log
```

Common patterns for Python projects:

```gitignore
venv/
__pycache__/
*.pyc
.pytest_cache/
.mypy_cache/
```

> **Note**: `files.exclude` in VS Code settings has no effect on comP's daemon-side
> indexing. Use `.gitignore` or `.comp/ignore` to control what the daemon indexes.

## Automatic limits

The following limits are applied automatically and require no configuration:

| Limit | Value | Behavior |
| --- | --- | --- |
| Max file size | 5 MiB | Files larger than 5 MiB are silently skipped during indexing. Useful for large generated files, binary assets, or data files inadvertently left in the workspace. |
| Large-workspace warning | 2 000 files | When more than 2 000 files are found after exclusions, a warning is logged listing the top directories by file count. Use this as a hint for which directories to add to `.comp/ignore`. |

## Manual re-indexing

Run `comP: Force Re-index Workspace` from the Command Palette (`Ctrl+Shift+P`)
to rebuild the index from scratch.
