<p align="center">
  <img src="resources/comp-icon.png" width="128" height="128" alt="comP Logo">
</p>

# comP - Code Context Engine for AI Agents

**An open-source, local-first code indexing engine that provides AI coding agents with optimal context.**

🌐 **[Official Website](https://tsucky230.github.io/comP/)**

> 🚀 **LLMエージェントが忘れない開発パートナーに変わる**
>
> comPは、あなたの開発プロジェクトを「知的記憶」に変えます。

✨ **入力トークン 94% 削減** — LLMに渡すコンテキストを劇的に圧縮。
同じ質問でも、60行のファイルを1行に。$0.10のコストが$0.006に。

🧠 **セッション記憶で「3ヶ月後も前の決定を覚えてる」**
チャット履歴をBM25インデックス化—セッション横断で完全検索可能。
デーモン再起動後も自動で過去の文脈を復元。

🔒 **完全ローカル実行 — クラウドに何も上げない**
データはマシンの中だけ。セキュリティ懸念ゼロ。企業・個人情報も安全。

🤖 **あらゆるAIエージェントに対応**
Claude Code / Cursor / Cline / Antigravity / GitHub Copilot など、MCP互換なら全部対応。

🔗 **MAGATAMA と自動連携**
comP のインデックスを MAGATAMA がそのまま活用。
「何が壊れるか」「イディオムパターン」を 1/500 トークンで取得可能に。

---

## What It Does

- **📑 Code Indexing**: Automatically builds a searchable graph of your codebase using tree-sitter (30+ languages) with **concurrent background indexing** to prevent editor startup blocking.
- **🎯 Smart Context**: Provides AI agents with only the most relevant code, reducing input tokens by **94%**.
- **🔍 Impact Analysis**: Shows what code could break when you change a symbol.
- **📊 Token Counter**: Displays exactly how many tokens your context uses, and tracks **accumulated token savings and efficiency** across sessions.
- **🔍 BM25 Search**: Complements symbol graph traversal with full-text search capability for Markdown files.
- **🧠 Session Memory**: Chat history automatically indexed and cross-session searchable. Daemon restarts restore context automatically—*LLM standard features can't do this*.
- **🤝 MCP Integration**: Works with Claude Code, Cursor, Cline, and other AI agents via Model Context Protocol.
- **100% Local**: Everything runs on your machine—no cloud calls, no data sharing.

**Supported Languages (30+)**: C, C++, C#, Go, Java, JavaScript, TypeScript, Python, Rust, Ruby, Bash, Kotlin, Swift, PHP, Dart, Elixir, Haskell, Lua, R, Zig, SQL, HTML, CSS, YAML, Scala, and more.

---

## Prerequisites

- **OS**: Windows, macOS, or Linux
- **VS Code**: Version 1.85 or later
- **Rust** (for development only): 1.70+
- **Node.js** (for development only): 18+

---

## Installation

### From VS Code Marketplace

1. Open **VS Code**
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

# Open in VS Code (F5) to test
```

---

## Quick Start

1. **Install** the comP extension.
2. **Open a folder** in VS Code (e.g., a Git repository).
3. **Start the daemon**:
   - **First-time workspace**: Open the comP sidebar by clicking the **comP icon** in the Activity Bar (left edge), and click the **"Start comP" button** to manually trigger the initial indexing.
   - **Subsequent times (when `.comp/` directory exists)**: comP will **automatically start indexing** your code in the background upon opening the folder.
4. Watch the **status bar** to see indexing progress.
5. Run the setup command:

```text
Ctrl+Shift+P → "comP: Setup Agents"
```

Choose your AI agent. comP will open a new Markdown tab with step-by-step instructions (and an LLM prompt) to easily set up your agent. You can also verify the agent's connection status directly in the comP sidebar.

---

## Agent Setup

```text
Ctrl+Shift+P → "comP: Setup Agents"
```

Select your agent, and comP will generate the configuration file and detailed setup instructions (Markdown tab).

| Agent | Setup Method |
| --- | --- |
| **Claude Code** | Run generated `claude mcp add` command in terminal |
| **GitHub Copilot** | Auto-writes to `.vscode/mcp.json` — **no extra steps** |
| **Antigravity** | Auto-writes to MCP config — **no extra steps** |
| **Aider** | Auto-writes to `.aider.conf.yml` — **no extra steps** |
| **Cursor** | Copy generated `cursor_config.json` to `~/.cursor/mcp.json` |
| **Cline** | Paste generated `cline_config.json` into Cline MCP settings |
| **Windsurf** | Copy generated `windsurf_config.json` to `~/.codeium/windsurf/mcp_config.json` |
| **Continue.dev** | Add generated `continue_config.py` to `~/.continue/config.py` |

**For detailed per-agent instructions**, see [docs/user/MCP_SETUP.md](docs/user/MCP_SETUP.md).

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
| **Aider** | ✅ Supported | `.aider.conf.yml` auto-written to workspace root |
| **Gemini** | ❌ Not supported | No native MCP client support |

Any MCP 2024-11-05 compliant client should work. If your agent needs a specific config, [open an issue](https://github.com/tsucky230/comP/issues/new).

**For detailed multi-agent setup instructions**, see [docs/user/MCP_SETUP.md](docs/user/MCP_SETUP.md).

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
| **comP: Copy Active File Compressed** | Copy current active file with AST compression (comments removed or skeletonized) to clipboard |
| **comP: Export Debug Log** | Open or export `session-memory.json` to inspect debug session data |

### Status Bar

Bottom-left of VS Code shows:

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

### VS Code Chat Participant (@comp)

You can use the `@comp` assistant directly inside the VS Code Chat panel (e.g. Copilot Chat) to query LLMs while automatically skeletonizing large files:

```markdown
@comp Explain what this function does using #file:src/main.rs
```

Attached files are automatically skeletonized (comments removed, function bodies replaced with `{ ... }`) on the fly, allowing you to feed large files (like API specs or type definitions) without blowing up the token count.

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

Hidden directories (names starting with `.`, such as `.venv`, `.pytest_cache`) and `.gitignore`-matched
paths are excluded automatically. The following non-hidden directory names are also skipped by default:
`node_modules`, `venv`, `__pycache__`, `coverage`, `vendor`, `out`.

To add more directory names without editing `.comp/ignore`, set `comp.exclude` in VS Code settings
(e.g. `["env", "data"]`); the extension syncs it into `.comp/config.json` and the daemon applies it on
the next Force Re-index.

### Index Limits

Create `.comp/config.json` to control indexing behavior for large repositories:

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

| Option | Values | Default | Description |
| --- | --- | --- | --- |
| `max_nodes` | integer | `200000` | Threshold for the total node count |
| `on_limit_exceeded` | `"warn"` \| `"stop"` | `"warn"` | `warn`: notify and continue · `stop`: halt indexing |
| `default_budget_tokens` | integer | — | Token budget for `run_pipeline`; triggers auto compression level selection (0→1→2) |
| `compression_rules` | object | — | Per-extension compression level overrides. Glob pattern → level (0/1/2). Applied before auto-budget selection. |

> **Expected database size** (`.comp/index.db`): ~1–5 MB for small projects (~1k files), ~20–80 MB for medium (~10k files), ~200 MB–1 GB for large repositories (100k+ files). Only symbol metadata is stored—no raw file content.
>
> **Tip for monorepos**: Open only the relevant subdirectory in VS Code. comP indexes only the open workspace folder.

---

## How It Works

### Architecture

1. **Indexer (Rust daemon)**: Scans your workspace, parses code with tree-sitter, stores graph in SQLite
2. **Search Engine**: Finds relevant code using semantic search + graph traversal + BM25 complementary
3. **MCP Server**: Exposes tools to AI agents via Model Context Protocol
4. **VS Code Extension**: Manages the daemon, displays UI, handles user commands

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

## Security & Privacy

- **🔐 Complete Local Execution**: Code and chat history are never sent to the cloud.
- **🛡️ Data Protection**: The `.comp/` directory is automatically added to `.gitignore`—your workspace data never leaves your machine.
- **📋 Auditable**: All processing happens inside your machine—no external APIs, no telemetry, no data collection.
- **🏢 Enterprise-Ready**: Safe for confidential code. Works entirely on-premises with zero privacy concerns.

---

## Troubleshooting

### Issue: "comP is not indexing"

- Check the **Status Bar** (bottom-left) for progress.
- If stuck: **Ctrl+Shift+P** → "comP: Force Re-index"
- Check `.comp/` folder exists and `.gitignore` includes it.

### Issue: "MCP connection failed"

- Run **"comP: Setup Agents"** again.
- Verify your agent has the `.comp/mcp-config.json` file.
- Check VS Code Output panel (View → Output → "comP") for errors.

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
| **v0.3** | PDF (.pdf) support, advanced impact analysis (`max_depth`), TF-IDF search wired to `run_pipeline`, multi-path indexing, AST compression for `get_symbol` | ✅ **Released** |
| **v0.4** | `run_pipeline` content mode (`include_content`/`compression_level`), `get_git_diff_context` tool for PR review, enhanced `get_project_overview` with language distribution | ✅ **Released** |
| **v0.5** | Clipboard copy of compressed active file (`copyActiveFileCompressed`), `@comp` Chat Participant integration using VS Code Chat Participant API, and automatic indexing & BM25 search support for Parquet (.parquet) files. | ✅ **Released** |
| **v0.6** | Dynamic budget: `run_pipeline` reads `default_budget_tokens` from `.comp/config.json` and auto-selects compression level 0→1→2 to fit within budget. Response includes `compression_level_applied` and `budget_adjusted` flags. | ✅ **Released** |
| **v0.7** | Per-extension compression rules in `.comp/config.json` (e.g. keep Markdown at level 0, skeleton Rust at level 2). Aider agent support via `.aider.conf.yml`. New `comP: Export Debug Log` command. Token visualization state bug fixes. | ✅ **Released** |
| **v0.8** | Directory-walk overhaul (ripgrep `ignore` crate) that prunes `.venv`/`node_modules` subtrees so large Python repos no longer time out. `.comp/ignore` file, `comp.exclude` setting, 5 MiB file-size skip and large-workspace warning. `workspace_root` centralised in daemon state. | ✅ **Released** |
| **v0.9** | **Session history & memory**: `session_log` and `session_recall` MCP tools for persistent chat history across daemon restarts. Automatic transcript recording via Stop hook, full-text BM25 indexing of session logs, UserPromptSubmit hook auto-injection of recent conversation context for seamless context recovery across sessions. *LLM standard features cannot replicate this capability—only comP enables cross-session persistent memory with automatic context injection.* | ✅ **Released** |
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
  ongoing development
- 💖 **Star this repository** — Help others discover comP

Your support enables faster development, better maintenance, and new
features. Thank you! 🙏

---

## Questions?

- 📖 **Read the Docs**: [docs/](docs/)
- 🐛 **Found a Bug**: [Open an Issue](https://github.com/tsucky230/comP/issues/new)
- 💬 **Have Ideas**: [Start a Discussion](https://github.com/tsucky230/comP/discussions/new)
- 👥 **Want to Help**: [CONTRIBUTING.md](CONTRIBUTING.md)
