/// Markdown Parser with Mermaid Diagram Support
///
/// This library parses Markdown text into a structured Abstract Syntax Tree (AST).
/// It provides special handling for Mermaid diagrams, distinguishing them from
/// standard code blocks.

use serde::{Deserialize, Serialize};

/// Represents inline elements within text (bold, italic, links, plain text)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Inline {
    /// Plain text content
    Text(String),
    /// Bold text (**text**)
    Bold(Vec<Inline>),
    /// Italic text (*text*)
    Italic(Vec<Inline>),
    /// Link [text](url)
    Link {
        text: Vec<Inline>,
        url: String,
    },
}

/// Represents a node in the Markdown Abstract Syntax Tree
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Node {
    /// A heading with level (1-6) and content
    Heading {
        level: u8,
        content: Vec<Inline>,
    },
    /// A paragraph of text
    Paragraph(Vec<Inline>),
    /// An unordered list item
    ListItem(Vec<Inline>),
    /// A fenced code block with optional language identifier
    CodeBlock {
        lang: Option<String>,
        code: String,
    },
    /// A Mermaid diagram (distinct from CodeBlock)
    MermaidDiagram(String),
}

/// Parser for converting Markdown text into an AST
pub struct Parser {
    input: String,
}

impl Parser {
    /// Create a new parser from a Markdown string
    pub fn new(input: String) -> Self {
        Self { input }
    }

    /// Parse the input Markdown into a vector of AST nodes
    pub fn parse(&self) -> Vec<Node> {
        let mut nodes = Vec::new();
        let lines: Vec<&str> = self.input.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i].trim();

            // Skip empty lines
            if line.is_empty() {
                i += 1;
                continue;
            }

            // Check for fenced code blocks (```)
            if line.starts_with("```") {
                let lang_tag = line[3..].trim();
                let lang = if lang_tag.is_empty() {
                    None
                } else {
                    Some(lang_tag.to_string())
                };

                // Collect code block content until closing fence
                let mut code_lines = Vec::new();
                i += 1;
                while i < lines.len() {
                    if lines[i].trim() == "```" {
                        break;
                    }
                    code_lines.push(lines[i]);
                    i += 1;
                }

                let code = code_lines.join("\n");

                // Special handling for Mermaid diagrams
                if lang.as_ref().map(|s| s.to_lowercase()) == Some("mermaid".to_string()) {
                    nodes.push(Node::MermaidDiagram(code));
                } else {
                    nodes.push(Node::CodeBlock { lang, code });
                }
                i += 1;
                continue;
            }

            // Check for headings (# syntax)
            if line.starts_with('#') {
                let mut level = 0;
                let mut chars = line.chars();
                while chars.next() == Some('#') && level < 6 {
                    level += 1;
                }

                if level > 0 && level <= 6 {
                    let content = line[level..].trim().to_string();
                    if !content.is_empty() {
                        nodes.push(Node::Heading(level as u8, content));
                        i += 1;
                        continue;
                    }
                }
            }

            // Collect paragraph lines (until empty line or block element)
            let mut para_lines = Vec::new();
            while i < lines.len() {
                let current_line = lines[i].trim();

                // Stop at empty line or block elements
                if current_line.is_empty() {
                    break;
                }
                if current_line.starts_with('#') || current_line.starts_with("```") {
                    break;
                }

                para_lines.push(current_line);
                i += 1;
            }

            if !para_lines.is_empty() {
                let para_text = para_lines.join(" ");
                nodes.push(Node::Paragraph(para_text));
            }
        }

        nodes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_paragraph() {
        let input = "This is a simple paragraph.".to_string();
        let parser = Parser::new(input);
        let result = parser.parse();

        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0],
            Node::Paragraph("This is a simple paragraph.".to_string())
        );
    }

    #[test]
    fn test_multiple_paragraphs() {
        let input = "First paragraph.\n\nSecond paragraph.".to_string();
        let parser = Parser::new(input);
        let result = parser.parse();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Node::Paragraph("First paragraph.".to_string()));
        assert_eq!(result[1], Node::Paragraph("Second paragraph.".to_string()));
    }

    #[test]
    fn test_empty_input() {
        let input = String::new();
        let parser = Parser::new(input);
        let result = parser.parse();

        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_whitespace_only() {
        let input = "   \n\n   ".to_string();
        let parser = Parser::new(input);
        let result = parser.parse();

        assert_eq!(result.len(), 0);
    }

    // Phase 2 Tests

    #[test]
    fn test_standard_code_block() {
        let input = "```rust\nfn main() {\n    println!(\"Hello\");\n}\n```".to_string();
        let parser = Parser::new(input);
        let result = parser.parse();

        assert_eq!(result.len(), 1);
        match &result[0] {
            Node::CodeBlock { lang, code } => {
                assert_eq!(lang.as_ref(), Some(&"rust".to_string()));
                assert_eq!(code, "fn main() {\n    println!(\"Hello\");\n}");
            }
            _ => panic!("Expected CodeBlock, got {:?}", result[0]),
        }
    }

    #[test]
    fn test_code_block_without_language() {
        let input = "```\nSome code here\n```".to_string();
        let parser = Parser::new(input);
        let result = parser.parse();

        assert_eq!(result.len(), 1);
        match &result[0] {
            Node::CodeBlock { lang, code } => {
                assert_eq!(lang, &None);
                assert_eq!(code, "Some code here");
            }
            _ => panic!("Expected CodeBlock, got {:?}", result[0]),
        }
    }

    #[test]
    fn test_mermaid_diagram() {
        let input = "```mermaid\ngraph TD\n    A-->B\n```".to_string();
        let parser = Parser::new(input);
        let result = parser.parse();

        assert_eq!(result.len(), 1);
        match &result[0] {
            Node::MermaidDiagram(diagram) => {
                assert_eq!(diagram, "graph TD\n    A-->B");
            }
            _ => panic!("Expected MermaidDiagram, got {:?}", result[0]),
        }
    }

    #[test]
    fn test_mermaid_vs_codeblock_distinction() {
        let input = "```rust\nfn main() {}\n```\n\n```mermaid\ngraph TD\n    A-->B\n```".to_string();
        let parser = Parser::new(input);
        let result = parser.parse();

        assert_eq!(result.len(), 2);
        // First should be CodeBlock
        match &result[0] {
            Node::CodeBlock { lang, .. } => {
                assert_eq!(lang.as_ref(), Some(&"rust".to_string()));
            }
            _ => panic!("First block should be CodeBlock, got {:?}", result[0]),
        }
        // Second should be MermaidDiagram
        match &result[1] {
            Node::MermaidDiagram(_) => {}
            _ => panic!("Second block should be MermaidDiagram, got {:?}", result[1]),
        }
    }

    #[test]
    fn test_heading_h1() {
        let input = "# Heading 1".to_string();
        let parser = Parser::new(input);
        let result = parser.parse();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Node::Heading(1, "Heading 1".to_string()));
    }

    #[test]
    fn test_heading_h2() {
        let input = "## Heading 2".to_string();
        let parser = Parser::new(input);
        let result = parser.parse();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Node::Heading(2, "Heading 2".to_string()));
    }

    #[test]
    fn test_heading_h6() {
        let input = "###### Heading 6".to_string();
        let parser = Parser::new(input);
        let result = parser.parse();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Node::Heading(6, "Heading 6".to_string()));
    }

    #[test]
    fn test_mixed_content() {
        let input = "# Title\n\nSome paragraph.\n\n```rust\nfn main() {}\n```\n\n```mermaid\ngraph TD\n    A-->B\n```".to_string();
        let parser = Parser::new(input);
        let result = parser.parse();

        assert_eq!(result.len(), 4);
        assert_eq!(result[0], Node::Heading(1, "Title".to_string()));
        assert_eq!(result[1], Node::Paragraph("Some paragraph.".to_string()));
        match &result[2] {
            Node::CodeBlock { lang, .. } => {
                assert_eq!(lang.as_ref(), Some(&"rust".to_string()));
            }
            _ => panic!("Expected CodeBlock"),
        }
        match &result[3] {
            Node::MermaidDiagram(_) => {}
            _ => panic!("Expected MermaidDiagram"),
        }
    }
}

