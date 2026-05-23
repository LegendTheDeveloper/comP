// DaemonManager - Manages Rust daemon lifecycle and IPC communication
//
// Responsibilities:
// - Start daemon process
// - Stop daemon gracefully
// - Send JSON-RPC requests to daemon via stdio
// - Handle reconnection on failures
// - Log daemon output for debugging

import * as vscode from "vscode";
import { spawn, ChildProcess } from "child_process";
import * as path from "path";
import { readFileSync } from "fs";

interface JSONRPCRequest {
  jsonrpc: "2.0";
  id: number | string;
  method: string;
  params?: unknown;
}

interface JSONRPCResponse {
  jsonrpc: "2.0";
  id: number | string;
  result?: unknown;
  error?: { code: number; message: string };
}

export class DaemonManager {
  private context: vscode.ExtensionContext;
  private process: ChildProcess | null = null;
  private requestId = 0;
  private pendingRequests = new Map<number, (response: JSONRPCResponse) => void>();
  private isReady = false;

  constructor(context: vscode.ExtensionContext) {
    this.context = context;
  }

  /**
   * Start the Rust daemon
   *
   * Process:
   * 1. Find daemon binary
   * 2. Spawn process with environment variables
   * 3. Attach stdout/stderr handlers
   * 4. Wait for ready signal
   */
  async start(): Promise<void> {
    const daemonPath = this.getDaemonPath();
    const workspaceRoot = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath || ".";

    console.log("[comP] Starting daemon from:", daemonPath);

    try {
      this.process = spawn(daemonPath, [], {
        env: {
          ...process.env,
          COMP_WORKSPACE_ROOT: workspaceRoot,
          RUST_LOG: "info",
        },
        cwd: workspaceRoot,
      });

      // Handle daemon output
      this.process.stdout?.on("data", (data) => {
        console.log("[comP daemon]", data.toString().trim());
      });

      this.process.stderr?.on("data", (data) => {
        console.error("[comP daemon err]", data.toString().trim());
      });

      this.process.on("exit", (code) => {
        console.log("[comP] Daemon exited with code:", code);
        this.isReady = false;
      });

      this.isReady = true;
      console.log("[comP] Daemon started successfully");
    } catch (error) {
      console.error("[comP] Failed to start daemon:", error);
      throw new Error(`Failed to start comP daemon: ${error instanceof Error ? error.message : String(error)}`);
    }
  }

  /**
   * Stop the daemon gracefully
   */
  async stop(): Promise<void> {
    if (this.process) {
      console.log("[comP] Stopping daemon...");
      this.process.kill("SIGTERM");

      // Wait up to 5 seconds for process to exit
      await new Promise((resolve) => {
        const timeout = setTimeout(() => {
          if (this.process) {
            this.process.kill("SIGKILL");
          }
          resolve(null);
        }, 5000);

        this.process?.on("exit", () => {
          clearTimeout(timeout);
          resolve(null);
        });
      });

      this.process = null;
      this.isReady = false;
    }
  }

  /**
   * Send a request to daemon and wait for response
   *
   * # Protocol:
   * Request: { "jsonrpc": "2.0", "id": 1, "method": "...", "params": {...} }
   * Response: { "jsonrpc": "2.0", "id": 1, "result": {...} }
   */
  async request(method: string, params?: unknown): Promise<unknown> {
    if (!this.isReady || !this.process) {
      throw new Error("Daemon is not running");
    }

    const id = ++this.requestId;
    const request: JSONRPCRequest = {
      jsonrpc: "2.0",
      id,
      method,
      params,
    };

    return new Promise((resolve, reject) => {
      // Set timeout based on method (getStats may be very slow on large indexes)
      const timeoutMs = method === "getStats" ? 30000 : 10000;
      const timeout = setTimeout(() => {
        this.pendingRequests.delete(id);
        reject(new Error(`Request timeout for method: ${method}`));
      }, timeoutMs);

      // Wait for response
      this.pendingRequests.set(id, (response) => {
        clearTimeout(timeout);
        if (response.error) {
          reject(new Error(response.error.message));
        } else {
          resolve(response.result);
        }
      });

      // Send request to daemon
      this.process!.stdin?.write(JSON.stringify(request) + "\n");
    });
  }

  /**
   * Index a single file (incremental update)
   */
  async indexFile(filePath: string): Promise<void> {
    await this.request("indexFile", { path: filePath });
  }

  /**
   * Remove file from index
   */
  async removeFile(filePath: string): Promise<void> {
    await this.request("removeFile", { path: filePath });
  }

  /**
   * Check if daemon is running
   */
  isRunning(): boolean {
    return this.isReady && this.process !== null;
  }

  /**
   * Get current index statistics
   *
   * Returns current state of the index
   */
  async getStats(): Promise<{
    total_files: number;
    total_nodes: number;
    total_edges: number;
  }> {
    const result = await this.request("getStats", {});
    if (!result || typeof result !== "object") {
      throw new Error("Invalid stats response from daemon");
    }
    const stats = result as any;
    return {
      total_files: Number(stats.total_files) || 0,
      total_nodes: Number(stats.total_nodes) || 0,
      total_edges: Number(stats.total_edges) || 0,
    };
  }

  /**
   * Get symbols in a file for CodeLens display
   *
   * Returns array of symbols with dependent counts
   */
  async getSymbols(filePath: string): Promise<any[]> {
    return (await this.request("getSymbols", { path: filePath })) as any[];
  }

  /**
   * Get path to daemon binary
   *
   * Checks:
   * 1. Bundled release binary (production)
   * 2. Cargo build output (development)
   */
  private getDaemonPath(): string {
    let binaryName = "comp-daemon";
    if (process.platform === "win32") {
      binaryName += ".exe";
    }

    // Production: bundled binary
    const bundledPath = path.join(
      this.context.extensionPath,
      "daemon",
      "target",
      "release",
      binaryName
    );

    // Development: check workspace daemon build
    const workspacePath = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
    if (workspacePath) {
      const devPath = path.join(workspacePath, "daemon", "target", "release", binaryName);
      try {
        readFileSync(devPath);
        return devPath;
      } catch {
        // File doesn't exist, fall through
      }
    }

    return bundledPath;
  }
}
