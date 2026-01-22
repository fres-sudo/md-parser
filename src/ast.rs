//! Shared AST types for the Markdown parser.

use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

/// Errors that can occur during parsing
#[derive(Debug, Clone)]
pub enum ParseError {
    /// Error compiling a regex pattern
    RegexCompilationError(String),
    /// Error extracting capture groups from regex match
    InvalidCaptureError(String),
    /// Error serializing AST to JSON
    SerializationError(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::RegexCompilationError(msg) => {
                write!(f, "Regex compilation error: {}", msg)
            }
            ParseError::InvalidCaptureError(msg) => {
                write!(f, "Invalid capture error: {}", msg)
            }
            ParseError::SerializationError(msg) => {
                write!(f, "Serialization error: {}", msg)
            }
        }
    }
}

impl Error for ParseError {}

/// Represents inline elements within text (bold, italic, links, plain text)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Inline {
    /// Plain text content
    #[serde(rename = "text")]
    Text { content: String },
    /// Bold text (**text**)
    #[serde(rename = "bold")]
    Bold { content: Vec<Inline> },
    /// Italic text (*text*)
    #[serde(rename = "italic")]
    Italic { content: Vec<Inline> },
    /// Link [text](url)
    #[serde(rename = "link")]
    Link { text: Vec<Inline>, url: String },
}

/// Represents a node in the Markdown Abstract Syntax Tree
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Node {
    /// A heading with level (1-6) and content
    #[serde(rename = "heading")]
    Heading { level: u8, content: Vec<Inline> },
    /// A paragraph of text
    #[serde(rename = "paragraph")]
    Paragraph { content: Vec<Inline> },
    /// An unordered list item
    #[serde(rename = "list_item")]
    ListItem { content: Vec<Inline> },
    /// A fenced code block with optional language identifier
    #[serde(rename = "code_block")]
    CodeBlock { lang: Option<String>, code: String },
    /// A Mermaid diagram (distinct from CodeBlock)
    #[serde(rename = "mermaid_diagram")]
    MermaidDiagram { diagram: String },
}
