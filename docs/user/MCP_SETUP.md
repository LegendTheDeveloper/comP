# MCP Server Setup for Multiple Agents

comP runs as an MCP server, making it compatible with any MCP-capable AI agent. This guide covers setup for:

- **Claude Code** (local CLI)
- **GitHub Copilot** (VS Code extension)
- **Cursor** (editor)
- **Cline** (VS Code extension)
- **Antigravity**

---

## Prerequisites

1. Install comP from the VSCode Marketplace or build locally
2. Run the setup command from VSCode:

   ```
   Ctrl+Shift+P → "comP: Setup Agents"
   ```

3. This generates configuration files in `.comp/config/`

---

## Agent-Specific Setup

### Claude Code (Recommended)

**Windows (PowerShell)**:

```powershell
$configPath = "$env:APPDATA\Claude\claude_desktop_config.json"
```

**macOS/Linux**:

```bash
$configPath = ~/.claude/claude_desktop_config.json
```

Copy the contents of `.comp/config/claude_desktop_config.json` into the `mcpServers` section:

```json
{
  "mcpServers": {
    "comp": {
      "command": "/path/to/comp-daemon",
      "env": {
        "COMP_WORKSPACE_ROOT": "/path/to/workspace"
      }
    }
  }
}
```

Restart Claude Code. Run `@comp` in the chat to verify.

---

### GitHub Copilot (VSCode Extension)

GitHub Copilot in VSCode uses MCP via extension-level configuration. Run:

```
Ctrl+Shift+P → "comP: Setup Agents" → Select "GitHub Copilot"
```

This registers comP in VSCode settings. Copilot will automatically discover `run_pipeline` and related tools.

---

### Cursor

Cursor uses MCP similarly to Claude Code:

1. Open Cursor settings (`Cmd+,` or `Ctrl+,`)
2. Search for **"MCP"**
3. Paste the MCP server config from `.comp/config/claude_desktop_config.json`
4. Restart Cursor

---

### Cline (VSCode)

Cline discovers MCP servers from VSCode settings. No additional config needed after install.

To verify, check Cline's settings UI:

- Look for **"MCP Servers"** section
- comP should appear in the list

---

### Antigravity

Antigravity uses the Anthropic MCP manifest system. Setup:

1. Create `~/.claude/mcp-servers-manifest.json` (or update existing):

```json
{
  "servers": {
    "comp": {
      "command": "comp-daemon",
      "args": [],
      "env": {
        "COMP_WORKSPACE_ROOT": "/path/to/workspace"
      }
    }
  }
}
```

1. Ensure `comp-daemon` is in `$PATH`:

**Windows**:

```powershell
$env:PATH = "$env:PATH;C:\Users\YourName\AppData\Local\Programs\comp"
[Environment]::SetEnvironmentVariable("PATH", $env:PATH, "User")
```

**macOS/Linux**:

```bash
# Add to ~/.zshrc or ~/.bashrc
export PATH="/usr/local/bin/comp:$PATH"
```

1. Restart Antigravity. Run `@comp` to verify.

---

## Troubleshooting

### "MCP server not found"

- Verify `comp-daemon` binary exists at the configured path
- Check `COMP_WORKSPACE_ROOT` environment variable is set correctly
- Restart the agent application

### No tools appear in chat

- Run `comP: Force Re-index` to rebuild the index
- Check VSCode output panel (`View → Output → "comP"`) for errors
- Verify `.comp/index.db` exists in the workspace

### Token compression not working

- Update `comp.maxContextTokens` in VSCode settings (default: 8000)
- Run `run_pipeline` with increased `max_tokens` parameter

---

## Multi-Workspace Setup

To use comP in multiple workspaces:

1. Open each workspace in VSCode separately
2. Run `comP: Force Re-index` in each
3. Each workspace gets its own `.comp/index.db`
4. In Claude Code / Cursor, set different `COMP_WORKSPACE_ROOT` values per workspace

---

## What's Next?

- See [CONFIGURATION.md](./CONFIGURATION.md) for VSCode settings
- See [MCP_TOOLS.md](./MCP_TOOLS.md) for available MCP tools
- Check [GETTING_STARTED.md](./GETTING_STARTED.md) for usage tips
