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
        let mut line_num = 1u32;

        for line in content.lines() {
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
            line_num += 1;
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
