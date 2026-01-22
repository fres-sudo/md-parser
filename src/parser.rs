//! Markdown parsing logic.

use crate::ast::{Inline, Node, ParseError};
use regex::Regex;

/// Maximum heading level supported (1-6)
const MAX_HEADING_LEVEL: u8 = 6;

/// Compiled regex patterns for inline element parsing
struct RegexPatterns {
    link: Regex,
    bold: Regex,
    italic: Regex,
}

impl RegexPatterns {
    /// Compile all regex patterns
    fn new() -> Result<Self, ParseError> {
        Ok(RegexPatterns {
            link: Regex::new(r"\[([^\]]+)\]\(([^)]+)\)")
                .map_err(|e| ParseError::RegexCompilationError(format!("Link regex: {}", e)))?,
            bold: Regex::new(r"\*\*((?:[^*]|\*[^*\n])+?)\*\*")
                .map_err(|e| ParseError::RegexCompilationError(format!("Bold regex: {}", e)))?,
            italic: Regex::new(r"\*([^*\n]+?)\*")
                .map_err(|e| ParseError::RegexCompilationError(format!("Italic regex: {}", e)))?,
        })
    }
}

/// Parser for converting Markdown text into an AST
pub struct Parser {
    input: String,
    regex_patterns: RegexPatterns,
}

impl Parser {
    /// Create a new parser from a Markdown string
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if regex patterns fail to compile
    pub fn new(input: String) -> Result<Self, ParseError> {
        let regex_patterns = RegexPatterns::new()?;
        Ok(Self {
            input,
            regex_patterns,
        })
    }

    /// Find the earliest match among all inline patterns
    fn find_earliest_match(&self, text: &str) -> Option<(usize, usize, &'static str)> {
        let mut earliest_pos = text.len();
        let mut match_type = None;
        let mut match_range = (0, 0);

        // Check for links
        if let Some(m) = self.regex_patterns.link.find(text) {
            if m.start() < earliest_pos {
                earliest_pos = m.start();
                match_type = Some("link");
                match_range = (m.start(), m.end());
            }
        }

        // Check for bold (must check before italic to avoid conflicts)
        if let Some(m) = self.regex_patterns.bold.find(text) {
            if m.start() < earliest_pos {
                earliest_pos = m.start();
                match_type = Some("bold");
                match_range = (m.start(), m.end());
            }
        }

        // Check for italic (only if not part of bold - check that it's not **)
        if let Some(m) = self.regex_patterns.italic.find(text) {
            let start = m.start();
            let end = m.end();
            // Make sure it's not part of bold (check for ** before or after)
            let is_bold = (start > 0 && text.as_bytes()[start - 1] == b'*')
                || (end < text.len() && text.as_bytes()[end] == b'*');

            if !is_bold && start < earliest_pos {
                match_type = Some("italic");
                match_range = (start, end);
            }
        }

        match_type.map(|mt| (match_range.0, match_range.1, mt))
    }

    /// Process a link match and add it to inlines
    fn process_link_match<'a>(
        &self,
        remaining: &'a str,
        match_range: (usize, usize),
        inlines: &mut Vec<Inline>,
    ) -> Result<&'a str, ParseError> {
        // Add text before the link
        if match_range.0 > 0 {
            let text_before = &remaining[..match_range.0];
            if !text_before.is_empty() {
                inlines.push(Inline::Text {
                    content: text_before.to_string(),
                });
            }
        }

        let match_text = &remaining[match_range.0..match_range.1];
        let caps = self
            .regex_patterns
            .link
            .captures(match_text)
            .ok_or_else(|| {
                ParseError::InvalidCaptureError("Failed to capture link groups".to_string())
            })?;

        let link_text = caps
            .get(1)
            .ok_or_else(|| {
                ParseError::InvalidCaptureError("Failed to capture link text".to_string())
            })?
            .as_str();
        let link_url = caps
            .get(2)
            .ok_or_else(|| {
                ParseError::InvalidCaptureError("Failed to capture link URL".to_string())
            })?
            .as_str();

        let text_inlines = self.parse_inline(link_text)?;
        inlines.push(Inline::Link {
            text: text_inlines,
            url: link_url.to_string(),
        });

        Ok(&remaining[match_range.1..])
    }

    /// Process a bold match and add it to inlines
    fn process_bold_match<'a>(
        &self,
        remaining: &'a str,
        match_range: (usize, usize),
        inlines: &mut Vec<Inline>,
    ) -> Result<&'a str, ParseError> {
        // Add text before the bold
        if match_range.0 > 0 {
            let text_before = &remaining[..match_range.0];
            if !text_before.is_empty() {
                inlines.push(Inline::Text {
                    content: text_before.to_string(),
                });
            }
        }

        let match_text = &remaining[match_range.0..match_range.1];
        let caps = self
            .regex_patterns
            .bold
            .captures(match_text)
            .ok_or_else(|| {
                ParseError::InvalidCaptureError("Failed to capture bold groups".to_string())
            })?;

        let bold_text = caps
            .get(1)
            .ok_or_else(|| {
                ParseError::InvalidCaptureError("Failed to capture bold text".to_string())
            })?
            .as_str();

        let bold_inlines = self.parse_inline(bold_text)?;
        inlines.push(Inline::Bold {
            content: bold_inlines,
        });

        Ok(&remaining[match_range.1..])
    }

    /// Process an italic match and add it to inlines
    fn process_italic_match<'a>(
        &self,
        remaining: &'a str,
        match_range: (usize, usize),
        inlines: &mut Vec<Inline>,
    ) -> Result<&'a str, ParseError> {
        // Add text before the italic
        if match_range.0 > 0 {
            let text_before = &remaining[..match_range.0];
            if !text_before.is_empty() {
                inlines.push(Inline::Text {
                    content: text_before.to_string(),
                });
            }
        }

        let match_text = &remaining[match_range.0..match_range.1];
        let caps = self
            .regex_patterns
            .italic
            .captures(match_text)
            .ok_or_else(|| {
                ParseError::InvalidCaptureError("Failed to capture italic groups".to_string())
            })?;

        let italic_text = caps
            .get(1)
            .ok_or_else(|| {
                ParseError::InvalidCaptureError("Failed to capture italic text".to_string())
            })?
            .as_str();

        let italic_inlines = self.parse_inline(italic_text)?;
        inlines.push(Inline::Italic {
            content: italic_inlines,
        });

        Ok(&remaining[match_range.1..])
    }

    /// Parse inline elements from a text string
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if regex matching fails
    fn parse_inline(&self, text: &str) -> Result<Vec<Inline>, ParseError> {
        let mut inlines = Vec::new();
        let mut remaining = text;

        while !remaining.is_empty() {
            if let Some((start, end, match_type)) = self.find_earliest_match(remaining) {
                let match_range = (start, end);
                remaining = match match_type {
                    "link" => self.process_link_match(remaining, match_range, &mut inlines)?,
                    "bold" => self.process_bold_match(remaining, match_range, &mut inlines)?,
                    "italic" => self.process_italic_match(remaining, match_range, &mut inlines)?,
                    _ => {
                        // Unexpected match type (should not happen)
                        &remaining[end..]
                    }
                };
            } else {
                // No more matches, add remaining text
                if !remaining.is_empty() {
                    inlines.push(Inline::Text {
                        content: remaining.to_string(),
                    });
                }
                break;
            }
        }

        // If no inline elements were found, return a single text node
        if inlines.is_empty() && !text.is_empty() {
            inlines.push(Inline::Text {
                content: text.to_string(),
            });
        }

        Ok(inlines)
    }

    /// Parse a fenced code block starting at the given line index
    ///
    /// Returns the node and the new line index after the code block
    fn parse_code_block(
        &self,
        lines: &[&str],
        start_idx: usize,
    ) -> Result<(Node, usize), ParseError> {
        let line = lines[start_idx].trim();
        let lang_tag = line[3..].trim();
        let lang = if lang_tag.is_empty() {
            None
        } else {
            Some(lang_tag.to_string())
        };

        // Collect code block content until closing fence
        let mut code_lines = Vec::new();
        let mut i = start_idx + 1;
        while i < lines.len() {
            if lines[i].trim() == "```" {
                break;
            }
            code_lines.push(lines[i]);
            i += 1;
        }

        let code = code_lines.join("\n");

        // Special handling for Mermaid diagrams
        let node = if lang.as_ref().map(|s| s.to_lowercase()) == Some("mermaid".to_string()) {
            Node::MermaidDiagram { diagram: code }
        } else {
            Node::CodeBlock { lang, code }
        };

        Ok((node, i + 1))
    }

    /// Parse a heading from a line
    ///
    /// Returns Some(node) if a valid heading is found, None otherwise
    fn parse_heading(&self, line: &str) -> Result<Option<Node>, ParseError> {
        if !line.starts_with('#') {
            return Ok(None);
        }

        let mut level = 0;
        let mut chars = line.chars();
        while chars.next() == Some('#') && level < MAX_HEADING_LEVEL as usize {
            level += 1;
        }

        if level > 0 && level <= MAX_HEADING_LEVEL as usize {
            let content = line[level..].trim();
            if !content.is_empty() {
                let inline_content = self.parse_inline(content)?;
                return Ok(Some(Node::Heading {
                    level: level as u8,
                    content: inline_content,
                }));
            }
        }

        Ok(None)
    }

    /// Collect paragraph lines starting at the given index
    ///
    /// Returns the paragraph text and the new line index after the paragraph
    fn collect_paragraph_lines(&self, lines: &[&str], start_idx: usize) -> (String, usize) {
        let mut para_lines = Vec::new();
        let mut i = start_idx;

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

        let para_text = para_lines.join(" ");
        (para_text, i)
    }

    /// Parse the input Markdown into a vector of AST nodes
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if parsing fails
    pub fn parse(&self) -> Result<Vec<Node>, ParseError> {
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
                let (node, new_idx) = self.parse_code_block(&lines, i)?;
                nodes.push(node);
                i = new_idx;
                continue;
            }

            // Check for headings (# syntax)
            if let Some(heading_node) = self.parse_heading(line)? {
                nodes.push(heading_node);
                i += 1;
                continue;
            }

            // Collect paragraph lines (until empty line or block element)
            let (para_text, new_idx) = self.collect_paragraph_lines(&lines, i);
            if !para_text.is_empty() {
                let inline_content = self.parse_inline(&para_text)?;
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
    pub fn to_json(&self) -> Result<String, ParseError> {
        let ast = self.parse()?;
        serde_json::to_string_pretty(&ast).map_err(|e| {
            ParseError::SerializationError(format!("JSON serialization failed: {}", e))
        })
    }
}
