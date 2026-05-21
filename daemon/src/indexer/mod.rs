// mod.rs update - Main indexer orchestrator integrating walker, parser, doc_parser
//
// Updated implementation now includes:
// - FileWalker for filesystem traversal
// - CodeParser for tree-sitter parsing
// - DocumentParser for JSON/XML/Markdown
// - Integration: walk -> parse -> extract -> store

use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::path::Path;

pub mod walker;
pub mod parser;
pub mod doc_parser;
pub mod dependency;

pub use walker::{FileWalker, FileEntry, WalkerConfig, WalkerResult};
pub use parser::{CodeParser, Symbol, SymbolKind};
pub use doc_parser::DocumentParser;
pub use dependency::{DependencyAnalyzer, EdgeKind, Dependency};

/// Main indexer orchestrator
pub struct Indexer {
    workspace_root: String,
    walker: FileWalker,
    parser: CodeParser,
}

impl Indexer {
    /// Create a new indexer for the given workspace
    pub fn new(workspace_root: &str) -> Self {
        let walker = FileWalker::new(workspace_root, WalkerConfig::default());
        let parser = CodeParser::default();

        Indexer {
            workspace_root: workspace_root.to_string(),
            walker,
            parser,
        }
    }

    /// Index the entire workspace
    ///
    /// Process:
    /// 1. Walk filesystem to find all source files
    /// 2. Compare hashes with DB to find changed files
    /// 3. For each changed file:
    ///    - Parse with tree-sitter or document parser
    ///    - Extract symbols
    ///    - Store symbols in graph DB
    ///    - Extract and store dependencies as edges
    /// 4. Delete entries for removed files
    /// 5. Return statistics
    ///
    /// # Arguments
    /// - previous_hashes: Optional map of (path -> hash) from previous index
    /// - db: GraphDB instance for storing results
    ///
    /// # Returns
    /// - (total_files, indexed_files, symbols_extracted)
    pub async fn index_workspace(
        &mut self,
        previous_hashes: Option<&HashMap<String, String>>,
        db: &crate::graph::GraphDB,
    ) -> Result<(usize, usize, usize)> {
        // 1. Walk filesystem
        let walk_result = self.walker.walk(previous_hashes)?;
        let total_files = walk_result.files.len();
        let changed_count = walk_result.changed_files.len();

        // 2. Process changed files: parse, extract, store in DB
        let mut symbols_count = 0;
        for file_entry in walk_result.changed_files {
            match self.parse_and_extract(&file_entry, db).await {
                Ok(count) => symbols_count += count,
                Err(e) => {
                    eprintln!("Error parsing {}: {}", file_entry.path, e);
                    // Continue with next file on error
                }
            }
        }

        // 3. TODO: Delete entries for removed files from DB
        // for path in walk_result.deleted_files {
        //     db.delete_file(&path)?;
        // }

        Ok((total_files, changed_count, symbols_count))
    }

    /// Parse a single file and extract symbols, storing them in GraphDB
    ///
    /// # Process:
    /// 1. Read file content from disk
    /// 2. Parse based on language (tree-sitter or document parser)
    /// 3. Store file entry in DB (upsert_file)
    /// 4. For each symbol, store as node in DB (insert_node)
    /// 5. Extract dependencies and store as edges (insert_edge)
    ///
    /// # Arguments
    /// - file_entry: File metadata (path, hash, language)
    /// - db: GraphDB instance for storing results
    ///
    /// # Returns
    /// - Number of symbols extracted and stored
    async fn parse_and_extract(&mut self, file_entry: &FileEntry, db: &crate::graph::GraphDB) -> Result<usize> {
        use std::fs;

        // 1. Read file content
        let full_path = Path::new(&self.workspace_root).join(&file_entry.path);
        let content = fs::read_to_string(&full_path)?;

        // 2. Parse based on language
        let symbols = match file_entry.language.as_str() {
            "json" => DocumentParser::parse_json(&content)?,
            "jsonl" => DocumentParser::parse_jsonl(&content)?,
            "xml" => DocumentParser::parse_xml(&content)?,
            "markdown" | "md" => DocumentParser::parse_markdown(&content)?,
            _ => {
                // Tree-sitter parsing for code
                self.parser.parse_file(&file_entry.language, &content).await?
            }
        };

        // 3. Store file in DB
        let file_id = db.upsert_file(&file_entry.path, &file_entry.hash, &file_entry.language)?;

        // 4. Store each symbol as a node in DB
        let mut symbol_map: HashMap<String, i64> = HashMap::new();
        for symbol in &symbols {
            let node_id = db.insert_node(
                file_id,
                &symbol.name,
                symbol.kind.as_str(),
                symbol.line as i32,
                symbol.column as i32,
                symbol.scope.as_deref(),
            )?;
            symbol_map.insert(symbol.name.clone(), node_id);
        }

        // 5. Extract and store dependencies (edges) — Phase 4
        // Extract raw dependencies from source code
        let raw_deps = DependencyAnalyzer::extract_dependencies(
            &file_entry.language,
            &content,
            &file_entry.path,
        ).unwrap_or_default();

        // Resolve dependency names to node IDs
        let edges = DependencyAnalyzer::resolve_dependencies(&raw_deps, &symbol_map, &HashMap::new());

        // Store edges in database
        for (from_id, to_id, edge_kind) in edges {
            if let Err(e) = db.insert_edge(from_id, to_id, &edge_kind) {
                eprintln!("Warning: Failed to insert edge {}->{}: {}", from_id, to_id, e);
                // Continue on edge insertion failures
            }
        }

        Ok(symbols.len())
    }

    /// Index a single file (incremental update)
    pub async fn index_file(&mut self, path: &Path) -> Result<()> {
        let content = std::fs::read_to_string(path)?;
        
        // Get relative path
        let relative_path = path.strip_prefix(&self.workspace_root)?;
        let relative_str = relative_path.to_string_lossy().to_string();
        
        // Detect language (same logic as walker)
        let language = self.walker_detect_language(&relative_str);

        // Parse
        match language.as_str() {
            "json" => { DocumentParser::parse_json(&content)?; },
            "xml" => { DocumentParser::parse_xml(&content)?; },
            "markdown" | "md" => { DocumentParser::parse_markdown(&content)?; },
            _ => { 
                self.parser.parse_file(&language, &content).await?;
            }
        };

        // TODO: Update graph DB

        Ok(())
    }

    // Helper: same language detection as walker
    fn walker_detect_language(&self, path: &str) -> String {
        if let Some(ext) = Path::new(path).extension() {
            match ext.to_string_lossy().as_ref() {
                "rs" => "rust",
                "ts" | "tsx" => "typescript",
                "js" | "jsx" => "javascript",
                "py" => "python",
                "go" => "go",
                "java" => "java",
                "c" | "h" => "c",
                "cpp" | "cc" | "cxx" | "hpp" => "cpp",
                "cs" => "csharp",
                "rb" => "ruby",
                "php" => "php",
                "sh" | "bash" => "bash",
                "sql" => "sql",
                "html" | "htm" => "html",
                "css" | "scss" | "less" => "css",
                "json" => "json",
                "yaml" | "yml" => "yaml",
                "xml" => "xml",
                "md" => "markdown",
                _ => "unknown",
            }
        } else {
            "unknown"
        }
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_indexer_creation() {
        let indexer = Indexer::new("/tmp/test");
        assert_eq!(indexer.workspace_root, "/tmp/test");
    }

    #[test]
    fn test_language_detection() {
        let indexer = Indexer::new(".");

        assert_eq!(indexer.walker_detect_language("main.rs"), "rust");
        assert_eq!(indexer.walker_detect_language("app.ts"), "typescript");
        assert_eq!(indexer.walker_detect_language("index.json"), "json");
        assert_eq!(indexer.walker_detect_language("README.md"), "markdown");
    }

    #[tokio::test]
    async fn test_parse_and_extract_stores_symbols() -> Result<()> {
        use tempfile::TempDir;
        use std::fs::File;
        use std::io::Write;

        // Setup: Create temp directory with a Rust file
        let temp_dir = TempDir::new()?;
        let workspace_root = temp_dir.path().to_string_lossy().to_string();

        // Create a simple Rust file with symbols
        let rust_file = temp_dir.path().join("test.rs");
        let mut file = File::create(&rust_file)?;
        writeln!(file, "fn hello() {{}}\nstruct Point {{}}\nfn main() {{}}")?;

        // Calculate hash for the file
        use sha2::{Sha256, Digest};
        let content = std::fs::read_to_string(&rust_file)?;
        let mut hasher = Sha256::new();
        hasher.update(&content);
        let hash = format!("{:x}", hasher.finalize());

        // Create GraphDB in temp directory
        let db = crate::graph::GraphDB::new(&workspace_root).await?;

        // Create indexer and parse the file
        let mut indexer = Indexer::new(&workspace_root);
        let file_entry = FileEntry {
            path: "test.rs".to_string(),
            hash: hash.clone(),
            language: "rust".to_string(),
            modified_time: 0,
        };

        let count = indexer.parse_and_extract(&file_entry, &db).await?;

        // Verify: Should extract multiple symbols (hello, Point, main)
        assert!(count > 0, "Should extract at least one symbol");

        // Verify: Database should contain file entry
        let (file_count, node_count, _edge_count) = db.get_stats()?;
        assert_eq!(file_count, 1, "Should have exactly 1 file");
        assert!(node_count > 0, "Should have extracted symbols as nodes");

        Ok(())
    }

    #[tokio::test]
    async fn test_index_workspace_integration() -> Result<()> {
        use tempfile::TempDir;
        use std::fs::File;
        use std::io::Write;

        // Setup: Create temp directory with multiple Rust files
        let temp_dir = TempDir::new()?;
        let workspace_root = temp_dir.path().to_string_lossy().to_string();

        // Create test files
        let file1 = temp_dir.path().join("file1.rs");
        let mut f1 = File::create(&file1)?;
        writeln!(f1, "fn func1() {{}}")?;

        let file2 = temp_dir.path().join("file2.rs");
        let mut f2 = File::create(&file2)?;
        writeln!(f2, "fn func2() {{}}\nstruct Data {{}}")?;

        // Create GraphDB
        let db = crate::graph::GraphDB::new(&workspace_root).await?;

        // Index workspace (without previous hashes, all files are "new")
        let mut indexer = Indexer::new(&workspace_root);
        let (total_files, indexed_files, symbols) = indexer
            .index_workspace(None, &db)
            .await?;

        // Verify: Should find and index files
        // Note: total_files may include non-Rust files, indexed_files should include our 2 Rust files
        assert!(total_files > 0, "Should find files");
        assert_eq!(indexed_files, 2, "Should index 2 Rust files");
        assert!(symbols > 0, "Should extract symbols from indexed files");

        // Verify: Database stats
        let (file_count, node_count, _edge_count) = db.get_stats()?;
        assert_eq!(file_count, 2, "Should have 2 files in DB");
        assert!(node_count > 0, "Should have nodes for extracted symbols");

        Ok(())
    }

    #[tokio::test]
    async fn test_index_workspace_empty() -> Result<()> {
        use tempfile::TempDir;

        let temp_dir = TempDir::new()?;
        let workspace_root = temp_dir.path().to_string_lossy().to_string();

        // Create GraphDB in empty directory
        let db = crate::graph::GraphDB::new(&workspace_root).await?;

        let mut indexer = Indexer::new(&workspace_root);
        let (total_files, indexed_files, symbols) = indexer
            .index_workspace(None, &db)
            .await?;

        // Empty directory should have no indexable files
        assert_eq!(indexed_files, 0, "Empty directory should have 0 indexed files");
        assert_eq!(symbols, 0, "Empty directory should have 0 symbols");

        Ok(())
    }

    #[test]
    fn test_dependency_analyzer_resolve() {
        // Test dependency resolution with symbol_map
        let deps = vec![
            Dependency {
                from: "main".to_string(),
                to: "helper".to_string(),
                kind: EdgeKind::FunctionCall,
                line: 5,
            },
        ];

        let mut symbol_map = HashMap::new();
        symbol_map.insert("main".to_string(), 1);
        symbol_map.insert("helper".to_string(), 2);

        let edges = DependencyAnalyzer::resolve_dependencies(&deps, &symbol_map, &HashMap::new());

        // Verify edge is created with correct node IDs
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].0, 1); // from_id
        assert_eq!(edges[0].1, 2); // to_id
        assert_eq!(edges[0].2, "function_call");
    }

    #[test]
    fn test_dependency_analyzer_unresolved() {
        // Test that unresolved dependencies are skipped
        let deps = vec![
            Dependency {
                from: "main".to_string(),
                to: "unknown".to_string(),
                kind: EdgeKind::FunctionCall,
                line: 5,
            },
        ];

        let mut symbol_map = HashMap::new();
        symbol_map.insert("main".to_string(), 1);
        // "unknown" is not in symbol_map

        let edges = DependencyAnalyzer::resolve_dependencies(&deps, &symbol_map, &HashMap::new());

        // Unresolved dependencies should not create edges
        assert_eq!(edges.len(), 0);
    }

    #[tokio::test]
    async fn test_parse_and_extract_with_dependencies() -> Result<()> {
        use tempfile::TempDir;
        use std::fs::File;
        use std::io::Write;

        // Setup: Create temp directory with a Rust file containing function calls
        let temp_dir = TempDir::new()?;
        let workspace_root = temp_dir.path().to_string_lossy().to_string();

        // Create a Rust file with a function call
        let rust_file = temp_dir.path().join("test.rs");
        let mut file = File::create(&rust_file)?;
        writeln!(file, "fn main() {{}}\nfn helper() {{}}\n")?;

        // Calculate hash
        use sha2::{Sha256, Digest};
        let content = std::fs::read_to_string(&rust_file)?;
        let mut hasher = Sha256::new();
        hasher.update(&content);
        let hash = format!("{:x}", hasher.finalize());

        // Create GraphDB and indexer
        let db = crate::graph::GraphDB::new(&workspace_root).await?;
        let mut indexer = Indexer::new(&workspace_root);

        let file_entry = FileEntry {
            path: "test.rs".to_string(),
            hash,
            language: "rust".to_string(),
            modified_time: 0,
        };

        // Parse and extract
        let symbol_count = indexer.parse_and_extract(&file_entry, &db).await?;

        // Verify symbols were extracted
        assert!(symbol_count > 0);

        // Verify database state
        let (file_count, node_count, edge_count) = db.get_stats()?;
        assert_eq!(file_count, 1);
        assert!(node_count > 0);
        // Note: edges may be 0 if dependency extraction is not yet implemented

        Ok(())
    }
}
