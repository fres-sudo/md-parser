//! Inline element parsing (bold, italic, links, images, strikethrough).

use crate::ast::{Inline, ParseError};
use regex::{Regex, RegexSet};

/// Type of inline element match found during parsing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum InlineMatchType {
    Image,
    Link,
    Code,
    Strikethrough,
    Bold,
    Italic,
}

/// Compiled regex patterns for inline element parsing
pub(super) struct RegexPatterns {
    /// RegexSet for efficient multi-pattern matching
    set: RegexSet,
    /// Individual regexes for getting match positions and captures
    image: Regex,
    link: Regex,
    code: Regex,
    strikethrough: Regex,
    bold: Regex,
    italic: Regex,
}

impl RegexPatterns {
    /// Compile all regex patterns
    pub(super) fn new() -> Result<Self, ParseError> {
        // Pattern strings in order: image, link, code, strikethrough, bold, italic
        let pattern_strings = [
            r"!\[([^\]]*)\]\(([^)]+)\)",    // image
            r"\[([^\]]+)\]\(([^)]+)\)",     // link
            r"`([^`]+)`",                   // code - backticks with one or more non-backtick chars
            r"~~([^~]+?)~~",                // strikethrough
            r"\*\*((?:[^*]|\*[^*])+?)\*\*", // bold - allows * (for italic) but not ** inside
            r"\*((?:[^*]|\*\*)+)\*", // italic - allows ** (for bold) inside, greedy to match full span
        ];

        let set = RegexSet::new(pattern_strings).map_err(|e| {
            ParseError::RegexCompilationError(format!("RegexSet compilation: {}", e))
        })?;

        Ok(RegexPatterns {
            set,
            image: Regex::new(pattern_strings[0])
                .map_err(|e| ParseError::RegexCompilationError(format!("Image regex: {}", e)))?,
            link: Regex::new(pattern_strings[1])
                .map_err(|e| ParseError::RegexCompilationError(format!("Link regex: {}", e)))?,
            code: Regex::new(pattern_strings[2])
                .map_err(|e| ParseError::RegexCompilationError(format!("Code regex: {}", e)))?,
            strikethrough: Regex::new(pattern_strings[3]).map_err(|e| {
                ParseError::RegexCompilationError(format!("Strikethrough regex: {}", e))
            })?,
            bold: Regex::new(pattern_strings[4])
                .map_err(|e| ParseError::RegexCompilationError(format!("Bold regex: {}", e)))?,
            italic: Regex::new(pattern_strings[5])
                .map_err(|e| ParseError::RegexCompilationError(format!("Italic regex: {}", e)))?,
        })
    }

    /// Find the earliest match among all inline patterns
    pub(super) fn find_earliest_match(
        &self,
        text: &str,
    ) -> Option<(usize, usize, InlineMatchType)> {
        // Use RegexSet to quickly identify which patterns match
        let matches = self.set.matches(text);

        // If no patterns match, return early
        if !matches.matched_any() {
            return None;
        }

        let mut earliest_pos = text.len();
        let mut match_type = None;
        let mut match_range = (0, 0);

        // Check patterns in priority order: image (0), link (1), code (2), strikethrough (3), bold (4), italic (5)
        // Only check patterns that RegexSet identified as matching

        // Check for images (must check before links since images start with !)
        if matches.matched(0) {
            if let Some(m) = self.image.find(text) {
                if m.start() < earliest_pos {
                    earliest_pos = m.start();
                    match_type = Some(InlineMatchType::Image);
                    match_range = (m.start(), m.end());
                }
            }
        }

        // Check for links
        if matches.matched(1) {
            if let Some(m) = self.link.find(text) {
                if m.start() < earliest_pos {
                    earliest_pos = m.start();
                    match_type = Some(InlineMatchType::Link);
                    match_range = (m.start(), m.end());
                }
            }
        }

        // Check for code (must check before bold/italic to avoid conflicts)
        if matches.matched(2) {
            if let Some(m) = self.code.find(text) {
                if m.start() < earliest_pos {
                    earliest_pos = m.start();
                    match_type = Some(InlineMatchType::Code);
                    match_range = (m.start(), m.end());
                }
            }
        }

        // Check for strikethrough (must check before bold/italic to avoid conflicts)
        if matches.matched(3) {
            if let Some(m) = self.strikethrough.find(text) {
                if m.start() < earliest_pos {
                    earliest_pos = m.start();
                    match_type = Some(InlineMatchType::Strikethrough);
                    match_range = (m.start(), m.end());
                }
            }
        }

        // Check for bold (must check before italic to avoid conflicts)
        if matches.matched(4) {
            if let Some(m) = self.bold.find(text) {
                if m.start() < earliest_pos {
                    earliest_pos = m.start();
                    match_type = Some(InlineMatchType::Bold);
                    match_range = (m.start(), m.end());
                }
            }
        }

        // Check for italic (only if not part of bold - check that it's not **)
        if matches.matched(5) {
            if let Some(m) = self.italic.find(text) {
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
        }

        match_type.map(|mt| (match_range.0, match_range.1, mt))
    }

    /// Process an image match and add it to inlines
    pub(super) fn process_image_match<'a>(
        &self,
        remaining: &'a str,
        match_range: (usize, usize),
        inlines: &mut Vec<Inline>,
    ) -> Result<&'a str, ParseError> {
        // Add text before the image
        if match_range.0 > 0 {
            let text_before = &remaining[..match_range.0];
            if !text_before.is_empty() {
                inlines.push(Inline::Text {
                    content: text_before.to_string(),
                });
            }
        }

        let match_text = &remaining[match_range.0..match_range.1];
        let caps = self.image.captures(match_text).ok_or_else(|| {
            ParseError::InvalidCaptureError("Failed to capture image groups".to_string())
        })?;

        let alt_text = caps
            .get(1)
            .ok_or_else(|| {
                ParseError::InvalidCaptureError("Failed to capture image alt text".to_string())
            })?
            .as_str();
        let image_url = caps
            .get(2)
            .ok_or_else(|| {
                ParseError::InvalidCaptureError("Failed to capture image URL".to_string())
            })?
            .as_str();

        inlines.push(Inline::Image {
            alt: alt_text.to_string(),
            url: image_url.to_string(),
        });

        Ok(&remaining[match_range.1..])
    }

    /// Process a link match and add it to inlines
    pub(super) fn process_link_match<'a>(
        &self,
        remaining: &'a str,
        match_range: (usize, usize),
        inlines: &mut Vec<Inline>,
        parse_inline_fn: impl Fn(&str) -> Result<Vec<Inline>, ParseError>,
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
        let caps = self.link.captures(match_text).ok_or_else(|| {
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

        let text_inlines = parse_inline_fn(link_text)?;
        inlines.push(Inline::Link {
            text: text_inlines,
            url: link_url.to_string(),
        });

        Ok(&remaining[match_range.1..])
    }

    /// Process a bold match and add it to inlines
    pub(super) fn process_bold_match<'a>(
        &self,
        remaining: &'a str,
        match_range: (usize, usize),
        inlines: &mut Vec<Inline>,
        parse_inline_fn: impl Fn(&str) -> Result<Vec<Inline>, ParseError>,
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
        let caps = self.bold.captures(match_text).ok_or_else(|| {
            ParseError::InvalidCaptureError("Failed to capture bold groups".to_string())
        })?;

        let bold_text = caps
            .get(1)
            .ok_or_else(|| {
                ParseError::InvalidCaptureError("Failed to capture bold text".to_string())
            })?
            .as_str();

        let bold_inlines = parse_inline_fn(bold_text)?;
        inlines.push(Inline::Bold {
            content: bold_inlines,
        });

        Ok(&remaining[match_range.1..])
    }

    /// Process a strikethrough match and add it to inlines
    pub(super) fn process_strikethrough_match<'a>(
        &self,
        remaining: &'a str,
        match_range: (usize, usize),
        inlines: &mut Vec<Inline>,
        parse_inline_fn: impl Fn(&str) -> Result<Vec<Inline>, ParseError>,
    ) -> Result<&'a str, ParseError> {
        // Add text before the strikethrough
        if match_range.0 > 0 {
            let text_before = &remaining[..match_range.0];
            if !text_before.is_empty() {
                inlines.push(Inline::Text {
                    content: text_before.to_string(),
                });
            }
        }

        let match_text = &remaining[match_range.0..match_range.1];
        let caps = self.strikethrough.captures(match_text).ok_or_else(|| {
            ParseError::InvalidCaptureError("Failed to capture strikethrough groups".to_string())
        })?;

        let strikethrough_text = caps
            .get(1)
            .ok_or_else(|| {
                ParseError::InvalidCaptureError("Failed to capture strikethrough text".to_string())
            })?
            .as_str();

        let strikethrough_inlines = parse_inline_fn(strikethrough_text)?;
        inlines.push(Inline::Strikethrough {
            content: strikethrough_inlines,
        });

        Ok(&remaining[match_range.1..])
    }

    /// Process an italic match and add it to inlines
    pub(super) fn process_italic_match<'a>(
        &self,
        remaining: &'a str,
        match_range: (usize, usize),
        inlines: &mut Vec<Inline>,
        parse_inline_fn: impl Fn(&str) -> Result<Vec<Inline>, ParseError>,
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
        let caps = self.italic.captures(match_text).ok_or_else(|| {
            ParseError::InvalidCaptureError("Failed to capture italic groups".to_string())
        })?;

        let italic_text = caps
            .get(1)
            .ok_or_else(|| {
                ParseError::InvalidCaptureError("Failed to capture italic text".to_string())
            })?
            .as_str();

        let italic_inlines = parse_inline_fn(italic_text)?;
        inlines.push(Inline::Italic {
            content: italic_inlines,
        });

        Ok(&remaining[match_range.1..])
    }

    /// Process a code match and add it to inlines
    pub(super) fn process_code_match<'a>(
        &self,
        remaining: &'a str,
        match_range: (usize, usize),
        inlines: &mut Vec<Inline>,
    ) -> Result<&'a str, ParseError> {
        // Add text before the code
        if match_range.0 > 0 {
            let text_before = &remaining[..match_range.0];
            if !text_before.is_empty() {
                inlines.push(Inline::Text {
                    content: text_before.to_string(),
                });
            }
        }

        let match_text = &remaining[match_range.0..match_range.1];
        let caps = self.code.captures(match_text).ok_or_else(|| {
            ParseError::InvalidCaptureError("Failed to capture code groups".to_string())
        })?;

        let code_content = caps
            .get(1)
            .ok_or_else(|| {
                ParseError::InvalidCaptureError("Failed to capture code content".to_string())
            })?
            .as_str();

        // Code content is stored as plain text (no recursive parsing)
        inlines.push(Inline::Code {
            content: code_content.to_string(),
        });

        Ok(&remaining[match_range.1..])
    }
}

/// Parse inline elements from a text string
pub(super) fn parse_inline(
    text: &str,
    regex_patterns: &RegexPatterns,
) -> Result<Vec<Inline>, ParseError> {
    let mut inlines = Vec::new();
    let mut remaining = text;

    while !remaining.is_empty() {
        if let Some((start, end, match_type)) = regex_patterns.find_earliest_match(remaining) {
            let match_range = (start, end);
            remaining = match match_type {
                InlineMatchType::Image => {
                    regex_patterns.process_image_match(remaining, match_range, &mut inlines)?
                }
                InlineMatchType::Link => regex_patterns.process_link_match(
                    remaining,
                    match_range,
                    &mut inlines,
                    |t| parse_inline(t, regex_patterns),
                )?,
                InlineMatchType::Code => {
                    regex_patterns.process_code_match(remaining, match_range, &mut inlines)?
                }
                InlineMatchType::Strikethrough => regex_patterns.process_strikethrough_match(
                    remaining,
                    match_range,
                    &mut inlines,
                    |t| parse_inline(t, regex_patterns),
                )?,
                InlineMatchType::Bold => regex_patterns.process_bold_match(
                    remaining,
                    match_range,
                    &mut inlines,
                    |t| parse_inline(t, regex_patterns),
                )?,
                InlineMatchType::Italic => regex_patterns.process_italic_match(
                    remaining,
                    match_range,
                    &mut inlines,
                    |t| parse_inline(t, regex_patterns),
                )?,
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
