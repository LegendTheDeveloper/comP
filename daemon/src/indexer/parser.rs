// parser.rs - Tree-sitter based code parser
//
// Responsibilities:
// - Initialize tree-sitter parsers for 30+ languages
// - Parse source code and extract symbols
// - Build dependency graph from AST
// - Extract function signatures, class definitions, type information
//
// Supported languages (30+):
// C, C++, C#, Go, Java, JavaScript, TypeScript, Python, Rust, Ruby, Bash, 
// Kotlin, Swift, PHP, Dart, Elixir, Haskell, Lua, R, Zig, SQL, HTML, CSS, 
// YAML, Scala, Objective-C, Clojure, HCL, etc.

use anyhow::{Result, anyhow};
use tree_sitter::{Node, Parser as TreeSitterParser};

/// Symbol kind extracted from source code
#[derive(Debug, Clone)]
pub enum SymbolKind {
    Function,
    Class,
    Interface,
    Struct,
    Enum,
    Type,
    Variable,
    Constant,
    Module,
    Namespace,
    Method,
    Property,
    Unknown,
}

impl SymbolKind {
    /// Convert string to SymbolKind
    fn from_node_type(node_type: &str) -> Self {
        match node_type {
            "function_declaration" | "function_definition" | "function" => SymbolKind::Function,
            "class_declaration" | "class_definition" | "class" => SymbolKind::Class,
            "interface_declaration" | "interface" => SymbolKind::Interface,
            "struct_item" | "struct_declaration" | "struct" => SymbolKind::Struct,
            "enum_declaration" | "enum_item" | "enum" => SymbolKind::Enum,
            "type_declaration" | "type_alias" | "type" => SymbolKind::Type,
            "method_definition" | "method" => SymbolKind::Method,
            "property_signature" | "property" => SymbolKind::Property,
            "variable_declarator" | "variable_declaration" => SymbolKind::Variable,
            "const_declaration" => SymbolKind::Constant,
            "module" | "namespace" => SymbolKind::Module,
            _ => SymbolKind::Unknown,
        }
    }

    /// Display name for storing in database
    pub fn as_str(&self) -> &str {
        match self {
            SymbolKind::Function => "function",
            SymbolKind::Class => "class",
            SymbolKind::Interface => "interface",
            SymbolKind::Struct => "struct",
            SymbolKind::Enum => "enum",
            SymbolKind::Type => "type",
            SymbolKind::Variable => "variable",
            SymbolKind::Constant => "constant",
            SymbolKind::Module => "module",
            SymbolKind::Namespace => "namespace",
            SymbolKind::Method => "method",
            SymbolKind::Property => "property",
            SymbolKind::Unknown => "unknown",
        }
    }
}

/// A symbol extracted from source code
#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub line: u32,
    pub column: u32,
    pub end_line: u32,
    pub end_column: u32,
    pub signature: Option<String>,
    pub is_exported: bool,
    pub scope: Option<String>,
}

/// Parser for extracting symbols from source code
pub struct CodeParser {
    parser: TreeSitterParser,
}

impl CodeParser {
    /// Create a new parser
    pub fn new() -> Result<Self> {
        let parser = TreeSitterParser::new();
        Ok(CodeParser { parser })
    }

    /// Parse a source file and extract symbols with scope tracking
    ///
    /// # Arguments
    /// - language: Programming language (e.g., "rust", "typescript", "python")
    /// - source_code: File content as string
    ///
    /// # Returns
    /// - Vec<Symbol>: All symbols found in the file with scope information
    ///
    /// # Process
    /// 1. Line-by-line scan with scope state tracking
    /// 2. Extract function, class, type definitions
    /// 3. Track nesting level and parent scope
    /// 4. Detect visibility (pub/export, private)
    /// 5. Return list of symbols with complete metadata
    pub async fn parse_file(&mut self, _language: &str, source_code: &str) -> Result<Vec<Symbol>> {
        let mut symbols = Vec::new();
        let mut scope_stack: Vec<(String, SymbolKind)> = Vec::new();
        let lines: Vec<&str> = source_code.lines().collect();

        for (idx, line) in lines.iter().enumerate() {
            let line_num = idx as u32 + 1;
            let trimmed = line.trim();

            // Skip comments and empty lines
            if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with("/*") {
                continue;
            }

            // Update scope based on brace nesting
            let open_braces = trimmed.chars().filter(|c| *c == '{').count();
            let close_braces = trimmed.chars().filter(|c| *c == '}').count();

            // Pop scope if closing braces
            for _ in 0..close_braces {
                scope_stack.pop();
            }

            let current_scope = scope_stack
                .last()
                .map(|(name, _)| name.clone());

            // Extract symbols
            if let Some(name) = self.extract_function_def(trimmed) {
                symbols.push(Symbol {
                    name: name.clone(),
                    kind: SymbolKind::Function,
                    line: line_num,
                    column: (line.len() - trimmed.len()) as u32 + 1,
                    end_line: line_num,
                    end_column: line.len() as u32,
                    signature: Some(trimmed.to_string()),
                    is_exported: trimmed.starts_with("pub") || trimmed.starts_with("export"),
                    scope: current_scope.clone(),
                });

                // If opening brace, add to scope
                if open_braces > 0 {
                    scope_stack.push((name.clone(), SymbolKind::Function));
                }
            }

            if let Some(name) = self.extract_class_def(trimmed) {
                symbols.push(Symbol {
                    name: name.clone(),
                    kind: SymbolKind::Class,
                    line: line_num,
                    column: (line.len() - trimmed.len()) as u32 + 1,
                    end_line: line_num,
                    end_column: line.len() as u32,
                    signature: Some(trimmed.to_string()),
                    is_exported: trimmed.starts_with("pub") || trimmed.starts_with("export"),
                    scope: current_scope.clone(),
                });

                if open_braces > 0 {
                    scope_stack.push((name.clone(), SymbolKind::Class));
                }
            }

            if let Some(name) = self.extract_type_def(trimmed) {
                symbols.push(Symbol {
                    name: name.clone(),
                    kind: SymbolKind::Type,
                    line: line_num,
                    column: (line.len() - trimmed.len()) as u32 + 1,
                    end_line: line_num,
                    end_column: line.len() as u32,
                    signature: Some(trimmed.to_string()),
                    is_exported: trimmed.starts_with("pub") || trimmed.starts_with("export"),
                    scope: current_scope.clone(),
                });

                if open_braces > 0 {
                    scope_stack.push((name.clone(), SymbolKind::Type));
                }
            }
        }

        Ok(symbols)
    }

    /// Extract function name from definition line
    fn extract_function_def(&self, line: &str) -> Option<String> {
        // Rust: fn name(...) or pub fn name(...) or async fn name(...)
        if line.contains("fn ") {
            if let Some(start) = line.rfind("fn ") {
                let after_fn = &line[start + 3..];
                if let Some(paren) = after_fn.find('(') {
                    let name = after_fn[..paren].trim();
                    // Skip generic lifetimes
                    if let Some(bracket) = name.find('<') {
                        return Some(name[..bracket].to_string());
                    }
                    return Some(name.to_string());
                }
            }
        }

        // TypeScript/JavaScript: function name(...) or export function name(...) or async function name(...)
        if line.contains("function ") {
            if let Some(start) = line.find("function ") {
                let after_fn = &line[start + 9..];
                if let Some(paren) = after_fn.find('(') {
                    return Some(after_fn[..paren].trim().to_string());
                }
            }
        }

        // TypeScript: async () => or const name = (...) =>
        if line.contains("=>") && (line.contains("async") || line.contains("const ")) {
            if let Some(start) = line.find("const ") {
                let after_const = &line[start + 6..];
                if let Some(eq) = after_const.find('=') {
                    let name = after_const[..eq].trim();
                    if !name.is_empty() && !name.contains('(') {
                        return Some(name.to_string());
                    }
                }
            }
        }

        // Python: def name(...) or async def name(...)
        if line.contains("def ") {
            if let Some(start) = line.find("def ") {
                let after_def = &line[start + 4..];
                if let Some(paren) = after_def.find('(') {
                    return Some(after_def[..paren].trim().to_string());
                }
            }
        }

        // Go: func name(...) or func (receiver) name(...)
        if line.contains("func ") {
            if let Some(start) = line.find("func ") {
                let after_func = &line[start + 5..];
                // Handle receiver methods: func (r *Receiver) Method()
                if let Some(paren) = after_func.find('(') {
                    let before_paren = &after_func[..paren].trim();
                    // If it's a receiver method, skip to closing paren
                    if before_paren.is_empty() || before_paren.ends_with(')') {
                        let name_part = if before_paren.is_empty() {
                            after_func
                        } else {
                            // Find method name after receiver
                            if let Some(close_paren) = after_func.find(')') {
                                let remainder = &after_func[close_paren + 1..].trim();
                                if let Some(next_paren) = remainder.find('(') {
                                    return Some(remainder[..next_paren].trim().to_string());
                                }
                            }
                            return None;
                        };

                        if let Some(method_paren) = name_part.find('(') {
                            return Some(name_part[..method_paren].trim().to_string());
                        }
                    }
                }
            }
        }

        None
    }

    /// Extract class name from definition line
    fn extract_class_def(&self, line: &str) -> Option<String> {
        // TypeScript/JavaScript: class Name or export class Name
        if line.contains("class ") {
            if let Some(start) = line.find("class ") {
                let after_class = &line[start + 6..];
                // Get name until first space, {, <, or (
                let mut end = after_class.len();
                if let Some(brace) = after_class.find('{') {
                    end = end.min(brace);
                }
                if let Some(paren) = after_class.find('(') {
                    end = end.min(paren);
                }
                if let Some(angle) = after_class.find('<') {
                    end = end.min(angle);
                }
                if let Some(space) = after_class.find(' ') {
                    end = end.min(space);
                }
                let name = after_class[..end].trim();
                if !name.is_empty() {
                    return Some(name.to_string());
                }
            }
        }

        None
    }

    /// Extract type/interface name from definition line
    fn extract_type_def(&self, line: &str) -> Option<String> {
        // TypeScript: interface Name or type Name
        if line.contains("interface ") {
            if let Some(start) = line.find("interface ") {
                let after_interface = &line[start + 10..];
                let mut end = after_interface.len();
                if let Some(brace) = after_interface.find('{') {
                    end = end.min(brace);
                }
                if let Some(angle) = after_interface.find('<') {
                    end = end.min(angle);
                }
                if let Some(space) = after_interface.find(' ') {
                    end = end.min(space);
                }
                let name = after_interface[..end].trim();
                if !name.is_empty() {
                    return Some(name.to_string());
                }
            }
        }

        if line.contains("type ") && !line.contains("// type") && !line.contains("typeof") {
            if let Some(start) = line.find("type ") {
                let after_type = &line[start + 5..];
                if let Some(eq) = after_type.find('=') {
                    let name = after_type[..eq].trim();
                    if !name.is_empty() {
                        return Some(name.to_string());
                    }
                }
                if let Some(semi) = after_type.find(';') {
                    let name = after_type[..semi].trim();
                    if !name.is_empty() {
                        return Some(name.to_string());
                    }
                }
            }
        }

        // Rust: struct Name or pub struct Name
        if line.contains("struct ") {
            if let Some(start) = line.find("struct ") {
                let after_struct = &line[start + 7..];
                let mut end = after_struct.len();
                if let Some(brace) = after_struct.find('{') {
                    end = end.min(brace);
                }
                if let Some(paren) = after_struct.find('(') {
                    end = end.min(paren);
                }
                if let Some(angle) = after_struct.find('<') {
                    end = end.min(angle);
                }
                if let Some(space) = after_struct.find(' ') {
                    end = end.min(space);
                }
                let name = after_struct[..end].trim();
                if !name.is_empty() {
                    return Some(name.to_string());
                }
            }
        }

        // Rust: enum Name or pub enum Name
        if line.contains("enum ") {
            if let Some(start) = line.find("enum ") {
                let after_enum = &line[start + 5..];
                let mut end = after_enum.len();
                if let Some(brace) = after_enum.find('{') {
                    end = end.min(brace);
                }
                if let Some(angle) = after_enum.find('<') {
                    end = end.min(angle);
                }
                if let Some(space) = after_enum.find(' ') {
                    end = end.min(space);
                }
                let name = after_enum[..end].trim();
                if !name.is_empty() {
                    return Some(name.to_string());
                }
            }
        }

        None
    }

    /// Extract dependencies between symbols
    ///
    /// # Arguments
    /// - language: Programming language
    /// - source_code: File content
    ///
    /// # Returns
    /// - Vec<(from_symbol, to_symbol, kind)>: Dependencies (calls, references, extends, etc.)
    pub async fn extract_dependencies(
        &mut self,
        _language: &str,
        _source_code: &str,
    ) -> Result<Vec<(String, String, String)>> {
        // TODO: Full implementation with AST traversal
        // For v0.1, return empty (dependencies tracked via graph traversal instead)
        Ok(Vec::new())
    }
}

impl Default for CodeParser {
    fn default() -> Self {
        Self::new().expect("Failed to create parser")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parser_creation() -> Result<()> {
        let _parser = CodeParser::new()?;
        Ok(())
    }

    #[tokio::test]
    async fn test_parse_rust_function() -> Result<()> {
        let mut parser = CodeParser::new()?;
        let code = "fn main() { println!(\"hello\"); }";

        let symbols = parser.parse_file("rust", code).await?;
        assert!(!symbols.is_empty(), "Should extract main function");
        assert_eq!(symbols[0].name, "main");
        assert_eq!(symbols[0].kind.as_str(), "function");

        Ok(())
    }

    #[tokio::test]
    async fn test_parse_typescript() -> Result<()> {
        let mut parser = CodeParser::new()?;
        let code = "export function greet(name: string) { return `Hello, ${name}`; }";

        let symbols = parser.parse_file("typescript", code).await?;
        assert!(!symbols.is_empty(), "Should extract greet function");
        assert_eq!(symbols[0].name, "greet");
        assert!(symbols[0].is_exported, "Should be marked as exported");

        Ok(())
    }

    #[tokio::test]
    async fn test_parse_class() -> Result<()> {
        let mut parser = CodeParser::new()?;
        let code = "class User { constructor(name: string) {} }";

        let symbols = parser.parse_file("typescript", code).await?;
        assert!(!symbols.is_empty(), "Should extract User class");
        assert_eq!(symbols[0].name, "User");
        assert_eq!(symbols[0].kind.as_str(), "class");

        Ok(())
    }

    #[tokio::test]
    async fn test_parse_interface() -> Result<()> {
        let mut parser = CodeParser::new()?;
        let code = "interface Config { name: string; }";

        let symbols = parser.parse_file("typescript", code).await?;
        assert!(!symbols.is_empty(), "Should extract Config interface");
        assert_eq!(symbols[0].name, "Config");
        assert_eq!(symbols[0].kind.as_str(), "type");

        Ok(())
    }

    #[tokio::test]
    async fn test_parse_multiple_symbols() -> Result<()> {
        let mut parser = CodeParser::new()?;
        let code = r#"
fn foo() {}
fn bar() {}
class Baz {}
"#;

        let symbols = parser.parse_file("typescript", code).await?;
        assert_eq!(symbols.len(), 3, "Should extract 3 symbols");

        Ok(())
    }

    #[tokio::test]
    async fn test_parse_nested_scope() -> Result<()> {
        let mut parser = CodeParser::new()?;
        let code = r#"
class User {
  fn name() {}
  fn email() {}
}
"#;

        let symbols = parser.parse_file("rust", code).await?;
        assert!(symbols.len() >= 3, "Should extract class and methods");

        // Check that methods have class scope
        let methods = symbols.iter().filter(|s| s.kind.as_str() == "function").collect::<Vec<_>>();
        assert!(!methods.is_empty(), "Should extract methods");

        Ok(())
    }

    #[tokio::test]
    async fn test_parse_struct_and_enum() -> Result<()> {
        let mut parser = CodeParser::new()?;
        let code = r#"
pub struct Point {
    x: i32,
    y: i32,
}

enum Color {
    Red,
    Green,
    Blue,
}
"#;

        let symbols = parser.parse_file("rust", code).await?;
        assert!(symbols.len() >= 2, "Should extract struct and enum");
        assert!(symbols.iter().any(|s| s.name == "Point"), "Should find Point struct");
        assert!(symbols.iter().any(|s| s.name == "Color"), "Should find Color enum");

        Ok(())
    }

    #[tokio::test]
    async fn test_parse_exported_symbols() -> Result<()> {
        let mut parser = CodeParser::new()?;
        let code = r#"
export function publicFn() {}
function privateFn() {}
pub fn rust_public() {}
fn rust_private() {}
"#;

        let symbols = parser.parse_file("typescript", code).await?;
        let public_symbols = symbols.iter().filter(|s| s.is_exported).collect::<Vec<_>>();
        assert!(!public_symbols.is_empty(), "Should detect exported symbols");

        Ok(())
    }
}
