// CodeLens Provider Tests
//
// Coverage:
// - Language support filtering
// - CodeLens generation
// - CodeLens resolution
// - Dependent count display

import { expect } from "chai";
import * as vscode from "vscode";
import { DependencyCodeLensProvider } from "../CodeLens";
import { DaemonManager } from "../../daemon/DaemonManager";

// Mock DaemonManager
class MockDaemonManager implements Partial<DaemonManager> {
  async request(_method: string, _params?: unknown): Promise<unknown> {
    return { symbols: [], dependents: [] };
  }
}

describe("DependencyCodeLensProvider", () => {
  let provider: DependencyCodeLensProvider;
  let mockDaemon: MockDaemonManager;

  beforeEach(() => {
    mockDaemon = new MockDaemonManager();
    provider = new DependencyCodeLensProvider(mockDaemon as any);
  });

  afterEach(() => {
    provider.dispose();
  });

  describe("isSupportedLanguage", () => {
    // private メソッドなので provideCodeLenses 経由でテスト

    it("should return empty array for unsupported language", async () => {
      const doc = {
        languageId: "markdown",
        uri: { fsPath: "/path/to/file.md" },
      } as any;

      const lenses = await provider.provideCodeLenses(doc, null as any);
      expect(lenses).to.deep.equal([]);
    });

    it("should handle typescript language", async () => {
      const doc = {
        languageId: "typescript",
        uri: { fsPath: "/path/to/file.ts" },
      } as any;

      const lenses = await provider.provideCodeLenses(doc, null as any);
      expect(Array.isArray(lenses)).to.be.true;
    });

    it("should handle python language", async () => {
      const doc = {
        languageId: "python",
        uri: { fsPath: "/path/to/file.py" },
      } as any;

      const lenses = await provider.provideCodeLenses(doc, null as any);
      expect(Array.isArray(lenses)).to.be.true;
    });
  });

  describe("provideCodeLenses", () => {
    it("should return empty array for empty document", async () => {
      const doc = {
        languageId: "typescript",
        uri: { fsPath: "/path/to/file.ts" },
      } as any;

      const lenses = await provider.provideCodeLenses(doc, null as any);
      expect(Array.isArray(lenses)).to.be.true;
    });

    it("should handle daemon errors gracefully", async () => {
      // Mock daemon that throws error
      const errorDaemon = {
        request: async () => {
          throw new Error("Daemon unavailable");
        },
      };
      const errorProvider = new DependencyCodeLensProvider(
        errorDaemon as any
      );

      const doc = {
        languageId: "typescript",
        uri: { fsPath: "/path/to/file.ts" },
      } as any;

      const lenses = await errorProvider.provideCodeLenses(doc, null as any);
      expect(Array.isArray(lenses)).to.be.true;
      expect(lenses.length).to.equal(0);
      errorProvider.dispose();
    });
  });

  describe("resolveCodeLens", () => {
    it("should set command for symbol with dependents", () => {
      const range = new vscode.Range(
        new vscode.Position(0, 0),
        new vscode.Position(0, 5)
      );
      const symbolInfo = {
        id: 1,
        name: "myFunction",
        kind: "function",
        line: 0,
        column: 0,
        dependentCount: 3,
      };

      const codeLens = new vscode.CodeLens(range, undefined);
      (codeLens as any).data = symbolInfo;

      const resolved = provider.resolveCodeLens(codeLens, null as any);

      expect(resolved.command).to.exist;
      expect(resolved.command?.title).to.include("3 references");
      expect(resolved.command?.command).to.equal("vscode.executeReferenceProvider");
    });

    it("should handle singular reference", () => {
      const range = new vscode.Range(
        new vscode.Position(0, 0),
        new vscode.Position(0, 5)
      );
      const symbolInfo = {
        id: 1,
        name: "myFunction",
        kind: "function",
        line: 0,
        column: 0,
        dependentCount: 1,
      };

      const codeLens = new vscode.CodeLens(range, undefined);
      (codeLens as any).data = symbolInfo;

      const resolved = provider.resolveCodeLens(codeLens, null as any);

      expect(resolved.command?.title).to.include("1 reference");
      expect(resolved.command?.title).not.to.include("1 references");
    });

    it("should set no-op command for symbol with no dependents", () => {
      const range = new vscode.Range(
        new vscode.Position(0, 0),
        new vscode.Position(0, 5)
      );
      const symbolInfo = {
        id: 1,
        name: "myFunction",
        kind: "function",
        line: 0,
        column: 0,
        dependentCount: 0,
      };

      const codeLens = new vscode.CodeLens(range, undefined);
      (codeLens as any).data = symbolInfo;

      const resolved = provider.resolveCodeLens(codeLens, null as any);

      expect(resolved.command?.title).to.equal("No references");
      expect(resolved.command?.command).to.equal("");
    });
  });

  describe("refresh", () => {
    it("should emit change event", (done) => {
      provider.onDidChangeCodeLenses((_e) => {
        done();
      });

      provider.refresh();
    });
  });

  describe("dispose", () => {
    it("should clean up resources", () => {
      expect(() => {
        provider.dispose();
      }).not.to.throw();
    });
  });
});
