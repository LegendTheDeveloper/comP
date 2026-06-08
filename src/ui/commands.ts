// Commands - Register VSCode commands for comP
//
// Commands:
// - comp.setupAgents: Configure MCP for AI agents
// - comp.forceReindex: Force complete re-indexing
// - comp.generateContext: Generate optimized context capsule
// - comp.showImpactGraph: Show impact analysis

import * as vscode from "vscode";
import * as fs from "fs";
import * as path from "path";
import { DaemonManager } from "../daemon/DaemonManager";
import { StatusBar } from "./StatusBar";
import { AgentSetupManager } from "../mcp/AgentSetup";

export function registerCommands(
  context: vscode.ExtensionContext,
  // WHY: A new DaemonManager is created on restart, so we accept a getter to get the latest
  // instance at invocation time instead of a static reference at registration.
  getDaemonManager: () => DaemonManager | null,
  statusBar: StatusBar
): void {
  const workspaceRoot = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath || ".";
  // _daemonManager is unused in AgentSetupManager (reserved for future expansion)
  const agentSetup = new AgentSetupManager(null as unknown as DaemonManager, workspaceRoot, context.extensionPath);

  // Command 1: comp.setupAgents
  // Setup MCP for AI agents (Claude Code, Cursor, Cline, etc.)
  context.subscriptions.push(
    vscode.commands.registerCommand("comp.setupAgents", async () => {
      const agents = ["Claude Code", "Cursor", "Cline", "Windsurf", "Continue", "Antigravity", "GitHub Copilot", "Aider"];
      const selected = await vscode.window.showQuickPick(agents, {
        placeHolder: "Select an AI agent to configure",
      });

      if (!selected) return;

      try {
        const result = await agentSetup.generateConfig(selected);

        if (result.success) {
          let mdContent = `# comP MCP Setup for ${selected}\n\n`;
          
          if (result.llmPrompt || result.command) {
            mdContent += `## 次の手順 (Next Steps)\n\n`;
            if (result.llmPrompt) {
              mdContent += `### LLM に設定を依頼する\n以下のプロンプトをコピーして、エージェントのチャット画面に貼り付けてください。\n\n\`\`\`text\n${result.llmPrompt}\n\`\`\`\n\n`;
            }
            if (result.command) {
              mdContent += `### ターミナルで設定する\n以下のコマンドをご自身のターミナルで実行してください。\n\n\`\`\`bash\n${result.command}\n\`\`\`\n\n`;
            }
          } else {
            mdContent += `設定は自動的に反映されました。\n\n`;
          }

          mdContent += `### 設定ファイルパス\n設定ファイルは以下の場所に生成されました:\n\`\`\`text\n${result.configPath}\n\`\`\`\n`;

          if (result.constitutionGuide) {
            const { llmInstruction } = result.constitutionGuide;
            mdContent += `\n---\n\n## comP を確実に使わせるための設定（推奨）\n\n`;
            mdContent += `以下のプロンプトをそのままエージェントのチャットに貼り付けてください。\n`;
            mdContent += `エージェントが設定ファイルへの追記を自動で行います。\n\n`;
            mdContent += `\`\`\`text\n${llmInstruction}\n\`\`\`\n`;
          }

          const doc = await vscode.workspace.openTextDocument({ content: mdContent, language: "markdown" });
          await vscode.window.showTextDocument(doc, { preview: false });
          vscode.window.showInformationMessage(`${selected} 向けの設定を生成しました。開かれたタブの手順に従ってください。`);
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
        statusBar.updateStats(stats.total_nodes, stats.total_files, "Ready", stats.efficiency || "0%");
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

  // Command 6: comp.exportDebugLog
  // Export session-memory.json to a user-chosen location
  context.subscriptions.push(
    vscode.commands.registerCommand("comp.exportDebugLog", async () => {
      const sessionMemoryPath = path.join(workspaceRoot, ".comp", "session-memory.json");

      if (!fs.existsSync(sessionMemoryPath)) {
        vscode.window.showWarningMessage(
          "No session memory found. Run a query via MCP first to generate logs."
        );
        return;
      }

      const choice = await vscode.window.showQuickPick(
        [
          { label: "Open in Editor", description: "View session-memory.json in a new tab" },
          { label: "Export to File", description: "Save a copy to a chosen location" },
        ],
        { placeHolder: "How do you want to view the debug log?" }
      );

      if (!choice) return;

      if (choice.label === "Open in Editor") {
        const uri = vscode.Uri.file(sessionMemoryPath);
        const doc = await vscode.workspace.openTextDocument(uri);
        await vscode.window.showTextDocument(doc, { preview: false });
        return;
      }

      // Export to file
      const defaultUri = vscode.Uri.file(
        path.join(workspaceRoot, `comp-debug-${Date.now()}.json`)
      );
      const saveUri = await vscode.window.showSaveDialog({
        defaultUri,
        filters: { "JSON": ["json"] },
        title: "Export comP Debug Log",
      });

      if (!saveUri) return;

      try {
        const content = fs.readFileSync(sessionMemoryPath, "utf-8");
        fs.writeFileSync(saveUri.fsPath, content, "utf-8");
        const openDoc = await vscode.window.showInformationMessage(
          `Debug log exported to ${path.basename(saveUri.fsPath)}`,
          "Open File"
        );
        if (openDoc === "Open File") {
          const doc = await vscode.workspace.openTextDocument(saveUri);
          await vscode.window.showTextDocument(doc, { preview: false });
        }
      } catch (error) {
        vscode.window.showErrorMessage(
          `Export failed: ${error instanceof Error ? error.message : String(error)}`
        );
      }
    })
  );

  // Command 7: comp.copyActiveFileCompressed
  // Copy current active file with AST compression
  context.subscriptions.push(
    vscode.commands.registerCommand("comp.copyActiveFileCompressed", async () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor) {
        vscode.window.showErrorMessage("No active editor. Open a file first.");
        return;
      }

      const dm = getDaemonManager();
      if (!dm?.isRunning()) {
        vscode.window.showErrorMessage("comP daemon is not running. Start it from the comP sidebar first.");
        return;
      }

      const levels = [
        { label: "Full Source", description: "No compression, copy full file", value: 0 },
        { label: "Compact", description: "Remove comments and empty lines", value: 1 },
        { label: "Skeleton", description: "Extract declarations only (signatures)", value: 2 }
      ];

      const selected = await vscode.window.showQuickPick(levels, {
        placeHolder: "Select compression level"
      });

      if (selected === undefined) return;

      const filePath = editor.document.uri.fsPath;
      try {
        await vscode.window.withProgress({
          location: vscode.ProgressLocation.Notification,
          title: "comP: Compressing file...",
          cancellable: false
        }, async () => {
          const { text, compressionRate } = await dm.compressFile(filePath, selected.value);
          await vscode.env.clipboard.writeText(text);
          vscode.window.showInformationMessage(`Copied to clipboard (${selected.label} mode, ${compressionRate} reduction).`);
        });
      } catch (error) {
        vscode.window.showErrorMessage(
          `Failed to compress file: ${error instanceof Error ? error.message : String(error)}`
        );
      }
    })
  );
}
