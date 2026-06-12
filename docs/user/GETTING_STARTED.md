# Getting Started with comP

## Prerequisites

- VSCode 1.80+
- A workspace with TypeScript, JavaScript, Python, or Rust code

## Installation

### From VSIX (recommended)

1. Download `comP-<version>.vsix` from [GitHub Releases](https://github.com/tsucky230/comP/releases)
2. In VSCode: `Extensions` → `...` → `Install from VSIX`
3. Select the downloaded file

### From source

```bash
git clone https://github.com/tsucky230/comP.git
cd comP
npm install
npm run compile
npm run daemon:build
```

Then press `F5` in VSCode to launch the extension in a development host.

## First use

1. Open a workspace folder in VSCode
2. comP starts indexing automatically (status bar shows `◈ comP: Indexing`)
3. Wait for `◈ comP: Ready` — indexing is complete
4. Open the comP sidebar panel to see index statistics

## Python projects

comP indexes Python source files automatically. Keep indexing fast by ensuring
virtual environments are excluded:

| Directory name | Excluded automatically? | Action needed |
| --- | --- | --- |
| `.venv/` | ✅ Yes (hidden dir) | None |
| `venv/` | ✅ Yes (built-in skip list) | None |
| `__pycache__/` | ✅ Yes (built-in skip list) | None |
| Custom venv name (e.g. `env/`) | ❌ No | Add to `.gitignore` or `.comp/ignore` |

If you see a large number of files indexed (the status bar will warn when > 2 000
files are found), check the daemon log and add the offending directory to
`.comp/ignore`:

```gitignore
# .comp/ignore
env/
.tox/
.nox/
```

> **Tip**: Run `comP: Force Re-index Workspace` from the Command Palette after
> editing `.comp/ignore` to apply the exclusions immediately.

## Next steps

- [MCP Tools](MCP_TOOLS.md) — use comP as an AI context provider
- [Configuration](CONFIGURATION.md) — adjust settings and exclusions
