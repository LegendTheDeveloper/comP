# Configuration

All settings are under `comp.*` in VSCode settings (`Ctrl+,`).

| Setting | Type | Default | Description |
|---|---|---|---|
| `comp.maxTokens` | number | `8000` | Maximum tokens for `run_pipeline` context capsule |
| `comp.enableCodeLens` | boolean | `true` | Show dependency counts as CodeLens above symbols |
| `comp.autoIndex` | boolean | `true` | Automatically index files on workspace open |

## Workspace vs User settings

Settings can be applied at user level (`~/.config/Code/User/settings.json`) or
per-workspace (`.vscode/settings.json`). Workspace settings take precedence.

## Excluding files from indexing

comP respects `.gitignore`. To exclude additional paths, add them to `.gitignore`
or configure `files.exclude` in VSCode settings.

## Manual re-indexing

Run `comP: Force Re-index Workspace` from the Command Palette (`Ctrl+Shift+P`)
to rebuild the index from scratch.
