// Test setup - mock vscode module for unit tests
import Module from 'module';

const originalRequire = Module.prototype.require;

// WHY: キャッシュしないと require('vscode') のたびに新オブジェクトが生成され、
//      テストファイルとソースファイルで異なる参照を持つ。モックのプロパティ上書きが
//      ソース側に伝わらないため、全モジュールで同一インスタンスを共有する。
let vscodeMock: any = null;

Module.prototype.require = function (id: string) {
  if (id === 'vscode') {
    if (vscodeMock) return vscodeMock;
    // VSCode EventEmitter: fire() で listeners に通知、event プロパティで購読
    class MockVSCodeEventEmitter<T = void> {
      private listeners: Array<(e: T) => any> = [];

      // .event は "リスナー登録関数" を返す (VSCode API の仕様)
      get event() {
        return (listener: (e: T) => any): { dispose: () => void } => {
          this.listeners.push(listener);
          return { dispose: () => this.fire = this.fire };
        };
      }

      fire(data?: T): void {
        for (const listener of this.listeners) {
          listener(data as T);
        }
      }

      dispose(): void {
        this.listeners = [];
      }
    }

    // VSCode Position: 行・文字位置を保持するだけの値オブジェクト
    class MockPosition {
      line: number;
      character: number;
      constructor(line: number, character: number) {
        this.line = line;
        this.character = character;
      }
    }

    // VSCode Range: start/end の Position ペア
    class MockRange {
      start: MockPosition;
      end: MockPosition;
      constructor(start: MockPosition, end: MockPosition) {
        this.start = start;
        this.end = end;
      }
    }

    // VSCode CodeLens: Range と任意の Command を持つ
    class MockCodeLens {
      range: MockRange;
      command: any;
      isResolved: boolean;
      constructor(range: MockRange, command?: any) {
        this.range = range;
        this.command = command;
        this.isResolved = false;
      }
    }

    vscodeMock = {
      ExtensionContext: class {},
      EventEmitter: MockVSCodeEventEmitter,
      Position: MockPosition,
      Range: MockRange,
      CodeLens: MockCodeLens,
      workspace: {
        workspaceFolders: undefined,
      },
      StatusBarAlignment: { Left: 1, Right: 2 },
      ThemeColor: class {
        id: string;
        constructor(id: string) { this.id = id; }
      },
      commands: {
        registerCommand: () => ({ dispose: () => {} }),
      },
      window: {
        createWebviewPanel: () => {},
        createStatusBarItem: () => ({
          text: "",
          tooltip: "",
          command: "",
          backgroundColor: undefined,
          show: () => {},
          dispose: () => {},
        }),
        showInformationMessage: () => Promise.resolve(undefined),
        showErrorMessage: () => Promise.resolve(undefined),
        showWarningMessage: () => Promise.resolve(undefined),
        showQuickPick: () => Promise.resolve(undefined),
        showInputBox: () => Promise.resolve(undefined),
        activeTextEditor: undefined,
      },
    };
    return vscodeMock;
  }
  return originalRequire.apply(this, [id] as any);
};
