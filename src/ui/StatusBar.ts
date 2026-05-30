// StatusBar - Display comP status in VSCode status bar
//
// Shows:
// - Status (Ready, Indexing, Error)
// - Node count from graph DB
// - Click to show statistics dashboard

import * as vscode from "vscode";

export class StatusBar {
  public static instance: StatusBar | null = null;
  private item: vscode.StatusBarItem | null = null;

  constructor() {
    StatusBar.instance = this;
    // Create status bar item on the left side
    this.item = vscode.window.createStatusBarItem(
      vscode.StatusBarAlignment.Left,
      100
    );
    this.item.command = "comp.showStats";
    this.item.tooltip = "Click to open comP statistics";
  }

  /** Update status bar to "Initializing" state */
  show(status: string): void {
    if (!this.item) return;
    this.item.text = `◈ comP: ${status}`;
    if (status === "Error") {
      this.item.backgroundColor = new vscode.ThemeColor("statusBarItem.errorBackground");
    } else {
      this.item.backgroundColor = undefined;
    }
    this.item.show();
  }

  /** Update with statistics from daemon */
  updateStats(totalNodes: number, totalFiles: number, status: "Ready" | "Indexing" | "Error" = "Ready", efficiency?: string): void {
    if (!this.item) return;

    const statusIcon = status === "Ready" ? "✓" : status === "Indexing" ? "⟳" : "⚠";
    const savingsText = efficiency && efficiency !== "0%" ? ` | ${efficiency} saved` : "";
    this.item.text = `◈ comP: ${totalNodes} symbols${savingsText} | ${statusIcon} ${status}`;
    this.item.tooltip = `${totalFiles} files indexed • ${totalNodes} symbols • Status: ${status}${efficiency ? ` • Efficiency: ${efficiency}` : ""}`;

    // Set background color based on status
    if (status === "Error") {
      this.item.backgroundColor = new vscode.ThemeColor("statusBarItem.errorBackground");
    } else if (status === "Indexing") {
      this.item.backgroundColor = undefined;
    } else {
      this.item.backgroundColor = undefined;
    }

    this.item.show();
  }

  /** Update to show indexing progress */
  updateProgress(current: number, total: number): void {
    if (!this.item) return;
    const percent = Math.round((current / total) * 100);
    this.item.text = `◈ comP: Indexing ${percent}%`;
    this.item.show();
  }

  /** Dispose status bar */
  dispose(): void {
    if (this.item) {
      this.item.dispose();
    }
  }
}
