# Configuration

All settings are under `comp.*` in VSCode settings (`Ctrl+,`).

| Setting | Type | Default | Description |
| --- | --- | --- | --- |
| `comp.maxTokens` | number | `8000` | Maximum tokens for `run_pipeline` context capsule |
| `comp.enableCodeLens` | boolean | `true` | Show dependency counts as CodeLens above symbols |
| `comp.autoIndex` | boolean | `true` | Automatically index files on workspace open |

## Workspace vs User settings

Settings can be applied at user level (`~/.config/Code/User/settings.json`) or
per-workspace (`.vscode/settings.json`). Workspace settings take precedence.

## Multi-Agent Configuration

comP works with multiple AI agents simultaneously. Each agent gets its own configuration:

### VSCode Integrated Agents

For agents running inside VSCode (Copilot, Cline), configure MCP servers in `.vscode/mcp.json`:

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

All paths are indexed into the primary workspace's `.comp/index.db`.
Relative paths are resolved from the workspace root.

---

## Excluding files from indexing

comP respects `.gitignore`. To exclude additional paths, add them to `.gitignore`
or configure `files.exclude` in VSCode settings.

## Manual re-indexing

Run `comP: Force Re-index Workspace` from the Command Palette (`Ctrl+Shift+P`)
to rebuild the index from scratch.

