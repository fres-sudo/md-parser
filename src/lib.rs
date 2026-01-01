/// Markdown Parser with Mermaid Diagram Support
///
/// This library parses Markdown text into a structured Abstract Syntax Tree (AST).
/// It provides special handling for Mermaid diagrams, distinguishing them from
/// standard code blocks.

/// Represents a node in the Markdown Abstract Syntax Tree
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    /// A heading with level (1-6) and content
    Heading(u8, String),
    /// A paragraph of text
    Paragraph(String),
    /// An unordered list item
    ListItem(String),
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
        // Phase 1: Simple paragraph parsing
        // Split by double newlines to get paragraphs
        let paragraphs: Vec<&str> = self
            .input
            .split("\n\n")
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        paragraphs
            .into_iter()
            .map(|text| Node::Paragraph(text.to_string()))
            .collect()
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
}

