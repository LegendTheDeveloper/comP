<p align="center">
  <img src="resources/comp-icon.png" width="128" height="128" alt="comP Logo">
</p>

# comP - Code Context Engine for AI Agents

**An open-source, local-first code indexing engine that provides AI coding agents with optimal context.**

🌐 **[Official Website](https://tsucky230.github.io/comP/)**

comP enables Claude Code, Cursor, Cline, and other AI agents to understand and analyze your codebase efficiently. It indexes your project, builds a semantic code graph, and estimates token usage—all running locally on your machine.

---

## What It Does

- **📑 Code Indexing**: Automatically builds a searchable graph of your codebase using tree-sitter (30+ languages) with **concurrent background indexing** to prevent editor startup blocking.
- **🎯 Smart Context**: Provides AI agents with only the most relevant code, reducing tokens by ~60%.
- **🔍 Impact Analysis**: Shows what code could break when you change a symbol.
- **📊 Token Counter**: Displays exactly how many tokens your context uses, and tracks **accumulated token savings and efficiency** across sessions.
- **🔍 BM25 Search**: Complements symbol graph traversal with full-text search capability for Markdown files.
- **🤝 MCP Integration**: Works with Claude Code, Cursor, Cline, and other AI agents via Model Context Protocol.
- **100% Local**: Everything runs on your machine—no cloud calls, no data sharing.

**Supported Languages (30+)**: C, C++, C#, Go, Java, JavaScript, TypeScript, Python, Rust, Ruby, Bash, Kotlin, Swift, PHP, Dart, Elixir, Haskell, Lua, R, Zig, SQL, HTML, CSS, YAML, Scala, and more.

---

## Prerequisites

- **OS**: Windows, macOS, or Linux
- **VSCode**: Version 1.85 or later
- **Rust** (for development only): 1.70+
- **Node.js** (for development only): 18+

---

## Installation

### From VSCode Marketplace

1. Open **VSCode**
2. Go to **Extensions** (`Ctrl+Shift+X`)
3. Search for **"comP - Code Context Engine"**
4. Click **Install**

### From GitHub (Development)

```bash
git clone https://github.com/tsucky230/comP.git
cd comP

# Install dependencies
npm install

# Build the extension
npm run compile

# Build the Rust daemon
npm run daemon:build

# Open in VSCode (F5) to test
```

---

## Quick Start

1. **Install** the comP extension.
2. **Open a folder** in VSCode (e.g., a Git repository).
3. **Start the daemon**:
   - **First-time workspace**: Open the comP sidebar by clicking the **comP icon** in the Activity Bar (left edge), and click the **"Start comP" button** to manually trigger the initial indexing.
   - **Subsequent times (when `.comp/` directory exists)**: comP will **automatically start indexing** your code in the background upon opening the folder.
4. Watch the **status bar** to see indexing progress.
5. Run the setup command:

```bash
Ctrl+Shift+P → "comP: Setup Agents"
```

Choose your AI agent (Claude Code, Cursor, Cline, Antigravity, etc.), and comP will configure the MCP connection.

---

## Agent Compatibility

| Agent | Status | Notes |
| --- | --- | --- |
| **Claude Code** | ✅ Supported & Verified | Verified by maintainers |
| **GitHub Copilot** | ✅ Supported & Verified | Verified by maintainers |
| **Antigravity** | ✅ Supported & Verified | Premium agent in Antigravity IDE |
| **Cursor** | ✅ Supported | MCP 2024-11-05 compliant |
| **Cline** | ✅ Supported | MCP 2024-11-05 compliant |
| **Windsurf** | ✅ Supported | MCP 2024-11-05 compliant |
| **Gemini** | ❌ Not supported | No native MCP client support |

Any MCP 2024-11-05 compliant client should work. If your agent needs a specific config, [open an issue](https://github.com/tsucky230/comP/issues/new).

---

## Usage

### Command Palette

**Ctrl+Shift+P** to access:

| Command | What It Does |
| --- | --- |
| **comP: Setup Agents** | Configure MCP settings for Claude Code, Cursor, Cline, etc. |
| **comP: Force Re-index** | Rebuild the entire codebase index |
| **comP: Generate Context Capsule** | Extract optimized code for your current task |
| **comP: Show Impact Graph** | See what code depends on the symbol at your cursor |

### Status Bar

Bottom-left of VSCode shows:

```text
◈ comP: 12,534 nodes | ✓ Ready
```

Click to open the **Statistics Dashboard** with:

- Indexing progress
- Total nodes and edges
- Token reduction estimate (e.g., 60% saved)
- Session-specific and accumulated token metrics (Sent, Saved, Efficiency %)

### AI Agent Usage

Once configured, AI agents like Claude Code can call comP's tools:

```markdown
# In Claude Code chat:
@comP run_pipeline
Analyze the impact of changing the `authenticate()` function
```

---

## Configuration

comP stores its configuration in `.comp/` at your project root (excluded from git by default).

### Excluding Files and Directories

Create `.comp/ignore` to exclude paths from indexing (same syntax as `.gitignore`):

```gitignore
node_modules/
vendor/
dist/
build/
target/
__pycache__/
*.min.js
```

The following patterns are excluded by default: `node_modules/`, `.git/`, `dist/`, `build/`, `target/`.

### Index Limits

Create `.comp/config.json` to control indexing behavior for large repositories:

```json
{
  "max_nodes": 100000,
  "on_limit_exceeded": "warn"
}
```

| Option | Values | Default | Description |
| --- | --- | --- | --- |
| `max_nodes` | integer | `200000` | Threshold for the total node count |
| `on_limit_exceeded` | `"warn"` \| `"stop"` | `"warn"` | `warn`: notify and continue · `stop`: halt indexing |

> **Expected database size** (`.comp/index.db`): ~1–5 MB for small projects (~1k files), ~20–80 MB for medium (~10k files), ~200 MB–1 GB for large repositories (100k+ files). Only symbol metadata is stored—no raw file content.
>
> **Tip for monorepos**: Open only the relevant subdirectory in VSCode. comP indexes only the open workspace folder.

---

## How It Works

### Architecture

1. **Indexer (Rust daemon)**: Scans your workspace, parses code with tree-sitter, stores graph in SQLite
2. **Search Engine**: Finds relevant code using semantic search + graph traversal + BM25 complementary
3. **MCP Server**: Exposes tools to AI agents via Model Context Protocol
4. **VSCode Extension**: Manages the daemon, displays UI, handles user commands

### Data Flow

```text
Your Code Files
        ↓
  tree-sitter (parses 30+ languages)
        ↓
  SQLite Graph DB (.comp/index.db)
        ↓
  Semantic Search & BM25
        ↓
  MCP Tools (run_pipeline, get_context, etc.)
        ↓
  AI Agent (Claude Code, Cursor, Cline, Antigravity)
```

---

## Output & Results

When you run **"comP: Generate Context Capsule"**:

```text
📌 Pivot Files (full content):
  - src/auth/authenticate.ts (150 lines)

📎 Related Files (signatures only):
  - src/auth/session.ts (14 fn signatures)
  - src/types/user.ts (5 type definitions)

📊 Context Summary:
  - Total tokens: 2,340
  - Savings vs. full file: 65%
  - Estimated cost: $0.04 (vs. $0.11 for raw repo)
```

---

## Troubleshooting

### Issue: "comP is not indexing"

- Check the **Status Bar** (bottom-left) for progress.
- If stuck: **Ctrl+Shift+P** → "comP: Force Re-index"
- Check `.comp/` folder exists and `.gitignore` includes it.

### Issue: "MCP connection failed"

- Run **"comP: Setup Agents"** again.
- Verify your agent has the `.comp/mcp-config.json` file.
- Check VSCode Output panel (View → Output → "comP") for errors.

### Issue: "Indexing is slow"

- Large repos (>100k files) take time on first run.
- Subsequent updates are incremental (fast).
- Check system resources: comP uses <500MB RAM.

### Issue: "Not all languages are recognized"

- comP supports 30+ languages out-of-the-box.
- **Unsupported files** are silently skipped (not indexed).
- Word (.docx) support coming in v2.

---

## Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for details.

### Development Setup

```bash
# Clone and install
git clone https://github.com/tsucky230/comP.git
cd comP
npm install

# Run tests
cargo test --all --manifest-path daemon/Cargo.toml
npm test

# Watch mode for development
npm run watch
npm run daemon:build -- --watch  # in another terminal

# Lint Markdown
npm run lint:md:fix
```

---

## License

**MIT License** — See [LICENSE](LICENSE) for details.

---

## Roadmap

| Version | Features | Status |
| --- | --- | --- |
| **v0.1** | Core indexing, basic MCP, 30 languages, JSON/XML/Markdown, background indexing, and token stats. | ✅ **Released** |
| **v0.2** | Word (.docx), PowerPoint (.pptx), and Excel (.xlsx) automatic indexing, BM25 search. | ✅ **Released** |
| **v0.3** | PDF (.pdf) support, advanced impact analysis, embedding-based search, cross-repo indexing | ⚪ Planning |
| **v0.4** | Auto project docs generation, Git diff-aware context for PR review | ⚪ Planning |
| **v1.0** | Stable API, wider agent support, integrations | ⚪ Planning |

---

## Resources

- **GitHub**: <https://github.com/tsucky230/comP>
- **Issues**: <https://github.com/tsucky230/comP/issues>
- **Discussions**: <https://github.com/tsucky230/comP/discussions>
- **Architecture Docs**: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)
- **API Reference**: [docs/API.md](docs/API.md)

---

## Support This Project

comP is free and open-source. If you find it valuable, consider supporting development:

- ☕ **[GitHub Sponsors](https://github.com/sponsors/tsucky230)** — Support
  ongoing development (coming soon)
- 💖 **Star this repository** — Help others discover comP

Your support enables faster development, better maintenance, and new
features. Thank you! 🙏

---

## Questions?

- 📖 **Read the Docs**: [docs/](docs/)
- 🐛 **Found a Bug**: [Open an Issue](https://github.com/tsucky230/comP/issues/new)
- 💬 **Have Ideas**: [Start a Discussion](https://github.com/tsucky230/comP/discussions/new)
- 👥 **Want to Help**: [CONTRIBUTING.md](CONTRIBUTING.md)
