//! List parsing (unordered, ordered, task lists).

use crate::ast::{Inline, ListItem, Node, ParseError};

use super::inline::parse_inline;
use super::inline::RegexPatterns;

/// Check if a raw line (with indentation) matches the ordered list pattern
///
/// Returns Some((indent_level, number, content)) if it's an ordered list line, None otherwise.
/// Indent level is calculated as number of 2-space increments (0 = no indent, 1 = 2 spaces, etc.)
/// Pattern: one or more digits followed by `.` and a space
pub(super) fn detect_ordered_list_line(line: &str) -> Option<(usize, u32, &str)> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Find the position of the first digit
    let digit_start = line.find(|c: char| c.is_ascii_digit())?;

    // Find where the digits end
    let mut digit_end = digit_start;
    while digit_end < line.len() && line.as_bytes()[digit_end].is_ascii_digit() {
        digit_end += 1;
    }

    // Must be followed by `.` and a space
    if digit_end + 2 > line.len() {
        return None;
    }
    if line.as_bytes()[digit_end] != b'.' || line.as_bytes()[digit_end + 1] != b' ' {
        return None;
    }

    // Parse the number
    let number_str = &line[digit_start..digit_end];
    let number: u32 = number_str.parse().ok()?;

    // Calculate indent: count leading spaces, divide by 2 (round down)
    let leading_spaces = line[..digit_start]
        .chars()
        .take_while(|&c| c == ' ')
        .count();
    let indent_level = leading_spaces / 2;

    // Extract content after "number. "
    let content = line[digit_end + 2..].trim();
    Some((indent_level, number, content))
}

/// Check if a raw line (with indentation) matches the list pattern
///
/// Returns Some((indent_level, marker, content, checked)) if it's a list line, None otherwise.
/// Indent level is calculated as number of 2-space increments (0 = no indent, 1 = 2 spaces, etc.)
/// checked is Some(bool) for task list items, None for regular list items.
pub(super) fn detect_list_line(line: &str) -> Option<(usize, char, &str, Option<bool>)> {
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
pub(super) fn detect_continuation_line(line: &str) -> Option<usize> {
    if line.trim().is_empty() {
        return None;
    }

    // Must start with spaces (indented)
    let leading_spaces = line.chars().take_while(|&c| c == ' ').count();
    if leading_spaces == 0 {
        return None;
    }

    // Must NOT match list pattern (no marker)
    if detect_list_line(line).is_some() || detect_ordered_list_line(line).is_some() {
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
pub(super) fn parse_unordered_list(
    lines: &[&str],
    start_idx: usize,
    config: &crate::config::ParserConfig,
    regex_patterns: &RegexPatterns,
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
        if trimmed.starts_with('#') || trimmed.starts_with(&config.code_fence_pattern) {
            break;
        }

        // Check if it's a list line
        if let Some((indent_level, _marker, content, checked)) = detect_list_line(line) {
            // Parse the content as inline elements
            let inline_content = if content.is_empty() {
                Vec::new()
            } else {
                parse_inline(content, regex_patterns)?
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
        } else if let Some(_continuation_indent) = detect_continuation_line(line) {
            // Continuation line - append to the most recently added item
            let continuation_text = line.trim();
            if !continuation_text.is_empty() && !last_item_path.is_empty() {
                let continuation_inlines = parse_inline(continuation_text, regex_patterns)?;

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

/// Parse an ordered list starting at the given line index
///
/// Returns the node and the new line index after the list
pub(super) fn parse_ordered_list(
    lines: &[&str],
    start_idx: usize,
    config: &crate::config::ParserConfig,
    regex_patterns: &RegexPatterns,
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
        if trimmed.starts_with('#') || trimmed.starts_with(&config.code_fence_pattern) {
            break;
        }

        // Check if it's an ordered list line
        if let Some((indent_level, _number, content)) = detect_ordered_list_line(line) {
            // Parse the content as inline elements
            let inline_content = if content.is_empty() {
                Vec::new()
            } else {
                parse_inline(content, regex_patterns)?
            };

            let new_item = ListItem {
                content: inline_content,
                children: Vec::new(),
                checked: None, // Ordered lists don't support task lists
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
        } else if let Some(_continuation_indent) = detect_continuation_line(line) {
            // Continuation line - append to the most recently added item
            let continuation_text = line.trim();
            if !continuation_text.is_empty() && !last_item_path.is_empty() {
                let continuation_inlines = parse_inline(continuation_text, regex_patterns)?;

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
        } else if let Some((indent_level, _marker, content, checked)) = detect_list_line(line) {
            // Unordered list line - could be nested within ordered list
            // Parse the content as inline elements
            let inline_content = if content.is_empty() {
                Vec::new()
            } else {
                parse_inline(content, regex_patterns)?
            };

            let new_item = ListItem {
                content: inline_content,
                children: Vec::new(),
                checked,
            };

            // Truncate last_items to current indent level
            last_items.truncate(indent_level + 1);

            // Add the new item to the appropriate location
            if indent_level == 0 {
                // Top-level item - end the ordered list
                break;
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
                        // No parent found, end the ordered list
                        break;
                    }
                } else {
                    // Parent level doesn't exist, end the ordered list
                    break;
                }
            }

            i += 1;
        } else {
            // Not an ordered list line, continuation, or unordered list line - end of list
            break;
        }
    }

    Ok((Node::OrderedList { items }, i))
}
