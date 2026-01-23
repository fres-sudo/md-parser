//! Markdown parsing logic.

mod blockquotes;
mod blocks;
mod horizontal_rules;
mod inline;
mod lists;
mod mermaid;
mod tables;

use crate::ast::{Node, ParseError};
use crate::config::ParserConfig;

use inline::RegexPatterns;

/// Parser for converting Markdown text into an AST
pub struct Parser {
    input: String,
    regex_patterns: RegexPatterns,
    warnings: Vec<String>,
    config: ParserConfig,
}

impl Parser {
    /// Create a new parser from a Markdown string with default configuration
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if regex patterns fail to compile
    pub fn new(input: String) -> Result<Self, ParseError> {
        Self::with_config(input, ParserConfig::default())
    }

    /// Create a new parser from a Markdown string with custom configuration
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if regex patterns fail to compile
    pub fn with_config(input: String, config: ParserConfig) -> Result<Self, ParseError> {
        let regex_patterns = RegexPatterns::new()?;
        Ok(Self {
            input,
            regex_patterns,
            warnings: Vec::new(),
            config,
        })
    }

    /// Parse the input Markdown into a vector of AST nodes
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if parsing fails
    pub fn parse(&mut self) -> Result<Vec<Node>, ParseError> {
        // Clear warnings at the start of each parse
        self.warnings.clear();

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

            // Check for fenced code blocks
            if line.starts_with(&self.config.code_fence_pattern) {
                let (node, new_idx, warnings) =
                    blocks::parse_code_block(&lines, i, &self.config, &self.regex_patterns)?;
                self.warnings.extend(warnings);
                nodes.push(node);
                i = new_idx;
                continue;
            }

            // Check for headings (# syntax)
            let line_number = i + 1;
            if let Some(heading_node) =
                blocks::parse_heading(line, line_number, &self.config, &self.regex_patterns)?
            {
                nodes.push(heading_node);
                i += 1;
                continue;
            }

            // Check for ordered lists (must check before unordered lists, must check raw line, not trimmed, to detect indentation)
            if lists::detect_ordered_list_line(lines[i]).is_some() {
                let (list_node, new_idx) =
                    lists::parse_ordered_list(&lines, i, &self.config, &self.regex_patterns)?;
                nodes.push(list_node);
                i = new_idx;
                continue;
            }

            // Check for unordered lists (must check raw line, not trimmed, to detect indentation)
            if lists::detect_list_line(lines[i]).is_some() {
                let (list_node, new_idx) =
                    lists::parse_unordered_list(&lines, i, &self.config, &self.regex_patterns)?;
                nodes.push(list_node);
                i = new_idx;
                continue;
            }

            // Check for tables (must check if current line is a table row and next line is separator)
            if tables::detect_table_row(lines[i]) {
                // Check if next line is a separator
                if i + 1 < lines.len() && tables::detect_table_separator(lines[i + 1]) {
                    let (table_node, new_idx) =
                        tables::parse_table(&lines, i, &self.config, &self.regex_patterns)?;
                    nodes.push(table_node);
                    i = new_idx;
                    continue;
                }
            }

            // Check for blockquotes
            if blockquotes::detect_blockquote_line(lines[i]).is_some() {
                let (blockquote_node, new_idx) =
                    blockquotes::parse_blockquote(&lines, i, &self.config, &self.regex_patterns)?;
                nodes.push(blockquote_node);
                i = new_idx;
                continue;
            }

            // Check for horizontal rules
            if horizontal_rules::detect_horizontal_rule(lines[i]) {
                nodes.push(Node::HorizontalRule);
                i += 1;
                continue;
            }

            // Collect paragraph lines (until empty line or block element)
            let (para_text, new_idx) = blocks::collect_paragraph_lines(&lines, i, &self.config);
            if !para_text.is_empty() {
                let inline_content = inline::parse_inline(&para_text, &self.regex_patterns)?;
                nodes.push(Node::Paragraph {
                    content: inline_content,
                });
            }
            i = new_idx;
        }

        Ok(nodes)
    }

    /// Serialize the AST to JSON string
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if parsing or serialization fails
    pub fn to_json(&mut self) -> Result<String, ParseError> {
        let ast = self.parse()?;
        serde_json::to_string_pretty(&ast).map_err(|e| {
            ParseError::SerializationError(format!("JSON serialization failed: {}", e))
        })
    }

    /// Get a reference to the warnings collected during parsing
    ///
    /// Warnings are generated for issues like unclosed code blocks.
    /// The warnings vector is cleared at the start of each `parse()` call.
    pub fn warnings(&self) -> &[String] {
        &self.warnings
    }
}
