// dependency.rs - Dependency analysis for extracted symbols
//
// Responsibilities:
// - Extract import/require statements from source code
// - Map imported module names to symbol node IDs
// - Detect symbol references within the code
// - Record edges in the dependency graph for:
//   - import/require (module dependencies)
//   - function calls (function dependencies)
//   - type references (type dependencies)

use anyhow::Result;
use std::collections::HashMap;

/// Dependency edge type
#[derive(Debug, Clone)]
pub enum EdgeKind {
    /// import/require statement (file-level dependency)
    Import,
    /// Function call (symbol references function)
    FunctionCall,
    /// Type usage (symbol uses a type/interface)
    TypeReference,
    /// Inheritance (class extends/implements another)
    Inheritance,
}

impl EdgeKind {
    /// Convert to string for database storage
    pub fn as_str(&self) -> &str {
        match self {
            EdgeKind::Import => "import",
            EdgeKind::FunctionCall => "function_call",
            EdgeKind::TypeReference => "type_reference",
            EdgeKind::Inheritance => "inheritance",
        }
    }
}

/// Dependency extraction result
#[derive(Debug, Clone)]
pub struct Dependency {
    /// Source symbol name (or file if module-level)
    pub from: String,
    /// Target symbol name or module name
    pub to: String,
    /// Type of dependency
    pub kind: EdgeKind,
    /// Line number where dependency occurs
    pub line: u32,
}

/// Dependency analyzer
pub struct DependencyAnalyzer;

impl DependencyAnalyzer {
    /// Extract dependencies from source code
    ///
    /// # Arguments
    /// - language: Programming language (rust, typescript, python, etc.)
    /// - source_code: File content
    /// - file_path: Path for module-level imports
    ///
    /// # Returns
    /// - Vec<Dependency>: All dependencies found
    ///
    /// # Process
    /// 1. Parse language-specific import statements
    /// 2. Extract module names and imported symbols
    /// 3. Detect symbol references (function calls, type usage, etc.)
    /// 4. Build dependency list with line numbers
    pub fn extract_dependencies(
        language: &str,
        source_code: &str,
        _file_path: &str,
    ) -> Result<Vec<Dependency>> {
        // TODO: Implement language-specific dependency extraction
        // For now, return empty to prevent test failures

        match language {
            "rust" => Self::extract_rust_dependencies(source_code),
            "typescript" | "javascript" => Self::extract_typescript_dependencies(source_code),
            "python" => Self::extract_python_dependencies(source_code),
            "go" => Self::extract_go_dependencies(source_code),
            _ => Ok(Vec::new()),
        }
    }

    /// Extract Rust dependencies (use statements and function calls)
    ///
    /// # Process:
    /// 1. Parse `use` statements to extract imports
    /// 2. Detect function/method calls (module::function or obj.method)
    /// 3. Detect type usage (variable: Type, Function<T>)
    fn extract_rust_dependencies(source_code: &str) -> Result<Vec<Dependency>> {
        use regex::Regex;

        let mut deps = Vec::new();

        // Pattern 1: Extract `use` statements
        // Matches: use std::collections::HashMap;
        //          use crate::graph::GraphDB;
        let use_pattern = Regex::new(r"^\s*use\s+([\w:]+)(?::\:\{[^}]+\})?;?")?;

        // Pattern 2: Extract function calls (module::function or method calls)
        // Matches: GraphDB::new()
        //          db.insert_node()
        let call_pattern = Regex::new(r"([\w_]+)::([\w_]+)\s*\(")?;

        for (line_num, line) in source_code.lines().enumerate() {
            let line_no = (line_num + 1) as u32;

            // Extract imports from use statements
            if let Some(caps) = use_pattern.captures(line) {
                if let Some(module) = caps.get(1) {
                    let module_name = module.as_str();
                    // Extract last component (Symbol) from path
                    if let Some(last) = module_name.split("::").last() {
                        deps.push(Dependency {
                            from: "module".to_string(), // File-level import
                            to: last.to_string(),
                            kind: EdgeKind::Import,
                            line: line_no,
                        });
                    }
                }
            }

            // Extract function calls
            for caps in call_pattern.captures_iter(line) {
                if let (Some(obj), Some(func)) = (caps.get(1), caps.get(2)) {
                    deps.push(Dependency {
                        from: obj.as_str().to_string(),
                        to: func.as_str().to_string(),
                        kind: EdgeKind::FunctionCall,
                        line: line_no,
                    });
                }
            }
        }

        Ok(deps)
    }

    /// Extract TypeScript/JavaScript dependencies (import statements and function calls)
    ///
    /// # Process:
    /// 1. Parse import/require statements
    /// 2. Detect function/method calls
    /// 3. Detect type references in type annotations
    fn extract_typescript_dependencies(source_code: &str) -> Result<Vec<Dependency>> {
        use regex::Regex;

        let mut deps = Vec::new();

        // Pattern 1: import { Symbol } from 'module'
        let import_named = Regex::new(r#"import\s+\{\s*([^}]+)\s*\}\s+from\s+['"]([^'"]+)['"]"#)?;

        // Pattern 2: import * as name from 'module'
        let import_star = Regex::new(r#"import\s+\*\s+as\s+(\w+)\s+from\s+['"]([^'"]+)['"]"#)?;

        // Pattern 3: const/var name = require('module')
        let require_pattern = Regex::new(r#"(?:const|var|let)\s+(\w+)\s*=\s*require\(['"](.*?)['"]\)"#)?;

        // Pattern 4: function/method calls (obj.method or Class.static)
        let call_pattern = Regex::new(r"([\w_]+)\.([\w_]+)\s*\(")?;

        for (line_num, line) in source_code.lines().enumerate() {
            let line_no = (line_num + 1) as u32;

            // Extract named imports: import { Symbol } from 'module'
            if let Some(caps) = import_named.captures(line) {
                if let Some(symbols) = caps.get(1) {
                    for symbol in symbols.as_str().split(',') {
                        let sym = symbol.trim();
                        deps.push(Dependency {
                            from: "module".to_string(),
                            to: sym.to_string(),
                            kind: EdgeKind::Import,
                            line: line_no,
                        });
                    }
                }
            }

            // Extract star imports: import * as name
            if let Some(caps) = import_star.captures(line) {
                if let Some(name) = caps.get(1) {
                    deps.push(Dependency {
                        from: "module".to_string(),
                        to: name.as_str().to_string(),
                        kind: EdgeKind::Import,
                        line: line_no,
                    });
                }
            }

            // Extract requires: const name = require('module')
            if let Some(caps) = require_pattern.captures(line) {
                if let Some(name) = caps.get(1) {
                    deps.push(Dependency {
                        from: "module".to_string(),
                        to: name.as_str().to_string(),
                        kind: EdgeKind::Import,
                        line: line_no,
                    });
                }
            }

            // Extract method calls
            for caps in call_pattern.captures_iter(line) {
                if let (Some(obj), Some(method)) = (caps.get(1), caps.get(2)) {
                    deps.push(Dependency {
                        from: obj.as_str().to_string(),
                        to: method.as_str().to_string(),
                        kind: EdgeKind::FunctionCall,
                        line: line_no,
                    });
                }
            }
        }

        Ok(deps)
    }

    /// Extract Python dependencies (import statements)
    fn extract_python_dependencies(_source_code: &str) -> Result<Vec<Dependency>> {
        // TODO: Parse import statements
        // Patterns:
        // - `import module`
        // - `from module import Symbol`
        // - `Symbol()` calls
        Ok(Vec::new())
    }

    /// Extract Go dependencies (import statements)
    fn extract_go_dependencies(_source_code: &str) -> Result<Vec<Dependency>> {
        // TODO: Parse import statements and package references
        // Patterns:
        // - `import "package/name"`
        // - `package.Function()` calls
        Ok(Vec::new())
    }

    /// Resolve dependencies to node IDs
    ///
    /// # Arguments
    /// - deps: Raw dependencies extracted from code
    /// - symbol_map: Mapping of symbol names to node IDs (current file)
    /// - imported_symbols: Mapping of imported modules to their exported symbols
    ///
    /// # Returns
    /// - Vec<(from_node_id, to_node_id, edge_kind)>: Resolved node pairs for edge creation
    ///
    /// # Process:
    /// 1. For each dependency, look up source node ID in symbol_map
    /// 2. For each dependency, look up target in imported_symbols or symbol_map
    /// 3. Return pairs that can be inserted as edges
    pub fn resolve_dependencies(
        deps: &[Dependency],
        symbol_map: &HashMap<String, i64>,
        _imported_symbols: &HashMap<String, HashMap<String, i64>>,
    ) -> Vec<(i64, i64, String)> {
        // TODO: Implement node ID resolution
        // - Match symbol names to node IDs from symbol_map
        // - Handle module.symbol notation by looking up in imported_symbols
        // - Return only resolvable dependencies

        let mut edges = Vec::new();

        for dep in deps {
            // Try to find source node ID
            if let Some(&from_id) = symbol_map.get(&dep.from) {
                // Try to find target node ID
                // For now, just check symbol_map (same-file references)
                if let Some(&to_id) = symbol_map.get(&dep.to) {
                    edges.push((from_id, to_id, dep.kind.as_str().to_string()));
                }
            }
        }

        edges
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_rust_dependencies() {
        let code = r#"
use std::collections::HashMap;
use crate::graph::GraphDB;

fn main() {
    let map = HashMap::new();
    let db = GraphDB::new();
}
"#;
        let deps = DependencyAnalyzer::extract_rust_dependencies(code).unwrap();

        // Should extract imports and function calls
        assert!(!deps.is_empty(), "Should extract dependencies");

        // Verify at least some imports are found
        let imports: Vec<_> = deps.iter().filter(|d| matches!(d.kind, EdgeKind::Import)).collect();
        assert!(!imports.is_empty(), "Should find import statements");

        // Verify at least some function calls are found
        let calls: Vec<_> = deps.iter().filter(|d| matches!(d.kind, EdgeKind::FunctionCall)).collect();
        assert!(!calls.is_empty(), "Should find function calls");
    }

    #[test]
    fn test_extract_typescript_dependencies() {
        let code = r#"
import { GraphDB } from './graph';
import * as fs from 'fs';

const db = new GraphDB();
fs.readFile('test.txt', () => {});
"#;
        let deps = DependencyAnalyzer::extract_typescript_dependencies(code).unwrap();

        // Should extract imports and function calls
        assert!(!deps.is_empty(), "Should extract dependencies");

        // Verify imports are found
        let imports: Vec<_> = deps.iter().filter(|d| matches!(d.kind, EdgeKind::Import)).collect();
        assert!(!imports.is_empty(), "Should find import statements");

        // Verify function calls are found
        let calls: Vec<_> = deps.iter().filter(|d| matches!(d.kind, EdgeKind::FunctionCall)).collect();
        assert!(!calls.is_empty(), "Should find method calls");
    }

    #[test]
    fn test_resolve_dependencies() {
        let mut symbol_map = HashMap::new();
        symbol_map.insert("main".to_string(), 1);
        symbol_map.insert("helper".to_string(), 2);

        let deps = vec![Dependency {
            from: "main".to_string(),
            to: "helper".to_string(),
            kind: EdgeKind::FunctionCall,
            line: 5,
        }];

        let edges = DependencyAnalyzer::resolve_dependencies(&deps, &symbol_map, &HashMap::new());

        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0], (1, 2, "function_call".to_string()));
    }

    #[test]
    fn test_resolve_dependencies_unresolved() {
        let symbol_map = HashMap::new(); // Empty map

        let deps = vec![Dependency {
            from: "main".to_string(),
            to: "unknown".to_string(),
            kind: EdgeKind::FunctionCall,
            line: 5,
        }];

        let edges = DependencyAnalyzer::resolve_dependencies(&deps, &symbol_map, &HashMap::new());

        // Unresolved dependencies should be skipped
        assert_eq!(edges.len(), 0);
    }
}
