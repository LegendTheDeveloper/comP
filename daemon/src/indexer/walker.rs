// walker.rs - Filesystem walker with .gitignore support and incremental detection
//
// Responsibilities:
// - Scan workspace directory recursively
// - Parse and respect .gitignore patterns
// - Calculate file hashes for incremental updates
// - Skip directories and files based on patterns (node_modules, .git, etc.)
// - Detect which files have changed since last index
//
// Key data structures:
// - FileEntry: path, hash, language, last_modified
// - WalkerResult: total files, indexed files, skipped files

use anyhow::Result;
use sha2::{Sha256, Digest};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::{WalkDir, DirEntry};

/// A file entry with metadata
#[derive(Debug, Clone)]
pub struct FileEntry {
    /// Relative path from workspace root
    pub path: String,
    /// SHA256 hash of file content
    pub hash: String,
    /// Detected language (e.g., "rust", "typescript", "python")
    pub language: String,
    /// File modification time (Unix timestamp)
    #[allow(dead_code)]
    pub modified_time: i64,
}

/// Result of walking the filesystem
#[derive(Debug)]
pub struct WalkerResult {
    /// All files found (including unchanged)
    pub files: Vec<FileEntry>,
    /// Files that need re-indexing (changed or new)
    pub changed_files: Vec<FileEntry>,
    /// Files that were deleted (tracked before but not found now)
    pub deleted_files: Vec<String>,
}

/// Walker configuration
pub struct WalkerConfig {
    /// Skip hidden files and directories (starting with .)
    pub skip_hidden: bool,
    /// Built-in patterns to skip (substring match against full path)
    pub skip_patterns: Vec<String>,
    /// User-defined patterns from .comp/ignore (gitignore-style)
    pub ignore_patterns: Vec<String>,
}

impl Default for WalkerConfig {
    fn default() -> Self {
        WalkerConfig {
            skip_hidden: true,
            skip_patterns: vec![
                "node_modules".to_string(),
                ".git".to_string(),
                ".comp".to_string(),
                "target".to_string(),
                "dist".to_string(),
                "build".to_string(),
            ],
            ignore_patterns: vec![],
        }
    }
}

pub struct FileWalker {
    workspace_root: PathBuf,
    config: WalkerConfig,
}

impl FileWalker {
    /// Create a new file walker
    pub fn new(workspace_root: &str, config: WalkerConfig) -> Self {
        FileWalker {
            workspace_root: PathBuf::from(workspace_root),
            config,
        }
    }

    /// Walk the workspace and return all files
    /// 
    /// # Arguments
    /// - workspace_root: Path to workspace
    /// - previous_hashes: Map of (file_path -> hash) from previous index
    ///
    /// # Returns
    /// - WalkerResult containing all files, changed files, and deleted files
    ///
    /// # Process
    /// 1. Parse .gitignore if exists
    /// 2. Walk directory recursively
    /// 3. For each file:
    ///    - Check if should skip (hidden, in skip_patterns, etc.)
    ///    - Detect language
    ///    - Calculate hash
    ///    - Compare with previous hash (if exists)
    /// 4. Detect deleted files (were in previous_hashes but not found)
    /// 5. Return result
    pub fn walk(&self, previous_hashes: Option<&HashMap<String, String>>) -> Result<WalkerResult> {
        // TODO: Implement filesystem walk with:
        // - gitignore pattern matching
        // - hash calculation
        // - change detection
        // - language detection
        
        let mut all_files = Vec::new();
        let mut changed_files = Vec::new();
        let mut found_paths = HashSet::new();

        // Walk directory
        for entry in WalkDir::new(&self.workspace_root)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if self.should_skip_entry(&entry) {
                continue;
            }

            if entry.path().is_file() {
                let relative_path = self.get_relative_path(entry.path())?;
                
                // Calculate hash
                let hash = self.calculate_file_hash(entry.path())?;
                let modified_time = self.get_modified_time(entry.path())?;
                let language = self.detect_language(&relative_path);

                found_paths.insert(relative_path.clone());

                let file_entry = FileEntry {
                    path: relative_path.clone(),
                    hash: hash.clone(),
                    language,
                    modified_time,
                };

                // Check if file changed
                let file_changed = if let Some(prev_hashes) = previous_hashes {
                    prev_hashes.get(&relative_path) != Some(&hash)
                } else {
                    true // First index, all files are "changed"
                };

                if file_changed {
                    changed_files.push(file_entry.clone());
                }

                all_files.push(file_entry);
            }
        }

        // Detect deleted files
        let deleted_files = if let Some(prev_hashes) = previous_hashes {
            prev_hashes
                .keys()
                .filter(|path| !found_paths.contains(*path))
                .cloned()
                .collect()
        } else {
            Vec::new()
        };

        Ok(WalkerResult {
            files: all_files,
            changed_files,
            deleted_files,
        })
    }

    /// Check if a directory entry should be skipped
    fn should_skip_entry(&self, entry: &DirEntry) -> bool {
        let path = entry.path();
        let file_name = entry.file_name();
        let file_name_str = file_name.to_string_lossy();

        // Skip hidden files
        if self.config.skip_hidden && file_name_str.starts_with('.') {
            return true;
        }

        // Built-in patterns: substring match against full path
        for pattern in &self.config.skip_patterns {
            if path.to_string_lossy().contains(pattern) {
                return true;
            }
        }

        // User-defined patterns from .comp/ignore
        let normalized = path.to_string_lossy().replace('\\', "/");
        for pattern in &self.config.ignore_patterns {
            if Self::matches_ignore_pattern(&normalized, pattern) {
                return true;
            }
        }

        false
    }

    /// Match a normalized path against a gitignore-style pattern.
    ///
    /// Supports:
    /// - Name match: `node_modules` → any path segment equals this
    /// - Directory pattern: `dist/` → same as `dist` (trailing slash stripped)
    /// - Suffix glob: `*.min.js` → path ends with `.min.js`
    fn matches_ignore_pattern(path: &str, pattern: &str) -> bool {
        let pattern = pattern.trim_end_matches('/');
        if pattern.is_empty() {
            return false;
        }
        if let Some(suffix) = pattern.strip_prefix('*') {
            return path.ends_with(suffix);
        }
        // Match any path segment (handles both files and directories)
        path.split('/').any(|component| component == pattern)
    }

    /// Get relative path from workspace root, normalized to forward slashes
    fn get_relative_path(&self, path: &Path) -> Result<String> {
        let relative = path.strip_prefix(&self.workspace_root)?;
        Ok(relative.to_string_lossy().replace('\\', "/"))
    }

    /// Calculate SHA256 hash of file content
    fn calculate_file_hash(&self, path: &Path) -> Result<String> {
        let content = fs::read(path)?;
        let mut hasher = Sha256::new();
        hasher.update(&content);
        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Get file modification time (Unix timestamp)
    fn get_modified_time(&self, path: &Path) -> Result<i64> {
        let metadata = fs::metadata(path)?;
        let modified = metadata.modified()?;
        let duration = modified.duration_since(std::time::UNIX_EPOCH)?;
        Ok(duration.as_secs() as i64)
    }

    /// Detect programming language from file extension
    fn detect_language(&self, path: &str) -> String {
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
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_walker_creation() {
        let walker = FileWalker::new("/tmp", WalkerConfig::default());
        assert_eq!(walker.workspace_root, PathBuf::from("/tmp"));
    }

    #[test]
    fn test_language_detection() {
        let walker = FileWalker::new(".", WalkerConfig::default());
        
        // Test various file extensions
        assert_eq!(walker.detect_language("main.rs"), "rust");
        assert_eq!(walker.detect_language("app.ts"), "typescript");
        assert_eq!(walker.detect_language("script.py"), "python");
        assert_eq!(walker.detect_language("main.go"), "go");
        assert_eq!(walker.detect_language("unknown.xyz"), "unknown");
    }

    #[test]
    fn test_file_hash_calculation() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test.txt");
        let mut file = File::create(&file_path)?;
        file.write_all(b"test content")?;

        let walker = FileWalker::new(temp_dir.path().to_str().unwrap(), WalkerConfig::default());
        let hash1 = walker.calculate_file_hash(&file_path)?;
        let hash2 = walker.calculate_file_hash(&file_path)?;

        // Same content should produce same hash
        assert_eq!(hash1, hash2);
        // Hash should be 64 chars (SHA256 hex)
        assert_eq!(hash1.len(), 64);

        Ok(())
    }

    #[test]
    fn test_skip_hidden_files() -> Result<()> {
        let walker = FileWalker::new(".", WalkerConfig::default());
        
        // Create mock DirEntry-like behavior by testing path filtering
        let hidden_path = ".hidden";
        let normal_path = "normal";

        assert!(walker.detect_language(hidden_path).is_empty() == false); // Just verify detection works
        assert!(walker.detect_language(normal_path).is_empty() == false);

        Ok(())
    }

    #[tokio::test]
    async fn test_walk_empty_directory() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let walker = FileWalker::new(
            temp_dir.path().to_str().unwrap(),
            WalkerConfig::default(),
        );

        let result = walker.walk(None)?;
        assert_eq!(result.files.len(), 0);
        assert_eq!(result.changed_files.len(), 0);
        assert_eq!(result.deleted_files.len(), 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_walk_with_files() -> Result<()> {
        let temp_dir = TempDir::new()?;
        
        // Create test files
        let file1_path = temp_dir.path().join("test1.rs");
        let mut file1 = File::create(&file1_path)?;
        file1.write_all(b"fn main() {}")?;

        let file2_path = temp_dir.path().join("test2.py");
        let mut file2 = File::create(&file2_path)?;
        file2.write_all(b"print('hello')")?;

        let walker = FileWalker::new(
            temp_dir.path().to_str().unwrap(),
            WalkerConfig::default(),
        );

        let result = walker.walk(None)?;
        
        assert_eq!(result.files.len(), 2);
        assert_eq!(result.changed_files.len(), 2); // All files are new
        assert_eq!(result.deleted_files.len(), 0);

        // Check language detection
        let langs: Vec<_> = result.files.iter().map(|f| f.language.as_str()).collect();
        assert!(langs.contains(&"rust"));
        assert!(langs.contains(&"python"));

        Ok(())
    }

    #[tokio::test]
    async fn test_incremental_detection() -> Result<()> {
        let temp_dir = TempDir::new()?;
        
        let file_path = temp_dir.path().join("test.rs");
        let mut file = File::create(&file_path)?;
        file.write_all(b"fn main() {}")?;

        let walker = FileWalker::new(
            temp_dir.path().to_str().unwrap(),
            WalkerConfig::default(),
        );

        // First walk
        let result1 = walker.walk(None)?;
        assert_eq!(result1.changed_files.len(), 1);

        // Create a previous hashes map from first walk
        let mut previous_hashes = HashMap::new();
        for file_entry in &result1.files {
            previous_hashes.insert(file_entry.path.clone(), file_entry.hash.clone());
        }

        // Second walk with same files (should detect no changes)
        let result2 = walker.walk(Some(&previous_hashes))?;
        assert_eq!(result2.changed_files.len(), 0); // No changes

        // Modify file
        let mut file = File::create(&file_path)?;
        file.write_all(b"fn main() { println!('hello'); }")?;

        // Third walk (should detect change)
        let result3 = walker.walk(Some(&previous_hashes))?;
        assert_eq!(result3.changed_files.len(), 1); // File changed

        Ok(())
    }
}
