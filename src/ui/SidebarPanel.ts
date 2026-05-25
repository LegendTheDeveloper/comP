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

/**
 * Sidebar panel - WebviewViewProvider として実装
 *
 * # 入力
 * - context: VSCode ExtensionContext（バージョン読み込み・DaemonManager 生成に使用）
 *
 * # 出力
 * - resolveWebviewView: VSCode が sidebar を表示するときに呼び出す
 * - setDaemonManager: extension.ts が daemon 起動後に呼び出す
 */
/**
 * extension.ts が SidebarPanel に注入する起動/停止コールバック。
 * WHY: 以前 SidebarPanel が new DaemonManager() で独自に二重生成していた結果、
 * extension.ts のグローバル daemonManager と分離され、commands.ts (forceReindex 等) が
 * 古いインスタンスを叩く不整合があった。生成責務を extension.ts に一本化する。
 */
export interface DaemonLifecycleCallbacks {
  onStartRequest: () => Promise<DaemonManager | null>;
  onStopRequest: () => Promise<void>;
}

export class SidebarPanel implements vscode.WebviewViewProvider {
  public static readonly viewType = "comp-stats";
  private static instance: SidebarPanel | undefined;

  // WebviewView は resolveWebviewView が呼ばれるまで undefined
  private view?: vscode.WebviewView;
  private daemonManager: DaemonManager | null = null;
  private statsInterval: NodeJS.Timeout | null = null;
  private logs: string[] = [];
  private maxLogs = 100;
  private version = "0.1.0";
  private lifecycleCallbacks: DaemonLifecycleCallbacks | null = null;

  private constructor(context: vscode.ExtensionContext) {
    // package.json からバージョンを取得
    try {
      const pkgPath = path.join(context.extensionPath, "package.json");
      const pkg = JSON.parse(fs.readFileSync(pkgPath, "utf-8"));
      this.version = pkg.version || this.version;
    } catch {
      // フォールバック
    }
  }

  /**
   * extension.ts から呼ばれる初期化メソッド
   * 後方互換のため旧シグネチャを維持
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
   * VSCode がサイドバーに WebView を表示するときに呼ばれる
   *
   * # 処理
   * - WebView の HTML をセット
   * - メッセージハンドラーを登録
   * - daemon が起動済みなら stats 取得を開始
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

    // visible になったとき stats を更新
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
   * daemon 起動後に extension.ts から呼ばれる
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
   * extension.ts が起動/停止ロジックを注入する。
   * WHY: SidebarPanel は UI 層に専念し、DaemonManager のライフサイクル管理を行わない。
   */
  public setLifecycleCallbacks(callbacks: DaemonLifecycleCallbacks): void {
    this.lifecycleCallbacks = callbacks;
  }

  /**
   * WebView からのメッセージを処理
   *
   * # 処理分岐
   * - "refresh": stats 更新
   * - "startDaemon": daemon 起動
   * - "stopDaemon": daemon 停止
   * - "clearLogs": ログクリア
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
      this.view.webview.postMessage({
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
  <h2>comP <span style="font-weight:normal;opacity:0.6;font-size:11px;">v${this.version}</span></h2>
  <div class="control-panel">
    <div class="button-group">
      <button id="startBtn" onclick="startDaemon()">▶ Start</button>
      <button id="stopBtn" onclick="stopDaemon()" disabled>■ Stop</button>
    </div>
    <div class="status-indicator">
      <div class="status-dot" id="statusDot"></div>
      <span id="statusText">Daemon stopped</span>
    </div>
  </div>
  <div class="stats-container">
    <div class="stat-item"><div class="stat-label">Files</div><div class="stat-value" id="fileCount">--</div></div>
    <div class="stat-item"><div class="stat-label">Nodes</div><div class="stat-value" id="symbolCount">--</div></div>
    <div class="stat-item"><div class="stat-label">Edges</div><div class="stat-value" id="edgeCount">--</div></div>
    <div class="stat-item"><div class="stat-label">Updated</div><div class="stat-value" id="lastUpdated" style="font-size:11px;">--</div></div>
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
    window.addEventListener('message', (event) => {
      const msg = event.data;
      if (msg.type === 'statsUpdate') {
        const d = msg.data;
        document.getElementById('fileCount').textContent = d.totalFiles ?? '--';
        document.getElementById('symbolCount').textContent = d.totalNodes ?? '--';
        document.getElementById('edgeCount').textContent = d.totalEdges ?? '--';
        document.getElementById('lastUpdated').textContent = d.lastUpdated ?? '--';
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
    function updateStatus(running) {
      document.getElementById('statusDot').classList.toggle('running', running);
      document.getElementById('statusText').textContent = running ? 'Daemon running' : 'Daemon stopped';
      document.getElementById('startBtn').disabled = running;
      document.getElementById('stopBtn').disabled = !running;
    }
    vscode.postMessage({ command: 'refresh' });
  </script>
</body>
</html>`;
  }
}
