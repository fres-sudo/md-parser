//! Table parsing.

use crate::ast::{Alignment, Inline, Node, ParseError, Span};

use super::inline::parse_inline;
use super::inline::RegexPatterns;

/// Check if a line is a table row (starts with | and contains at least one more |)
pub(super) fn detect_table_row(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with('|') && trimmed[1..].contains('|')
}

/// Check if a line is a table separator (matches pattern like |:---|, |---:|, |:---:|, or |---|)
pub(super) fn detect_table_separator(line: &str) -> bool {
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
pub(super) fn parse_table_separator(line: &str) -> Vec<Option<Alignment>> {
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
pub(super) fn parse_table_row(
    line: &str,
    regex_patterns: &RegexPatterns,
) -> Result<Vec<Vec<Inline>>, ParseError> {
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
            parse_inline(cell_content, regex_patterns)?
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
pub(super) fn parse_table(
    lines: &[&str],
    start_idx: usize,
    config: &crate::config::ParserConfig,
    regex_patterns: &RegexPatterns,
) -> Result<(Node, usize), ParseError> {
    let mut i = start_idx;

    // Parse header row
    if !detect_table_row(lines[i]) {
        // Not a table - this shouldn't be called if not a table
        return Err(ParseError::MalformedMarkdown {
            message: "Expected table row".to_string(),
            span: Span {
                line: i + 1,
                column: None,
            },
        });
    }

    let headers = parse_table_row(lines[i], regex_patterns)?;
    i += 1;

    // Parse separator row
    if i >= lines.len() || !detect_table_separator(lines[i]) {
        return Err(ParseError::MalformedMarkdown {
            message: "Expected table separator row".to_string(),
            span: Span {
                line: i + 1,
                column: None,
            },
        });
    }

    let alignments = parse_table_separator(lines[i]);
    i += 1;

    // Parse data rows until a non-table line is encountered
    let mut rows = Vec::new();
    while i < lines.len() {
        let line = lines[i].trim();

        // Stop at empty line or block elements
        if line.is_empty() {
            break;
        }
        if line.starts_with('#') || line.starts_with(&config.code_fence_pattern) {
            break;
        }

        // Stop at list lines
        if super::lists::detect_list_line(lines[i]).is_some()
            || super::lists::detect_ordered_list_line(lines[i]).is_some()
        {
            break;
        }

        // Check if it's a table row
        if detect_table_row(lines[i]) {
            let row = parse_table_row(lines[i], regex_patterns)?;
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
