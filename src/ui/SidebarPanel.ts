// SidebarPanel.ts - WebView-based sidebar for comP statistics dashboard
//
// Responsibilities:
// - Implement WebviewViewProvider to render in the activity bar sidebar
// - Handle messages from webview (start/stop/refresh/clearLogs)
// - Send index statistics to webview periodically
// - Manage daemon lifecycle from the sidebar UI
//
// WHY WebviewViewProvider instead of createWebviewPanel:
// package.json registers "comp-stats" under views/comp-explorer with "type": "webview"
// This requires WebviewViewProvider + registerWebviewViewProvider.
// createWebviewPanel opens in the editor area, not the sidebar.

import * as vscode from "vscode";
import * as path from "path";
import * as fs from "fs";
import { DaemonManager } from "../daemon/DaemonManager";
import { StatusBar } from "./StatusBar";
import { SessionMemoryManager } from "../mcp/sessionMemory";

/**
 * Sidebar panel - Implemented as a WebviewViewProvider.
 *
 * # Inputs
 * - context: VSCode ExtensionContext (used for loading version and creating DaemonManager)
 *
 * # Outputs
 * - resolveWebviewView: Called by VSCode to display the sidebar
 * - setDaemonManager: Called by extension.ts after starting the daemon
 */
/**
 * Lifecycle callbacks injected by extension.ts for starting/stopping the daemon.
 * WHY: Avoid duplicate creation of DaemonManager in SidebarPanel which caused inconsistency
 * with commands (e.g. forceReindex) calling an outdated instance. Creation is centralized in extension.ts.
 */
export interface DaemonLifecycleCallbacks {
  onStartRequest: () => Promise<DaemonManager | null>;
  onStopRequest: () => Promise<void>;
}

export class SidebarPanel implements vscode.WebviewViewProvider {
  public static readonly viewType = "comp-stats";
  private static instance: SidebarPanel | undefined;

  // WebviewView remains undefined until resolveWebviewView is called
  private view?: vscode.WebviewView;
  private daemonManager: DaemonManager | null = null;
  private statsInterval: NodeJS.Timeout | null = null;
  private logs: string[] = [];
  private maxLogs = 100;
  private version = "0.1.0";
  private lifecycleCallbacks: DaemonLifecycleCallbacks | null = null;
  // Track indexing state across polls so we can emit per-repo / finished log lines.
  private prevIndexing = false;
  private prevCurrentRepo: string | null = null;

  private constructor(context: vscode.ExtensionContext) {
    // Load version from package.json
    try {
      const pkgPath = path.join(context.extensionPath, "package.json");
      const pkg = JSON.parse(fs.readFileSync(pkgPath, "utf-8"));
      this.version = pkg.version || this.version;
    } catch {
      // Fallback
    }
  }

  /**
   * Initialization method called by extension.ts.
   * Maintains original signature for backward compatibility.
   */
  public static createOrShow(
    _extensionPath: string,
    _daemonManager: DaemonManager | null,
    context?: vscode.ExtensionContext
  ): SidebarPanel {
    if (!context) {
      throw new Error("ExtensionContext is required for SidebarPanel");
    }
    if (!SidebarPanel.instance) {
      SidebarPanel.instance = new SidebarPanel(context);
    }
    return SidebarPanel.instance;
  }

  /**
   * Called by VSCode when displaying the webview in the sidebar.
   *
   * # Actions
   * - Sets HTML content for the WebView
   * - Registers the message handlers
   * - Starts polling stats if the daemon is already running
   */
  public resolveWebviewView(
    webviewView: vscode.WebviewView,
    _context: vscode.WebviewViewResolveContext,
    _token: vscode.CancellationToken
  ): void {
    this.view = webviewView;

    webviewView.webview.options = { enableScripts: true };
    webviewView.webview.html = this.getHtml();

    webviewView.webview.onDidReceiveMessage((message) => {
      this.handleWebviewMessage(message);
    });

    // Refresh stats when the panel becomes visible
    webviewView.onDidChangeVisibility(() => {
      if (webviewView.visible) {
        this.refreshStats();
      }
    });

    this.addLog("Panel initialized");

    if (this.daemonManager) {
      this.startStatsRefresh();
    }
  }

  /**
   * Called by extension.ts after starting the daemon
   */
  public setDaemonManager(daemonManager: DaemonManager | null): void {
    this.daemonManager = daemonManager;
    if (daemonManager) {
      this.addLog("✓ Indexing started");
      this.startStatsRefresh();
      this.view?.webview.postMessage({ type: "daemonStatus", running: true });
    } else {
      if (this.statsInterval) {
        clearInterval(this.statsInterval);
        this.statsInterval = null;
      }
      this.view?.webview.postMessage({ type: "daemonStatus", running: false });
    }
  }

  /**
   * Injects lifecycle callbacks from extension.ts.
   * WHY: SidebarPanel should focus on UI only and delegate lifecycle management.
   */
  public setLifecycleCallbacks(callbacks: DaemonLifecycleCallbacks): void {
    this.lifecycleCallbacks = callbacks;
  }

  /**
   * Handles incoming messages from the WebView.
   *
   * # Branches
   * - "refresh": Refreshes statistics
   * - "startDaemon": Starts the daemon
   * - "stopDaemon": Stops the daemon
   * - "clearLogs": Clears UI logs
   */
  private async handleWebviewMessage(message: {
    command: string;
    [key: string]: unknown;
  }): Promise<void> {
    try {
      switch (message.command) {
        case "refresh":
          await this.refreshStats();
          break;
        case "startDaemon":
          await this.handleStartDaemon();
          break;
        case "stopDaemon":
          await this.handleStopDaemon();
          break;
        case "clearLogs":
          this.logs = [];
          this.sendLogsUpdate();
          break;
        case "reindex":
          await vscode.commands.executeCommand("comp.forceReindex");
          break;
        case "addRepo":
          await this.handleAddRepo();
          break;
        case "removeRepo":
          await this.handleRemoveRepo(String(message.alias ?? ""));
          break;
        default:
          console.warn("[comP] Unknown message command:", message.command);
      }
    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : String(error);
      this.addLog(`Error: ${errorMsg}`);
    }
  }

  private async handleStartDaemon(): Promise<void> {
    if (this.daemonManager?.isRunning()) {
      this.addLog("Daemon already running");
      return;
    }
    if (!this.lifecycleCallbacks) {
      this.addLog("✗ Lifecycle callbacks not configured");
      return;
    }

    try {
      this.addLog("Starting daemon...");
      const dm = await this.lifecycleCallbacks.onStartRequest();
      if (dm) {
        this.daemonManager = dm;
        this.addLog("✓ Daemon started successfully");
        this.startStatsRefresh();
        this.view?.webview.postMessage({ type: "daemonStatus", running: true });
      } else {
        this.addLog("✗ Daemon start returned null");
      }
    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : String(error);
      this.addLog(`✗ Failed to start daemon: ${errorMsg}`);
      this.daemonManager = null;
    }
  }

  private async handleStopDaemon(): Promise<void> {
    if (!this.daemonManager) {
      this.addLog("Daemon not running");
      return;
    }
    if (!this.lifecycleCallbacks) {
      this.addLog("✗ Lifecycle callbacks not configured");
      return;
    }

    try {
      this.addLog("Stopping daemon...");
      if (this.statsInterval) {
        clearInterval(this.statsInterval);
        this.statsInterval = null;
      }
      await this.lifecycleCallbacks.onStopRequest();
      this.daemonManager = null;
      this.addLog("Daemon stopped");
      this.view?.webview.postMessage({ type: "daemonStatus", running: false });
    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : String(error);
      this.addLog(`✗ Failed to stop daemon: ${errorMsg}`);
    }
  }

  /**
   * Prompts for a folder via the native OS picker and registers it as a new
   * repo. Indexing runs in the daemon's background; refreshStats() picks up
   * progress on the next poll (or immediately, since we trigger one here).
   */
  private async handleAddRepo(): Promise<void> {
    if (!this.daemonManager) {
      this.addLog("✗ Daemon not running");
      return;
    }

    const uris = await vscode.window.showOpenDialog({
      canSelectFolders: true,
      canSelectFiles: false,
      canSelectMany: false,
      openLabel: "Add Repository",
    });
    if (!uris || uris.length === 0) {
      return;
    }
    const repoPath = uris[0].fsPath;

    try {
      this.addLog(`Adding repo: ${repoPath}`);
      const result = await this.daemonManager.addRepo(repoPath);
      this.addLog(`✓ Repo added: ${result.alias} (indexing in background)`);
      await this.refreshStats();
    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : String(error);
      this.addLog(`✗ Failed to add repo: ${errorMsg}`);
    }
  }

  /**
   * Confirms with the user (deletion is not undoable short of re-adding and
   * re-indexing) then removes a repo and its indexed files/nodes/edges.
   */
  private async handleRemoveRepo(alias: string): Promise<void> {
    if (!alias) {
      return;
    }
    if (!this.daemonManager) {
      this.addLog("✗ Daemon not running");
      return;
    }

    const confirmed = await vscode.window.showWarningMessage(
      `Remove "${alias}" from the index? This deletes its indexed files, symbols, and dependencies.`,
      { modal: true },
      "Remove"
    );
    if (confirmed !== "Remove") {
      return;
    }

    try {
      this.addLog(`Removing repo: ${alias}`);
      const result = await this.daemonManager.removeRepo(alias);
      this.addLog(`✓ Repo removed: ${result.alias} (${result.removed_files} files)`);
      await this.refreshStats();
    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : String(error);
      this.addLog(`✗ Failed to remove repo: ${errorMsg}`);
    }
  }

  private async refreshStats(): Promise<void> {
    if (!this.view) {
      return;
    }

    if (!this.daemonManager) {
      this.view.webview.postMessage({ type: "daemonStatus", running: false });
      return;
    }

    try {
      const stats = await this.daemonManager.getStats();
      const efficiency: string = stats.efficiency || "0%";
      const tokensSaved: number = stats.tokens_saved || 0;
      const queriesCount: number = stats.queries_count || 0;

      // Update efficiency stats in the status bar
      StatusBar.instance?.updateStats(
        stats.total_nodes || 0,
        stats.total_files || 0,
        "Ready",
        efficiency
      );

      let lastAgentConnectionStr = "Waiting...";
      try {
        const workspaceRoot = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath || ".";
        const memoryManager = new SessionMemoryManager(workspaceRoot);
        const memory = memoryManager.load();
        let lastTimestamp = 0;
        for (const session of memory.sessions) {
          for (const call of session.calls) {
            if (call.timestamp > lastTimestamp) {
              lastTimestamp = call.timestamp;
            }
          }
        }
        if (lastTimestamp > 0) {
          lastAgentConnectionStr = new Date(lastTimestamp).toLocaleTimeString();
        }
      } catch (e) {
        // ignore file read errors
      }

      const isIndexing: boolean = stats.indexing?.is_indexing ?? false;
      const currentRepo: string | null = stats.indexing?.current_repo ?? null;
      // Emit per-repo / finished log lines into the sidebar Logs panel as the
      // daemon advances through repos (state polled every 5s).
      if (currentRepo && currentRepo !== this.prevCurrentRepo) {
        if (this.prevCurrentRepo) {
          this.addLog(`✓ Indexed ${this.prevCurrentRepo}`);
        }
        this.addLog(`⟳ Indexing ${currentRepo}…`);
      }
      if (this.prevIndexing && !isIndexing) {
        if (this.prevCurrentRepo) {
          this.addLog(`✓ Indexed ${this.prevCurrentRepo}`);
        }
        this.addLog("✓ Indexing finished");
      }
      this.prevIndexing = isIndexing;
      this.prevCurrentRepo = isIndexing ? currentRepo : null;

      // Recent searches recorded by the daemon. Non-critical: an error here
      // must not break the stats refresh.
      let recentSearches: Awaited<ReturnType<DaemonManager["getSearchHistory"]>> = [];
      try {
        recentSearches = await this.daemonManager.getSearchHistory(30);
      } catch (e) {
        console.debug("[comP] getSearchHistory failed", e);
      }

      this.view.webview.postMessage({
        type: "statsUpdate",
        data: {
          daemonRunning: true,
          totalFiles: stats.total_files || 0,
          totalNodes: stats.total_nodes || 0,
          totalEdges: stats.total_edges || 0,
          lastUpdated: new Date().toLocaleTimeString(),
          efficiency,
          tokensSaved,
          queriesCount,
          lastAgentConnection: lastAgentConnectionStr,
          repos: stats.repos || [],
          indexing: stats.indexing ? { isIndexing, currentRepo } : undefined,
          recentSearches,
        },
      });
    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : String(error);
      if (errorMsg.includes("timeout")) {
        console.debug("[comP] Stats timeout (daemon still indexing)");
        return;
      }
      this.view.webview.postMessage({
        type: "statsError",
        message: `Unable to fetch stats: ${errorMsg}`,
        daemonRunning: !!this.daemonManager,
      });
    }
  }

  private startStatsRefresh(): void {
    if (this.statsInterval) {
      clearInterval(this.statsInterval);
    }
    this.statsInterval = setInterval(() => this.refreshStats(), 5000);
    this.refreshStats();
  }

  private addLog(message: string): void {
    const timestamp = new Date().toLocaleTimeString();
    const logEntry = `[${timestamp}] ${message}`;
    this.logs.push(logEntry);
    if (this.logs.length > this.maxLogs) {
      this.logs.shift();
    }
    console.log(`[comP] ${logEntry}`);
    this.sendLogsUpdate();
  }

  private sendLogsUpdate(): void {
    this.view?.webview.postMessage({ type: "logsUpdate", logs: this.logs });
  }

  public dispose(): void {
    if (this.statsInterval) {
      clearInterval(this.statsInterval);
      this.statsInterval = null;
    }
  }

  private getHtml(): string {
    return `<!DOCTYPE html>
<html>
<head>
  <meta charset="UTF-8">
  <style>
    * { margin: 0; padding: 0; box-sizing: border-box; }
    body {
      font-family: var(--vscode-font-family);
      color: var(--vscode-foreground);
      background: var(--vscode-sideBar-background);
      padding: 12px;
      overflow-y: auto;
    }
    h2 {
      font-size: 13px;
      font-weight: 600;
      margin-bottom: 12px;
      border-bottom: 1px solid var(--vscode-sideBarSectionHeader-border, #555);
      padding-bottom: 6px;
    }
    .control-panel {
      margin-bottom: 12px;
      padding: 10px;
      background: var(--vscode-input-background);
      border: 1px solid var(--vscode-editorWidget-border);
      border-radius: 4px;
    }
    .button-group { display: flex; gap: 6px; margin-bottom: 8px; }
    button {
      flex: 1;
      padding: 6px 10px;
      background: var(--vscode-button-background);
      color: var(--vscode-button-foreground);
      border: none;
      border-radius: 3px;
      cursor: pointer;
      font-size: 11px;
      font-weight: 500;
    }
    button:hover { background: var(--vscode-button-hoverBackground); }
    button:disabled { opacity: 0.5; cursor: not-allowed; }
    .status-indicator { display: flex; align-items: center; gap: 6px; font-size: 11px; }
    .status-dot { width: 7px; height: 7px; border-radius: 50%; background: var(--vscode-errorForeground); flex-shrink: 0; }
    .status-dot.running { background: #4CAF50; }
    .status-dot.indexing { background: var(--vscode-terminal-ansiYellow, #e2c08d); animation: comp-pulse 1.2s ease-in-out infinite; }
    @keyframes comp-pulse { 0%, 100% { opacity: 1; } 50% { opacity: 0.25; } }
    #indexingIndicator { margin-top: 4px; }
    #indexingText { color: var(--vscode-terminal-ansiYellow, #e2c08d); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
    .stats-container { display: grid; grid-template-columns: repeat(2, 1fr); gap: 8px; margin-bottom: 12px; }
    .stat-item {
      padding: 10px 8px;
      background: var(--vscode-input-background);
      border: 1px solid var(--vscode-editorWidget-border);
      border-radius: 4px;
      text-align: center;
    }
    .stat-label { font-size: 10px; opacity: 0.7; text-transform: uppercase; margin-bottom: 4px; }
    .stat-value { font-size: 18px; font-weight: bold; color: var(--vscode-terminal-ansiBlue); }
    .repos-section {
      padding: 10px;
      background: var(--vscode-input-background);
      border: 1px solid var(--vscode-editorWidget-border);
      border-radius: 4px;
      margin-bottom: 12px;
    }
    .repos-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 6px; }
    .repos-header h3 { font-size: 11px; font-weight: 600; }
    .repos-header button { flex: 0; padding: 2px 8px; font-size: 10px; }
    .repos-content { max-height: 160px; overflow-y: auto; }
    .repo-row {
      display: flex;
      justify-content: space-between;
      align-items: center;
      gap: 6px;
      padding: 4px 2px;
      font-size: 11px;
      border-bottom: 1px solid var(--vscode-editorWidget-border);
    }
    .repo-row:last-child { border-bottom: none; }
    .repo-row.indexing {
      background: var(--vscode-list-activeSelectionBackground, rgba(226,192,141,0.14));
      border-left: 2px solid var(--vscode-terminal-ansiYellow, #e2c08d);
      padding-left: 4px;
      border-radius: 3px;
    }
    .repo-row.indexing .repo-alias { color: var(--vscode-terminal-ansiYellow, #e2c08d); font-weight: 600; }
    .repo-spinner {
      width: 10px; height: 10px; flex-shrink: 0;
      border: 2px solid var(--vscode-terminal-ansiYellow, #e2c08d);
      border-top-color: transparent;
      border-radius: 50%;
      animation: comp-spin 0.8s linear infinite;
    }
    @keyframes comp-spin { to { transform: rotate(360deg); } }
    .fork-badge {
      font-weight: normal;
      font-size: 11px;
      color: var(--vscode-terminal-ansiBlue, #00a8ff);
      opacity: 0.85;
    }
    .repo-alias {
      opacity: 0.9;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
      flex: 1;
      min-width: 0;
    }
    .repo-counts { flex-shrink: 0; color: var(--vscode-terminal-ansiBlue); font-weight: 600; }
    .repo-counts .repo-nodes { opacity: 0.6; font-weight: normal; margin-left: 4px; }
    .repo-remove-btn {
      flex: 0;
      padding: 0 5px;
      font-size: 11px;
      line-height: 16px;
      background: transparent;
      color: var(--vscode-errorForeground);
      border: 1px solid var(--vscode-editorWidget-border);
      border-radius: 3px;
    }
    .repo-remove-btn:hover { background: var(--vscode-inputValidation-errorBackground, rgba(255,0,0,0.15)); }
    .searches-section {
      padding: 10px;
      background: var(--vscode-input-background);
      border: 1px solid var(--vscode-editorWidget-border);
      border-radius: 4px;
      margin-bottom: 12px;
    }
    .searches-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 6px; }
    .searches-header h3 { font-size: 11px; font-weight: 600; }
    .searches-hint { font-size: 10px; opacity: 0.6; }
    .searches-content { max-height: 240px; overflow-y: auto; }
    .search-entry { border-bottom: 1px solid var(--vscode-editorWidget-border); }
    .search-entry:last-child { border-bottom: none; }
    .search-entry summary {
      display: flex;
      align-items: center;
      gap: 6px;
      padding: 4px 2px;
      font-size: 11px;
      cursor: pointer;
      list-style: none;
    }
    .search-entry summary::-webkit-details-marker { display: none; }
    .search-entry summary:hover { background: var(--vscode-list-hoverBackground, rgba(255,255,255,0.04)); }
    .conf-dot { width: 7px; height: 7px; border-radius: 50%; flex-shrink: 0; }
    .conf-high { background: var(--vscode-terminal-ansiGreen, #4caf50); }
    .conf-medium { background: var(--vscode-terminal-ansiYellow, #e2c08d); }
    .conf-low { background: var(--vscode-errorForeground, #f44747); }
    .conf-none { background: var(--vscode-descriptionForeground, #888); opacity: 0.5; }
    .search-query {
      flex: 1;
      min-width: 0;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
      opacity: 0.9;
    }
    .search-time { flex-shrink: 0; font-size: 10px; opacity: 0.55; }
    .search-weak-badge {
      flex-shrink: 0;
      font-size: 9px;
      padding: 0 4px;
      border-radius: 3px;
      color: var(--vscode-errorForeground);
      border: 1px solid var(--vscode-errorForeground);
      opacity: 0.85;
    }
    .search-details { padding: 2px 4px 6px 15px; font-size: 10px; }
    .search-meta { opacity: 0.65; margin-bottom: 3px; }
    .search-pivot {
      display: flex;
      justify-content: space-between;
      gap: 8px;
      padding: 1px 0;
      font-family: var(--vscode-editor-font-family, monospace);
    }
    .search-pivot-path {
      flex: 1;
      min-width: 0;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
      opacity: 0.85;
    }
    .search-pivot-score { flex-shrink: 0; color: var(--vscode-terminal-ansiBlue); }
    .logs-section {
      padding: 10px;
      background: var(--vscode-input-background);
      border: 1px solid var(--vscode-editorWidget-border);
      border-radius: 4px;
    }
    .logs-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 6px; }
    .logs-header h3 { font-size: 11px; font-weight: 600; }
    .logs-header button { flex: 0; padding: 2px 6px; font-size: 10px; }
    .logs-content {
      max-height: 120px;
      overflow-y: auto;
      font-family: var(--vscode-editor-font-family, monospace);
      font-size: 10px;
      background: var(--vscode-editor-background);
      padding: 6px;
      border-radius: 3px;
    }
    .log-entry { margin: 1px 0; opacity: 0.85; }
    .log-entry.error { color: var(--vscode-errorForeground); }
    .log-entry.info { color: var(--vscode-terminal-ansiBlue); }
  </style>
</head>
<body>
  <h2>comP <span class="fork-badge">v${this.version} · Legend Fork</span></h2>
  <div class="control-panel">
    <div class="button-group">
      <button id="startBtn" onclick="startDaemon()">▶ Start</button>
      <button id="stopBtn" onclick="stopDaemon()" disabled>■ Stop</button>
    </div>
    <div class="button-group">
      <button id="reindexBtn" onclick="reindex()" disabled>↺ Re-index</button>
    </div>
    <div class="status-indicator">
      <div class="status-dot" id="statusDot"></div>
      <span id="statusText">Daemon stopped</span>
    </div>
    <div class="status-indicator" id="indexingIndicator" style="display:none;">
      <div class="status-dot indexing"></div>
      <span id="indexingText">Indexing...</span>
    </div>
  </div>
  <div class="stats-container">
    <div class="stat-item"><div class="stat-label">Files</div><div class="stat-value" id="fileCount">--</div></div>
    <div class="stat-item"><div class="stat-label">Nodes</div><div class="stat-value" id="symbolCount">--</div></div>
    <div class="stat-item"><div class="stat-label">Edges</div><div class="stat-value" id="edgeCount">--</div></div>
    <div class="stat-item"><div class="stat-label">Updated</div><div class="stat-value" id="lastUpdated" style="font-size:11px;">--</div></div>
  </div>
  <div class="stats-container" style="margin-top:0;">
    <div class="stat-item" style="grid-column:span 2;">
      <div class="stat-label">Last Agent Connection</div>
      <div class="stat-value" id="lastAgentConnection" style="font-size:14px;color:var(--vscode-terminal-ansiMagenta);">Waiting...</div>
    </div>
    <div class="stat-item" style="grid-column:span 2;">
      <div class="stat-label">Compression Ratio</div>
      <div style="display:flex;align-items:baseline;justify-content:center;gap:8px;">
        <div class="stat-value" id="tokenEfficiency" style="color:var(--vscode-terminal-ansiGreen);">--</div>
        <div style="font-size:11px;opacity:0.7;" id="tokensSaved"></div>
      </div>
      <div style="font-size:10px;opacity:0.5;margin-top:2px;" id="queriesCount"></div>
    </div>
  </div>
  <div class="repos-section" id="reposSection" style="display:none;">
    <div class="repos-header">
      <h3>Repositories</h3>
      <button id="addRepoBtn" onclick="addRepo()">+ Add</button>
    </div>
    <div class="repos-content" id="reposContent"></div>
  </div>
  <div class="searches-section" id="searchesSection" style="display:none;">
    <div class="searches-header">
      <h3>Recent Searches</h3>
      <span class="searches-hint" id="searchesCount"></span>
    </div>
    <div class="searches-content" id="searchesContent"></div>
  </div>
  <div class="logs-section">
    <div class="logs-header"><h3>Logs</h3><button onclick="clearLogs()">Clear</button></div>
    <div class="logs-content" id="logsContent"><div class="log-entry">Initializing...</div></div>
  </div>
  <script>
    const vscode = acquireVsCodeApi();
    function startDaemon() { vscode.postMessage({ command: 'startDaemon' }); }
    function stopDaemon() { vscode.postMessage({ command: 'stopDaemon' }); }
    function clearLogs() { vscode.postMessage({ command: 'clearLogs' }); }
    function reindex() { vscode.postMessage({ command: 'reindex' }); }
    function addRepo() { vscode.postMessage({ command: 'addRepo' }); }
    function removeRepo(alias) { vscode.postMessage({ command: 'removeRepo', alias }); }
    window.addEventListener('message', (event) => {
      const msg = event.data;
      if (msg.type === 'statsUpdate') {
        const d = msg.data;
        document.getElementById('fileCount').textContent = d.totalFiles ?? '--';
        document.getElementById('symbolCount').textContent = d.totalNodes ?? '--';
        document.getElementById('edgeCount').textContent = d.totalEdges ?? '--';
        document.getElementById('lastUpdated').textContent = d.lastUpdated ?? '--';
        document.getElementById('tokenEfficiency').textContent = d.efficiency || '0%';
        if (document.getElementById('lastAgentConnection')) {
          document.getElementById('lastAgentConnection').textContent = d.lastAgentConnection || 'Waiting...';
        }
        const avgSaved = d.queriesCount > 0 ? Math.round(d.tokensSaved / d.queriesCount) : 0;
        const avgSavedStr = avgSaved > 1000 ? (avgSaved / 1000).toFixed(1) + 'K' : String(avgSaved);
        document.getElementById('tokensSaved').textContent = d.queriesCount > 0 ? '~' + avgSavedStr + ' tokens/query' : '';
        document.getElementById('queriesCount').textContent = 'vs full codebase · ' + (d.queriesCount || 0) + ' queries';
        const reposSection = document.getElementById('reposSection');
        const reposContent = document.getElementById('reposContent');
        if (d.repos && d.repos.length > 0) {
          reposSection.style.display = 'block';
          reposContent.innerHTML = '';
          const activeRepo = (d.indexing && d.indexing.isIndexing) ? d.indexing.currentRepo : null;
          d.repos.forEach(r => {
            const row = document.createElement('div');
            row.className = 'repo-row';
            const isActive = activeRepo && r.alias === activeRepo;
            if (isActive) {
              row.classList.add('indexing');
              const spinner = document.createElement('span');
              spinner.className = 'repo-spinner';
              spinner.title = 'Indexing…';
              row.appendChild(spinner);
            }
            const alias = document.createElement('span');
            alias.className = 'repo-alias';
            alias.textContent = r.alias + (r.is_root ? ' (root)' : '');
            alias.title = r.root_path || r.alias;
            const counts = document.createElement('span');
            counts.className = 'repo-counts';
            counts.innerHTML = r.files + ' files<span class="repo-nodes">' + r.nodes + ' nodes</span>';
            row.appendChild(alias);
            row.appendChild(counts);
            if (!r.is_root) {
              const removeBtn = document.createElement('button');
              removeBtn.className = 'repo-remove-btn';
              removeBtn.textContent = '×';
              removeBtn.title = 'Remove ' + r.alias;
              removeBtn.onclick = () => removeRepo(r.alias);
              row.appendChild(removeBtn);
            }
            reposContent.appendChild(row);
          });
        } else {
          reposSection.style.display = 'none';
        }
        renderRecentSearches(d.recentSearches || []);
        const indexingIndicator = document.getElementById('indexingIndicator');
        if (d.indexing && d.indexing.isIndexing) {
          indexingIndicator.style.display = 'flex';
          document.getElementById('indexingText').textContent =
            'Indexing' + (d.indexing.currentRepo ? ': ' + d.indexing.currentRepo : '...');
        } else {
          indexingIndicator.style.display = 'none';
        }
        if (d.daemonRunning) updateStatus(true);
      } else if (msg.type === 'daemonStatus') {
        updateStatus(msg.running);
      } else if (msg.type === 'logsUpdate') {
        const el = document.getElementById('logsContent');
        el.innerHTML = '';
        (msg.logs || []).forEach(log => {
          const div = document.createElement('div');
          div.className = 'log-entry';
          if (log.includes('✗') || log.includes('Error')) div.classList.add('error');
          else if (log.includes('✓') || log.includes('Starting')) div.classList.add('info');
          div.textContent = log;
          el.appendChild(div);
        });
        el.scrollTop = el.scrollHeight;
      } else if (msg.type === 'statsError') {
        const el = document.getElementById('logsContent');
        const div = document.createElement('div');
        div.className = 'log-entry error';
        div.textContent = '✗ ' + msg.message;
        el.appendChild(div);
      }
    });
    function formatAgo(unixSeconds) {
      if (!unixSeconds) return '';
      const s = Math.max(0, Math.floor(Date.now() / 1000) - unixSeconds);
      if (s < 60) return s + 's ago';
      if (s < 3600) return Math.floor(s / 60) + 'm ago';
      if (s < 86400) return Math.floor(s / 3600) + 'h ago';
      return Math.floor(s / 86400) + 'd ago';
    }
    function renderRecentSearches(searches) {
      const section = document.getElementById('searchesSection');
      const content = document.getElementById('searchesContent');
      if (!searches.length) {
        section.style.display = 'none';
        return;
      }
      section.style.display = 'block';
      document.getElementById('searchesCount').textContent = searches.length + ' recorded';
      // Keep expanded entries expanded across the 5s stats refresh.
      const openIds = new Set();
      content.querySelectorAll('details[open]').forEach(el => openIds.add(el.dataset.sid));
      const wasAtTop = content.scrollTop === 0;
      content.innerHTML = '';
      searches.forEach(s => {
        const details = document.createElement('details');
        details.className = 'search-entry';
        details.dataset.sid = s.timestamp + '|' + s.tool + '|' + s.query;
        if (openIds.has(details.dataset.sid)) details.open = true;

        const summary = document.createElement('summary');
        const dot = document.createElement('span');
        dot.className = 'conf-dot conf-' + (s.confidence || 'none');
        dot.title = s.confidence ? 'Confidence: ' + s.confidence : s.tool;
        const q = document.createElement('span');
        q.className = 'search-query';
        q.textContent = s.query;
        q.title = s.query;
        summary.appendChild(dot);
        summary.appendChild(q);
        if (s.weak_results) {
          const weak = document.createElement('span');
          weak.className = 'search-weak-badge';
          weak.textContent = 'weak';
          weak.title = 'No confident matches: the agent was told to fall back to its own search';
          summary.appendChild(weak);
        }
        const time = document.createElement('span');
        time.className = 'search-time';
        time.textContent = formatAgo(s.timestamp);
        summary.appendChild(time);
        details.appendChild(summary);

        const body = document.createElement('div');
        body.className = 'search-details';
        const meta = document.createElement('div');
        meta.className = 'search-meta';
        const parts = [s.tool];
        if (s.pivot_count != null) parts.push(s.pivot_count + ' pivots');
        if (s.dropped_low_relevance) parts.push(s.dropped_low_relevance + ' dropped');
        if (s.total_tokens != null) {
          parts.push((s.total_tokens > 1000 ? (s.total_tokens / 1000).toFixed(1) + 'K' : s.total_tokens) + ' tokens');
        }
        if (s.duration_ms != null) parts.push(s.duration_ms + ' ms');
        meta.textContent = parts.join(' · ');
        body.appendChild(meta);
        (s.top_pivots || []).forEach(p => {
          const row = document.createElement('div');
          row.className = 'search-pivot';
          const path = document.createElement('span');
          path.className = 'search-pivot-path';
          const fullPath = p.path || p.symbol || '';
          const segs = fullPath.split('/');
          path.textContent = segs.length > 2 ? '…/' + segs.slice(-2).join('/') : fullPath;
          path.title = fullPath + (p.reasons && p.reasons.length ? '  ::  ' + p.reasons.join(', ') : '');
          row.appendChild(path);
          if (typeof p.score === 'number') {
            const score = document.createElement('span');
            score.className = 'search-pivot-score';
            score.textContent = p.score.toFixed(2);
            score.title = 'Relevance score (per-query scale)';
            row.appendChild(score);
          }
          body.appendChild(row);
        });
        details.appendChild(body);
        content.appendChild(details);
      });
      if (wasAtTop) content.scrollTop = 0;
    }
    function updateStatus(running) {
      document.getElementById('statusDot').classList.toggle('running', running);
      document.getElementById('statusText').textContent = running ? 'Daemon running' : 'Daemon stopped';
      document.getElementById('startBtn').disabled = running;
      document.getElementById('stopBtn').disabled = !running;
      document.getElementById('reindexBtn').disabled = !running;
      document.getElementById('addRepoBtn').disabled = !running;
      if (!running) document.getElementById('indexingIndicator').style.display = 'none';
    }
    vscode.postMessage({ command: 'refresh' });
  </script>
</body>
</html>`;
  }
}
