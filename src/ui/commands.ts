// Commands - Register VSCode commands for comP
//
// Commands:
// - comp.setupAgents: Configure MCP for AI agents
// - comp.forceReindex: Force complete re-indexing
// - comp.generateContext: Generate optimized context capsule
// - comp.showImpactGraph: Show impact analysis

import * as vscode from "vscode";
import { DaemonManager } from "../daemon/DaemonManager";
import { StatusBar } from "./StatusBar";
import { AgentSetupManager } from "../mcp/AgentSetup";

export function registerCommands(
  context: vscode.ExtensionContext,
  // WHY: 再起動のたびに新しい DaemonManager が生成されるため、登録時点の参照ではなく
  // 呼び出し時点の最新インスタンスを得るゲッターを受け取る。
  getDaemonManager: () => DaemonManager | null,
  statusBar: StatusBar
): void {
  const workspaceRoot = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath || ".";
  // _daemonManager は AgentSetupManager では未使用（将来の拡張用）
  const agentSetup = new AgentSetupManager(null as unknown as DaemonManager, workspaceRoot, context.extensionPath);

  // Command 1: comp.setupAgents
  // Setup MCP for AI agents (Claude Code, Cursor, Cline, etc.)
  context.subscriptions.push(
    vscode.commands.registerCommand("comp.setupAgents", async () => {
      const agents = ["Claude Code", "Cursor", "Cline", "Windsurf", "Continue", "Antigravity"];
      const selected = await vscode.window.showQuickPick(agents, {
        placeHolder: "Select an AI agent to configure",
      });

      if (!selected) return;

      try {
        const result = await agentSetup.generateConfig(selected);

        if (result.success) {
          vscode.window.showInformationMessage(result.message);
          // Ask user if they want to open the config file
          const openFile = await vscode.window.showInformationMessage(
            `Config file created at ${result.configPath}. Open it?`,
            "Open",
            "Done"
          );

          if (openFile === "Open") {
            const uri = vscode.Uri.file(result.configPath);
            await vscode.window.showTextDocument(uri);
          }
        } else {
          vscode.window.showErrorMessage(result.message);
        }
      } catch (error) {
        vscode.window.showErrorMessage(
          `Failed to setup MCP: ${error instanceof Error ? error.message : String(error)}`
        );
      }
    })
  );

  // Command 2: comp.forceReindex
  // Perform complete workspace re-indexing
  context.subscriptions.push(
    vscode.commands.registerCommand("comp.forceReindex", async () => {
      const proceed = await vscode.window.showWarningMessage(
        "This will re-index your entire workspace. Continue?",
        "Yes",
        "Cancel"
      );

      if (proceed !== "Yes") return;

      const dm = getDaemonManager();
      if (!dm?.isRunning()) {
        vscode.window.showErrorMessage("comP daemon is not running. Start it from the comP sidebar first.");
        return;
      }
      statusBar.show("Indexing...");
      try {
        await dm.request("forceReindex");
        const stats = await dm.getStats();
        statusBar.updateStats(stats.total_nodes, stats.total_files, "Ready");
        vscode.window.showInformationMessage(`Re-indexing completed: ${stats.total_nodes} symbols found`);
      } catch (error) {
        statusBar.show("Error");
        vscode.window.showErrorMessage(
          `Re-indexing failed: ${error instanceof Error ? error.message : String(error)}`
        );
      }
    })
  );

  // Command 3: comp.generateContext
  // Generate optimized context for current task
  context.subscriptions.push(
    vscode.commands.registerCommand("comp.generateContext", async () => {
      const task = await vscode.window.showInputBox({
        prompt: "Describe what you're working on",
        placeHolder: "e.g., add user authentication",
      });

      if (!task) return;

      vscode.window.showInformationMessage("Generating optimized context...");
      // TODO: Call daemon run_pipeline tool
      // TODO: Show results in output panel or new editor
    })
  );

  // Command 4: comp.showImpactGraph
  // Show impact analysis for symbol at cursor
  context.subscriptions.push(
    vscode.commands.registerCommand("comp.showImpactGraph", async () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor) {
        vscode.window.showErrorMessage("No active editor");
        return;
      }

      const document = editor.document;
      const position = editor.selection.active;

      vscode.window.showInformationMessage(
        `Impact graph for: ${document.fileName}:${position.line}:${position.character}`
      );
      // TODO: Query daemon for impact graph at position
      // TODO: Display results
    })
  );

  // Command 5: comp.showStats (internal)
  // Show index statistics dashboard
  context.subscriptions.push(
    vscode.commands.registerCommand("comp.showStats", async () => {
      const dm = getDaemonManager();
      try {
        if (!dm?.isRunning()) throw new Error("Daemon is not running");
        const stats = await dm.getStats();
        const message = `comP Statistics\n\nFiles: ${stats.total_files}\nSymbols: ${stats.total_nodes}\nDependencies: ${stats.total_edges}`;
        vscode.window.showInformationMessage(message);
      } catch (error) {
        vscode.window.showErrorMessage(`Failed to fetch stats: ${error instanceof Error ? error.message : String(error)}`);
      }
    })
  );
}
