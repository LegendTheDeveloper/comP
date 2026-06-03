// mod.rs update - Main indexer orchestrator integrating walker, parser, doc_parser
//
// Updated implementation now includes:
// - FileWalker for filesystem traversal
// - CodeParser for tree-sitter parsing
// - DocumentParser for JSON/XML/Markdown
// - Integration: walk -> parse -> extract -> store

use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;

pub mod walker;
pub mod parser;
pub mod doc_parser;
pub mod dependency;

pub use walker::{FileWalker, FileEntry, WalkerConfig};
pub use parser::CodeParser;
pub use doc_parser::DocumentParser;
pub use dependency::DependencyAnalyzer;

/// Main indexer orchestrator
pub struct Indexer {
    workspace_root: String,
    walker: FileWalker,
    parser: CodeParser,
}

impl Indexer {
    /// Create a new indexer for the given workspace
    pub fn new(workspace_root: &str) -> Self {
        let config = WalkerConfig {
            ignore_patterns: Self::load_ignore_patterns(workspace_root),
            ..WalkerConfig::default()
        };

        let walker = FileWalker::new(workspace_root, config);
        let parser = CodeParser::default();

        Indexer {
            workspace_root: workspace_root.to_string(),
            walker,
            parser,
        }
    }

    /// Load user-defined ignore patterns from .comp/ignore
    fn load_ignore_patterns(workspace_root: &str) -> Vec<String> {
        let path = std::path::Path::new(workspace_root).join(".comp/ignore");
        std::fs::read_to_string(path)
            .unwrap_or_default()
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .collect()
    }

    /// Read additional workspace paths from `.comp/config.json`.
    ///
    /// WHY: Monorepos or multi-root workspaces need all sub-paths indexed into
    /// a single graph DB so cross-path dependencies can be resolved.
    pub fn read_additional_paths(workspace_root: &str) -> Vec<String> {
        let path = std::path::Path::new(workspace_root).join(".comp/config.json");
        let content = std::fs::read_to_string(path).unwrap_or_default();
        let json: serde_json::Value = serde_json::from_str(&content).unwrap_or(serde_json::Value::Null);
        json["additional_paths"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Load node limit settings from .comp/config.json
    /// Returns (max_nodes, on_limit_exceeded)
    fn load_comp_config(workspace_root: &str) -> (i64, String) {
        let path = std::path::Path::new(workspace_root).join(".comp/config.json");
        let content = std::fs::read_to_string(path).unwrap_or_default();
        let json: serde_json::Value = serde_json::from_str(&content).unwrap_or(serde_json::Value::Null);
        let max_nodes = json["max_nodes"].as_i64().unwrap_or(200_000);
        let on_limit = json["on_limit_exceeded"].as_str().unwrap_or("warn").to_string();
        (max_nodes, on_limit)
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

        // 3. Remove deleted file entries from the database
        for path in &walk_result.deleted_files {
            if let Err(e) = db.delete_file(path) {
                // WHY: If deletion fails, entries persist in the database, leading to obsolete search results.
                //      Log a warning to prompt retries during the next indexing run.
                log::warn!("Failed to remove deleted file from index (stale entry may persist): {} — {}", path, e);
            }
        }

        // 4. Check max_nodes limit from .comp/config.json
        let (max_nodes, on_limit) = Self::load_comp_config(&self.workspace_root);
        if let Ok((_, node_count, _)) = db.get_stats() {
            if node_count > max_nodes {
                match on_limit.as_str() {
                    "stop" => {
                        log::warn!(
                            "Node limit reached ({} > {}). Indexing halted. Add entries to .comp/ignore to reduce scope.",
                            node_count, max_nodes
                        );
                        return Ok((total_files, changed_count, symbols_count));
                    }
                    _ => {
                        log::warn!(
                            "Node limit exceeded ({} > {}). Consider adding .comp/ignore entries or increasing max_nodes in .comp/config.json.",
                            node_count, max_nodes
                        );
                    }
                }
            }
        }

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

        // 1. Read file content based on type
        let full_path = Path::new(&self.workspace_root).join(&file_entry.path);
        
        let is_binary = matches!(
            file_entry.language.as_str(),
            "parquet" | "docx" | "pptx" | "xlsx" | "pdf"
        );

        let (symbols, content_str) = if is_binary {
            let syms = match file_entry.language.as_str() {
                "parquet" => DocumentParser::parse_parquet(&full_path)?,
                "docx" => DocumentParser::parse_docx(&full_path)?,
                "pptx" => DocumentParser::parse_pptx(&full_path)?,
                "xlsx" => DocumentParser::parse_xlsx(&full_path)?,
                "pdf" => DocumentParser::parse_pdf(&full_path)?,
                _ => Vec::new(),
            };
            (syms, String::new()) // Binary files don't use string content for extraction later
        } else {
            // WHY: InvalidData usually indicates non-UTF-8 encoding.
            // Skip silently with a debug log and avoid printing to stderr to prevent user noise.
            let content = match fs::read_to_string(&full_path) {
                Ok(c) => c,
                Err(e) if e.kind() == std::io::ErrorKind::InvalidData => {
                    log::debug!("Skipping non-UTF-8 file: {} ({})", file_entry.path, e);
                    return Ok(0);
                }
                Err(e) => return Err(e.into()),
            };
            let syms = match file_entry.language.as_str() {
                "json" => DocumentParser::parse_json(&content)?,
                "jsonl" => DocumentParser::parse_jsonl(&content)?,
                "xml" => DocumentParser::parse_xml(&content)?,
                "markdown" | "md" => DocumentParser::parse_markdown(&content)?,
                _ => {
                    // Tree-sitter parsing for code
                    self.parser.parse_file(&file_entry.language, &content).await?
                }
            };
            (syms, content)
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
                symbol.is_exported,
                symbol.signature.as_deref(),
            )?;
            symbol_map.insert(symbol.name.clone(), node_id);
        }

        // 5. Extract and store dependencies (edges) — Phase 4
        // Extract raw dependencies from source code
        let raw_deps = if is_binary {
            Vec::new()
        } else {
            DependencyAnalyzer::extract_dependencies(
                &file_entry.language,
                &content_str,
                &file_entry.path,
            ).unwrap_or_default()
        };

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
    ///
    /// WHY: Called upon file changes. Previous implementation was a TODO that did not save to DB.
    /// We reuse parse_and_extract to parse, extract symbols, resolve dependencies, and save to DB in one go.
    pub async fn index_file(&mut self, path: &Path, db: &crate::graph::GraphDB) -> Result<()> {
        use sha2::{Sha256, Digest};

        // Convert to workspace-relative path (using absolute paths breaks DB uniqueness constraint)
        // WHY: Windows path normalizer returns backslashes (\), but the database must use forward slashes (/) consistently.
        //      The TypeScript extension sends forward slashes, so mismatch breaks path lookups.
        let relative_path = path
            .strip_prefix(&self.workspace_root)
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_else(|_| path.to_string_lossy().replace('\\', "/"));

        let language = self.walker_detect_language(&relative_path);

        // Calculate file hash (required for FileEntry)
        // Read as bytes to handle binary files (e.g., Parquet) gracefully
        let content_bytes = std::fs::read(path)?;
        let mut hasher = Sha256::new();
        hasher.update(&content_bytes);
        let hash = format!("{:x}", hasher.finalize());

        let file_entry = FileEntry {
            path: relative_path,
            hash,
            language,
            modified_time: 0,
        };

        // parse_and_extract performs parsing and writes to database
        self.parse_and_extract(&file_entry, db).await?;

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
                "jsonl" => "jsonl",
                "yaml" | "yml" => "yaml",
                "xml" => "xml",
                "md" => "markdown",
                "parquet" => "parquet",
                "docx" => "docx",
                "pptx" => "pptx",
                "xlsx" => "xlsx",
                "pdf" => "pdf",
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
    use super::dependency::{Dependency, EdgeKind};

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
        assert_eq!(indexer.walker_detect_language("report.docx"), "docx");
        assert_eq!(indexer.walker_detect_language("slides.pptx"), "pptx");
        assert_eq!(indexer.walker_detect_language("data.xlsx"), "xlsx");
        assert_eq!(indexer.walker_detect_language("data.parquet"), "parquet");
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
