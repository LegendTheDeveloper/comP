// doc_parser.rs - Parser for document formats (JSON, XML, Markdown, JSONL, Parquet)
//
// Responsibilities:
// - Parse JSON/JSONL files and extract structure
// - Parse XML files and extract tags/attributes
// - Parse Markdown and extract headings/sections
// - Parse Parquet files and extract schema fields
// - Create pseudo-symbols for document elements

use anyhow::Result;
use serde_json::Value;
use std::path::Path;
use std::fs::File;
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use parquet::file::reader::{FileReader, SerializedFileReader};

use super::parser::Symbol;

/// Document parser for non-code formats
pub struct DocumentParser;

impl DocumentParser {
    /// Parse JSON and extract structure as pseudo-symbols
    pub fn parse_json(content: &str) -> Result<Vec<Symbol>> {
        let parsed: Value = serde_json::from_str(content).unwrap_or(Value::Null);
        let mut symbols = Vec::new();

        if let Some(obj) = parsed.as_object() {
            for (key, _) in obj {
                // Determine line number roughly by finding the key in the string
                let line_num = content.lines().position(|l| l.contains(&format!("\"{}\"", key)))
                    .map(|idx| (idx + 1) as u32)
                    .unwrap_or(1);
                
                symbols.push(Symbol {
                    name: key.clone(),
                    kind: super::parser::SymbolKind::Property,
                    line: line_num,
                    column: 1,
                    end_line: line_num,
                    end_column: key.len() as u32 + 1,
                    signature: None,
                    is_exported: true,
                    scope: None,
                });
            }
        }
        
        Ok(symbols)
    }

    /// Parse JSONL (JSON Lines) and extract structure
    pub fn parse_jsonl(content: &str) -> Result<Vec<Symbol>> {
        let mut symbols = Vec::new();
        
        if let Some(first_line) = content.lines().find(|l| !l.trim().is_empty()) {
            if let Ok(Value::Object(obj)) = serde_json::from_str(first_line) {
                for (key, _) in obj {
                    symbols.push(Symbol {
                        name: key.clone(),
                        kind: super::parser::SymbolKind::Property,
                        line: 1,
                        column: 1,
                        end_line: 1,
                        end_column: key.len() as u32 + 1,
                        signature: None,
                        is_exported: true,
                        scope: None,
                    });
                }
            }
        }
        
        Ok(symbols)
    }

    /// Parse XML and extract tags
    pub fn parse_xml(content: &str) -> Result<Vec<Symbol>> {
        let mut reader = Reader::from_str(content);
        reader.config_mut().trim_text(true);

        let mut symbols = Vec::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    
                    if !symbols.iter().any(|s: &Symbol| s.name == name) {
                        let pos = reader.buffer_position() as usize;
                        let line_num = content[..pos].lines().count() as u32;

                        symbols.push(Symbol {
                            name,
                            kind: super::parser::SymbolKind::Module,
                            line: line_num.max(1),
                            column: 1,
                            end_line: line_num.max(1),
                            end_column: 1,
                            signature: None,
                            is_exported: true,
                            scope: None,
                        });
                    }
                }
                Ok(Event::Eof) => break,
                Err(_) => break, // Skip errors to gracefully handle partial/invalid XML
                _ => {}
            }
            buf.clear();
        }
        
        Ok(symbols)
    }

    /// Parse Markdown and extract structure
    pub fn parse_markdown(content: &str) -> Result<Vec<Symbol>> {
        let mut symbols = Vec::new();

        for (i, line) in content.lines().enumerate() {
            let line_num = (i + 1) as u32;
            let level = line.len() - line.trim_start_matches('#').len();
            if level > 0 && level <= 6 {
                let heading = line.trim_start_matches('#').trim();
                if !heading.is_empty() {
                    symbols.push(Symbol {
                        name: heading.to_string(),
                        kind: super::parser::SymbolKind::Module,
                        line: line_num,
                        column: (level + 1) as u32,
                        end_line: line_num,
                        end_column: (heading.len() + level + 1) as u32,
                        signature: None,
                        is_exported: true,
                        scope: None,
                    });
                }
            }
        }

        Ok(symbols)
    }

    /// Parse Parquet and extract schema fields
    pub fn parse_parquet(path: &Path) -> Result<Vec<Symbol>> {
        let file = File::open(path)?;
        let reader = SerializedFileReader::new(file)?;
        let metadata = reader.metadata();
        let file_metadata = metadata.file_metadata();
        let schema = file_metadata.schema();

        let mut symbols = Vec::new();
        for field in schema.get_fields() {
            let name = field.name().to_string();
            symbols.push(Symbol {
                name,
                kind: super::parser::SymbolKind::Property,
                line: 1,
                column: 1,
                end_line: 1,
                end_column: 1,
                signature: Some(format!("{:?}", field.get_physical_type())),
                is_exported: true,
                scope: None,
            });
        }
        Ok(symbols)
    }
}

/// BM25 full-text scorer for document files.
///
/// Reads files at query time and computes Okapi BM25 scores across the corpus.
/// Suitable for < 1000-file corpora (e.g. Markdown novel chapters) where
/// per-query I/O cost is acceptable and a persistent inverted index is overkill.
pub struct Bm25Scorer;

impl Bm25Scorer {
    // Standard Okapi BM25 hyperparameters
    const K1: f64 = 1.5;
    const B: f64 = 0.75;

    /// Read and BM25-score `file_paths` against `query_terms`.
    ///
    /// Returns (path, score) pairs sorted by score descending, capped at `top_k`.
    /// Files with score 0 (no query term present) are excluded.
    pub fn search_files(
        workspace_root: &str,
        file_paths: &[String],
        query_terms: &[&str],
        top_k: usize,
    ) -> Vec<(String, f64)> {
        use std::collections::HashMap;

        if query_terms.is_empty() || file_paths.is_empty() {
            return Vec::new();
        }

        let query_lower: Vec<String> = query_terms.iter().map(|t| t.to_lowercase()).collect();

        // Read and tokenize all files; skip unreadable ones silently
        let docs: Vec<(String, Vec<String>)> = file_paths
            .iter()
            .filter_map(|path| {
                let full = Path::new(workspace_root).join(path);
                let content = std::fs::read_to_string(&full).ok()?;
                let tokens = Self::tokenize(&content);
                if tokens.is_empty() { return None; }
                Some((path.clone(), tokens))
            })
            .collect();

        if docs.is_empty() {
            return Vec::new();
        }

        let num_docs = docs.len();
        let avg_doc_len =
            docs.iter().map(|(_, t)| t.len()).sum::<usize>() as f64 / num_docs as f64;

        // WHY: IDF requires knowing document frequency per term across the whole corpus.
        // We compute it once here before scoring individual documents.
        let doc_freq: HashMap<String, usize> = query_lower
            .iter()
            .map(|term| {
                let df = docs.iter().filter(|(_, tokens)| tokens.contains(term)).count();
                (term.clone(), df)
            })
            .collect();

        // Score every document
        let mut scored: Vec<(String, f64)> = docs
            .iter()
            .filter_map(|(path, tokens)| {
                let doc_len = tokens.len() as f64;
                let mut tf_map: HashMap<String, u32> = HashMap::new();
                for t in tokens {
                    *tf_map.entry(t.clone()).or_insert(0) += 1;
                }

                let score: f64 = query_lower
                    .iter()
                    .map(|term| {
                        let tf = *tf_map.get(term).unwrap_or(&0) as f64;
                        if tf == 0.0 {
                            return 0.0;
                        }
                        let df = *doc_freq.get(term).unwrap_or(&0) as f64;
                        // WHY: +1 inside ln prevents negative IDF for very common terms
                        let idf = ((num_docs as f64 - df + 0.5) / (df + 0.5) + 1.0).ln();
                        let tf_norm = tf * (Self::K1 + 1.0)
                            / (tf + Self::K1 * (1.0 - Self::B + Self::B * doc_len / avg_doc_len));
                        idf * tf_norm
                    })
                    .sum();

                if score > 0.0 { Some((path.clone(), score)) } else { None }
            })
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);
        scored
    }

    /// Tokenize into lowercase alphanumeric words, discarding single-char tokens.
    fn tokenize(text: &str) -> Vec<String> {
        text.split(|c: char| !c.is_alphanumeric())
            .filter(|w| w.len() >= 2)
            .map(|w| w.to_lowercase())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_json_simple() -> Result<()> {
        let json = r#"{"name": "test", "version": "1.0.0"}"#;
        let symbols = DocumentParser::parse_json(json)?;
        
        assert!(!symbols.is_empty());
        assert!(symbols.iter().any(|s| s.name == "name"));
        assert!(symbols.iter().any(|s| s.name == "version"));
        
        Ok(())
    }

    #[test]
    fn test_parse_markdown_headings() -> Result<()> {
        let markdown = r#"# Title
## Section 1
### Subsection
## Section 2
"#;
        let symbols = DocumentParser::parse_markdown(markdown)?;
        
        assert_eq!(symbols.len(), 4);
        assert_eq!(symbols[0].name, "Title");
        assert_eq!(symbols[1].name, "Section 1");
        
        Ok(())
    }

    #[test]
    fn test_parse_xml_tags() -> Result<()> {
        let xml = r#"<?xml version="1.0"?>
<root>
  <element1>content</element1>
  <element2>content</element2>
</root>"#;
        let symbols = DocumentParser::parse_xml(xml)?;
        
        assert_eq!(symbols.len(), 3);
        assert!(symbols.iter().any(|s| s.name == "root"));
        assert!(symbols.iter().any(|s| s.name == "element1"));
        assert!(symbols.iter().any(|s| s.name == "element2"));
        
        Ok(())
    }
}
