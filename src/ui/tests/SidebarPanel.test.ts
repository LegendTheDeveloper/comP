// SidebarPanel Unit Tests
//
// Coverage:
// - createOrShow() throws when context is missing
// - createOrShow() returns singleton on repeated calls
// - resolveWebviewView() sets HTML on the webview
// - setDaemonManager(null) posts daemonStatus: false
// - setDaemonManager(daemon) posts daemonStatus: true
// - setLifecycleCallbacks() stores callbacks for use in message handling

import { expect } from "chai";
import * as sinon from "sinon";
import { SidebarPanel } from "../SidebarPanel";

describe("SidebarPanel", () => {
  let mockContext: any;
  let mockWebviewView: any;
  let postedMessages: any[];
  let clock: sinon.SinonFakeTimers;

  beforeEach(() => {
    // Reset singleton between tests
    (SidebarPanel as any).instance = undefined;

    clock = sinon.useFakeTimers();
    postedMessages = [];

    mockContext = {
      extensionPath: "/nonexistent/path/for/test",
    };

    mockWebviewView = {
      webview: {
        options: {},
        html: "",
        postMessage: sinon.stub().callsFake((msg: any) => {
          postedMessages.push(msg);
        }),
        onDidReceiveMessage: sinon.stub().returns({ dispose: () => {} }),
      },
      onDidChangeVisibility: sinon.stub().returns({ dispose: () => {} }),
      visible: true,
    };
  });

  afterEach(() => {
    clock.restore();
    (SidebarPanel as any).instance = undefined;
  });

  it("createOrShow() throws when ExtensionContext is missing", () => {
    expect(() => SidebarPanel.createOrShow("/path", null, undefined)).to.throw();
  });

  it("createOrShow() returns an instance when context is provided", () => {
    const panel = SidebarPanel.createOrShow("/path", null, mockContext);
    expect(panel).to.be.instanceOf(SidebarPanel);
  });

  it("createOrShow() returns the same singleton on repeated calls", () => {
    const first = SidebarPanel.createOrShow("/path", null, mockContext);
    const second = SidebarPanel.createOrShow("/path", null, mockContext);
    expect(first).to.equal(second);
  });

  it("resolveWebviewView() sets HTML containing comP branding", () => {
    const panel = SidebarPanel.createOrShow("/path", null, mockContext);
    panel.resolveWebviewView(mockWebviewView, {} as any, {} as any);
    expect(mockWebviewView.webview.html).to.include("comP");
  });

  it("resolveWebviewView() registers a message handler on the webview", () => {
    const panel = SidebarPanel.createOrShow("/path", null, mockContext);
    panel.resolveWebviewView(mockWebviewView, {} as any, {} as any);
    expect(mockWebviewView.webview.onDidReceiveMessage.calledOnce).to.be.true;
  });

  it("setDaemonManager(null) posts daemonStatus: false", () => {
    const panel = SidebarPanel.createOrShow("/path", null, mockContext);
    panel.resolveWebviewView(mockWebviewView, {} as any, {} as any);
    postedMessages.length = 0;

    panel.setDaemonManager(null);

    const msg = postedMessages.find((m) => m.type === "daemonStatus");
    expect(msg).to.not.be.undefined;
    expect(msg.running).to.be.false;
  });

  it("setDaemonManager(daemon) posts daemonStatus: true", () => {
    const mockDaemon: any = {
      getStats: sinon.stub().resolves({ total_files: 5, total_nodes: 10, total_edges: 3 }),
      isRunning: () => true,
    };
    const panel = SidebarPanel.createOrShow("/path", null, mockContext);
    panel.resolveWebviewView(mockWebviewView, {} as any, {} as any);
    postedMessages.length = 0;

    panel.setDaemonManager(mockDaemon);

    const msg = postedMessages.find((m) => m.type === "daemonStatus");
    expect(msg).to.not.be.undefined;
    expect(msg.running).to.be.true;
  });

  it("getHtml() includes Re-index button with id reindexBtn", () => {
    const panel = SidebarPanel.createOrShow("/path", null, mockContext);
    panel.resolveWebviewView(mockWebviewView, {} as any, {} as any);
    expect(mockWebviewView.webview.html).to.include("reindexBtn");
    expect(mockWebviewView.webview.html).to.include("Re-index");
  });

  it("handleWebviewMessage('reindex') calls comp.forceReindex command", async () => {
    const vscode = require("vscode");
    const executeCommand = sinon.stub(vscode.commands, "executeCommand").resolves();

    const panel = SidebarPanel.createOrShow("/path", null, mockContext);
    panel.resolveWebviewView(mockWebviewView, {} as any, {} as any);

    const handler = mockWebviewView.webview.onDidReceiveMessage.firstCall.args[0];
    await handler({ command: "reindex" });

    expect(executeCommand.calledWith("comp.forceReindex")).to.be.true;
    executeCommand.restore();
  });

  it("setLifecycleCallbacks() stores callbacks without throwing", () => {
    const panel = SidebarPanel.createOrShow("/path", null, mockContext);
    const callbacks = {
      onStartRequest: sinon.stub().resolves(null),
      onStopRequest: sinon.stub().resolves(),
    };
    expect(() => panel.setLifecycleCallbacks(callbacks)).to.not.throw();
  });

  it("handleWebviewMessage('clearLogs') posts empty logsUpdate", async () => {
    const panel = SidebarPanel.createOrShow("/path", null, mockContext);
    panel.resolveWebviewView(mockWebviewView, {} as any, {} as any);
    postedMessages.length = 0;
    const handler = mockWebviewView.webview.onDidReceiveMessage.firstCall.args[0];
    await handler({ command: "clearLogs" });
    const msg = postedMessages.find((m) => m.type === "logsUpdate");
    expect(msg).to.not.be.undefined;
    expect(msg.logs).to.deep.equal([]);
  });

  it("handleWebviewMessage('refresh') calls getStats when daemon is set", async () => {
    const mockDaemon: any = {
      getStats: sinon.stub().resolves({ total_files: 1, total_nodes: 2, total_edges: 0 }),
      isRunning: () => true,
    };
    const panel = SidebarPanel.createOrShow("/path", null, mockContext);
    panel.resolveWebviewView(mockWebviewView, {} as any, {} as any);
    panel.setDaemonManager(mockDaemon);
    postedMessages.length = 0;
    const handler = mockWebviewView.webview.onDidReceiveMessage.firstCall.args[0];
    await handler({ command: "refresh" });
    expect(mockDaemon.getStats.called).to.be.true;
  });

  it("handleWebviewMessage('startDaemon') calls onStartRequest", async () => {
    const mockDaemon: any = {
      isRunning: () => true,
      getStats: sinon.stub().resolves({ total_files: 0, total_nodes: 0, total_edges: 0 }),
    };
    const callbacks = {
      onStartRequest: sinon.stub().resolves(mockDaemon),
      onStopRequest: sinon.stub().resolves(),
    };
    const panel = SidebarPanel.createOrShow("/path", null, mockContext);
    panel.resolveWebviewView(mockWebviewView, {} as any, {} as any);
    panel.setLifecycleCallbacks(callbacks);
    const handler = mockWebviewView.webview.onDidReceiveMessage.firstCall.args[0];
    await handler({ command: "startDaemon" });
    expect(callbacks.onStartRequest.calledOnce).to.be.true;
  });

  it("handleWebviewMessage('stopDaemon') calls onStopRequest", async () => {
    const mockDaemon: any = {
      isRunning: () => true,
      getStats: sinon.stub().resolves({ total_files: 0, total_nodes: 0, total_edges: 0 }),
    };
    const callbacks = {
      onStartRequest: sinon.stub().resolves(null),
      onStopRequest: sinon.stub().resolves(),
    };
    const panel = SidebarPanel.createOrShow("/path", null, mockContext);
    panel.resolveWebviewView(mockWebviewView, {} as any, {} as any);
    panel.setDaemonManager(mockDaemon);
    panel.setLifecycleCallbacks(callbacks);
    const handler = mockWebviewView.webview.onDidReceiveMessage.firstCall.args[0];
    await handler({ command: "stopDaemon" });
    expect(callbacks.onStopRequest.calledOnce).to.be.true;
  });

  it("handleWebviewMessage('addRepo') adds the repo picked via showOpenDialog", async () => {
    const vscode = require("vscode");
    const showOpenDialog = sinon
      .stub(vscode.window, "showOpenDialog")
      .resolves([{ fsPath: "/some/new/repo" }]);

    const mockDaemon: any = {
      isRunning: () => true,
      getStats: sinon.stub().resolves({ total_files: 0, total_nodes: 0, total_edges: 0 }),
      addRepo: sinon.stub().resolves({ alias: "repo", root_path: "/some/new/repo" }),
    };
    const panel = SidebarPanel.createOrShow("/path", null, mockContext);
    panel.resolveWebviewView(mockWebviewView, {} as any, {} as any);
    panel.setDaemonManager(mockDaemon);

    const handler = mockWebviewView.webview.onDidReceiveMessage.firstCall.args[0];
    await handler({ command: "addRepo" });

    expect(mockDaemon.addRepo.calledWith("/some/new/repo")).to.be.true;
    showOpenDialog.restore();
  });

  it("handleWebviewMessage('addRepo') does nothing when the dialog is cancelled", async () => {
    const vscode = require("vscode");
    const showOpenDialog = sinon.stub(vscode.window, "showOpenDialog").resolves(undefined);

    const mockDaemon: any = {
      isRunning: () => true,
      getStats: sinon.stub().resolves({ total_files: 0, total_nodes: 0, total_edges: 0 }),
      addRepo: sinon.stub().resolves({ alias: "repo", root_path: "/some/new/repo" }),
    };
    const panel = SidebarPanel.createOrShow("/path", null, mockContext);
    panel.resolveWebviewView(mockWebviewView, {} as any, {} as any);
    panel.setDaemonManager(mockDaemon);

    const handler = mockWebviewView.webview.onDidReceiveMessage.firstCall.args[0];
    await handler({ command: "addRepo" });

    expect(mockDaemon.addRepo.called).to.be.false;
    showOpenDialog.restore();
  });

  it("handleWebviewMessage('removeRepo') removes the repo after confirmation", async () => {
    const vscode = require("vscode");
    (vscode.window as any).showWarningMessage = sinon.stub().resolves("Remove");

    const mockDaemon: any = {
      isRunning: () => true,
      getStats: sinon.stub().resolves({ total_files: 0, total_nodes: 0, total_edges: 0 }),
      removeRepo: sinon.stub().resolves({ alias: "Alpha", removed_files: 3 }),
    };
    const panel = SidebarPanel.createOrShow("/path", null, mockContext);
    panel.resolveWebviewView(mockWebviewView, {} as any, {} as any);
    panel.setDaemonManager(mockDaemon);

    const handler = mockWebviewView.webview.onDidReceiveMessage.firstCall.args[0];
    await handler({ command: "removeRepo", alias: "Alpha" });

    expect(mockDaemon.removeRepo.calledWith("Alpha")).to.be.true;
  });

  it("handleWebviewMessage('removeRepo') skips removal when the confirmation is dismissed", async () => {
    const vscode = require("vscode");
    (vscode.window as any).showWarningMessage = sinon.stub().resolves(undefined);

    const mockDaemon: any = {
      isRunning: () => true,
      getStats: sinon.stub().resolves({ total_files: 0, total_nodes: 0, total_edges: 0 }),
      removeRepo: sinon.stub().resolves({ alias: "Alpha", removed_files: 3 }),
    };
    const panel = SidebarPanel.createOrShow("/path", null, mockContext);
    panel.resolveWebviewView(mockWebviewView, {} as any, {} as any);
    panel.setDaemonManager(mockDaemon);

    const handler = mockWebviewView.webview.onDidReceiveMessage.firstCall.args[0];
    await handler({ command: "removeRepo", alias: "Alpha" });

    expect(mockDaemon.removeRepo.called).to.be.false;
  });

  it("getHtml() includes the Add Repo button", () => {
    const panel = SidebarPanel.createOrShow("/path", null, mockContext);
    panel.resolveWebviewView(mockWebviewView, {} as any, {} as any);
    expect(mockWebviewView.webview.html).to.include("addRepoBtn");
  });

  it("dispose() clears the stats interval without throwing", () => {
    const mockDaemon: any = {
      getStats: sinon.stub().resolves({ total_files: 0, total_nodes: 0, total_edges: 0 }),
      isRunning: () => true,
    };
    const panel = SidebarPanel.createOrShow("/path", null, mockContext);
    panel.resolveWebviewView(mockWebviewView, {} as any, {} as any);
    panel.setDaemonManager(mockDaemon);
    expect(() => panel.dispose()).not.to.throw();
  });
});
