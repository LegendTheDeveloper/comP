// VSCode Extension Entry Point - comP
//
// Responsibilities:
// 1. Activate on startup
// 2. Start Rust daemon (or connect if already running)
// 3. Register commands (Setup Agents, Force Re-index, Generate Context, Show Impact)
// 4. Set up status bar
// 5. Set up event listeners for auto-indexing

import * as vscode from "vscode";
import * as fs from "fs";
import * as path from "path";
import { DaemonManager } from "./daemon/DaemonManager";
import { StatusBar } from "./ui/StatusBar";
import { SidebarPanel } from "./ui/SidebarPanel";
import { DependencyCodeLensProvider } from "./ui/CodeLens";
import { registerCommands } from "./ui/commands";

/** Global context */
let daemonManager: DaemonManager | null = null;
let statusBar: StatusBar | null = null;
let sidebarPanel: SidebarPanel | null = null;
let codeLensProvider: DependencyCodeLensProvider | null = null;

// Check if .comp directory exists in workspace root
function hasCompDirectory(): boolean {
  const workspaceRoot = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
  if (!workspaceRoot) return false;

  const compPath = path.join(workspaceRoot, ".comp");
  return fs.existsSync(compPath);
}

/** Activation: called when extension is loaded */
export async function activate(context: vscode.ExtensionContext): Promise<void> {
  console.log("[comP] Extension activating...");

  try {
    // Check if .comp directory exists
    const autoStartDaemon = hasCompDirectory();
    console.log(`[comP] .comp directory exists: ${autoStartDaemon}`);

    // 1. Initialize sidebar panel (always, for manual control)
    sidebarPanel = SidebarPanel.createOrShow(context.extensionPath, null, context);
    context.subscriptions.push(sidebarPanel as any);

    // 2. Initialize daemon manager only if .comp exists
    if (autoStartDaemon) {
      daemonManager = new DaemonManager(context);

      // 2a. Initialize status bar
      statusBar = new StatusBar();
      statusBar.show("Initializing...");

      await daemonManager.start();

      // 2b. Update sidebar panel with daemon manager
      sidebarPanel?.setDaemonManager(daemonManager);

      // 4. Register commands
      registerCommands(context, daemonManager, statusBar);

      // 5. Register CodeLens provider
      codeLensProvider = new DependencyCodeLensProvider(daemonManager);
      context.subscriptions.push(
        vscode.languages.registerCodeLensProvider(
          ["typescript", "javascript", "python", "go", "rust", "java", "csharp"],
          codeLensProvider
        )
      );

      // 6. Set up file system watchers for auto-indexing
      setupFileWatchers(context, daemonManager, codeLensProvider);

      statusBar.show("Ready");
      console.log("[comP] Extension activated successfully (auto-mode)");
    } else {
      // No .comp directory - manual startup mode
      console.log("[comP] No .comp directory found - sidebar will show startup controls");
      console.log("[comP] Extension activated successfully (manual-mode)");
    }
  } catch (error) {
    console.error("[comP] Activation failed:", error);
    vscode.window.showErrorMessage(
      `comP failed to activate: ${error instanceof Error ? error.message : String(error)}`
    );
  }
}

/** Deactivation: called when extension is unloaded */
export async function deactivate(): Promise<void> {
  console.log("[comP] Extension deactivating...");

  if (daemonManager) {
    await daemonManager.stop();
  }

  if (statusBar) {
    statusBar.dispose();
  }

  if (codeLensProvider) {
    codeLensProvider.dispose();
  }
}

/** Setup file system watchers for auto-indexing */
function setupFileWatchers(
  context: vscode.ExtensionContext,
  daemonManager: DaemonManager,
  codeLensProvider: DependencyCodeLensProvider
): void {
  const config = vscode.workspace.getConfiguration("comp");
  const autoIndex = config.get<boolean>("autoIndex", true);

  if (!autoIndex) {
    console.log("[comP] Auto-indexing disabled");
    return;
  }

  // Debounce timer for rapid file changes
  let debounceTimer: NodeJS.Timeout | null = null;

  // Watch for file changes (only code files)
  const sourcePattern = "**/*.{ts,tsx,js,jsx,py,go,rs,java,cs,rb,php,sql,json,yaml,xml,md}";
  const watcher = vscode.workspace.createFileSystemWatcher(sourcePattern, false, false, false);

  watcher.onDidChange(async (uri) => {
    // Debounce rapid changes (wait 500ms after last change)
    if (debounceTimer) {
      clearTimeout(debounceTimer);
    }

    debounceTimer = setTimeout(async () => {
      try {
        await daemonManager.indexFile(uri.fsPath);
        // Invalidate CodeLens cache for this file
        codeLensProvider.invalidateFile(uri.fsPath);
        codeLensProvider.refresh();
      } catch (error) {
        console.error("[comP] Error indexing file:", error);
      }
    }, 500);
  });

  watcher.onDidDelete(async (uri) => {
    try {
      // Notify daemon that file was deleted
      await daemonManager.removeFile(uri.fsPath);
      // Invalidate CodeLens cache for deleted file
      codeLensProvider.invalidateFile(uri.fsPath);
      codeLensProvider.refresh();
    } catch (error) {
      console.error("[comP] Error removing file from index:", error);
    }
  });

  context.subscriptions.push(watcher);
  console.log("[comP] Auto-indexing enabled");
}
