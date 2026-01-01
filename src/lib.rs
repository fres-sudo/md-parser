/// Markdown Parser with Mermaid Diagram Support
///
/// This library parses Markdown text into a structured Abstract Syntax Tree (AST).
/// It provides special handling for Mermaid diagrams, distinguishing them from
/// standard code blocks.

use serde::{Deserialize, Serialize};

/// Represents inline elements within text (bold, italic, links, plain text)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Inline {
    /// Plain text content
    #[serde(rename = "text")]
    Text {
        content: String,
    },
    /// Bold text (**text**)
    #[serde(rename = "bold")]
    Bold {
        content: Vec<Inline>,
    },
    /// Italic text (*text*)
    #[serde(rename = "italic")]
    Italic {
        content: Vec<Inline>,
    },
    /// Link [text](url)
    #[serde(rename = "link")]
    Link {
        text: Vec<Inline>,
        url: String,
    },
}

/// Represents a node in the Markdown Abstract Syntax Tree
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Node {
    /// A heading with level (1-6) and content
    #[serde(rename = "heading")]
    Heading {
        level: u8,
        content: Vec<Inline>,
    },
    /// A paragraph of text
    #[serde(rename = "paragraph")]
    Paragraph {
        content: Vec<Inline>,
    },
    /// An unordered list item
    #[serde(rename = "list_item")]
    ListItem {
        content: Vec<Inline>,
    },
    /// A fenced code block with optional language identifier
    #[serde(rename = "code_block")]
    CodeBlock {
        lang: Option<String>,
        code: String,
    },
    /// A Mermaid diagram (distinct from CodeBlock)
    #[serde(rename = "mermaid_diagram")]
    MermaidDiagram {
        diagram: String,
    },
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

    /// Parse inline elements from a text string
    fn parse_inline(&self, text: &str) -> Vec<Inline> {
        use regex::Regex;
        let mut inlines = Vec::new();
        let mut remaining = text;

        // Regex patterns for inline elements
        // Note: Process bold before italic to avoid conflicts
        let link_re = Regex::new(r"\[([^\]]+)\]\(([^)]+)\)").unwrap();
        // Bold: **...** - match content that doesn't contain **
        // We'll use a pattern that matches **, then content (allowing single *), then **
        // Simple approach: match ** then any chars until **, but we need to be careful
        let bold_re = Regex::new(r"\*\*((?:[^*]|\*[^*\n])+?)\*\*").unwrap();
        // Italic: single * not preceded or followed by *
        // We'll check manually that it's not part of bold
        let italic_re = Regex::new(r"\*([^*\n]+?)\*").unwrap();

        while !remaining.is_empty() {
            // Find the earliest match among all patterns
            let mut earliest_pos = remaining.len();
            let mut match_type = None;
            let mut match_range = (0, 0);

            // Check for links
            if let Some(m) = link_re.find(remaining) {
                if m.start() < earliest_pos {
                    earliest_pos = m.start();
                    match_type = Some("link");
                    match_range = (m.start(), m.end());
                }
            }

            // Check for bold (must check before italic to avoid conflicts)
            if let Some(m) = bold_re.find(remaining) {
                if m.start() < earliest_pos {
                    earliest_pos = m.start();
                    match_type = Some("bold");
                    match_range = (m.start(), m.end());
                }
            }

            // Check for italic (only if not part of bold - check that it's not **)
            if let Some(m) = italic_re.find(remaining) {
                let start = m.start();
                let end = m.end();
                // Make sure it's not part of bold (check for ** before or after)
                let is_bold = (start > 0 && remaining.as_bytes()[start - 1] == b'*')
                    || (end < remaining.len() && remaining.as_bytes()[end] == b'*');

                if !is_bold && start < earliest_pos {
                    match_type = Some("italic");
                    match_range = (start, end);
                    earliest_pos = start;
                }
            }

            // Process the match
            match match_type {
                Some("link") => {
                    // Add text before the link
                    if match_range.0 > 0 {
                        let text_before = &remaining[..match_range.0];
                        if !text_before.is_empty() {
                            inlines.push(Inline::Text { content: text_before.to_string() });
                        }
                    }
                    if let Some(caps) = link_re.captures(&remaining[match_range.0..match_range.1]) {
                        let link_text = caps.get(1).unwrap().as_str();
                        let link_url = caps.get(2).unwrap().as_str();
                        let text_inlines = self.parse_inline(link_text);
                        inlines.push(Inline::Link {
                            text: text_inlines,
                            url: link_url.to_string(),
                        });
                    }
                    remaining = &remaining[match_range.1..];
                }
                Some("bold") => {
                    // Add text before the bold
                    if match_range.0 > 0 {
                        let text_before = &remaining[..match_range.0];
                        if !text_before.is_empty() {
                            inlines.push(Inline::Text { content: text_before.to_string() });
                        }
                    }
                    if let Some(caps) = bold_re.captures(&remaining[match_range.0..match_range.1]) {
                        let bold_text = caps.get(1).unwrap().as_str();
                        let bold_inlines = self.parse_inline(bold_text);
                        inlines.push(Inline::Bold { content: bold_inlines });
                    }
                    remaining = &remaining[match_range.1..];
                }
                Some("italic") => {
                    // Add text before the italic
                    if match_range.0 > 0 {
                        let text_before = &remaining[..match_range.0];
                        if !text_before.is_empty() {
                            inlines.push(Inline::Text { content: text_before.to_string() });
                        }
                    }
                    if let Some(caps) = italic_re.captures(&remaining[match_range.0..match_range.1]) {
                        let italic_text = caps.get(1).unwrap().as_str();
                        let italic_inlines = self.parse_inline(italic_text);
                        inlines.push(Inline::Italic { content: italic_inlines });
                    }
                    remaining = &remaining[match_range.1..];
                }
                Some(_) => {
                    // Unexpected match type (should not happen)
                    remaining = &remaining[match_range.1..];
                }
                None => {
                    // No more matches, add remaining text
                    if !remaining.is_empty() {
                        inlines.push(Inline::Text { content: remaining.to_string() });
                    }
                    break;
                }
            }
        }

        // If no inline elements were found, return a single text node
        if inlines.is_empty() && !text.is_empty() {
            inlines.push(Inline::Text { content: text.to_string() });
        }

        inlines
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
                    nodes.push(Node::MermaidDiagram { diagram: code });
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
                    let content = line[level..].trim();
                    if !content.is_empty() {
                        let inline_content = self.parse_inline(content);
                        nodes.push(Node::Heading {
                            level: level as u8,
                            content: inline_content,
                        });
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
                let inline_content = self.parse_inline(&para_text);
                nodes.push(Node::Paragraph { content: inline_content });
            }
        }

        nodes
    }

    /// Serialize the AST to JSON string
    pub fn to_json(&self) -> String {
        let ast = self.parse();
        serde_json::to_string_pretty(&ast).unwrap_or_else(|e| format!("Error serializing: {}", e))
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
        match &result[0] {
            Node::Paragraph { content: inlines } => {
                assert_eq!(inlines.len(), 1);
                assert_eq!(inlines[0], Inline::Text { content: "This is a simple paragraph.".to_string() });
            }
            _ => panic!("Expected Paragraph"),
        }
    }

    #[test]
    fn test_multiple_paragraphs() {
        let input = "First paragraph.\n\nSecond paragraph.".to_string();
        let parser = Parser::new(input);
        let result = parser.parse();

        assert_eq!(result.len(), 2);
        match &result[0] {
            Node::Paragraph { content: inlines } => {
                assert_eq!(inlines.len(), 1);
                assert_eq!(inlines[0], Inline::Text { content: "First paragraph.".to_string() });
            }
            _ => panic!("Expected Paragraph"),
        }
        match &result[1] {
            Node::Paragraph { content: inlines } => {
                assert_eq!(inlines.len(), 1);
                assert_eq!(inlines[0], Inline::Text { content: "Second paragraph.".to_string() });
            }
            _ => panic!("Expected Paragraph"),
        }
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
            Node::MermaidDiagram { diagram } => {
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
            Node::MermaidDiagram { .. } => {}
            _ => panic!("Second block should be MermaidDiagram, got {:?}", result[1]),
        }
    }

    #[test]
    fn test_heading_h1() {
        let input = "# Heading 1".to_string();
        let parser = Parser::new(input);
        let result = parser.parse();

        assert_eq!(result.len(), 1);
        match &result[0] {
            Node::Heading { level, content } => {
                assert_eq!(*level, 1);
                assert_eq!(content.len(), 1);
                assert_eq!(content[0], Inline::Text { content: "Heading 1".to_string() });
            }
            _ => panic!("Expected Heading"),
        }
    }

    #[test]
    fn test_heading_h2() {
        let input = "## Heading 2".to_string();
        let parser = Parser::new(input);
        let result = parser.parse();

        assert_eq!(result.len(), 1);
        match &result[0] {
            Node::Heading { level, content } => {
                assert_eq!(*level, 2);
                assert_eq!(content.len(), 1);
                assert_eq!(content[0], Inline::Text { content: "Heading 2".to_string() });
            }
            _ => panic!("Expected Heading"),
        }
    }

    #[test]
    fn test_heading_h6() {
        let input = "###### Heading 6".to_string();
        let parser = Parser::new(input);
        let result = parser.parse();

        assert_eq!(result.len(), 1);
        match &result[0] {
            Node::Heading { level, content } => {
                assert_eq!(*level, 6);
                assert_eq!(content.len(), 1);
                assert_eq!(content[0], Inline::Text { content: "Heading 6".to_string() });
            }
            _ => panic!("Expected Heading"),
        }
    }

    #[test]
    fn test_mixed_content() {
        let input = "# Title\n\nSome paragraph.\n\n```rust\nfn main() {}\n```\n\n```mermaid\ngraph TD\n    A-->B\n```".to_string();
        let parser = Parser::new(input);
        let result = parser.parse();

        assert_eq!(result.len(), 4);
        match &result[0] {
            Node::Heading { level, .. } => {
                assert_eq!(*level, 1);
            }
            _ => panic!("Expected Heading"),
        }
        match &result[1] {
            Node::Paragraph { content: inlines } => {
                assert_eq!(inlines.len(), 1);
                assert_eq!(inlines[0], Inline::Text { content: "Some paragraph.".to_string() });
            }
            _ => panic!("Expected Paragraph"),
        }
        match &result[2] {
            Node::CodeBlock { lang, .. } => {
                assert_eq!(lang.as_ref(), Some(&"rust".to_string()));
            }
            _ => panic!("Expected CodeBlock"),
        }
        match &result[3] {
            Node::MermaidDiagram { .. } => {}
            _ => panic!("Expected MermaidDiagram"),
        }
    }

    // Phase 3 Tests

    #[test]
    fn test_bold_text() {
        let input = "This is **bold** text.".to_string();
        let parser = Parser::new(input);
        let result = parser.parse();

        assert_eq!(result.len(), 1);
        match &result[0] {
            Node::Paragraph { content: inlines } => {
                assert_eq!(inlines.len(), 3);
                assert_eq!(inlines[0], Inline::Text { content: "This is ".to_string() });
                match &inlines[1] {
                    Inline::Bold { content: bold_inlines } => {
                        assert_eq!(bold_inlines.len(), 1);
                        assert_eq!(bold_inlines[0], Inline::Text { content: "bold".to_string() });
                    }
                    _ => panic!("Expected Bold"),
                }
                assert_eq!(inlines[2], Inline::Text { content: " text.".to_string() });
            }
            _ => panic!("Expected Paragraph"),
        }
    }

    #[test]
    fn test_italic_text() {
        let input = "This is *italic* text.".to_string();
        let parser = Parser::new(input);
        let result = parser.parse();

        assert_eq!(result.len(), 1);
        match &result[0] {
            Node::Paragraph { content: inlines } => {
                assert_eq!(inlines.len(), 3);
                assert_eq!(inlines[0], Inline::Text { content: "This is ".to_string() });
                match &inlines[1] {
                    Inline::Italic { content: italic_inlines } => {
                        assert_eq!(italic_inlines.len(), 1);
                        assert_eq!(italic_inlines[0], Inline::Text { content: "italic".to_string() });
                    }
                    _ => panic!("Expected Italic"),
                }
                assert_eq!(inlines[2], Inline::Text { content: " text.".to_string() });
            }
            _ => panic!("Expected Paragraph"),
        }
    }

    #[test]
    fn test_link() {
        let input = "Visit [Rust](https://rust-lang.org) today!".to_string();
        let parser = Parser::new(input);
        let result = parser.parse();

        assert_eq!(result.len(), 1);
        match &result[0] {
            Node::Paragraph { content: inlines } => {
                assert_eq!(inlines.len(), 3);
                assert_eq!(inlines[0], Inline::Text { content: "Visit ".to_string() });
                match &inlines[1] {
                    Inline::Link { text, url } => {
                        assert_eq!(text.len(), 1);
                        assert_eq!(text[0], Inline::Text { content: "Rust".to_string() });
                        assert_eq!(url, "https://rust-lang.org");
                    }
                    _ => panic!("Expected Link"),
                }
                assert_eq!(inlines[2], Inline::Text { content: " today!".to_string() });
            }
            _ => panic!("Expected Paragraph"),
        }
    }

    #[test]
    fn test_nested_bold_italic() {
        let input = "This is **bold with *italic* inside**.".to_string();
        let parser = Parser::new(input);
        let result = parser.parse();

        assert_eq!(result.len(), 1);
        match &result[0] {
            Node::Paragraph { content: inlines } => {
                // Should have at least "This is " text and a Bold element
                assert!(inlines.len() >= 1);
                // Check that we have a Bold element somewhere
                let has_bold = inlines.iter().any(|inline| {
                    matches!(inline, Inline::Bold { .. })
                });
                assert!(has_bold, "Expected at least one Bold element");
                // If we have text before bold, verify it
                if let Some(Inline::Text { content }) = inlines.first() {
                    assert!(content.contains("This is") || content.is_empty());
                }
            }
            _ => panic!("Expected Paragraph"),
        }
    }

    #[test]
    fn test_heading_with_inline() {
        let input = "# This is a **bold** heading".to_string();
        let parser = Parser::new(input);
        let result = parser.parse();

        assert_eq!(result.len(), 1);
        match &result[0] {
            Node::Heading { level, content } => {
                assert_eq!(*level, 1);
                assert!(content.len() >= 2);
            }
            _ => panic!("Expected Heading"),
        }
    }

    #[test]
    fn test_mixed_inline_elements() {
        let input = "Check out [Rust](https://rust-lang.org) and **bold** and *italic*.".to_string();
        let parser = Parser::new(input);
        let result = parser.parse();

        assert_eq!(result.len(), 1);
        match &result[0] {
            Node::Paragraph { content: inlines } => {
                // Should have multiple inline elements
                assert!(inlines.len() >= 3);
            }
            _ => panic!("Expected Paragraph"),
        }
    }
}

