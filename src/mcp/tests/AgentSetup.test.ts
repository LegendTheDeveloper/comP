// AgentSetup Tests
//
// Coverage:
// - Config generation for each agent
// - File path resolution
// - Error handling for unsupported agents
// - Directory creation for config files

import { expect } from "chai";
import * as path from "path";
import * as fs from "fs";
import { AgentSetupManager } from "../AgentSetup";
import { DaemonManager } from "../../daemon/DaemonManager";

// Mock DaemonManager
class MockDaemonManager implements Partial<DaemonManager> {
  async request(_method: string, _params?: unknown): Promise<unknown> {
    return {};
  }
}

describe("AgentSetupManager", () => {
  let manager: AgentSetupManager;
  let mockDaemon: MockDaemonManager;
  const testWorkspace = "/tmp/test-workspace";

  beforeEach(() => {
    mockDaemon = new MockDaemonManager();
    manager = new AgentSetupManager(mockDaemon as any, testWorkspace);
  });

  describe("getAgentConfig", () => {
    it("should return Claude Code config", () => {
      const config = manager.getAgentConfig("Claude Code");

      expect(config).to.exist;
      expect(config?.name).to.equal("Claude Code");
      expect(config?.configPath).to.equal(".mcp.json");
    });

    it("should return Cursor config", () => {
      const config = manager.getAgentConfig("Cursor");

      expect(config).to.exist;
      expect(config?.name).to.equal("Cursor");
      expect(config?.configPath).to.include("cursor_config.json");
    });

    it("should return Cline config", () => {
      const config = manager.getAgentConfig("Cline");

      expect(config).to.exist;
      expect(config?.name).to.equal("Cline");
      expect(config?.configPath).to.include("cline_config.json");
    });

    it("should return Windsurf config", () => {
      const config = manager.getAgentConfig("Windsurf");

      expect(config).to.exist;
      expect(config?.name).to.equal("Windsurf");
      expect(config?.configPath).to.include("windsurf_config.json");
    });

    it("should return Continue config", () => {
      const config = manager.getAgentConfig("Continue");

      expect(config).to.exist;
      expect(config?.name).to.equal("Continue");
      expect(config?.configPath).to.include("continue_config.py");
    });

    it("should return GitHub Copilot config", () => {
      const config = manager.getAgentConfig("GitHub Copilot");

      expect(config).to.exist;
      expect(config?.name).to.equal("GitHub Copilot");
      expect(config?.configPath).to.include("mcp.json");
    });

    it("should return null for unsupported agent", () => {
      const config = manager.getAgentConfig("UnsupportedAgent");

      expect(config).to.be.null;
    });
  });

  describe("generateConfig", () => {
    it("should generate Claude Code MCP configuration", async () => {
      const result = await manager.generateConfig("Claude Code");

      expect(result.success).to.be.true;
      expect(result.configPath).to.include(".mcp.json");
      expect(result.message).to.include("Claude Code");
      expect(result.command).to.exist;
      expect(result.command).to.include("claude mcp add comp");
      expect(result.command).to.include(`COMP_WORKSPACE_ROOT="${testWorkspace}"`);
      expect(result.llmPrompt).to.exist;
      expect(result.llmPrompt).to.include(".mcp.json");

      // Verify content is valid JSON
      const content = fs.readFileSync(result.configPath, "utf-8");
      const config = JSON.parse(content);
      expect(config.mcpServers).to.exist;
      expect(config.mcpServers.comp).to.exist;
    });

    it("should generate Cursor MCP configuration", async () => {
      const result = await manager.generateConfig("Cursor");

      expect(result.success).to.be.true;
      expect(result.configPath).to.include("cursor_config.json");
    });

    it("should generate Cline MCP configuration", async () => {
      const result = await manager.generateConfig("Cline");

      expect(result.success).to.be.true;
      expect(result.configPath).to.include("cline_config.json");
    });

    it("should generate Windsurf MCP configuration", async () => {
      const result = await manager.generateConfig("Windsurf");

      expect(result.success).to.be.true;
      expect(result.configPath).to.include("windsurf_config.json");
    });

    it("should generate GitHub Copilot MCP configuration", async () => {
      const result = await manager.generateConfig("GitHub Copilot");

      expect(result.success).to.be.true;
      expect(result.configPath).to.include("mcp.json");

      // Verify content
      const content = fs.readFileSync(result.configPath, "utf-8");
      const config = JSON.parse(content);
      expect(config.servers).to.exist;
      expect(config.servers.comp).to.exist;
    });

    it("should fail for unsupported agent", async () => {
      const result = await manager.generateConfig("UnsupportedAgent");

      expect(result.success).to.be.false;
      expect(result.message).to.include("not supported");
    });

    it("should create directory if it does not exist", async () => {
      const result = await manager.generateConfig("Claude Code");

      expect(result.success).to.be.true;
      const dir = path.dirname(result.configPath);
      expect(fs.existsSync(dir)).to.be.true;
    });
  });

  describe("config content validation", () => {
    it("Claude Code config should contain MCP servers object", async () => {
      const result = await manager.generateConfig("Claude Code");
      const content = fs.readFileSync(result.configPath, "utf-8");
      const config = JSON.parse(content);

      expect(config.mcpServers.comp.command).to.exist;
      expect(config.mcpServers.comp.env).to.exist;
      expect(config.mcpServers.comp.env.COMP_WORKSPACE_ROOT).to.equal(testWorkspace);
    });

    it("Cline config should contain MCP servers object", async () => {
      const result = await manager.generateConfig("Cline");
      const content = fs.readFileSync(result.configPath, "utf-8");
      const config = JSON.parse(content);

      expect(config.mcpServers).to.exist;
      expect(config.mcpServers.comp).to.exist;
    });

    it("Continue config should be Python-compatible", async () => {
      const result = await manager.generateConfig("Continue");
      const content = fs.readFileSync(result.configPath, "utf-8");

      // Should contain Python-like syntax
      expect(content).to.include("mcp_servers");
      expect(content).to.include("COMP_WORKSPACE_ROOT");
    });
  });
});

