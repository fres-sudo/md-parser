//! Shared AST types for the Markdown parser.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

/// Source location in the input (1-based line for user-facing messages).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    /// 1-based line number
    pub line: usize,
    /// Optional 1-based column (when available)
    pub column: Option<usize>,
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.column {
            Some(col) => write!(f, "line {}, column {}", self.line, col),
            None => write!(f, "line {}", self.line),
        }
    }
}

/// Errors that can occur during parsing
#[derive(Debug, Clone)]
pub enum ParseError {
    /// Error compiling a regex pattern
    RegexCompilationError(String),
    /// Error extracting capture groups from regex match
    InvalidCaptureError(String),
    /// Error serializing AST to JSON
    SerializationError(String),
    /// Heading with more than 6 `#` characters
    InvalidHeadingLevel { level: u8, span: Span },
    /// Code fence opened, EOF before closing ```
    UnclosedCodeBlock { span: Span },
    /// Generic structural issues (future use)
    MalformedMarkdown { message: String, span: Span },
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
            ParseError::InvalidHeadingLevel { level, span } => {
                write!(f, "{}: invalid heading level {} (max 6)", span, level)
            }
            ParseError::UnclosedCodeBlock { span } => {
                write!(f, "{}: unclosed code block", span)
            }
            ParseError::MalformedMarkdown { message, span } => {
                write!(f, "{}: malformed markdown: {}", span, message)
            }
        }
    }
}

impl Error for ParseError {}

/// Column alignment for tables
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Alignment {
    /// Left alignment
    Left,
    /// Center alignment
    Center,
    /// Right alignment
    Right,
}

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
    /// Image ![alt](url)
    #[serde(rename = "image")]
    Image { alt: String, url: String },
}

/// A single item in an unordered list; may contain nested sub-lists.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListItem {
    /// Inline content of the list item
    pub content: Vec<Inline>,
    /// Nested sub-lists (indentation-based)
    pub children: Vec<ListItem>,
    /// Task list checkbox state: None for regular items, Some(false) for unchecked, Some(true) for checked
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checked: Option<bool>,
}

/// Validation status for Mermaid diagrams
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "lowercase")]
pub enum ValidationStatus {
    /// Diagram syntax is valid
    Valid,
    /// Diagram syntax is invalid with error messages
    Invalid { errors: Vec<String> },
    /// Diagram has not been validated
    NotValidated,
}

/// Configuration for Mermaid diagram rendering
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MermaidConfig {
    /// Theme name (default, neutral, dark, forest, base)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
    /// Font size (e.g., "16px")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_size: Option<String>,
    /// Font family (e.g., "trebuchet ms, verdana, arial")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_family: Option<String>,
    /// Additional theme variables as a JSON-like map
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme_variables: Option<HashMap<String, String>>,
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
    /// An unordered list (markers `-`, `*`, `+`) with optional nesting
    #[serde(rename = "unordered_list")]
    UnorderedList { items: Vec<ListItem> },
    /// A fenced code block with optional language identifier
    #[serde(rename = "code_block")]
    CodeBlock { lang: Option<String>, code: String },
    /// A Mermaid diagram (distinct from CodeBlock)
    #[serde(rename = "mermaid_diagram")]
    MermaidDiagram {
        /// The diagram content
        diagram: String,
        /// Diagram-specific configuration (merged from global and inline)
        #[serde(skip_serializing_if = "Option::is_none")]
        config: Option<MermaidConfig>,
        /// Validation status of the diagram
        validation_status: ValidationStatus,
        /// Validation warnings (non-fatal issues)
        #[serde(skip_serializing_if = "Vec::is_empty")]
        warnings: Vec<String>,
    },
    /// A markdown table
    #[serde(rename = "table")]
    Table {
        /// Header row cells (each cell is a vector of inline elements)
        headers: Vec<Vec<Inline>>,
        /// Data rows (each row is a vector of cells, each cell is a vector of inline elements)
        rows: Vec<Vec<Vec<Inline>>>,
        /// Column alignments (None = default/left, Some(Alignment) for explicit alignment)
        alignments: Vec<Option<Alignment>>,
    },
}
