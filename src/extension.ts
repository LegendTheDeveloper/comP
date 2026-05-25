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

let watcherDisposable: vscode.Disposable | null = null;
let codeLensDisposable: vscode.Disposable | null = null;
let commandsRegistered = false;

/** Activation: called when extension is loaded */
export async function activate(context: vscode.ExtensionContext): Promise<void> {
  console.log("[comP] Extension activating...");

  try {
    const autoStartDaemon = hasCompDirectory();
    console.log(`[comP] .comp directory exists: ${autoStartDaemon}`);

    // 1. Sidebar panel (常時。manual モードでも Start ボタンで起動できるよう)
    sidebarPanel = SidebarPanel.createOrShow(context.extensionPath, null, context);
    context.subscriptions.push(
      vscode.window.registerWebviewViewProvider(SidebarPanel.viewType, sidebarPanel, {
        webviewOptions: { retainContextWhenHidden: true },
      })
    );

    // 2. ライフサイクルコールバックを SidebarPanel に注入
    // WHY: SidebarPanel が new DaemonManager() で独自に二重生成していた問題を排除。
    // Start/Stop ボタンの実体は extension.ts が一元管理する。
    sidebarPanel.setLifecycleCallbacks({
      onStartRequest: async () => {
        await startDaemonStack(context);
        return daemonManager;
      },
      onStopRequest: async () => {
        await stopDaemonStack();
      },
    });

    // 3. auto モード: .comp 存在時は即起動
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
 * Daemon + 周辺機能を一括起動する。
 * 二重起動を防ぐため既存 daemonManager がある場合は何もしない。
 */
async function startDaemonStack(context: vscode.ExtensionContext): Promise<void> {
  if (daemonManager?.isRunning()) {
    console.log("[comP] Daemon already running, skipping startDaemonStack");
    return;
  }

  daemonManager = new DaemonManager(context);

  if (!statusBar) {
    statusBar = new StatusBar();
  }
  statusBar.show("Initializing...");

  await daemonManager.start();

  sidebarPanel?.setDaemonManager(daemonManager);

  // commands は activate 時に 1 度だけ登録（VSCode は同じ ID の重複登録を禁止）
  if (!commandsRegistered) {
    registerCommands(context, daemonManager, statusBar);
    commandsRegistered = true;
  }

  // CodeLens / FileWatcher は restart で再生成必要なので毎回登録 → dispose 管理
  if (codeLensDisposable) {
    codeLensDisposable.dispose();
  }
  codeLensProvider = new DependencyCodeLensProvider(daemonManager);
  codeLensDisposable = vscode.languages.registerCodeLensProvider(
    ["typescript", "javascript", "python", "go", "rust", "java", "csharp"],
    codeLensProvider
  );
  context.subscriptions.push(codeLensDisposable);

  if (watcherDisposable) {
    watcherDisposable.dispose();
    watcherDisposable = null;
  }
  watcherDisposable = setupFileWatchers(context, daemonManager, codeLensProvider);

  statusBar.show("Ready");
}

/**
 * Daemon + 周辺機能を一括停止する。commands 登録は VSCode lifecycle で
 * 管理されるため dispose しない (deactivate まで保持)。
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
): vscode.Disposable | null {
  const config = vscode.workspace.getConfiguration("comp");
  const autoIndex = config.get<boolean>("autoIndex", true);

  if (!autoIndex) {
    console.log("[comP] Auto-indexing disabled");
    return null;
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
  return watcher;
}
