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
import { DaemonManager } from "../daemon/DaemonManager";

/**
 * Statistics to display in dashboard
 */
interface IndexStats {
  totalFiles: number;
  totalNodes: number;
  totalEdges: number;
  languages: { [key: string]: number };
  lastIndexed: string;
  indexSize: string;
}

/**
 * Sidebar panel controller
 */
export class SidebarPanel {
  public static readonly viewType = "comp-stats";
  private static instance: SidebarPanel | undefined;

  private readonly webviewPanel: vscode.WebviewPanel;
  private readonly extensionPath: string;
  private daemonManager: DaemonManager | null = null;

  /**
   * Create or show the sidebar panel
   */
  public static createOrShow(extensionPath: string, daemonManager: DaemonManager): SidebarPanel {
    const column = vscode.ViewColumn.Sidebar;

    // If we already have a panel, show it
    if (SidebarPanel.instance) {
      SidebarPanel.instance.webviewPanel.reveal(column);
      return SidebarPanel.instance;
    }

    // Create new panel
    const webviewPanel = vscode.window.createWebviewPanel(
      SidebarPanel.viewType,
      "comP Statistics",
      column,
      {
        enableScripts: true,
        retainContextWhenHidden: true,
        localResourceRoots: [vscode.Uri.file(path.join(extensionPath, "src", "webview"))],
      }
    );

    SidebarPanel.instance = new SidebarPanel(webviewPanel, extensionPath, daemonManager);
    return SidebarPanel.instance;
  }

  /**
   * Constructor
   */
  private constructor(
    webviewPanel: vscode.WebviewPanel,
    extensionPath: string,
    daemonManager: DaemonManager
  ) {
    this.webviewPanel = webviewPanel;
    this.extensionPath = extensionPath;
    this.daemonManager = daemonManager;

    // Load webview content
    this.update();

    // Handle disposal
    this.webviewPanel.onDidDispose(() => {
      SidebarPanel.instance = undefined;
    });

    // Handle messages from webview
    this.webviewPanel.webview.onDidReceiveMessage(
      (message) => {
        this.handleWebviewMessage(message);
      },
      undefined,
      undefined
    );

    // Update stats periodically (every 5 seconds)
    const interval = setInterval(() => {
      this.refreshStats();
    }, 5000);

    this.webviewPanel.onDidDispose(() => {
      clearInterval(interval);
    });
  }

  /**
   * Handle messages from webview
   *
   * Message types:
   * - "refresh": Request stats update
   * - "forceIndex": Trigger re-indexing
   */
  private async handleWebviewMessage(message: { command: string; [key: string]: unknown }): Promise<void> {
    // TODO: Implement message handlers
    // - refresh: fetch stats and send to webview
    // - forceIndex: trigger daemon re-index
    // - openFile: open file from dependency graph
  }

  /**
   * Load and display webview content
   */
  private update(): void {
    // TODO: Load HTML from stats.html
    // - Replace resource URIs (CSS, JS) with webview URIs
    // - Inject nonce for script security
    // - Set webview HTML

    const statsHtmlPath = path.join(this.extensionPath, "src", "webview", "stats.html");
    
    // For now, show placeholder
    this.webviewPanel.webview.html = this.getPlaceholderHtml();
  }

  /**
   * Refresh statistics from daemon
   */
  private async refreshStats(): Promise<void> {
    if (!this.daemonManager) return;

    try {
      const stats = await this.daemonManager.getStats();
      
      // TODO: Send to webview
      // this.webviewPanel.webview.postMessage({
      //   type: "statsUpdate",
      //   stats
      // });
    } catch (error) {
      console.error("[comP] Failed to fetch stats:", error);
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
          body {
            font-family: var(--vscode-font-family);
            color: var(--vscode-foreground);
            background: var(--vscode-editor-background);
            padding: 20px;
          }
          .stat-item {
            margin: 15px 0;
            padding: 10px;
            background: var(--vscode-editor-background);
            border-left: 3px solid var(--vscode-activityBarBadge-background);
          }
          .stat-label {
            font-size: 12px;
            opacity: 0.7;
            text-transform: uppercase;
          }
          .stat-value {
            font-size: 24px;
            font-weight: bold;
            margin-top: 5px;
          }
        </style>
      </head>
      <body>
        <h2>comP Statistics</h2>
        <div class="stat-item">
          <div class="stat-label">Files Indexed</div>
          <div class="stat-value" id="fileCount">--</div>
        </div>
        <div class="stat-item">
          <div class="stat-label">Symbols Found</div>
          <div class="stat-value" id="symbolCount">--</div>
        </div>
        <div class="stat-item">
          <div class="stat-label">Dependencies</div>
          <div class="stat-value" id="edgeCount">--</div>
        </div>
        <p style="opacity: 0.6; font-size: 12px; margin-top: 20px;">Loading...</p>
      </body>
      </html>
    `;
  }
}
