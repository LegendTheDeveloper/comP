// SidebarPanel.ts - WebView-based sidebar for comP statistics dashboard
//
// Responsibilities:
// - Create and manage WebView panel
// - Load HTML/CSS/JS resources
// - Handle messages from webview (data requests)
// - Send index statistics to webview
// - Update stats in real-time as indexing progresses
//
// Architecture:
// Extension (main.ts)
//   ↓
// SidebarPanel (this file)
//   ↓ HTML/CSS/JS
// WebView (stats.html)
//   ↑↓ Message passing
// DaemonManager (IPC to Rust daemon)

import * as vscode from "vscode";
import * as path from "path";
import * as fs from "fs";
import { DaemonManager } from "../daemon/DaemonManager";


/**
 * Sidebar panel controller
 */
export class SidebarPanel {
  public static readonly viewType = "comp-stats";
  private static instance: SidebarPanel | undefined;

  private readonly webviewPanel: vscode.WebviewPanel;
  private daemonManager: DaemonManager | null = null;
  private extensionContext: vscode.ExtensionContext | null = null;
  private statsInterval: NodeJS.Timeout | null = null;
  private logs: string[] = [];
  private maxLogs = 100;

  /**
   * Create or show the sidebar panel
   */
  public static createOrShow(extensionPath: string, daemonManager: DaemonManager | null, context?: vscode.ExtensionContext): SidebarPanel {
    const column = vscode.ViewColumn.One;

    // Get version from package.json
    let version = "0.1.0";
    try {
      if (context) {
        const pkgPath = path.join(context.extensionPath, "package.json");
        const pkg = JSON.parse(fs.readFileSync(pkgPath, "utf-8"));
        version = pkg.version || version;
      }
    } catch {
      // Use default version if unable to read
    }

    // If we already have a panel, show it
    if (SidebarPanel.instance) {
      SidebarPanel.instance.webviewPanel.reveal(column);
      if (context && !SidebarPanel.instance.extensionContext) {
        SidebarPanel.instance.extensionContext = context;
      }
      return SidebarPanel.instance;
    }

    // Create new panel with version
    const webviewPanel = vscode.window.createWebviewPanel(
      SidebarPanel.viewType,
      `comP Statistics (v${version})`,
      column,
      {
        enableScripts: true,
        retainContextWhenHidden: true,
        localResourceRoots: [vscode.Uri.file(path.join(extensionPath, "src", "webview"))],
      }
    );

    SidebarPanel.instance = new SidebarPanel(webviewPanel, daemonManager, context || undefined);
    return SidebarPanel.instance;
  }

  /**
   * Set daemon manager after initialization (for manual startup mode)
   */
  public setDaemonManager(daemonManager: DaemonManager): void {
    this.daemonManager = daemonManager;
    this.startStatsRefresh();
  }

  /**
   * Constructor
   */
  private constructor(
    webviewPanel: vscode.WebviewPanel,
    daemonManager: DaemonManager | null,
    context?: vscode.ExtensionContext
  ) {
    this.webviewPanel = webviewPanel;
    this.daemonManager = daemonManager;
    this.extensionContext = context || null;

    // Load webview content
    this.update();

    // Handle disposal
    this.webviewPanel.onDidDispose(() => {
      SidebarPanel.instance = undefined;
      if (this.statsInterval) {
        clearInterval(this.statsInterval);
      }
    });

    // Handle messages from webview
    this.webviewPanel.webview.onDidReceiveMessage(
      (message) => {
        this.handleWebviewMessage(message);
      },
      undefined,
      undefined
    );

    // Start stats refresh if daemon is already available
    if (this.daemonManager) {
      this.startStatsRefresh();
    }

    this.addLog("Panel initialized");
  }

  /**
   * Start periodic stats refresh
   */
  private startStatsRefresh(): void {
    if (this.statsInterval) {
      clearInterval(this.statsInterval);
    }

    // Update stats periodically (every 5 seconds)
    this.statsInterval = setInterval(() => {
      this.refreshStats();
    }, 5000);

    // Initial refresh
    this.refreshStats();
  }

  /**
   * Handle messages from webview
   *
   * Message types:
   * - "refresh": Request stats update
   * - "startDaemon": Start daemon process
   * - "stopDaemon": Stop daemon process
   * - "clearLogs": Clear log buffer
   */
  private async handleWebviewMessage(message: { command: string; [key: string]: unknown }): Promise<void> {
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
        default:
          console.warn("[comP] Unknown message command:", message.command);
      }
    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : String(error);
      console.error("[comP] Error handling webview message:", errorMsg);
      this.addLog(`Error: ${errorMsg}`);
    }
  }

  /**
   * Handle start daemon command
   */
  private async handleStartDaemon(): Promise<void> {
    if (this.daemonManager) {
      this.addLog("Daemon already running");
      return;
    }

    if (!this.extensionContext) {
      this.addLog("Error: No extension context available");
      return;
    }

    try {
      this.addLog("Starting daemon...");
      // Create new daemon manager and start
      this.daemonManager = new DaemonManager(this.extensionContext);
      await this.daemonManager.start();
      this.addLog("Daemon started successfully");
      this.startStatsRefresh();

      // Notify webview that daemon is running
      this.webviewPanel.webview.postMessage({
        type: "daemonStatus",
        running: true,
      });
    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : String(error);
      this.addLog(`Failed to start daemon: ${errorMsg}`);
      throw error;
    }
  }

  /**
   * Handle stop daemon command
   */
  private async handleStopDaemon(): Promise<void> {
    if (!this.daemonManager) {
      this.addLog("Daemon not running");
      return;
    }

    try {
      this.addLog("Stopping daemon...");
      if (this.statsInterval) {
        clearInterval(this.statsInterval);
        this.statsInterval = null;
      }
      await this.daemonManager.stop();
      this.daemonManager = null;
      this.addLog("Daemon stopped");

      // Notify webview that daemon is stopped
      this.webviewPanel.webview.postMessage({
        type: "daemonStatus",
        running: false,
      });
    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : String(error);
      this.addLog(`Failed to stop daemon: ${errorMsg}`);
      throw error;
    }
  }

  /**
   * Add a log message
   */
  private addLog(message: string): void {
    const timestamp = new Date().toLocaleTimeString();
    const logEntry = `[${timestamp}] ${message}`;
    this.logs.push(logEntry);

    // Keep only last N logs
    if (this.logs.length > this.maxLogs) {
      this.logs.shift();
    }

    console.log(`[comP] ${logEntry}`);
    this.sendLogsUpdate();
  }

  /**
   * Send logs update to webview
   */
  private sendLogsUpdate(): void {
    this.webviewPanel.webview.postMessage({
      type: "logsUpdate",
      logs: this.logs,
    });
  }

  /**
   * Load and display webview content
   */
  private update(): void {
    // Load HTML from placeholder (can be extended to load from file)
    this.webviewPanel.webview.html = this.getPlaceholderHtml();
  }

  /**
   * Refresh statistics from daemon
   */
  private async refreshStats(): Promise<void> {
    if (!this.daemonManager) {
      this.webviewPanel.webview.postMessage({
        type: "daemonStatus",
        running: false,
      });
      return;
    }

    try {
      const stats = await this.daemonManager.getStats();

      // Update webview with stats
      this.webviewPanel.webview.postMessage({
        type: "statsUpdate",
        data: {
          daemonRunning: true,
          totalFiles: stats.total_files || 0,
          totalNodes: stats.total_nodes || 0,
          totalEdges: stats.total_edges || 0,
          lastUpdated: new Date().toLocaleTimeString(),
        },
      });
    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : String(error);

      // Silently log timeout errors - daemon is likely still indexing
      if (errorMsg.includes("timeout")) {
        console.debug("[comP] Stats timeout (daemon still indexing)");
        // Don't send error to UI, keep showing last known values
        return;
      }

      // Log other errors
      console.debug("[comP] Stats fetch error:", errorMsg);

      // Only show error for non-timeout errors
      this.webviewPanel.webview.postMessage({
        type: "statsError",
        message: `Unable to fetch stats: ${errorMsg}`,
        daemonRunning: !!this.daemonManager,
      });
    }
  }

  /**
   * Get placeholder HTML while webview loads
   */
  private getPlaceholderHtml(): string {
    return `
      <!DOCTYPE html>
      <html>
      <head>
        <meta charset="UTF-8">
        <style>
          * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
          }
          body {
            font-family: var(--vscode-font-family);
            color: var(--vscode-foreground);
            background: var(--vscode-editor-background);
            padding: 16px;
            overflow-y: auto;
          }
          h2 {
            font-size: 16px;
            margin-bottom: 16px;
            border-bottom: 1px solid var(--vscode-editorWidget-border);
            padding-bottom: 8px;
          }
          .control-panel {
            margin-bottom: 16px;
            padding: 12px;
            background: var(--vscode-input-background);
            border: 1px solid var(--vscode-editorWidget-border);
            border-radius: 4px;
          }
          .button-group {
            display: flex;
            gap: 8px;
            margin-bottom: 8px;
          }
          button {
            flex: 1;
            padding: 8px 12px;
            background: var(--vscode-button-background);
            color: var(--vscode-button-foreground);
            border: none;
            border-radius: 3px;
            cursor: pointer;
            font-size: 12px;
            font-weight: 500;
            transition: background 0.2s;
          }
          button:hover {
            background: var(--vscode-button-hoverBackground);
          }
          button:active {
            background: var(--vscode-button-background);
          }
          button:disabled {
            opacity: 0.5;
            cursor: not-allowed;
          }
          .status-indicator {
            display: flex;
            align-items: center;
            gap: 6px;
            font-size: 12px;
            padding: 4px 0;
          }
          .status-dot {
            width: 8px;
            height: 8px;
            border-radius: 50%;
            background: var(--vscode-errorForeground);
          }
          .status-dot.running {
            background: var(--vscode-testing-runIcon);
          }
          .stats-container {
            display: grid;
            grid-template-columns: repeat(2, 1fr);
            gap: 12px;
            margin-bottom: 16px;
          }
          .stat-item {
            padding: 12px;
            background: var(--vscode-input-background);
            border: 1px solid var(--vscode-editorWidget-border);
            border-radius: 4px;
            text-align: center;
          }
          .stat-label {
            font-size: 11px;
            opacity: 0.7;
            text-transform: uppercase;
            margin-bottom: 6px;
          }
          .stat-value {
            font-size: 20px;
            font-weight: bold;
            color: var(--vscode-terminal-ansiBlue);
          }
          .logs-section {
            margin-top: 16px;
            padding: 12px;
            background: var(--vscode-input-background);
            border: 1px solid var(--vscode-editorWidget-border);
            border-radius: 4px;
          }
          .logs-header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 8px;
          }
          .logs-header h3 {
            font-size: 12px;
            font-weight: 600;
          }
          .logs-content {
            max-height: 150px;
            overflow-y: auto;
            font-family: var(--vscode-editor-font-family);
            font-size: 11px;
            background: var(--vscode-editor-background);
            padding: 8px;
            border-radius: 3px;
            color: var(--vscode-editorTerminal-foreground);
          }
          .log-entry {
            margin: 2px 0;
            opacity: 0.8;
          }
          .log-entry.error {
            color: var(--vscode-errorForeground);
          }
          .log-entry.info {
            color: var(--vscode-terminal-ansiBlue);
          }
        </style>
      </head>
      <body>
        <h2>comP Dashboard</h2>

        <div class="control-panel">
          <div class="button-group">
            <button id="startBtn" onclick="startDaemon()">Start Indexing</button>
            <button id="stopBtn" onclick="stopDaemon()" disabled>Stop Indexing</button>
          </div>
          <div class="status-indicator">
            <div class="status-dot" id="statusDot"></div>
            <span id="statusText">Daemon stopped</span>
          </div>
        </div>

        <div class="stats-container">
          <div class="stat-item">
            <div class="stat-label">Files</div>
            <div class="stat-value" id="fileCount">--</div>
          </div>
          <div class="stat-item">
            <div class="stat-label">Nodes</div>
            <div class="stat-value" id="symbolCount">--</div>
          </div>
          <div class="stat-item">
            <div class="stat-label">Edges</div>
            <div class="stat-value" id="edgeCount">--</div>
          </div>
          <div class="stat-item">
            <div class="stat-label">Updated</div>
            <div class="stat-value" id="lastUpdated" style="font-size: 12px;">--</div>
          </div>
        </div>

        <div class="logs-section">
          <div class="logs-header">
            <h3>Debug Logs</h3>
            <button id="clearLogsBtn" style="flex: 0; padding: 4px 8px; font-size: 10px;" onclick="clearLogs()">Clear</button>
          </div>
          <div class="logs-content" id="logsContent">
            <div class="log-entry info">Waiting for logs...</div>
          </div>
        </div>

        <script>
          const vscode = acquireVsCodeApi();

          function startDaemon() {
            console.log('Starting daemon...');
            vscode.postMessage({ command: 'startDaemon' });
          }

          function stopDaemon() {
            console.log('Stopping daemon...');
            vscode.postMessage({ command: 'stopDaemon' });
          }

          function clearLogs() {
            vscode.postMessage({ command: 'clearLogs' });
          }

          window.addEventListener('message', (event) => {
            const message = event.data;
            console.log('Received message:', message);

            if (message.type === 'statsUpdate') {
              const data = message.data;
              document.getElementById('fileCount').textContent = data.totalFiles || '--';
              document.getElementById('symbolCount').textContent = data.totalNodes || '--';
              document.getElementById('edgeCount').textContent = data.totalEdges || '--';
              document.getElementById('lastUpdated').textContent = data.lastUpdated || '--';

              if (data.daemonRunning) {
                updateDaemonStatus(true);
              }
            } else if (message.type === 'daemonStatus') {
              updateDaemonStatus(message.running);
            } else if (message.type === 'logsUpdate') {
              const logsContent = document.getElementById('logsContent');
              logsContent.innerHTML = '';
              (message.logs || []).forEach(log => {
                const entry = document.createElement('div');
                entry.className = 'log-entry';
                if (log.includes('Error') || log.includes('error')) {
                  entry.classList.add('error');
                } else if (log.includes('Starting') || log.includes('Started')) {
                  entry.classList.add('info');
                }
                entry.textContent = log;
                logsContent.appendChild(entry);
              });
              logsContent.scrollTop = logsContent.scrollHeight;
            } else if (message.type === 'statsError') {
              const logsContent = document.getElementById('logsContent');
              const entry = document.createElement('div');
              entry.className = 'log-entry error';
              entry.textContent = '❌ ' + message.message;
              logsContent.appendChild(entry);
            }
          });

          function updateDaemonStatus(running) {
            const dot = document.getElementById('statusDot');
            const text = document.getElementById('statusText');
            const startBtn = document.getElementById('startBtn');
            const stopBtn = document.getElementById('stopBtn');

            if (running) {
              dot.classList.add('running');
              text.textContent = 'Daemon running';
              startBtn.disabled = true;
              stopBtn.disabled = false;
            } else {
              dot.classList.remove('running');
              text.textContent = 'Daemon stopped';
              startBtn.disabled = false;
              stopBtn.disabled = true;
            }
          }

          // Request initial stats
          vscode.postMessage({ command: 'refresh' });
        </script>
      </body>
      </html>
    `;
  }
}
