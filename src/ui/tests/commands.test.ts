// Commands Unit Tests
//
// Coverage:
// - registerCommands() registers all 5 expected commands
// - comp.showStats calls getStats and shows message
// - comp.forceReindex with "Yes" triggers daemon request
// - comp.forceReindex with "Cancel" skips reindex
// - comp.setupAgents with no selection does nothing
// - comp.generateContext with no input does nothing

import { expect } from "chai";
import * as sinon from "sinon";
import * as vscode from "vscode";
import { registerCommands } from "../commands";

describe("registerCommands", () => {
  let mockContext: any;
  let mockDaemon: any;
  let mockStatusBar: any;
  let handlers: Map<string, () => Promise<void>>;

  beforeEach(() => {
    handlers = new Map();
    (vscode.commands as any).registerCommand = (name: string, handler: () => Promise<void>) => {
      handlers.set(name, handler);
      return { dispose: () => {} };
    };

    mockContext = {
      subscriptions: { push: sinon.stub() },
    };

    mockDaemon = {
      request: sinon.stub().resolves({}),
      getStats: sinon.stub().resolves({ total_nodes: 10, total_files: 5, total_edges: 20 }),
      isRunning: sinon.stub().returns(true),
      compressFile: sinon.stub().resolves({ text: "compressed_output", compressionRate: "38%" }),
    };

    mockStatusBar = {
      show: sinon.stub(),
      updateStats: sinon.stub(),
    };

    (vscode.window as any).showQuickPick = sinon.stub().resolves(undefined);
    (vscode.window as any).showInputBox = sinon.stub().resolves(undefined);
    (vscode.window as any).showInformationMessage = sinon.stub().resolves(undefined);
    (vscode.window as any).showErrorMessage = sinon.stub().resolves(undefined);
    (vscode.window as any).showWarningMessage = sinon.stub().resolves("Cancel");
    (vscode.window as any).activeTextEditor = undefined;
    (vscode.window as any).withProgress = async (_options: any, task: () => Promise<any>) => {
      return await task();
    };
    (vscode.workspace as any).workspaceFolders = undefined;
    (vscode.env.clipboard as any).writeText = sinon.stub().resolves();
  });

  it("registers all 6 commands", () => {
    registerCommands(mockContext, () => mockDaemon, mockStatusBar);
    expect(handlers.size).to.equal(6);
    expect(handlers.has("comp.setupAgents")).to.be.true;
    expect(handlers.has("comp.forceReindex")).to.be.true;
    expect(handlers.has("comp.generateContext")).to.be.true;
    expect(handlers.has("comp.showImpactGraph")).to.be.true;
    expect(handlers.has("comp.showStats")).to.be.true;
    expect(handlers.has("comp.copyActiveFileCompressed")).to.be.true;
  });

  it("comp.showStats calls getStats and shows information message", async () => {
    registerCommands(mockContext, () => mockDaemon, mockStatusBar);
    await handlers.get("comp.showStats")!();
    expect(mockDaemon.getStats.calledOnce).to.be.true;
    expect((vscode.window as any).showInformationMessage.calledOnce).to.be.true;
  });

  it('comp.forceReindex with "Yes" calls request("forceReindex")', async () => {
    (vscode.window as any).showWarningMessage = sinon.stub().resolves("Yes");
    registerCommands(mockContext, () => mockDaemon, mockStatusBar);
    await handlers.get("comp.forceReindex")!();
    expect(mockDaemon.request.calledWith("forceReindex")).to.be.true;
  });

  it('comp.forceReindex with "Cancel" skips request', async () => {
    (vscode.window as any).showWarningMessage = sinon.stub().resolves("Cancel");
    registerCommands(mockContext, () => mockDaemon, mockStatusBar);
    await handlers.get("comp.forceReindex")!();
    expect(mockDaemon.request.called).to.be.false;
  });

  it("comp.forceReindex updates status bar on success", async () => {
    (vscode.window as any).showWarningMessage = sinon.stub().resolves("Yes");
    registerCommands(mockContext, () => mockDaemon, mockStatusBar);
    await handlers.get("comp.forceReindex")!();
    expect(mockStatusBar.updateStats.calledOnce).to.be.true;
  });

  it("comp.setupAgents with no selection does nothing", async () => {
    (vscode.window as any).showQuickPick = sinon.stub().resolves(undefined);
    registerCommands(mockContext, () => mockDaemon, mockStatusBar);
    await handlers.get("comp.setupAgents")!();
    expect((vscode.window as any).showInformationMessage.called).to.be.false;
  });

  it("comp.setupAgents with GitHub Copilot selection generates config", async () => {
    (vscode.window as any).showQuickPick = sinon.stub().resolves("GitHub Copilot");
    (vscode.window as any).showInformationMessage = sinon.stub().resolves("Done");
    registerCommands(mockContext, () => mockDaemon, mockStatusBar);
    await handlers.get("comp.setupAgents")!();
    expect((vscode.window as any).showInformationMessage.called).to.be.true;
  });

  it("comp.generateContext with no input does nothing", async () => {
    (vscode.window as any).showInputBox = sinon.stub().resolves(undefined);
    registerCommands(mockContext, () => mockDaemon, mockStatusBar);
    await handlers.get("comp.generateContext")!();
    expect(mockDaemon.request.called).to.be.false;
  });

  it("comp.showImpactGraph with no active editor shows error", async () => {
    (vscode.window as any).activeTextEditor = undefined;
    registerCommands(mockContext, () => mockDaemon, mockStatusBar);
    await handlers.get("comp.showImpactGraph")!();
    expect((vscode.window as any).showErrorMessage.calledOnce).to.be.true;
  });

  it("comp.copyActiveFileCompressed with no active editor shows error", async () => {
    (vscode.window as any).activeTextEditor = undefined;
    registerCommands(mockContext, () => mockDaemon, mockStatusBar);
    await handlers.get("comp.copyActiveFileCompressed")!();
    expect((vscode.window as any).showErrorMessage.calledOnce).to.be.true;
  });

  it("comp.copyActiveFileCompressed with active editor and selection calls compressFile and writes clipboard", async () => {
    // Setup active editor
    (vscode.window as any).activeTextEditor = {
      document: {
        uri: vscode.Uri.file("/workspace/test_file.rs"),
        languageId: "rust"
      }
    };
    // Mock QuickPick selection: Compact level
    const selection = { label: "Compact", value: 1 };
    (vscode.window as any).showQuickPick = sinon.stub().resolves(selection);

    registerCommands(mockContext, () => mockDaemon, mockStatusBar);
    await handlers.get("comp.copyActiveFileCompressed")!();

    expect(mockDaemon.compressFile.calledWith("/workspace/test_file.rs", 1)).to.be.true;
    expect((vscode.env as any).clipboard.writeText.calledWith("compressed_output")).to.be.true;
  });
});
