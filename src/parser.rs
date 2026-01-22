//! Markdown parsing logic.

use crate::ast::{Alignment, Inline, ListItem, Node, ParseError, Span};
use crate::config::ParserConfig;
use regex::Regex;

/// Type of inline element match found during parsing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InlineMatchType {
    Link,
    Bold,
    Italic,
}

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

    /// Find the earliest match among all inline patterns
    fn find_earliest_match(&self, text: &str) -> Option<(usize, usize, InlineMatchType)> {
        let mut earliest_pos = text.len();
        let mut match_type = None;
        let mut match_range = (0, 0);

        // Check for links
        if let Some(m) = self.regex_patterns.link.find(text) {
            if m.start() < earliest_pos {
                earliest_pos = m.start();
                match_type = Some(InlineMatchType::Link);
                match_range = (m.start(), m.end());
            }
        }

        // Check for bold (must check before italic to avoid conflicts)
        if let Some(m) = self.regex_patterns.bold.find(text) {
            if m.start() < earliest_pos {
                earliest_pos = m.start();
                match_type = Some(InlineMatchType::Bold);
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
                match_type = Some(InlineMatchType::Italic);
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
                    InlineMatchType::Link => {
                        self.process_link_match(remaining, match_range, &mut inlines)?
                    }
                    InlineMatchType::Bold => {
                        self.process_bold_match(remaining, match_range, &mut inlines)?
                    }
                    InlineMatchType::Italic => {
                        self.process_italic_match(remaining, match_range, &mut inlines)?
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
    /// Returns the node and the new line index after the code block.
    /// Errors with `UnclosedCodeBlock` if no closing fence is found before EOF.
    fn parse_code_block(
        &self,
        lines: &[&str],
        start_idx: usize,
    ) -> Result<(Node, usize), ParseError> {
        let line = lines[start_idx].trim();
        let lang_tag = line[self.config.code_fence_length..].trim();
        let lang = if lang_tag.is_empty() {
            None
        } else {
            Some(lang_tag.to_string())
        };

        // Collect code block content until closing fence
        let mut code_lines = Vec::new();
        let mut i = start_idx + 1;
        let mut is_closed = false;
        while i < lines.len() {
            if lines[i].trim() == self.config.code_fence_pattern {
                is_closed = true;
                break;
            }
            code_lines.push(lines[i]);
            i += 1;
        }

        if !is_closed {
            let span = Span {
                line: start_idx + 1,
                column: None,
            };
            return Err(ParseError::UnclosedCodeBlock { span });
        }

        let code = code_lines.join("\n");

        // Special handling for Mermaid diagrams
        let node = if lang.as_ref().map(|s| s.to_lowercase())
            == Some(self.config.mermaid_language.to_lowercase())
        {
            Node::MermaidDiagram { diagram: code }
        } else {
            Node::CodeBlock { lang, code }
        };

        Ok((node, i + 1))
    }

    /// Parse a heading from a line
    ///
    /// Returns `Some(node)` if a valid heading is found, `None` if not a heading.
    /// Errors with `InvalidHeadingLevel` if the line has more than 6 leading `#`.
    fn parse_heading(&self, line: &str, line_number: usize) -> Result<Option<Node>, ParseError> {
        if !line.starts_with('#') {
            return Ok(None);
        }

        let level = line.chars().take_while(|&c| c == '#').count();
        if level > self.config.max_heading_level as usize {
            let span = Span {
                line: line_number,
                column: None,
            };
            return Err(ParseError::InvalidHeadingLevel {
                level: level as u8,
                span,
            });
        }

        if level > 0 {
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

    /// Check if a raw line (with indentation) matches the list pattern
    ///
    /// Returns Some((indent_level, marker, content, checked)) if it's a list line, None otherwise.
    /// Indent level is calculated as number of 2-space increments (0 = no indent, 1 = 2 spaces, etc.)
    /// checked is Some(bool) for task list items, None for regular list items.
    fn detect_list_line(line: &str) -> Option<(usize, char, &str, Option<bool>)> {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return None;
        }

        // Check for list markers: -, *, or +
        let marker_pos = line.find(['-', '*', '+'])?;
        let marker = line.as_bytes()[marker_pos] as char;

        // Must be followed by a space
        if marker_pos + 1 >= line.len() || line.as_bytes()[marker_pos + 1] != b' ' {
            return None;
        }

        // Calculate indent: count leading spaces, divide by 2 (round down)
        let leading_spaces = line[..marker_pos].chars().take_while(|&c| c == ' ').count();
        let indent_level = leading_spaces / 2;

        // Check for task list pattern: - [ ] or - [x] or - [X]
        // Only applies to '-' marker
        if marker == '-' && marker_pos + 4 <= line.len() {
            let after_marker = &line[marker_pos + 2..];
            if after_marker.starts_with("[ ]") {
                // Unchecked task: - [ ] content (or just - [ ])
                if after_marker.len() == 3 {
                    // Empty task: - [ ]
                    return Some((indent_level, marker, "", Some(false)));
                } else if after_marker.as_bytes()[3] == b' ' {
                    // Task with content: - [ ] content
                    let content = after_marker[4..].trim();
                    return Some((indent_level, marker, content, Some(false)));
                }
            } else if after_marker.starts_with("[x]") || after_marker.starts_with("[X]") {
                // Checked task: - [x] or - [X] content (or just - [x])
                if after_marker.len() == 3 {
                    // Empty task: - [x] or - [X]
                    return Some((indent_level, marker, "", Some(true)));
                } else if after_marker.as_bytes()[3] == b' ' {
                    // Task with content: - [x] content
                    let content = after_marker[4..].trim();
                    return Some((indent_level, marker, content, Some(true)));
                }
            }
        }

        // Regular list item: extract content after marker and space
        let content = line[marker_pos + 2..].trim();
        Some((indent_level, marker, content, None))
    }

    /// Check if a line is a continuation line (indented, no marker)
    ///
    /// Returns Some(indent_level) if it's a continuation, None otherwise
    fn detect_continuation_line(line: &str) -> Option<usize> {
        if line.trim().is_empty() {
            return None;
        }

        // Must start with spaces (indented)
        let leading_spaces = line.chars().take_while(|&c| c == ' ').count();
        if leading_spaces == 0 {
            return None;
        }

        // Must NOT match list pattern (no marker)
        if Self::detect_list_line(line).is_some() {
            return None;
        }

        // Must not be a block element
        // Note: This is a static method, so we can't access config here.
        // We'll check for the default pattern "```" which is the standard.
        let trimmed = line.trim();
        if trimmed.starts_with('#') || trimmed.starts_with("```") {
            return None;
        }

        Some(leading_spaces / 2)
    }

    /// Parse an unordered list starting at the given line index
    ///
    /// Returns the node and the new line index after the list
    fn parse_unordered_list(
        &self,
        lines: &[&str],
        start_idx: usize,
    ) -> Result<(Node, usize), ParseError> {
        let mut items = Vec::new();
        let mut i = start_idx;
        // Track the last item at each indent level for easy access
        // last_items[0] = last top-level item, last_items[1] = last item at indent 1, etc.
        let mut last_items: Vec<Option<usize>> = Vec::new();
        // Track the path to the most recently added item for continuation lines
        let mut last_item_path: Vec<(usize, usize)> = Vec::new();

        while i < lines.len() {
            let line = lines[i];

            // Check for empty line - end of list
            if line.trim().is_empty() {
                break;
            }

            // Check for block elements - end of list
            let trimmed = line.trim();
            if trimmed.starts_with('#') || trimmed.starts_with(&self.config.code_fence_pattern) {
                break;
            }

            // Check if it's a list line
            if let Some((indent_level, _marker, content, checked)) = Self::detect_list_line(line) {
                // Parse the content as inline elements
                let inline_content = if content.is_empty() {
                    Vec::new()
                } else {
                    self.parse_inline(content)?
                };

                let new_item = ListItem {
                    content: inline_content,
                    children: Vec::new(),
                    checked,
                };

                // Truncate last_items to current indent level (we're going shallower or same)
                last_items.truncate(indent_level + 1);

                // Add the new item to the appropriate location
                if indent_level == 0 {
                    // Top-level item
                    let idx = items.len();
                    items.push(new_item);
                    if last_items.is_empty() {
                        last_items.push(Some(idx));
                    } else {
                        last_items[0] = Some(idx);
                    }
                    last_item_path = vec![(0, idx)];
                } else {
                    // Nested item: add to children of the last item at indent_level - 1
                    let parent_level = indent_level - 1;
                    if parent_level < last_items.len() {
                        if let Some(parent_idx) = last_items[parent_level] {
                            // Navigate to the parent item
                            let mut current = &mut items[parent_idx];
                            // Navigate through nested children to get to the right depth
                            for level in 1..indent_level {
                                if level < last_items.len() {
                                    if let Some(child_idx) = last_items[level] {
                                        if child_idx < current.children.len() {
                                            current = &mut current.children[child_idx];
                                        }
                                    }
                                }
                            }
                            // Add to current item's children
                            let child_idx = current.children.len();
                            current.children.push(new_item);
                            // Update last_items for this level
                            if indent_level >= last_items.len() {
                                last_items.resize(indent_level + 1, None);
                            }
                            last_items[indent_level] = Some(child_idx);
                            // Update path to track this new item
                            last_item_path.truncate(indent_level);
                            last_item_path.push((indent_level, child_idx));
                        } else {
                            // No parent found, add to top level as fallback
                            let idx = items.len();
                            items.push(new_item);
                            if last_items.is_empty() {
                                last_items.push(Some(idx));
                            } else {
                                last_items[0] = Some(idx);
                            }
                            last_item_path = vec![(0, idx)];
                        }
                    } else {
                        // Parent level doesn't exist, add to top level
                        let idx = items.len();
                        items.push(new_item);
                        if last_items.is_empty() {
                            last_items.push(Some(idx));
                        } else {
                            last_items[0] = Some(idx);
                        }
                        last_item_path = vec![(0, idx)];
                    }
                }

                i += 1;
            } else if let Some(_continuation_indent) = Self::detect_continuation_line(line) {
                // Continuation line - append to the most recently added item
                let continuation_text = line.trim();
                if !continuation_text.is_empty() && !last_item_path.is_empty() {
                    let continuation_inlines = self.parse_inline(continuation_text)?;

                    // Navigate to the item at last_item_path
                    let (first_level, first_idx) = last_item_path[0];
                    if first_level == 0 && first_idx < items.len() {
                        let mut current = &mut items[first_idx];
                        // Navigate through nested path
                        for (_level, idx) in &last_item_path[1..] {
                            if *idx < current.children.len() {
                                current = &mut current.children[*idx];
                            } else {
                                break;
                            }
                        }

                        // Append continuation to this item
                        if !current.content.is_empty() {
                            current.content.push(Inline::Text {
                                content: " ".to_string(),
                            });
                        }
                        current.content.extend(continuation_inlines);
                    } else if !items.is_empty() {
                        // Fallback: append to last top-level item
                        let item = items.last_mut().unwrap();
                        if !item.content.is_empty() {
                            item.content.push(Inline::Text {
                                content: " ".to_string(),
                            });
                        }
                        item.content.extend(continuation_inlines);
                    }
                }
                i += 1;
            } else {
                // Not a list line or continuation - end of list
                break;
            }
        }

        Ok((Node::UnorderedList { items }, i))
    }

    /// Check if a line is a table row (starts with | and contains at least one more |)
    fn detect_table_row(line: &str) -> bool {
        let trimmed = line.trim();
        trimmed.starts_with('|') && trimmed[1..].contains('|')
    }

    /// Check if a line is a table separator (matches pattern like |:---|, |---:|, |:---:|, or |---|)
    fn detect_table_separator(line: &str) -> bool {
        let trimmed = line.trim();
        if !trimmed.starts_with('|') {
            return false;
        }

        // Check if it matches the separator pattern: |:---|, |---:|, |:---:|, or |---|
        // The separator must have at least 3 dashes between pipes
        let parts: Vec<&str> = trimmed.split('|').collect();
        if parts.len() < 2 {
            return false;
        }

        // Check each cell separator (skip first and last empty parts)
        for part in parts.iter().skip(1) {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }
            // Must be all dashes with optional colons at start/end
            let has_colon_start = part.starts_with(':');
            let has_colon_end = part.ends_with(':');
            let dash_part = if has_colon_start && has_colon_end {
                &part[1..part.len() - 1]
            } else if has_colon_start {
                &part[1..]
            } else if has_colon_end {
                &part[..part.len() - 1]
            } else {
                part
            };

            // Must have at least 3 dashes
            if dash_part.len() < 3 || !dash_part.chars().all(|c| c == '-') {
                return false;
            }
        }

        true
    }

    /// Parse a table separator line and extract alignment information
    ///
    /// Returns a vector of alignment options (None = default/left, Some(Alignment) for explicit alignment)
    fn parse_table_separator(line: &str) -> Vec<Option<Alignment>> {
        let trimmed = line.trim();
        let parts: Vec<&str> = trimmed.split('|').collect();
        let mut alignments = Vec::new();

        // Skip first empty part (before first |) and process the rest
        for part in parts.iter().skip(1) {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }

            let has_colon_start = part.starts_with(':');
            let has_colon_end = part.ends_with(':');

            let alignment = if has_colon_start && has_colon_end {
                Some(Alignment::Center)
            } else if has_colon_end {
                Some(Alignment::Right)
            } else if has_colon_start {
                Some(Alignment::Left)
            } else {
                None // Default to left
            };

            alignments.push(alignment);
        }

        alignments
    }

    /// Parse a table row into cells, parsing inline content for each cell
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if inline parsing fails
    fn parse_table_row(&self, line: &str) -> Result<Vec<Vec<Inline>>, ParseError> {
        let trimmed = line.trim();
        let parts: Vec<&str> = trimmed.split('|').collect();
        let mut cells = Vec::new();

        // When splitting by '|', if line starts with '|', first part is empty
        // If line ends with '|', last part is empty
        // We want to process all parts between the pipes
        let start_idx = if !parts.is_empty() && parts[0].trim().is_empty() {
            1
        } else {
            0
        };
        let end_idx = if !parts.is_empty() && parts[parts.len() - 1].trim().is_empty() {
            parts.len() - 1
        } else {
            parts.len()
        };

        for part in &parts[start_idx..end_idx] {
            let cell_content = part.trim();
            let cell_inlines = if cell_content.is_empty() {
                Vec::new()
            } else {
                self.parse_inline(cell_content)?
            };
            cells.push(cell_inlines);
        }

        Ok(cells)
    }

    /// Parse a table starting at the given line index
    ///
    /// Returns the node and the new line index after the table.
    /// A table must have:
    /// 1. A header row (starts with |)
    /// 2. A separator row (matches separator pattern)
    /// 3. Zero or more data rows (each starts with |)
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if parsing fails
    fn parse_table(
        &self,
        lines: &[&str],
        start_idx: usize,
    ) -> Result<(Node, usize), ParseError> {
        let mut i = start_idx;

        // Parse header row
        if !Self::detect_table_row(lines[i]) {
            // Not a table - this shouldn't be called if not a table
            return Err(ParseError::MalformedMarkdown {
                message: "Expected table row".to_string(),
                span: Span {
                    line: i + 1,
                    column: None,
                },
            });
        }

        let headers = self.parse_table_row(lines[i])?;
        i += 1;

        // Parse separator row
        if i >= lines.len() || !Self::detect_table_separator(lines[i]) {
            return Err(ParseError::MalformedMarkdown {
                message: "Expected table separator row".to_string(),
                span: Span {
                    line: i + 1,
                    column: None,
                },
            });
        }

        let alignments = Self::parse_table_separator(lines[i]);
        i += 1;

        // Parse data rows until a non-table line is encountered
        let mut rows = Vec::new();
        while i < lines.len() {
            let line = lines[i].trim();

            // Stop at empty line or block elements
            if line.is_empty() {
                break;
            }
            if line.starts_with('#') || line.starts_with(&self.config.code_fence_pattern) {
                break;
            }

            // Stop at list lines
            if Self::detect_list_line(lines[i]).is_some() {
                break;
            }

            // Check if it's a table row
            if Self::detect_table_row(lines[i]) {
                let row = self.parse_table_row(lines[i])?;
                rows.push(row);
                i += 1;
            } else {
                // Not a table row, end of table
                break;
            }
        }

        Ok((
            Node::Table {
                headers,
                rows,
                alignments,
            },
            i,
        ))
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
            if current_line.starts_with('#') || current_line.starts_with(&self.config.code_fence_pattern) {
                break;
            }

            // Stop at list lines (list parsing happens before paragraph collection)
            if Self::detect_list_line(lines[i]).is_some() {
                break;
            }

            // Stop at table rows (table parsing happens before paragraph collection)
            if Self::detect_table_row(lines[i]) {
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
                let (node, new_idx) = self.parse_code_block(&lines, i)?;
                nodes.push(node);
                i = new_idx;
                continue;
            }

            // Check for headings (# syntax)
            let line_number = i + 1;
            if let Some(heading_node) = self.parse_heading(line, line_number)? {
                nodes.push(heading_node);
                i += 1;
                continue;
            }

            // Check for unordered lists (must check raw line, not trimmed, to detect indentation)
            if Self::detect_list_line(lines[i]).is_some() {
                let (list_node, new_idx) = self.parse_unordered_list(&lines, i)?;
                nodes.push(list_node);
                i = new_idx;
                continue;
            }

            // Check for tables (must check if current line is a table row and next line is separator)
            if Self::detect_table_row(lines[i]) {
                // Check if next line is a separator
                if i + 1 < lines.len() && Self::detect_table_separator(lines[i + 1]) {
                    let (table_node, new_idx) = self.parse_table(&lines, i)?;
                    nodes.push(table_node);
                    i = new_idx;
                    continue;
                }
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
