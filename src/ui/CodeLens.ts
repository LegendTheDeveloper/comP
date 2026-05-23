// CodeLens - Display dependency information as inline annotations
//
// Responsibilities:
// 1. Query daemon for symbols in active document
// 2. Calculate dependent count for each exported symbol
// 3. Render CodeLens inline showing "X references" for each symbol

import * as vscode from "vscode";
import { DaemonManager } from "../daemon/DaemonManager";

interface SymbolInfo {
  id: number;
  name: string;
  kind: string;
  line: number;
  column: number;
  dependentCount: number;
  scope?: string;
}

interface DaemonSymbol {
  id: number;
  name: string;
  kind: string;
  line: number;
  column: number;
  scope?: string;
  dependents?: number;
}

/**
 * CodeLens provider for displaying symbol dependency information
 *
 * # 責務
 * - サポート言語判定：TypeScript, Python, Go, Rust, Java, C#等
 * - daemon からシンボル情報を取得
 * - export/public シンボルのみを表示対象に
 * - 各シンボルの依存カウントを表示
 */
export class DependencyCodeLensProvider implements vscode.CodeLensProvider {
  private daemonManager: DaemonManager;
  private onDidChangeCodeLensesEmitter = new vscode.EventEmitter<void>();
  private cache = new Map<string, DaemonSymbol[]>();

  readonly onDidChangeCodeLenses = this.onDidChangeCodeLensesEmitter.event;

  constructor(daemonManager: DaemonManager) {
    this.daemonManager = daemonManager;
  }

  /**
   * Provide code lenses for document
   *
   * # 入力
   * - document: VSCode ドキュメント
   * - token: キャンセルトークン
   *
   * # 出力
   * - CodeLens 配列（export されたシンボルの依存情報）
   *
   * # 前提条件
   * - daemon が起動済み
   */
  async provideCodeLenses(
    document: vscode.TextDocument,
    _token: vscode.CancellationToken
  ): Promise<vscode.CodeLens[]> {
    // 言語フィルタ：サポート対象言語のみ処理
    if (!this.isSupportedLanguage(document.languageId)) {
      return [];
    }

    const codeLenses: vscode.CodeLens[] = [];

    try {
      const filePath = document.uri.fsPath;

      // キャッシュから取得、なければ daemon に問い合わせ
      let symbols = this.cache.get(filePath);
      if (!symbols) {
        try {
          symbols = (await this.daemonManager.getSymbols(filePath)) || [];
          this.cache.set(filePath, symbols);
        } catch (error) {
          // daemon 問い合わせに失敗した場合は空配列を返す
          console.debug("[comP] Failed to query symbols:", error);
          return [];
        }
      }

      // export/public シンボルのみフィルタ、CodeLens を生成
      for (const symbol of symbols) {
        // export や public シンボルのみ表示
        if (!this.isExportedSymbol(symbol)) {
          continue;
        }

        // 行番号が有効範囲内か確認
        if (symbol.line >= document.lineCount) {
          continue;
        }

        const line = document.lineAt(symbol.line);
        const range = new vscode.Range(
          new vscode.Position(symbol.line, symbol.column || 0),
          new vscode.Position(
            symbol.line,
            Math.min(
              symbol.column + (symbol.name?.length || 0),
              line.range.end.character
            )
          )
        );

        const symbolInfo: SymbolInfo = {
          id: symbol.id,
          name: symbol.name,
          kind: symbol.kind,
          line: symbol.line,
          column: symbol.column || 0,
          dependentCount: symbol.dependents || 0,
          scope: symbol.scope,
        };

        const codeLens = new vscode.CodeLens(range, undefined);
        (codeLens as any).data = symbolInfo;
        codeLenses.push(codeLens);
      }

      return codeLenses;
    } catch (error) {
      console.error("[comP] CodeLens error:", error);
      return [];
    }
  }

  /**
   * Resolve code lens command
   *
   * # 入力
   * - codeLens: VSCode CodeLens オブジェクト
   * - token: キャンセルトークン
   *
   * # 出力
   * - command が設定された CodeLens
   *
   * # 動作
   * - 依存カウント > 0: "X references" を表示、クリックで参照プロバイダを起動
   * - 依存カウント = 0: "No references" を表示（no-op コマンド）
   */
  resolveCodeLens(
    codeLens: vscode.CodeLens,
    _token: vscode.CancellationToken
  ): vscode.CodeLens {
    const symbolInfo = (codeLens as any).data as SymbolInfo;

    if (symbolInfo.dependentCount > 0) {
      const refText = `${symbolInfo.dependentCount} reference${
        symbolInfo.dependentCount === 1 ? "" : "s"
      }`;

      codeLens.command = {
        title: refText,
        command: "vscode.executeReferenceProvider",
        arguments: [codeLens.range.start],
      };
    } else {
      codeLens.command = {
        title: "No references",
        command: "",
      };
    }

    return codeLens;
  }

  /**
   * Check if symbol is exported/public
   *
   * # 入力
   * - symbol: daemon から返されたシンボル
   *
   * # 出力
   * - true if symbol is exported/public and should be displayed
   *
   * # フィルタ条件
   * - scope が "export", "public" を含む
   * - kind が function, class, interface, type 等（変数は除外）
   */
  private isExportedSymbol(symbol: DaemonSymbol): boolean {
    // 表示対象の kind
    const displayKinds = [
      "function",
      "class",
      "interface",
      "type",
      "enum",
      "struct",
      "trait",
      "module",
    ];

    if (!displayKinds.includes(symbol.kind.toLowerCase())) {
      return false;
    }

    // scope が export/public を含む場合のみ表示
    if (!symbol.scope) {
      return false;
    }

    const scopeLower = symbol.scope.toLowerCase();
    return (
      scopeLower.includes("export") ||
      scopeLower.includes("public") ||
      scopeLower.includes("default")
    );
  }

  /**
   * Check if language is supported by comP
   *
   * # 入力
   * - languageId: VSCode language ID
   *
   * # 出力
   * - true if supported, false otherwise
   *
   * # サポート言語
   * - TypeScript, JavaScript, Python, Go, Rust, Java, C#, C++, Ruby, PHP, SQL, JSON, YAML
   */
  private isSupportedLanguage(languageId: string): boolean {
    const supported = [
      "typescript",
      "javascript",
      "python",
      "go",
      "rust",
      "java",
      "csharp",
      "cpp",
      "ruby",
      "php",
      "sql",
      "json",
      "yaml",
    ];
    return supported.includes(languageId.toLowerCase());
  }

  /**
   * Clear cache for file (called when file is re-indexed)
   */
  invalidateFile(filePath: string): void {
    this.cache.delete(filePath);
  }

  /**
   * Clear all cache
   */
  clearCache(): void {
    this.cache.clear();
  }

  /**
   * Signal that code lenses should be refreshed
   */
  refresh(): void {
    this.onDidChangeCodeLensesEmitter.fire();
  }

  /**
   * Dispose provider resources
   */
  dispose(): void {
    this.onDidChangeCodeLensesEmitter.dispose();
    this.cache.clear();
  }
}
