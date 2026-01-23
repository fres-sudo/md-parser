//! Blockquote parsing.

use crate::ast::{Node, ParseError, Span};

use super::inline::parse_inline;
use super::inline::RegexPatterns;

/// Check if a line is a blockquote and return its nesting level
///
/// Returns `Some(level)` if the line starts with one or more `>` characters,
/// where level is the number of `>` characters (1 for `>`, 2 for `>>`, etc.).
/// Returns `None` if the line is not a blockquote.
pub(super) fn detect_blockquote_line(line: &str) -> Option<u8> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Count leading `>` characters
    let level = trimmed.chars().take_while(|&c| c == '>').count();
    if level > 0 {
        Some(level as u8)
    } else {
        None
    }
}

/// Collect blockquote lines starting at the given index
///
/// Returns the blockquote text (with `>` prefixes stripped) and the new line index after the blockquote.
/// Stops when encountering an empty line, a different nesting level, or other block elements.
pub(super) fn collect_blockquote_lines(
    lines: &[&str],
    start_idx: usize,
    config: &crate::config::ParserConfig,
) -> (String, usize) {
    let mut blockquote_lines = Vec::new();
    let mut i = start_idx;

    // Get the nesting level from the first line
    let nesting_level = match detect_blockquote_line(lines[i]) {
        Some(level) => level,
        None => return (String::new(), i), // Not a blockquote line
    };

    while i < lines.len() {
        let current_line = lines[i].trim();

        // Stop at empty line
        if current_line.is_empty() {
            break;
        }

        // Stop at other block elements
        if current_line.starts_with('#') || current_line.starts_with(&config.code_fence_pattern) {
            break;
        }

        // Stop at list lines
        if super::lists::detect_list_line(lines[i]).is_some()
            || super::lists::detect_ordered_list_line(lines[i]).is_some()
        {
            break;
        }

        // Stop at table rows
        if super::tables::detect_table_row(lines[i]) {
            break;
        }

        // Check if it's a blockquote line at the same nesting level
        if let Some(level) = detect_blockquote_line(lines[i]) {
            if level == nesting_level {
                // Strip the `>` prefix and optional space
                let content = lines[i]
                    .trim_start()
                    .chars()
                    .skip(level as usize)
                    .collect::<String>()
                    .trim_start()
                    .to_string();
                blockquote_lines.push(content);
                i += 1;
            } else {
                // Different nesting level, end this blockquote
                break;
            }
        } else {
            // Not a blockquote line, end the blockquote
            break;
        }
    }

    let blockquote_text = blockquote_lines.join(" ");
    (blockquote_text, i)
}

/// Parse a blockquote starting at the given line index
///
/// Returns the blockquote node and the new line index after the blockquote.
///
/// # Errors
///
/// Returns `ParseError` if inline parsing fails
pub(super) fn parse_blockquote(
    lines: &[&str],
    start_idx: usize,
    config: &crate::config::ParserConfig,
    regex_patterns: &RegexPatterns,
) -> Result<(Node, usize), ParseError> {
    // Detect nesting level from first line
    let level = match detect_blockquote_line(lines[start_idx]) {
        Some(l) => l,
        None => {
            return Err(ParseError::MalformedMarkdown {
                message: "Expected blockquote line".to_string(),
                span: Span {
                    line: start_idx + 1,
                    column: None,
                },
            });
        }
    };

    // Collect blockquote lines
    let (blockquote_text, new_idx) = collect_blockquote_lines(lines, start_idx, config);

    if blockquote_text.is_empty() {
        // Empty blockquote - skip it
        return Ok((
            Node::Blockquote {
                level,
                content: Vec::new(),
            },
            new_idx,
        ));
    }

    // Parse inline content
    let inline_content = parse_inline(&blockquote_text, regex_patterns)?;

    Ok((
        Node::Blockquote {
            level,
            content: inline_content,
        },
        new_idx,
    ))
}
