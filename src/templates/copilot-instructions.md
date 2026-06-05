# comP for GitHub Copilot

When using GitHub Copilot in VSCode with comP, you can leverage the `@comp` participant and MCP tools like `run_pipeline` for smarter code context.

## Quick Start

1. **Install comP** from VSCode Marketplace
2. **Open a folder** in VSCode
3. **Run setup**:

```text
Ctrl+Shift+P → "comP: Setup Agents" → Select "GitHub Copilot"
```

1. **Use in Copilot chat**:

- `@comp what files handle authentication?` (searches codebase)
- `@comp generate context for login refactor` (token-compressed context)

## Using `run_pipeline` Tool

Copilot can call `run_pipeline` automatically for code-related questions:

**Example**:

```text
What would break if I rename UserService to AccountService?
```

Copilot will:

1. Call `run_pipeline` with your query
2. Get impact analysis and dependent files
3. Return analysis in the response

## Available Tools

- `run_pipeline` — Generate optimized context for a task (code/documentation)
- `get_context` — Search symbols by keyword
- `get_impact_graph` — Analyze change impact on dependent code
- `list_indexed_files` — View indexing status and statistics
- `get_file_summary` — Get AST-based file overview
- `get_symbol` — Fetch specific function/class definition with dependencies

## Compression Levels

By default, `run_pipeline` returns compressed code (comments removed, 60-80% smaller).
To see full source:

```text
@comp show full source for UserService with compression_level=0
```

## Best Practices

- **Indexing**: Let comP finish indexing before using tools (watch status bar)
- **Context**: Use `@comp` for multi-file questions; it's token-efficient
- **Refinement**: If results miss something, refine your query and try again
