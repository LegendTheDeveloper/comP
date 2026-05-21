// AgentSetup - Generate MCP configuration for AI agents
//
// Responsibilities:
// 1. Generate config files for Claude Code, Cursor, Cline, Windsurf
// 2. Write MCP_SERVERS env var or config.json based on agent type
// 3. Provide UI for selecting agent and copying config

import * as path from "path";
import * as fs from "fs";
import { DaemonManager } from "../daemon/DaemonManager";

interface AgentConfig {
  name: string;
  configPath: string;
  envVar?: string;
  template: (daemonPath: string) => string;
}

/**
 * AgentSetup - Generate MCP configuration files for various AI agents
 *
 * # 対応エージェント
 * - Claude Code（claude_desktop_config.json）
 * - Cursor（.cursor/rules）
 * - Cline（.cline/config.json）
 * - Windsurf（.windsurf/config.json）
 * - GitHub Copilot（N/A - requires different approach）
 * - Continue.dev（.continue/config.py）
 *
 * # MCP サーバー構成
 * - stdio：daemon を MCP サーバーとしてサブプロセス起動
 * - 初期化パラメータ：workspace_root
 */
export class AgentSetupManager {
  private workspaceRoot: string;

  constructor(_daemonManager: DaemonManager, workspaceRoot: string) {
    // daemonManager reserved for future use (e.g., querying indexing status during config generation)
    this.workspaceRoot = workspaceRoot;
  }

  /**
   * Get configuration template for specific agent
   *
   * # 入力
   * - agentName: "Claude Code", "Cursor", "Cline", "Windsurf" 等
   *
   * # 出力
   * - AgentConfig オブジェクト（パス、テンプレート）
   *
   * # 前提条件
   * - daemon バイナリが既知の場所に存在
   */
  getAgentConfig(agentName: string): AgentConfig | null {

    switch (agentName) {
      case "Claude Code":
        return {
          name: "Claude Code",
          configPath: this.claudeDesktopConfigPath(),
          template: (path) => this.generateClaudeCodeConfig(path),
        };

      case "Cursor":
        return {
          name: "Cursor",
          configPath: this.cursorConfigPath(),
          template: (path) => this.generateCursorConfig(path),
        };

      case "Cline":
        return {
          name: "Cline",
          configPath: this.clineConfigPath(),
          template: (path) => this.generateClineConfig(path),
        };

      case "Windsurf":
        return {
          name: "Windsurf",
          configPath: this.windsurfConfigPath(),
          template: (path) => this.generateWindsurfConfig(path),
        };

      case "Continue":
        return {
          name: "Continue",
          configPath: this.continueConfigPath(),
          template: (path) => this.generateContinueConfig(path),
        };

      default:
        return null;
    }
  }

  /**
   * Generate and write configuration for agent
   *
   * # 入力
   * - agentName: エージェント名
   *
   * # 出力
   * - { configPath, success, message }
   *
   * # 前提条件
   * - ユーザーがファイル書き込み権限を持つ
   */
  async generateConfig(
    agentName: string
  ): Promise<{ configPath: string; success: boolean; message: string }> {
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
      const fullPath = path.join(this.workspaceRoot, config.configPath);

      // Create directories if needed
      const dir = path.dirname(fullPath);
      if (!fs.existsSync(dir)) {
        fs.mkdirSync(dir, { recursive: true });
      }

      // Write config file
      fs.writeFileSync(fullPath, configContent, "utf-8");

      return {
        configPath: fullPath,
        success: true,
        message: `MCP configuration created for ${agentName}`,
      };
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
   * # 出力
   * - daemon 実行ファイルの絶対パス
   */
  private getDaemonPath(): string {
    // TODO: DaemonManager から daemon パスを取得
    // 一時的には production bundled binary を想定
    const binaryName = process.platform === "win32" ? "comp-daemon.exe" : "comp-daemon";
    // Assuming daemon is in .comp/bin/
    return path.join(this.workspaceRoot, ".comp", "bin", binaryName);
  }

  /**
   * Generate Claude Code MCP configuration
   *
   * # フォーマット
   * - claude_desktop_config.json in ~/Library/Application Support/Claude/ (macOS)
   * - $APPDATA/Claude/claude_desktop_config.json (Windows)
   * - ~/.config/Claude/claude_desktop_config.json (Linux)
   *
   * # MCP サーバー設定
   * - command: daemon バイナリパス
   * - args: (なし)
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
    // Continue uses Python config
    const pythonConfig = `
# Continue.dev MCP Server Configuration
# Add this to your continue/config.py

mcp_servers = {
    "comp": {
        "command": "${daemonPath}",
        "args": [],
        "env": {
            "COMP_WORKSPACE_ROOT": "${this.workspaceRoot}",
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
}
