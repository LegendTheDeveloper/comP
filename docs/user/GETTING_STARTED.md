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

## Next steps

- [MCP Tools](MCP_TOOLS.md) — use comP as an AI context provider
- [Configuration](CONFIGURATION.md) — adjust settings
