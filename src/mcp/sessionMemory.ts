// SessionMemory - Persistent session-based storage of MCP tool invocations
//
// Responsibilities:
// 1. Maintain past calls (query, symbols, tokens, stale status) per session
// 2. Load/save data from/to .comp/session-memory.json
// 3. Mark matching entries as stale when source code files are modified

import * as fs from "fs";
import * as path from "path";

export interface SessionCall {
  query: string;
  symbols: string[];
  files: string[];
  tokens: number;
  stale: boolean;
  timestamp: number;
}

export interface Session {
  id: string;
  timestamp: number;
  calls: SessionCall[];
}

export interface SessionMemory {
  sessions: Session[];
}

export class SessionMemoryManager {
  private memoryFilePath: string;

  constructor(workspaceRoot: string) {
    this.memoryFilePath = path.join(workspaceRoot, ".comp", "session-memory.json");
  }

  /**
   * Load session memory from file
   */
  public load(): SessionMemory {
    if (!fs.existsSync(this.memoryFilePath)) {
      return { sessions: [] };
    }
    try {
      const content = fs.readFileSync(this.memoryFilePath, "utf8");
      return JSON.parse(content) as SessionMemory;
    } catch (e) {
      console.error("[comP] Failed to load session memory:", e);
      return { sessions: [] };
    }
  }

  /**
   * Save session memory to file
   */
  public save(memory: SessionMemory): void {
    try {
      const dir = path.dirname(this.memoryFilePath);
      if (!fs.existsSync(dir)) {
        fs.mkdirSync(dir, { recursive: true });
      }
      fs.writeFileSync(this.memoryFilePath, JSON.stringify(memory, null, 2), "utf8");
    } catch (e) {
      console.error("[comP] Failed to save session memory:", e);
    }
  }

  /**
   * Mark entries as stale if they are related to the modified file
   * @param relativeFilePath Relative path from workspace root
   */
  public markStaleForFile(relativeFilePath: string): void {
    const normalizedPath = relativeFilePath.replace(/\\/g, "/");
    const memory = this.load();
    let updated = false;

    for (const session of memory.sessions) {
      for (const call of session.calls) {
        if (!call.stale) {
          const match = call.files.some(f => f.replace(/\\/g, "/") === normalizedPath);
          if (match) {
            call.stale = true;
            updated = true;
          }
        }
      }
    }

    if (updated) {
      this.save(memory);
      console.log(`[comP] Session memory updated: marked entries stale for ${normalizedPath}`);
    }
  }
}
