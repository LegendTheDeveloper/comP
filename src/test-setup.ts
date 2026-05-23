// Test setup - mock vscode module for unit tests
import Module from 'module';

const originalRequire = Module.prototype.require;

Module.prototype.require = function (id: string) {
  if (id === 'vscode') {
    return {
      ExtensionContext: class {},
      workspace: {
        workspaceFolders: undefined,
      },
      commands: {
        registerCommand: () => {},
      },
      window: {
        createWebviewPanel: () => {},
      },
    };
  }
  return originalRequire.apply(this, [id] as any);
};
