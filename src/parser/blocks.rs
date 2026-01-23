//! Block-level element parsing (code blocks, headings, paragraphs).

use crate::ast::{Node, ParseError, Span, ValidationStatus};
use crate::config::ParserConfig;

use super::inline::parse_inline;
use super::inline::RegexPatterns;
use super::mermaid::MermaidValidator;

/// Parse a fenced code block starting at the given line index
///
/// Returns the node, the new line index after the code block, and any warnings.
/// Errors with `UnclosedCodeBlock` if no closing fence is found before EOF.
pub(super) fn parse_code_block(
    lines: &[&str],
    start_idx: usize,
    config: &ParserConfig,
    _regex_patterns: &RegexPatterns,
) -> Result<(Node, usize, Vec<String>), ParseError> {
    let line = lines[start_idx].trim();
    let lang_tag = line[config.code_fence_length..].trim();
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
        if lines[i].trim() == config.code_fence_pattern {
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
    if lang.as_ref().map(|s| s.to_lowercase()) == Some(config.mermaid_language.to_lowercase()) {
        // Parse frontmatter and extract configuration
        let (inline_config, diagram_content) = MermaidValidator::parse_frontmatter(&code);

        // Merge global and inline configuration
        let merged_config = MermaidValidator::merge_config(&config.mermaid, inline_config);

        // Validate syntax if enabled
        let (validation_status, validation_warnings) = if config.mermaid.validate_syntax {
            MermaidValidator::validate_syntax(&diagram_content, config.mermaid.use_cli_validation)
        } else {
            (ValidationStatus::NotValidated, Vec::new())
        };

        // Collect warnings to return
        let mut warnings = Vec::new();
        for warning in &validation_warnings {
            warnings.push(format!("Mermaid diagram validation warning: {}", warning));
        }

        // Add validation errors to warnings (but keep as MermaidDiagram as requested)
        if let ValidationStatus::Invalid { ref errors } = validation_status {
            for error in errors {
                warnings.push(format!("Mermaid diagram validation error: {}", error));
            }
        }

        let node = Node::MermaidDiagram {
            diagram: diagram_content,
            config: Some(merged_config),
            validation_status,
            warnings: validation_warnings,
        };

        Ok((node, i + 1, warnings))
    } else {
        Ok((Node::CodeBlock { lang, code }, i + 1, Vec::new()))
    }
}

/// Parse a heading from a line
///
/// Returns `Some(node)` if a valid heading is found, `None` if not a heading.
/// Errors with `InvalidHeadingLevel` if the line has more than 6 leading `#`.
pub(super) fn parse_heading(
    line: &str,
    line_number: usize,
    config: &ParserConfig,
    regex_patterns: &RegexPatterns,
) -> Result<Option<Node>, ParseError> {
    if !line.starts_with('#') {
        return Ok(None);
    }

    let level = line.chars().take_while(|&c| c == '#').count();
    if level > config.max_heading_level as usize {
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
            let inline_content = parse_inline(content, regex_patterns)?;
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
pub(super) fn collect_paragraph_lines(
    lines: &[&str],
    start_idx: usize,
    config: &ParserConfig,
) -> (String, usize) {
    let mut para_lines = Vec::new();
    let mut i = start_idx;

    while i < lines.len() {
        let current_line = lines[i].trim();

        // Stop at empty line or block elements
        if current_line.is_empty() {
            break;
        }
        if current_line.starts_with('#') || current_line.starts_with(&config.code_fence_pattern) {
            break;
        }

        // Stop at list lines (list parsing happens before paragraph collection)
        if super::lists::detect_list_line(lines[i]).is_some()
            || super::lists::detect_ordered_list_line(lines[i]).is_some()
        {
            break;
        }

        // Stop at table rows (table parsing happens before paragraph collection)
        if super::tables::detect_table_row(lines[i]) {
            break;
        }

        // Stop at blockquote lines (blockquote parsing happens before paragraph collection)
        if super::blockquotes::detect_blockquote_line(lines[i]).is_some() {
            break;
        }

        para_lines.push(current_line);
        i += 1;
    }

    let para_text = para_lines.join(" ");
    (para_text, i)
}
