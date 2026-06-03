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
import { SessionMemoryManager } from "./mcp/sessionMemory";
import { registerChatParticipant } from "./mcp/chatParticipant";

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

let watcherDisposable: vscode.Disposable | null = null;
let codeLensDisposable: vscode.Disposable | null = null;


/** Activation: called when extension is loaded */
export async function activate(context: vscode.ExtensionContext): Promise<void> {
  console.log("[comP] Extension activating...");

  try {
    const autoStartDaemon = hasCompDirectory();
    console.log(`[comP] .comp directory exists: ${autoStartDaemon}`);

    // 1. Sidebar panel (Always active so that start button is available even in manual mode)
    sidebarPanel = SidebarPanel.createOrShow(context.extensionPath, null, context);
    context.subscriptions.push(
      vscode.window.registerWebviewViewProvider(SidebarPanel.viewType, sidebarPanel, {
        webviewOptions: { retainContextWhenHidden: true },
      })
    );

    // 2. Always register the StatusBar and commands.
    // WHY: To prevent "command not found" errors on comp.showStats when daemon fails to start,
    // we register them exactly once in activate() regardless of startDaemonStack success.
    statusBar = new StatusBar();
    statusBar.show("Stopped");
    context.subscriptions.push({ dispose: () => statusBar?.dispose() });
    registerCommands(context, () => daemonManager, statusBar);
    registerChatParticipant(context, () => daemonManager);

    // 3. Inject lifecycle callbacks into SidebarPanel.
    // WHY: Prevent duplicate DaemonManager creation within SidebarPanel.
    // The actual start/stop execution is centralized in extension.ts.
    sidebarPanel.setLifecycleCallbacks({
      onStartRequest: async () => {
        await startDaemonStack(context);
        return daemonManager;
      },
      onStopRequest: async () => {
        await stopDaemonStack();
      },
    });

    // 4. Auto mode: immediately start if .comp directory exists
    if (autoStartDaemon) {
      await startDaemonStack(context);
      console.log("[comP] Extension activated successfully (auto-mode)");
    } else {
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

/**
 * Starts the daemon and helper services.
 * Skips starting if a running instance already exists to prevent duplicate processes.
 */
async function startDaemonStack(context: vscode.ExtensionContext): Promise<void> {
  if (daemonManager?.isRunning()) {
    console.log("[comP] Daemon already running, skipping startDaemonStack");
    return;
  }

  daemonManager = new DaemonManager(context);

  statusBar?.show("Initializing...");

  try {
    await daemonManager.start();
  } catch (error) {
    statusBar?.show("Error");
    throw error;
  }

  sidebarPanel?.setDaemonManager(daemonManager);

  // CodeLens and FileWatcher need to be re-created on restart -> manage via dispose
  if (codeLensDisposable) {
    codeLensDisposable.dispose();
  }
  codeLensProvider = new DependencyCodeLensProvider(daemonManager);
  codeLensDisposable = vscode.languages.registerCodeLensProvider(
    ["typescript", "javascript", "python", "go", "rust", "java", "csharp"],
    codeLensProvider
  );
  // WHY: Do not push to context.subscriptions since startDaemonStack is called on restarts.
  // Pushing would accumulate obsolete entries, leading to duplicate dispose errors during deactivation.
  // Instead, dispose manually during deactivate().

  if (watcherDisposable) {
    watcherDisposable.dispose();
    watcherDisposable = null;
  }
  watcherDisposable = setupFileWatchers(context, daemonManager, codeLensProvider);

  statusBar?.show("Ready");
}

/**
 * Stops the daemon and helper services.
 * Commands registered in activate() are not disposed here to keep them registered.
 */
async function stopDaemonStack(): Promise<void> {
  if (watcherDisposable) {
    watcherDisposable.dispose();
    watcherDisposable = null;
  }
  if (codeLensDisposable) {
    codeLensDisposable.dispose();
    codeLensDisposable = null;
  }
  codeLensProvider = null;

  if (daemonManager) {
    await daemonManager.stop();
    daemonManager = null;
  }

  sidebarPanel?.setDaemonManager(null);
  statusBar?.show("Stopped");
}

/** Deactivation: called when extension is unloaded */
export async function deactivate(): Promise<void> {
  console.log("[comP] Extension deactivating...");

  sidebarPanel?.dispose();

  if (daemonManager) {
    await daemonManager.stop();
  }

  if (statusBar) {
    statusBar.dispose();
  }

  if (codeLensDisposable) {
    codeLensDisposable.dispose();
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
): vscode.Disposable | null {
  const config = vscode.workspace.getConfiguration("comp");
  const autoIndex = config.get<boolean>("autoIndex", true);

  if (!autoIndex) {
    console.log("[comP] Auto-indexing disabled");
    return null;
  }

  const workspaceRoot = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
  const sessionMemoryManager = workspaceRoot ? new SessionMemoryManager(workspaceRoot) : null;

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
      debounceTimer = null;
      try {
        await daemonManager.indexFile(uri.fsPath);
        // Invalidate CodeLens cache for this file
        codeLensProvider.invalidateFile(uri.fsPath);
        codeLensProvider.refresh();

        // Mark session memory entries as stale if they depend on this file
        if (sessionMemoryManager) {
          const relativePath = vscode.workspace.asRelativePath(uri, false);
          sessionMemoryManager.markStaleForFile(relativePath);
        }
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

      // Mark session memory entries as stale if they depend on this file
      if (sessionMemoryManager) {
        const relativePath = vscode.workspace.asRelativePath(uri, false);
        sessionMemoryManager.markStaleForFile(relativePath);
      }
    } catch (error) {
      console.error("[comP] Error removing file from index:", error);
    }
  });

  // WHY: Return a composite disposable to ensure pending debounce timers are cleared when watcher is disposed.
  const disposable: vscode.Disposable = {
    dispose: () => {
      if (debounceTimer) {
        clearTimeout(debounceTimer);
        debounceTimer = null;
      }
      watcher.dispose();
    },
  };
  context.subscriptions.push(disposable);
  console.log("[comP] Auto-indexing enabled");
  return disposable;
}
