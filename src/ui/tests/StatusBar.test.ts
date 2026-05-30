// StatusBar Unit Tests
//
// Coverage:
// - constructor creates status bar item
// - show() updates text and respects Error state
// - updateStats() formats symbols/status correctly
// - updateProgress() calculates percentage
// - dispose() releases the item

import { expect } from "chai";
import * as sinon from "sinon";
import * as vscode from "vscode";
import { StatusBar } from "../StatusBar";

describe("StatusBar", () => {
  let mockItem: any;

  beforeEach(() => {
    mockItem = {
      text: "",
      tooltip: "",
      command: "",
      backgroundColor: undefined,
      show: sinon.stub(),
      dispose: sinon.stub(),
    };
    // Overwrite the shared mock object in test-setup.ts (referenced globally)
    (vscode.window as any).createStatusBarItem = sinon.stub().returns(mockItem);
  });

  it("constructor creates a status bar item", () => {
    new StatusBar();
    expect((vscode.window as any).createStatusBarItem.calledOnce).to.be.true;
  });

  it("show() sets text and calls show on the item", () => {
    const bar = new StatusBar();
    bar.show("Ready");
    expect(mockItem.text).to.equal("◈ comP: Ready");
    expect(mockItem.show.calledOnce).to.be.true;
  });

  it('show("Error") sets error background color', () => {
    const bar = new StatusBar();
    bar.show("Error");
    expect(mockItem.backgroundColor).to.not.be.undefined;
  });

  it('show("Ready") clears background color', () => {
    const bar = new StatusBar();
    bar.show("Error");
    bar.show("Ready");
    expect(mockItem.backgroundColor).to.be.undefined;
  });

  it("updateStats() formats node count and status into text", () => {
    const bar = new StatusBar();
    bar.updateStats(123, 50, "Ready");
    expect(mockItem.text).to.include("123");
    expect(mockItem.text).to.include("Ready");
  });

  it("updateStats() shows Indexing icon when status is Indexing", () => {
    const bar = new StatusBar();
    bar.updateStats(0, 0, "Indexing");
    expect(mockItem.text).to.include("Indexing");
  });

  it("updateStats() sets tooltip with file and symbol counts", () => {
    const bar = new StatusBar();
    bar.updateStats(100, 42, "Ready");
    expect(mockItem.tooltip).to.include("42");
    expect(mockItem.tooltip).to.include("100");
  });

  it("updateProgress() shows calculated percentage", () => {
    const bar = new StatusBar();
    bar.updateProgress(3, 10);
    expect(mockItem.text).to.include("30%");
  });

  it("updateProgress() handles 100%", () => {
    const bar = new StatusBar();
    bar.updateProgress(10, 10);
    expect(mockItem.text).to.include("100%");
  });

  it("dispose() calls dispose on the underlying item", () => {
    const bar = new StatusBar();
    bar.dispose();
    expect(mockItem.dispose.calledOnce).to.be.true;
  });
});
