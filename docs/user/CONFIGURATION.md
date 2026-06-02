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
