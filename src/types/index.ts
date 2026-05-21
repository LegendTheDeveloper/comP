// Type definitions for comP

/** Symbol kind (function, class, type, variable, etc.) */
export type SymbolKind =
  | "function"
  | "class"
  | "interface"
  | "type"
  | "variable"
  | "constant"
  | "method"
  | "property"
  | "enum"
  | "module"
  | "namespace"
  | "unknown";

/** Programming language */
export type Language =
  | "typescript"
  | "javascript"
  | "python"
  | "rust"
  | "go"
  | "java"
  | "c"
  | "cpp"
  | "csharp"
  | "ruby"
  | "php"
  | "bash"
  | "sql"
  | "html"
  | "css"
  | "json"
  | "yaml"
  | "markdown"
  | "xml"
  | "unknown";

/** A symbol/node in the code graph */
export interface CodeSymbol {
  id: number;
  name: string;
  kind: SymbolKind;
  file: string;
  line: number;
  column: number;
  signature?: string;
  scope?: string;
  isExported: boolean;
}

/** A file in the index */
export interface IndexedFile {
  id: number;
  path: string;
  language: Language;
  hash: string;
  lastIndexed: number;
  symbolCount: number;
}

/** Context capsule for AI agent */
export interface ContextCapsule {
  pivotFiles: {
    path: string;
    content: string;
    tokens: number;
  }[];
  relatedFiles: {
    path: string;
    symbols: CodeSymbol[];
    tokens: number;
  }[];
  totalTokens: number;
  tokenSavings: string; // e.g. "65%"
  estimatedCost: string; // e.g. "$0.04"
}

/** Impact analysis result */
export interface ImpactAnalysis {
  affectedFiles: Map<string, CodeSymbol[]>;
  impactCount: number;
  severity: "low" | "medium" | "high";
}
