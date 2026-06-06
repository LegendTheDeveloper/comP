// AgentSetup - Generate MCP configuration for AI agents
//
// Responsibilities:
// 1. Generate config files for Claude Code, Cursor, Cline, Windsurf
// 2. Write MCP_SERVERS env var or config.json based on agent type
// 3. Provide UI for selecting agent and copying config

import * as path from "path";
import * as fs from "fs";
import * as os from "os";
import { DaemonManager } from "../daemon/DaemonManager";

interface AgentConfig {
  name: string;
  configPath: string;
  envVar?: string;
  template: (daemonPath: string) => string;
  command?: (daemonPath: string) => string;
  llmPrompt?: (daemonPath: string, configPath: string) => string;
}

export interface GenerateConfigResult {
  configPath: string;
  success: boolean;
  message: string;
  command?: string;
  llmPrompt?: string;
}

/**
 * AgentSetup - Generate MCP configuration files for various AI agents
 *
 * # Supported Agents
 * - Claude Code (claude_desktop_config.json)
 * - Cursor (.cursor/rules)
 * - Cline (.cline/config.json)
 * - Windsurf (.windsurf/config.json)
 * - GitHub Copilot (N/A - requires different approach)
 * - Continue.dev (.continue/config.py)
 *
 * # MCP Server Setup
 * - stdio: Spawn daemon as an MCP server subprocess
 * - Init parameters: workspace_root
 */
export class AgentSetupManager {
  private workspaceRoot: string;
  private extensionPath: string | undefined;

  constructor(_daemonManager: DaemonManager, workspaceRoot: string, extensionPath?: string) {
    // daemonManager reserved for future use (e.g., querying indexing status during config generation)
    this.workspaceRoot = workspaceRoot;
    this.extensionPath = extensionPath;
  }

  /**
   * Get configuration template for specific agent
   *
   * # Inputs
   * - agentName: "Claude Code", "Cursor", "Cline", "Windsurf", etc.
   *
   * # Outputs
   * - AgentConfig object (path, template)
   *
   * # Prerequisites
   * - The daemon binary exists in a known location
   */
  getAgentConfig(agentName: string): AgentConfig | null {

    switch (agentName) {
      case "Claude Code":
        return {
          name: "Claude Code",
          configPath: this.claudeDesktopConfigPath(),
          template: (path) => this.generateClaudeCodeConfig(path),
          command: (path) => this.generateClaudeCodeCommand(path),
          llmPrompt: (path) => this.generateClaudeCodeLLMPrompt(path),
        };

      case "Cursor":
        return {
          name: "Cursor",
          configPath: this.cursorConfigPath(),
          template: (path) => this.generateCursorConfig(path),
          llmPrompt: (_, configPath) => `以下のMCPサーバー設定ファイルが生成されました。このプロジェクトで comP を MCP サーバーとして利用できるように、あなたの設定を更新してください。\n設定ファイルパス: ${configPath}`
        };

      case "Cline":
        return {
          name: "Cline",
          configPath: this.clineConfigPath(),
          template: (path) => this.generateClineConfig(path),
          llmPrompt: (_, configPath) => `以下のMCPサーバー設定ファイルが生成されました。このプロジェクトで comP を MCP サーバーとして利用できるように、あなたの設定を更新してください。\n設定ファイルパス: ${configPath}`
        };

      case "Windsurf":
        return {
          name: "Windsurf",
          configPath: this.windsurfConfigPath(),
          template: (path) => this.generateWindsurfConfig(path),
          llmPrompt: (_, configPath) => `以下のMCPサーバー設定ファイルが生成されました。このプロジェクトで comP を MCP サーバーとして利用できるように、あなたの設定を更新してください。\n設定ファイルパス: ${configPath}`
        };

      case "Continue":
        return {
          name: "Continue",
          configPath: this.continueConfigPath(),
          template: (path) => this.generateContinueConfig(path),
          llmPrompt: (_, configPath) => `以下のMCPサーバー設定ファイルが生成されました。このプロジェクトで comP を MCP サーバーとして利用できるように、あなたの設定を更新してください。\n設定ファイルパス: ${configPath}`
        };

      case "Antigravity":
        return {
          name: "Antigravity",
          configPath: this.antigravityConfigPath(),
          template: (path) => this.generateAntigravityConfig(path),
        };

      case "GitHub Copilot":
        return {
          name: "GitHub Copilot",
          configPath: this.copilotConfigPath(),
          template: (path) => this.generateCopilotConfig(path),
        };

      default:
        return null;
    }
  }

  /**
   * Generate and write configuration for agent
   *
   * # Inputs
   * - agentName: The name of the agent
   *
   * # Outputs
   * - { configPath, success, message }
   *
   * # Prerequisites
   * - The user has file write permissions
   */
  async generateConfig(agentName: string): Promise<GenerateConfigResult> {
    const config = this.getAgentConfig(agentName);

    if (!config) {
      return {
        configPath: "",
        success: false,
        message: `Agent ${agentName} is not supported`,
      };
    }

    try {
      const daemonPath = this.getDaemonPath();
      const configContent = config.template(daemonPath);
      // WHY: Global configs like Antigravity return absolute paths so no join is required
      const fullPath = path.isAbsolute(config.configPath)
        ? config.configPath
        : path.join(this.workspaceRoot, config.configPath);

      // Create directories if needed
      const dir = path.dirname(fullPath);
      if (!fs.existsSync(dir)) {
        fs.mkdirSync(dir, { recursive: true });
      }

      // Write config file
      fs.writeFileSync(fullPath, configContent, "utf-8");

      const result: GenerateConfigResult = {
        configPath: fullPath,
        success: true,
        message: `MCP configuration created for ${agentName}`,
      };

      if (config.command) {
        result.command = config.command(daemonPath);
      }
      if (config.llmPrompt) {
        result.llmPrompt = config.llmPrompt(daemonPath, fullPath);
      }

      return result;
    } catch (error) {
      return {
        configPath: "",
        success: false,
        message: `Failed to generate config: ${error instanceof Error ? error.message : String(error)}`,
      };
    }
  }

  /**
   * Get daemon binary path for MCP stdio communication
   *
   * # Outputs
   * - Absolute path of the daemon executable
   *
   * # Precedence
   * 1. Development build: <workspaceRoot>/daemon/target/release/
   * 2. Bundled package: <workspaceRoot>/.comp/bin/
   */
  private getDaemonPath(): string {
    const binaryName = process.platform === "win32" ? "comp-daemon.exe" : "comp-daemon";
    const bundledBinaryName = process.platform === "win32" ? "comp-daemon-win.exe"
      : process.platform === "darwin" ? "comp-daemon-macos"
      : "comp-daemon-linux";

    // Development: cargo build output in workspace (default cargo output name)
    const devPath = path.join(this.workspaceRoot, "daemon", "target", "release", binaryName);
    if (fs.existsSync(devPath)) return devPath;

    // Extension: binary bundled with the installed extension (platform-specific name)
    if (this.extensionPath) {
      const extPath = path.join(this.extensionPath, "daemon", "target", "release", bundledBinaryName);
      if (fs.existsSync(extPath)) return extPath;
    }

    // Production: bundled binary in workspace .comp/bin
    return path.join(this.workspaceRoot, ".comp", "bin", binaryName);
  }

  /**
   * Generate Claude Code MCP configuration
   *
   * # Format
   * - claude_desktop_config.json in ~/Library/Application Support/Claude/ (macOS)
   * - $APPDATA/Claude/claude_desktop_config.json (Windows)
   * - ~/.config/Claude/claude_desktop_config.json (Linux)
   *
   * # MCP Server Setup
   * - command: daemon binary path
   * - args: (none)
   * - env: COMP_WORKSPACE_ROOT
   */
  private generateClaudeCodeConfig(daemonPath: string): string {
    const config = {
      mcpServers: {
        comp: {
          command: daemonPath,
          args: [],
          env: {
            COMP_WORKSPACE_ROOT: this.workspaceRoot,
            RUST_LOG: "info",
          },
        },
      },
    };

    return JSON.stringify(config, null, 2);
  }

  private generateClaudeCodeCommand(daemonPath: string): string {
    const escapeQuotes = (value: string) => value.replace(/"/g, '\\"');
    const escapedPath = escapeQuotes(daemonPath);
    const escapedWorkspaceRoot = escapeQuotes(this.workspaceRoot);

    return `claude mcp add comp "${escapedPath}" -e COMP_WORKSPACE_ROOT="${escapedWorkspaceRoot}" -e RUST_LOG=info`;
  }

  private generateClaudeCodeLLMPrompt(daemonPath: string): string {
    const command = this.generateClaudeCodeCommand(daemonPath);
    return `このワークスペースを Claude Code の MCP サーバーとして登録するには、次のコマンドをターミナルで実行してください。\n\n${command}`;
  }

  /**
   * Generate Cursor MCP configuration
   */
  private generateCursorConfig(daemonPath: string): string {
    // Cursor uses .cursor/rules or environment variables
    // Format: CURSOR_MCP_SERVERS={"comp":{"command":"...","env":{...}}}
    const mcpConfig = {
      comp: {
        command: daemonPath,
        args: [],
        env: {
          COMP_WORKSPACE_ROOT: this.workspaceRoot,
          RUST_LOG: "info",
        },
      },
    };

    return JSON.stringify(mcpConfig, null, 2);
  }

  /**
   * Generate Cline MCP configuration
   */
  private generateClineConfig(daemonPath: string): string {
    const config = {
      mcpServers: {
        comp: {
          command: daemonPath,
          args: [],
          env: {
            COMP_WORKSPACE_ROOT: this.workspaceRoot,
            RUST_LOG: "info",
          },
        },
      },
    };

    return JSON.stringify(config, null, 2);
  }

  /**
   * Generate Windsurf MCP configuration
   */
  private generateWindsurfConfig(daemonPath: string): string {
    const config = {
      mcpServers: {
        comp: {
          command: daemonPath,
          args: [],
          env: {
            COMP_WORKSPACE_ROOT: this.workspaceRoot,
            RUST_LOG: "info",
          },
        },
      },
    };

    return JSON.stringify(config, null, 2);
  }

  /**
   * Generate Continue.dev MCP configuration (Python-based)
   */
  private generateContinueConfig(daemonPath: string): string {
    const escapePy = (s: string) => s.replace(/\\/g, "\\\\").replace(/"/g, '\\"');
    const pythonConfig = `
# Continue.dev MCP Server Configuration
# Add this to your continue/config.py

mcp_servers = {
    "comp": {
        "command": "${escapePy(daemonPath)}",
        "args": [],
        "env": {
            "COMP_WORKSPACE_ROOT": "${escapePy(this.workspaceRoot)}",
            "RUST_LOG": "info"
        }
    }
}
`;

    return pythonConfig;
  }

  /**
   * Configuration file paths for each agent
   */
  private claudeDesktopConfigPath(): string {
    return ".comp/config/claude_desktop_config.json";
  }

  private cursorConfigPath(): string {
    return ".comp/config/cursor_config.json";
  }

  private clineConfigPath(): string {
    return ".comp/config/cline_config.json";
  }

  private windsurfConfigPath(): string {
    return ".comp/config/windsurf_config.json";
  }

  private continueConfigPath(): string {
    return ".comp/config/continue_config.py";
  }

  /**
   * Antigravity MCP config path
   *
   * Antigravity (Google Gemini-based IDE) stores global MCP config at
   * ~/.gemini/antigravity-ide/mcp_config.json — absolute path so generateConfig
   * writes directly without prepending workspaceRoot.
   */
  private antigravityConfigPath(): string {
    return path.join(os.homedir(), ".gemini", "antigravity-ide", "mcp_config.json");
  }

  /**
   * Generate Antigravity MCP configuration
   *
   * Merges comP into the existing mcp_config.json rather than overwriting,
   * to preserve other MCP servers the user may have configured.
   */
  private generateAntigravityConfig(daemonPath: string): string {
    let existing: { mcpServers: Record<string, unknown> } = { mcpServers: {} };

    try {
      const raw = fs.readFileSync(this.antigravityConfigPath(), "utf-8");
      const parsed = JSON.parse(raw);
      existing = { mcpServers: {}, ...parsed };
    } catch {
      // File absent or invalid — start from scratch
    }

    existing.mcpServers["comp"] = {
      command: daemonPath,
      args: [],
      env: {
        COMP_WORKSPACE_ROOT: this.workspaceRoot,
        RUST_LOG: "info",
      },
    };

    return JSON.stringify(existing, null, 2);
  }

  private copilotConfigPath(): string {
    return ".vscode/mcp.json";
  }

  /**
   * Generate GitHub Copilot MCP configuration
   *
   * Writes to workspace .vscode/mcp.json, merging with existing config if present.
   */
  private generateCopilotConfig(daemonPath: string): string {
    let existing: { servers: Record<string, unknown> } = { servers: {} };

    const fullPath = path.join(this.workspaceRoot, this.copilotConfigPath());
    try {
      if (fs.existsSync(fullPath)) {
        const raw = fs.readFileSync(fullPath, "utf-8");
        const parsed = JSON.parse(raw);
        existing = { servers: {}, ...parsed };
      }
    } catch {
      // File absent or invalid — start from scratch
    }

    existing.servers["comp"] = {
      command: daemonPath,
      args: [],
      env: {
        COMP_WORKSPACE_ROOT: this.workspaceRoot,
        RUST_LOG: "info",
      },
    };

    return JSON.stringify(existing, null, 2);
  }
}
