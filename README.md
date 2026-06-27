<p align="center">
  <img src="resources/comp-icon.png" width="128" height="128" alt="comP Logo">
</p>

# comP — Your AI Assistant's Memory System

**Open-source, local-first code indexing engine for AI coding assistants. Works with Claude Code, Cursor, Cline, Antigravity, and more.**

🌐 **[Official Website](https://tsucky230.github.io/comP/)**

---

## The Problem AI Assistants Face

Claude Code, Cursor, and other AI coding assistants are powerful, but they have **one critical limitation**:

**Your AI reads the entire codebase every time you ask a question.**

```
Traditional workflow:
  Question → AI reads entire project → answers (5,000 tokens)
  Question → AI reads entire project again → answers (5,000 tokens)

Cost: $0.10 per question. Context forgotten between sessions.
```

---

## How comP Solves It

**comP creates a "project map + index"** so your AI understands "this file does X, that directory handles Y" **instantly—without reading every file.**

```
With comP:
  Question → comP extracts relevant files (300 tokens) → AI answers fast
  Question → comP finds related code → AI answers instantly

Cost: $0.006 per question (94% reduction). Session history auto-restored.
```

---

## What Actually Changes

| Metric | Before | After |
| --- | --- | --- |
| **Input tokens per question** | 5,000 | 300 |
| **Cost per question** | $0.10 | $0.006 |
| **Time to first response** | 15 seconds | 3 seconds |
| **Session memory** | Lost when IDE closes | Auto-preserved & searchable |
| **Data sent to cloud** | All files | None (100% local) |

---

## Installation & Setup (3 Steps)

### 1. Install from VS Code Marketplace

1. Open VS Code
2. Go to **Extensions** (`Ctrl+Shift+X`)
3. Search for **"comP - Code Context Engine"**
4. Click **Install**

### 2. Open Your Project

Open any folder in VS Code (Git repository recommended).

### 3. Start comP

- Click the **comP icon** in the Activity Bar (left sidebar)
- Click **"▶ Start"** button
- Indexing begins in the background
- Watch the status bar at the bottom for progress

```text
◈ comP: 12,534 symbols | ✓ Ready
```

---

## Connect Your AI Agent (One-Time Setup)

After installing comP, run this command to connect your AI assistant:

```text
Ctrl+Shift+P → "comP: Setup Agents"
```

Select your agent (Claude Code, Cursor, Cline, etc.), and comP generates the connection instructions.

| Agent | What to Do |
| --- | --- |
| **Claude Code** | Copy-paste the generated `claude mcp add` command in terminal |
| **GitHub Copilot** | Auto-configured (no manual steps) |
| **Cursor** | Copy generated config to `~/.cursor/mcp.json` |
| **Cline** | Paste into Cline's MCP settings |
| **Antigravity** | Auto-configured |
| **Aider** | Auto-configured in `.aider.conf.yml` |
| **Windsurf** | Copy to `~/.codeium/windsurf/mcp_config.json` |
| **Continue.dev** | Add to `~/.continue/config.py` |

Details: [docs/user/MCP_SETUP.md](docs/user/MCP_SETUP.md)

---

## How to Use It

### With Claude Code (Simplest)

```markdown
# In Claude Code chat:
@comP run_pipeline
Analyze what happens if I change the authenticate() function
```

comP finds all related files and Claude answers with minimal context.

### With VS Code Native Chat

```markdown
@comp #file:src/main.rs
Explain what this function does
```

The file is automatically compressed before being sent to the LLM.

### Available Commands

Press `Ctrl+Shift+P`:

| Command | What It Does |
| --- | --- |
| **comP: Setup Agents** | Configure Claude Code, Cursor, etc. |
| **comP: Force Re-index** | Scan entire project again (after adding files) |
| **comP: Show Impact Graph** | See what code breaks if you change a symbol |
| **comP: Copy Active File Compressed** | Copy current file in compressed form to clipboard |
| **comP: Export Debug Log** | Save session history for debugging |

### Status Bar (Bottom of VS Code)

```text
◈ comP: 12,534 nodes | ✓ Ready | 60% saved
```

Click it to see:

- Files indexed
- Total symbols found
- Tokens saved this session
- Last agent connection time

---

## Excluding Files & Folders

Create `.comp/ignore` in your project root (like `.gitignore`):

```gitignore
node_modules/
vendor/
dist/
build/
target/
__pycache__/
*.min.js
```

These are auto-excluded:

- `.venv`, `.pytest_cache` (hidden directories)
- Anything in `.gitignore`
- `node_modules`, `venv`, `__pycache__`, `coverage`, `vendor`, `out`

---

## Fine-Tuning Compression & Budget

Create `.comp/config.json` for large projects:

```json
{
  "max_nodes": 100000,
  "on_limit_exceeded": "warn",
  "default_budget_tokens": 8000,
  "compression_rules": {
    "*.md": 0,
    "*.rs": 2,
    "*.ts": 1
  }
}
```

| Setting | Meaning |
| --- | --- |
| `max_nodes` | Upper limit for symbols (default: 200,000) |
| `on_limit_exceeded` | `"warn"` = notify but continue / `"stop"` = halt |
| `default_budget_tokens` | Token budget—auto-picks compression level |
| `compression_rules` | Override compression per file type (0=full, 1=compact, 2=skeleton) |

> **Database size**: ~1–5 MB (1k files), 20–80 MB (10k files), 200 MB–1 GB (100k+ files). Only metadata stored—no raw code.

---

## How It Works (Technical Overview)

### Architecture

1. **Indexer (Rust daemon)**: Scans your workspace, parses code with tree-sitter, stores in SQLite
2. **Search Engine**: BM25 full-text search + graph traversal + semantic scoring
3. **MCP Server**: Exposes `run_pipeline`, `get_context`, `get_impact_graph` tools
4. **VS Code Extension**: Manages daemon, UI, commands

### Data Flow

```
Code files (30+ languages)
       ↓
   [Tree-sitter parsing]
       ↓
SQLite graph database (.comp/index.db)
       ↓
   [Search engine: BM25 + graph traversal]
       ↓
  [MCP server: run_pipeline, get_context]
       ↓
AI agent (Claude Code, Cursor, Cline, etc.)
       ↓
  [Compression: remove unnecessary content]
       ↓
LLM API (fewer tokens = lower cost)
```

### Supported Languages (30+)

C, C++, C#, Go, Java, JavaScript, TypeScript, Python, Rust, Ruby, Bash, Kotlin, Swift, PHP, Dart, Elixir, Haskell, Lua, R, Zig, SQL, HTML, CSS, YAML, Scala, and more.

---

## Security & Privacy

- **🔐 100% Local**: Code and chat history stay on your machine—never sent to the cloud
- **🛡️ Auto-Excluded**: `.comp/` folder auto-added to `.gitignore`
- **📋 Auditable**: Everything runs locally—no external APIs, no telemetry
- **🏢 Enterprise-Safe**: Secure for proprietary code, compliant environments, sensitive data

---

## Troubleshooting

### "comP isn't indexing"

1. Check the status bar (bottom of VS Code) for progress
2. If stuck: `Ctrl+Shift+P` → **"comP: Force Re-index"**
3. Verify `.comp/` folder exists and is in `.gitignore`

### "MCP connection failed"

1. Re-run `Ctrl+Shift+P` → **"comP: Setup Agents"**
2. Check that the config file was created
3. View logs in VS Code **Output** panel (View → Output → "comP")

### "Indexing is slow"

- First-time indexing of large repos (>100k files) takes time
- Subsequent runs are incremental (fast)
- comP uses <500MB RAM typically

### "Some languages not recognized"

- comP supports 30+ languages; unsupported files are skipped
- Office formats (Word, Excel) will be supported in v2.0

---

## FAQ: "My AI reads files directly even with comP installed"

**A:** AI assistants work best when you're explicit. Try these:

### 1. Use `@comP` in your prompt

```markdown
# Good:
@comP run_pipeline
Analyze the JWT authentication flow

# Avoids:
Look at src/auth/middleware.ts and explain it
```

### 2. Strengthen agent instructions

Add this to `.github/copilot-instructions.md`:

```markdown
## comP Usage is Mandatory

- Call `run_pipeline` FIRST on every coding task
- Never use grep/find/Bash for searches—use comP instead
- Why: comP saves 94% tokens vs full codebase reads
```

### 3. Use session memory

At the start of a new session:

```markdown
@comP session_recall
Remind me what we decided about JWT token expiry
```

Session history is auto-restored—your AI remembers past decisions.

---

## Contributing

Bug reports, feature requests, and PRs welcome! See [CONTRIBUTING.md](CONTRIBUTING.md).

### Development Setup

```bash
git clone https://github.com/tsucky230/comP.git
cd comP
npm install

# Tests
npm test
cargo test --manifest-path daemon/Cargo.toml

# Watch mode
npm run watch
npm run daemon:build -- --watch
```

---

## License

MIT — [LICENSE](LICENSE)

---

## Roadmap

| Version | Features | Status |
| --- | --- | --- |
| **v0.1** | Core indexing, MCP, 30 languages | ✅ Released |
| **v0.2** | Office formats (Word/Excel), BM25 search | ✅ Released |
| **v0.3** | PDF support, impact analysis | ✅ Released |
| **v0.4** | Content mode, git diff context | ✅ Released |
| **v0.5** | Code compression, @comp chat participant | ✅ Released |
| **v0.6** | Dynamic token budget | ✅ Released |
| **v0.7** | File-type compression rules | ✅ Released |
| **v0.8** | Large repo optimization | ✅ Released |
| **v0.9** | **Session history & persistent memory** | ✅ Released |
| **v1.0** | API stabilization, community tools | ⚪ Planned |

---

## Links

- **GitHub**: <https://github.com/tsucky230/comP>
- **Issues**: <https://github.com/tsucky230/comP/issues>
- **Discussions**: <https://github.com/tsucky230/comP/discussions>
- **Architecture Docs**: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)
- **MCP Tools Ref**: [docs/user/MCP_TOOLS.md](docs/user/MCP_TOOLS.md)

---

## Support This Project

comP is free and open-source. If it helps you, consider supporting development:

- ☕ **[GitHub Sponsors](https://github.com/sponsors/tsucky230)** — Fund development
- 💖 **Star this repo** — Help others discover it

---

## Questions & Feedback

- 📖 **Docs**: [docs/](docs/)
- 🐛 **Report a bug**: [Create an Issue](https://github.com/tsucky230/comP/issues/new)
- 💬 **Discuss**: [Start a Discussion](https://github.com/tsucky230/comP/discussions/new)
- 👥 **Contribute**: [CONTRIBUTING.md](CONTRIBUTING.md)
