// SessionMemoryManager Unit Tests
//
// Coverage:
// - load/save session memory
// - mark matching entries as stale based on file paths

import { expect } from "chai";
import * as fs from "fs";
import * as path from "path";
import * as os from "os";
import { SessionMemoryManager, SessionMemory } from "../sessionMemory";

describe("SessionMemoryManager", () => {
  let tempDir: string;
  let manager: SessionMemoryManager;

  beforeEach(() => {
    tempDir = path.join(os.tmpdir(), `comp-test-session-memory-${Date.now()}`);
    fs.mkdirSync(tempDir, { recursive: true });
    manager = new SessionMemoryManager(tempDir);
  });

  afterEach(() => {
    try {
      fs.rmSync(tempDir, { recursive: true, force: true });
    } catch {}
  });

  it("should load empty memory if file does not exist", () => {
    const memory = manager.load();
    expect(memory.sessions).to.be.an("array").that.is.empty;
  });

  it("should save and load session memory", () => {
    const mockMemory: SessionMemory = {
      sessions: [
        {
          id: "session-1",
          timestamp: 1234567,
          calls: [
            {
              query: "test query",
              symbols: ["testSymbol"],
              files: ["src/test.ts"],
              tokens: 100,
              stale: false,
              timestamp: 1234568,
            },
          ],
        },
      ],
    };

    manager.save(mockMemory);
    const loaded = manager.load();
    expect(loaded.sessions).to.have.lengthOf(1);
    expect(loaded.sessions[0].id).to.equal("session-1");
    expect(loaded.sessions[0].calls).to.have.lengthOf(1);
    expect(loaded.sessions[0].calls[0].query).to.equal("test query");
  });

  it("should mark matching files as stale", () => {
    const mockMemory: SessionMemory = {
      sessions: [
        {
          id: "session-1",
          timestamp: 1234567,
          calls: [
            {
              query: "test query",
              symbols: ["testSymbol"],
              files: ["src/test.ts", "src/other.ts"],
              tokens: 100,
              stale: false,
              timestamp: 1234568,
            },
            {
              query: "another query",
              symbols: ["anotherSymbol"],
              files: ["src/unrelated.ts"],
              tokens: 50,
              stale: false,
              timestamp: 1234569,
            },
          ],
        },
      ],
    };

    manager.save(mockMemory);

    // Mark 'src/test.ts' as stale
    manager.markStaleForFile("src\\test.ts");

    const loaded = manager.load();
    expect(loaded.sessions[0].calls[0].stale).to.be.true;
    expect(loaded.sessions[0].calls[1].stale).to.be.false;
  });
});
