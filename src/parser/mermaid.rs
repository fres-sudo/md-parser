//! Mermaid diagram validator and configuration parser.

use crate::ast::{MermaidConfig, ValidationStatus};
use crate::config::MermaidParserConfig;
use regex::Regex;
use std::collections::HashMap;

/// Mermaid diagram validator and configuration parser
pub(super) struct MermaidValidator;

impl MermaidValidator {
    /// Parse frontmatter configuration from Mermaid diagram
    ///
    /// Extracts inline configuration from Mermaid frontmatter syntax:
    /// `%%{init: {'theme':'dark', 'themeVariables': {'fontSize':'18px'}}}%%`
    ///
    /// Returns (config, diagram_without_frontmatter)
    pub(super) fn parse_frontmatter(diagram: &str) -> (Option<MermaidConfig>, String) {
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

        // Extract themeVariables - search for fontSize/fontFamily in nested themeVariables object
        if init_content.contains("themeVariables") {
            // Use a regex that finds fontSize within themeVariables section
            // Pattern: 'fontSize' followed by colon and quoted value, anywhere after themeVariables
            if let Ok(re) = Regex::new(r"themeVariables[^}]*'fontSize'\s*:\s*'([^']+)'") {
                if let Some(caps) = re.captures(init_content) {
                    if let Some(m) = caps.get(1) {
                        let fs_val = m.as_str().to_string();
                        font_size = Some(fs_val.clone());
                        let mut tv_map = HashMap::new();
                        tv_map.insert("fontSize".to_string(), fs_val);
                        theme_variables = Some(tv_map);
                    }
                }
            }

            // Fallback: try the original approach with extract_object
            if font_size.is_none() {
                if let Some(tv_start) = init_content.find("themeVariables:") {
                    let tv_section_start = tv_start + "themeVariables:".len();
                    let tv_content = &init_content[tv_section_start..].trim();
                    let mut tv_map = HashMap::new();

                    // Try extracting the nested object
                    if let Some(tv_obj) = Self::extract_object(tv_content) {
                        if let Some(fs) = Self::extract_string_value(&tv_obj, "fontSize") {
                            font_size = Some(fs.clone());
                            tv_map.insert("fontSize".to_string(), fs);
                        }
                        if let Some(ff) = Self::extract_string_value(&tv_obj, "fontFamily") {
                            font_family = Some(ff.clone());
                            tv_map.insert("fontFamily".to_string(), ff);
                        }
                    }

                    // If extract_object failed, try extract_string_value on tv_content
                    if font_size.is_none() {
                        if let Some(fs) = Self::extract_string_value(tv_content, "fontSize") {
                            font_size = Some(fs.clone());
                            if tv_map.is_empty() {
                                tv_map.insert("fontSize".to_string(), fs);
                            }
                        }
                    }
                    if font_family.is_none() {
                        if let Some(ff) = Self::extract_string_value(tv_content, "fontFamily") {
                            font_family = Some(ff.clone());
                            if let Some(ref mut map) = theme_variables {
                                map.insert("fontFamily".to_string(), ff);
                            } else {
                                tv_map.insert("fontFamily".to_string(), ff);
                            }
                        }
                    }

                    if !tv_map.is_empty() {
                        theme_variables = Some(tv_map);
                    }
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
        let pattern_single = format!("'{}'\\s*:\\s*'([^']+)'", regex::escape(key));
        if let Ok(re) = Regex::new(&pattern_single) {
            if let Some(caps) = re.captures(content) {
                if let Some(m) = caps.get(1) {
                    return Some(m.as_str().to_string());
                }
            }
        }

        // Fall back to double quotes
        let pattern_double = format!("\"{}\"\\s*:\\s*\"([^\"]+)\"", regex::escape(key));
        if let Ok(re) = Regex::new(&pattern_double) {
            if let Some(caps) = re.captures(content) {
                if let Some(m) = caps.get(1) {
                    return Some(m.as_str().to_string());
                }
            }
        }

        // Try without quotes around key
        let pattern_no_quote_key =
            format!("{}['\"]?\\s*:\\s*['\"]([^'\"]+)['\"]", regex::escape(key));
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
    pub(super) fn merge_config(
        default: &MermaidParserConfig,
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
    pub(super) fn validate_syntax(diagram: &str, use_cli: bool) -> (ValidationStatus, Vec<String>) {
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
