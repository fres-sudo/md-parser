//! Markdown parsing logic.

use crate::ast::{
    Alignment, Inline, ListItem, MermaidConfig, Node, ParseError, Span, ValidationStatus,
};
use crate::config::ParserConfig;
use regex::{Regex, RegexSet};
use std::collections::HashMap;

/// Type of inline element match found during parsing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InlineMatchType {
    Image,
    Link,
    Strikethrough,
    Bold,
    Italic,
}

/// Compiled regex patterns for inline element parsing
struct RegexPatterns {
    /// RegexSet for efficient multi-pattern matching
    set: RegexSet,
    /// Individual regexes for getting match positions and captures
    image: Regex,
    link: Regex,
    strikethrough: Regex,
    bold: Regex,
    italic: Regex,
}

impl RegexPatterns {
    /// Compile all regex patterns
    fn new() -> Result<Self, ParseError> {
        // Pattern strings in order: image, link, strikethrough, bold, italic
        let pattern_strings = [
            r"!\[([^\]]*)\]\(([^)]+)\)",      // image
            r"\[([^\]]+)\]\(([^)]+)\)",      // link
            r"~~([^~]+?)~~",                  // strikethrough
            r"\*\*((?:[^*]|\*[^*\n])+?)\*\*", // bold
            r"\*([^*\n]+?)\*",               // italic
        ];

        let set = RegexSet::new(&pattern_strings)
            .map_err(|e| ParseError::RegexCompilationError(format!("RegexSet compilation: {}", e)))?;

        Ok(RegexPatterns {
            set,
            image: Regex::new(pattern_strings[0])
                .map_err(|e| ParseError::RegexCompilationError(format!("Image regex: {}", e)))?,
            link: Regex::new(pattern_strings[1])
                .map_err(|e| ParseError::RegexCompilationError(format!("Link regex: {}", e)))?,
            strikethrough: Regex::new(pattern_strings[2])
                .map_err(|e| ParseError::RegexCompilationError(format!("Strikethrough regex: {}", e)))?,
            bold: Regex::new(pattern_strings[3])
                .map_err(|e| ParseError::RegexCompilationError(format!("Bold regex: {}", e)))?,
            italic: Regex::new(pattern_strings[4])
                .map_err(|e| ParseError::RegexCompilationError(format!("Italic regex: {}", e)))?,
        })
    }
}

/// Mermaid diagram validator and configuration parser
struct MermaidValidator;

impl MermaidValidator {
    /// Parse frontmatter configuration from Mermaid diagram
    ///
    /// Extracts inline configuration from Mermaid frontmatter syntax:
    /// `%%{init: {'theme':'dark', 'themeVariables': {'fontSize':'18px'}}}%%`
    ///
    /// Returns (config, diagram_without_frontmatter)
    fn parse_frontmatter(diagram: &str) -> (Option<MermaidConfig>, String) {
        // Look for frontmatter pattern: %%{init: {...}}%%
        // Frontmatter can be on first line or second line
        let lines: Vec<&str> = diagram.lines().collect();
        let mut frontmatter_line_idx = None;
        let mut frontmatter_content = None;

        // Check first two lines for frontmatter
        for (idx, line) in lines.iter().take(2).enumerate() {
            let trimmed_line = line.trim();
            if trimmed_line.starts_with("%%{") {
                if let Some(end_pos) = trimmed_line.find("}%%") {
                    // end_pos is the start of "}%%", so we need +3 to include all 3 chars
                    let frontmatter = &trimmed_line[..end_pos + 3];
                    if let Some(config) = Self::parse_frontmatter_config(frontmatter) {
                        frontmatter_line_idx = Some(idx);
                        frontmatter_content = Some(config);
                        break;
                    }
                }
            }
        }

        if let (Some(idx), Some(config)) = (frontmatter_line_idx, frontmatter_content) {
            // Remove the frontmatter line from diagram
            let mut diagram_lines = lines;
            diagram_lines.remove(idx);
            let diagram_content = diagram_lines.join("\n").trim().to_string();
            (Some(config), diagram_content)
        } else {
            (None, diagram.to_string())
        }
    }

    /// Parse frontmatter config from string like `%%{init: {'theme':'dark'}}%%`
    fn parse_frontmatter_config(frontmatter: &str) -> Option<MermaidConfig> {
        // Remove %%{ and }%%
        let content = frontmatter.strip_prefix("%%{")?.strip_suffix("}%%")?;

        // Look for init: {...}
        if !content.trim_start().starts_with("init:") {
            return None;
        }

        let init_content = content.trim_start().strip_prefix("init:")?.trim();

        // Basic parsing of the config object
        // This is a simplified parser - in production you might want a proper JSON parser
        // For now, we'll extract common fields using string matching
        let mut theme = None;
        let mut font_size = None;
        let mut font_family = None;
        let mut theme_variables = None;

        // Extract theme
        if let Some(theme_match) = Self::extract_string_value(init_content, "theme") {
            theme = Some(theme_match);
        }

        // Extract themeVariables
        if let Some(tv_start) = init_content.find("themeVariables:") {
            let tv_content = &init_content[tv_start + "themeVariables:".len()..].trim();
            if let Some(tv_obj) = Self::extract_object(tv_content) {
                let mut tv_map = HashMap::new();

                // Extract fontSize - try both with and without quotes in the extracted object
                if let Some(fs) = Self::extract_string_value(&tv_obj, "fontSize") {
                    font_size = Some(fs.clone());
                    tv_map.insert("fontSize".to_string(), fs);
                } else {
                    // Try extracting directly from tv_obj if it's just the value
                    // This handles cases where the object structure is different
                    let fs_pattern = Regex::new(r"'fontSize'\s*:\s*'([^']+)'").ok();
                    if let Some(re) = fs_pattern {
                        if let Some(caps) = re.captures(&tv_obj) {
                            if let Some(m) = caps.get(1) {
                                let fs_val = m.as_str().to_string();
                                font_size = Some(fs_val.clone());
                                tv_map.insert("fontSize".to_string(), fs_val);
                            }
                        }
                    }
                }

                // Extract fontFamily
                if let Some(ff) = Self::extract_string_value(&tv_obj, "fontFamily") {
                    font_family = Some(ff.clone());
                    tv_map.insert("fontFamily".to_string(), ff);
                }

                if !tv_map.is_empty() {
                    theme_variables = Some(tv_map);
                }
            }
        }

        // Only return config if we found something
        if theme.is_some()
            || font_size.is_some()
            || font_family.is_some()
            || theme_variables.is_some()
        {
            Some(MermaidConfig {
                theme,
                font_size,
                font_family,
                theme_variables,
            })
        } else {
            None
        }
    }

    /// Extract a string value from a config-like string
    /// Looks for patterns like 'key':'value' or "key":"value"
    fn extract_string_value(content: &str, key: &str) -> Option<String> {
        // Try with single quotes first (more common in Mermaid config)
        let pattern_single = format!(
            "'{}'\\s*:\\s*'([^']+)'",
            regex::escape(key)
        );
        if let Ok(re) = Regex::new(&pattern_single) {
            if let Some(caps) = re.captures(content) {
                if let Some(m) = caps.get(1) {
                    return Some(m.as_str().to_string());
                }
            }
        }

        // Fall back to double quotes
        let pattern_double = format!(
            "\"{}\"\\s*:\\s*\"([^\"]+)\"",
            regex::escape(key)
        );
        if let Ok(re) = Regex::new(&pattern_double) {
            if let Some(caps) = re.captures(content) {
                if let Some(m) = caps.get(1) {
                    return Some(m.as_str().to_string());
                }
            }
        }

        // Try without quotes around key
        let pattern_no_quote_key = format!(
            "{}['\"]?\\s*:\\s*['\"]([^'\"]+)['\"]",
            regex::escape(key)
        );
        if let Ok(re) = Regex::new(&pattern_no_quote_key) {
            if let Some(caps) = re.captures(content) {
                if let Some(m) = caps.get(1) {
                    return Some(m.as_str().to_string());
                }
            }
        }

        None
    }

    /// Extract an object from a string (simplified - finds content between {})
    fn extract_object(content: &str) -> Option<String> {
        let trimmed = content.trim();
        if !trimmed.starts_with('{') {
            return None;
        }

        let mut depth = 0;
        let mut start = 0;
        for (i, ch) in trimmed.char_indices() {
            if ch == '{' {
                if depth == 0 {
                    start = i + 1;
                }
                depth += 1;
            } else if ch == '}' {
                depth -= 1;
                if depth == 0 {
                    return Some(trimmed[start..i].to_string());
                }
            }
        }
        None
    }

    /// Merge global default config with inline config
    fn merge_config(
        default: &crate::config::MermaidParserConfig,
        inline: Option<MermaidConfig>,
    ) -> MermaidConfig {
        if let Some(inline_config) = inline {
            MermaidConfig {
                theme: inline_config
                    .theme
                    .or_else(|| Some(default.default_theme.clone())),
                font_size: inline_config
                    .font_size
                    .or_else(|| Some(default.default_font_size.clone())),
                font_family: inline_config
                    .font_family
                    .or_else(|| Some(default.default_font_family.clone())),
                theme_variables: inline_config.theme_variables,
            }
        } else {
            MermaidConfig {
                theme: Some(default.default_theme.clone()),
                font_size: Some(default.default_font_size.clone()),
                font_family: Some(default.default_font_family.clone()),
                theme_variables: None,
            }
        }
    }

    /// Validate Mermaid diagram syntax
    ///
    /// Returns validation status and warnings
    fn validate_syntax(diagram: &str, use_cli: bool) -> (ValidationStatus, Vec<String>) {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        let trimmed = diagram.trim();
        if trimmed.is_empty() {
            errors.push("Mermaid diagram is empty".to_string());
            return (ValidationStatus::Invalid { errors }, warnings);
        }

        // Check for valid diagram type keywords
        let valid_types = [
            "graph",
            "flowchart",
            "sequenceDiagram",
            "classDiagram",
            "stateDiagram",
            "stateDiagram-v2",
            "erDiagram",
            "journey",
            "gantt",
            "pie",
            "requirementDiagram",
            "gitgraph",
            "mindmap",
            "timeline",
            "C4Context",
            "C4Container",
            "C4Component",
        ];

        let first_line = trimmed.lines().next().unwrap_or("").trim();
        let mut found_type = false;
        for diagram_type in &valid_types {
            if first_line.starts_with(diagram_type) {
                found_type = true;
                break;
            }
        }

        if !found_type {
            errors.push(format!(
                "Invalid or missing diagram type. Expected one of: {}",
                valid_types.join(", ")
            ));
        }

        // Check bracket/parenthesis balance
        let mut paren_count = 0;
        let mut bracket_count = 0;
        let mut brace_count = 0;

        for ch in trimmed.chars() {
            match ch {
                '(' => paren_count += 1,
                ')' => {
                    paren_count -= 1;
                    if paren_count < 0 {
                        errors.push("Unmatched closing parenthesis".to_string());
                        break;
                    }
                }
                '[' => bracket_count += 1,
                ']' => {
                    bracket_count -= 1;
                    if bracket_count < 0 {
                        errors.push("Unmatched closing bracket".to_string());
                        break;
                    }
                }
                '{' => brace_count += 1,
                '}' => {
                    brace_count -= 1;
                    if brace_count < 0 {
                        errors.push("Unmatched closing brace".to_string());
                        break;
                    }
                }
                _ => {}
            }
        }

        if paren_count > 0 {
            errors.push(format!("{} unmatched opening parenthesis(es)", paren_count));
        }
        if bracket_count > 0 {
            errors.push(format!("{} unmatched opening bracket(s)", bracket_count));
        }
        if brace_count > 0 {
            errors.push(format!("{} unmatched opening brace(s)", brace_count));
        }

        // Check for common arrow syntax issues
        if trimmed.contains("-->") || trimmed.contains("---") || trimmed.contains("==>") {
            // Basic check - arrows should have nodes on both sides
            let arrow_pattern = Regex::new(r"(-->|==>|---)").ok();
            if let Some(re) = arrow_pattern {
                for mat in re.find_iter(trimmed) {
                    let before = &trimmed[..mat.start()].trim();
                    let after = &trimmed[mat.end()..].trim();

                    if before.is_empty() || after.is_empty() {
                        warnings.push("Arrow may be missing node on one side".to_string());
                    }
                }
            }
        }

        // Optional CLI validation
        if use_cli {
            if let Some(cli_errors) = Self::validate_with_cli(trimmed) {
                errors.extend(cli_errors);
            } else {
                warnings.push("Mermaid CLI not available, using basic validation only".to_string());
            }
        }

        if errors.is_empty() {
            (ValidationStatus::Valid, warnings)
        } else {
            (ValidationStatus::Invalid { errors }, warnings)
        }
    }

    /// Attempt to validate using Mermaid CLI (if available)
    fn validate_with_cli(diagram: &str) -> Option<Vec<String>> {
        use std::fs;
        use std::process::Command;

        // Check if mmdc is available
        if Command::new("mmdc").arg("--version").output().is_err() {
            return None;
        }

        // Create a temporary file
        let temp_dir = std::env::temp_dir();
        let input_file = temp_dir.join(format!(
            "mermaid_validate_{}.mmd",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        ));
        let output_file = temp_dir.join(format!(
            "mermaid_validate_{}.svg",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        ));

        // Write diagram to temp file
        if fs::write(&input_file, diagram).is_err() {
            return None;
        }

        // Try to render with mmdc
        let output = Command::new("mmdc")
            .arg("-i")
            .arg(&input_file)
            .arg("-o")
            .arg(&output_file)
            .output();

        // Clean up temp files
        let _ = fs::remove_file(&input_file);
        let _ = fs::remove_file(&output_file);

        if let Ok(result) = output {
            if !result.status.success() {
                let stderr = String::from_utf8_lossy(&result.stderr);
                return Some(vec![format!("Mermaid CLI validation failed: {}", stderr)]);
            }
        }

        None
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
        // Use RegexSet to quickly identify which patterns match
        let matches = self.regex_patterns.set.matches(text);

        // If no patterns match, return early
        if !matches.matched_any() {
            return None;
        }

        let mut earliest_pos = text.len();
        let mut match_type = None;
        let mut match_range = (0, 0);

        // Check patterns in priority order: image (0), link (1), strikethrough (2), bold (3), italic (4)
        // Only check patterns that RegexSet identified as matching

        // Check for images (must check before links since images start with !)
        if matches.matched(0) {
            if let Some(m) = self.regex_patterns.image.find(text) {
                if m.start() < earliest_pos {
                    earliest_pos = m.start();
                    match_type = Some(InlineMatchType::Image);
                    match_range = (m.start(), m.end());
                }
            }
        }

        // Check for links
        if matches.matched(1) {
            if let Some(m) = self.regex_patterns.link.find(text) {
                if m.start() < earliest_pos {
                    earliest_pos = m.start();
                    match_type = Some(InlineMatchType::Link);
                    match_range = (m.start(), m.end());
                }
            }
        }

        // Check for strikethrough (must check before bold/italic to avoid conflicts)
        if matches.matched(2) {
            if let Some(m) = self.regex_patterns.strikethrough.find(text) {
                if m.start() < earliest_pos {
                    earliest_pos = m.start();
                    match_type = Some(InlineMatchType::Strikethrough);
                    match_range = (m.start(), m.end());
                }
            }
        }

        // Check for bold (must check before italic to avoid conflicts)
        if matches.matched(3) {
            if let Some(m) = self.regex_patterns.bold.find(text) {
                if m.start() < earliest_pos {
                    earliest_pos = m.start();
                    match_type = Some(InlineMatchType::Bold);
                    match_range = (m.start(), m.end());
                }
            }
        }

        // Check for italic (only if not part of bold - check that it's not **)
        if matches.matched(4) {
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
        }

        match_type.map(|mt| (match_range.0, match_range.1, mt))
    }

    /// Process an image match and add it to inlines
    fn process_image_match<'a>(
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
        let caps = self
            .regex_patterns
            .image
            .captures(match_text)
            .ok_or_else(|| {
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

    /// Process a strikethrough match and add it to inlines
    fn process_strikethrough_match<'a>(
        &self,
        remaining: &'a str,
        match_range: (usize, usize),
        inlines: &mut Vec<Inline>,
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
        let caps = self
            .regex_patterns
            .strikethrough
            .captures(match_text)
            .ok_or_else(|| {
                ParseError::InvalidCaptureError("Failed to capture strikethrough groups".to_string())
            })?;

        let strikethrough_text = caps
            .get(1)
            .ok_or_else(|| {
                ParseError::InvalidCaptureError("Failed to capture strikethrough text".to_string())
            })?
            .as_str();

        let strikethrough_inlines = self.parse_inline(strikethrough_text)?;
        inlines.push(Inline::Strikethrough {
            content: strikethrough_inlines,
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
                    InlineMatchType::Image => {
                        self.process_image_match(remaining, match_range, &mut inlines)?
                    }
                    InlineMatchType::Link => {
                        self.process_link_match(remaining, match_range, &mut inlines)?
                    }
                    InlineMatchType::Strikethrough => {
                        self.process_strikethrough_match(remaining, match_range, &mut inlines)?
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
    /// Returns the node, the new line index after the code block, and any warnings.
    /// Errors with `UnclosedCodeBlock` if no closing fence is found before EOF.
    fn parse_code_block(
        &self,
        lines: &[&str],
        start_idx: usize,
    ) -> Result<(Node, usize, Vec<String>), ParseError> {
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
        if lang.as_ref().map(|s| s.to_lowercase())
            == Some(self.config.mermaid_language.to_lowercase())
        {
            // Parse frontmatter and extract configuration
            let (inline_config, diagram_content) = MermaidValidator::parse_frontmatter(&code);

            // Merge global and inline configuration
            let merged_config = MermaidValidator::merge_config(&self.config.mermaid, inline_config);

            // Validate syntax if enabled
            let (validation_status, validation_warnings) = if self.config.mermaid.validate_syntax {
                MermaidValidator::validate_syntax(
                    &diagram_content,
                    self.config.mermaid.use_cli_validation,
                )
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

    /// Check if a line is a blockquote and return its nesting level
    ///
    /// Returns `Some(level)` if the line starts with one or more `>` characters,
    /// where level is the number of `>` characters (1 for `>`, 2 for `>>`, etc.).
    /// Returns `None` if the line is not a blockquote.
    fn detect_blockquote_line(line: &str) -> Option<u8> {
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
    fn collect_blockquote_lines(&self, lines: &[&str], start_idx: usize) -> (String, usize) {
        let mut blockquote_lines = Vec::new();
        let mut i = start_idx;

        // Get the nesting level from the first line
        let nesting_level = match Self::detect_blockquote_line(lines[i]) {
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
            if current_line.starts_with('#')
                || current_line.starts_with(&self.config.code_fence_pattern)
            {
                break;
            }

            // Stop at list lines
            if Self::detect_list_line(lines[i]).is_some() {
                break;
            }

            // Stop at table rows
            if Self::detect_table_row(lines[i]) {
                break;
            }

            // Check if it's a blockquote line at the same nesting level
            if let Some(level) = Self::detect_blockquote_line(lines[i]) {
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
    fn parse_blockquote(&self, lines: &[&str], start_idx: usize) -> Result<(Node, usize), ParseError> {
        // Detect nesting level from first line
        let level = match Self::detect_blockquote_line(lines[start_idx]) {
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
        let (blockquote_text, new_idx) = self.collect_blockquote_lines(lines, start_idx);

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
        let inline_content = self.parse_inline(&blockquote_text)?;

        Ok((
            Node::Blockquote {
                level,
                content: inline_content,
            },
            new_idx,
        ))
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
    fn parse_table(&self, lines: &[&str], start_idx: usize) -> Result<(Node, usize), ParseError> {
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
            if current_line.starts_with('#')
                || current_line.starts_with(&self.config.code_fence_pattern)
            {
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

            // Stop at blockquote lines (blockquote parsing happens before paragraph collection)
            if Self::detect_blockquote_line(lines[i]).is_some() {
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
                let (node, new_idx, warnings) = self.parse_code_block(&lines, i)?;
                self.warnings.extend(warnings);
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

            // Check for blockquotes
            if Self::detect_blockquote_line(lines[i]).is_some() {
                let (blockquote_node, new_idx) = self.parse_blockquote(&lines, i)?;
                nodes.push(blockquote_node);
                i = new_idx;
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
