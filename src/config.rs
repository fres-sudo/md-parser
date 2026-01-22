//! Configuration management for the Markdown parser.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Configuration for Mermaid diagram parser settings
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MermaidParserConfig {
    /// Default theme (default, neutral, dark, forest, base)
    #[serde(default = "default_mermaid_theme")]
    pub default_theme: String,
    /// Default font size (e.g., "16px")
    #[serde(default = "default_mermaid_font_size")]
    pub default_font_size: String,
    /// Default font family
    #[serde(default = "default_mermaid_font_family")]
    pub default_font_family: String,
    /// Enable syntax validation
    #[serde(default = "default_true")]
    pub validate_syntax: bool,
    /// Use Mermaid CLI for validation if available (optional)
    #[serde(default = "default_false")]
    pub use_cli_validation: bool,
}

fn default_mermaid_theme() -> String {
    "default".to_string()
}

fn default_mermaid_font_size() -> String {
    "16px".to_string()
}

fn default_mermaid_font_family() -> String {
    "trebuchet ms, verdana, arial".to_string()
}

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

impl Default for MermaidParserConfig {
    fn default() -> Self {
        Self {
            default_theme: default_mermaid_theme(),
            default_font_size: default_mermaid_font_size(),
            default_font_family: default_mermaid_font_family(),
            validate_syntax: true,
            use_cli_validation: false,
        }
    }
}

/// Configuration for the parser settings
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ParserConfig {
    /// Maximum heading level supported (1-6)
    pub max_heading_level: u8,
    /// Length of code block fence (typically 3 for ```)
    pub code_fence_length: usize,
    /// Pattern for code block fence (typically "```")
    pub code_fence_pattern: String,
    /// Language identifier for Mermaid diagrams
    pub mermaid_language: String,
    /// Mermaid diagram configuration
    #[serde(default)]
    pub mermaid: MermaidParserConfig,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            max_heading_level: 6,
            code_fence_length: 3,
            code_fence_pattern: "```".to_string(),
            mermaid_language: "mermaid".to_string(),
            mermaid: MermaidParserConfig::default(),
        }
    }
}

/// Configuration for the renderer settings
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RendererConfig {
    /// Output directory for rendered files
    pub output_directory: String,
    /// Path to HTML header template file
    pub html_header_path: String,
    /// Path to HTML footer template file
    pub html_footer_path: String,
    /// Path to HTML body start template file
    pub html_body_start_path: String,
    /// Path to CSS styles file
    pub styles_css_path: String,
}

impl Default for RendererConfig {
    fn default() -> Self {
        Self {
            output_directory: "output".to_string(),
            html_header_path: "assets/html_header.html".to_string(),
            html_footer_path: "assets/html_footer.html".to_string(),
            html_body_start_path: "assets/html_body_start.html".to_string(),
            styles_css_path: "assets/styles.css".to_string(),
        }
    }
}

/// Configuration for output file settings
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OutputConfig {
    /// Output directory for all generated files
    pub directory: String,
    /// Filename for AST debug output
    pub ast_debug_filename: String,
    /// Filename for AST JSON output
    pub ast_json_filename: String,
    /// Filename for HTML output
    pub html_filename: String,
    /// Enable AST debug output
    pub enable_ast_debug: bool,
    /// Enable AST JSON output
    pub enable_ast_json: bool,
    /// Enable HTML output
    pub enable_html: bool,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            directory: "output".to_string(),
            ast_debug_filename: "ast.txt".to_string(),
            ast_json_filename: "ast.json".to_string(),
            html_filename: "output.html".to_string(),
            enable_ast_debug: true,
            enable_ast_json: true,
            enable_html: true,
        }
    }
}

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Config {
    /// Parser configuration
    pub parser: ParserConfig,
    /// Renderer configuration
    pub renderer: RendererConfig,
    /// Output configuration
    pub output: OutputConfig,
}

impl Config {
    /// Load configuration from `config.toml` file, or return default if file doesn't exist
    ///
    /// # Errors
    ///
    /// Returns an error if the config file exists but cannot be parsed
    pub fn load_config() -> Result<Self, String> {
        const CONFIG_PATH: &str = "config.toml";

        if !Path::new(CONFIG_PATH).exists() {
            return Ok(Self::default());
        }

        let contents = fs::read_to_string(CONFIG_PATH)
            .map_err(|e| format!("Failed to read config file '{}': {}", CONFIG_PATH, e))?;

        let config: Config = toml::from_str(&contents)
            .map_err(|e| format!("Failed to parse config file '{}': {}", CONFIG_PATH, e))?;

        // Validate config values
        config.validate()?;

        Ok(config)
    }

    /// Validate configuration values
    ///
    /// # Errors
    ///
    /// Returns an error if any configuration value is invalid
    fn validate(&self) -> Result<(), String> {
        // Validate max_heading_level (must be between 1 and 6)
        if self.parser.max_heading_level == 0 || self.parser.max_heading_level > 6 {
            return Err(format!(
                "Invalid max_heading_level: {}. Must be between 1 and 6",
                self.parser.max_heading_level
            ));
        }

        // Validate code_fence_length (must be at least 1)
        if self.parser.code_fence_length == 0 {
            return Err(format!(
                "Invalid code_fence_length: {}. Must be at least 1",
                self.parser.code_fence_length
            ));
        }

        // Validate code_fence_pattern (must not be empty)
        if self.parser.code_fence_pattern.is_empty() {
            return Err("code_fence_pattern cannot be empty".to_string());
        }

        // Validate mermaid_language (must not be empty)
        if self.parser.mermaid_language.is_empty() {
            return Err("mermaid_language cannot be empty".to_string());
        }

        Ok(())
    }
}
