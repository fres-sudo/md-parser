//! Markdown Parser with Mermaid Diagram Support
//!
//! This library parses Markdown text into a structured Abstract Syntax Tree (AST).
//! It provides special handling for Mermaid diagrams, distinguishing them from
//! standard code blocks.

mod ast;
mod config;
mod parser;
mod renderer;

pub use ast::{Alignment, Inline, MermaidConfig, Node, ParseError, Span, ValidationStatus};
pub use config::{Config, MermaidParserConfig, OutputConfig, ParserConfig, RendererConfig};
pub use parser::Parser;

use std::error::Error;

impl Parser {
    /// Generate a complete HTML document from the AST using default renderer config
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if parsing fails, or `Box<dyn Error>` if template loading fails
    pub fn to_html(&mut self) -> Result<String, Box<dyn Error>> {
        let ast = self.parse()?;
        let renderer_config = RendererConfig::default();
        renderer::render_to_html(&ast, &renderer_config)
    }

    /// Generate a complete HTML document from the AST using custom renderer config
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if parsing fails, or `Box<dyn Error>` if template loading fails
    pub fn to_html_with_config(
        &mut self,
        renderer_config: &RendererConfig,
    ) -> Result<String, Box<dyn Error>> {
        let ast = self.parse()?;
        renderer::render_to_html(&ast, renderer_config)
    }

    /// Save the HTML output to a file using default renderer config
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if parsing fails, or `Box<dyn Error>` if file operations fail
    pub fn to_html_file(&mut self, filename: &str) -> Result<(), Box<dyn Error>> {
        let ast = self.parse()?;
        let renderer_config = RendererConfig::default();
        renderer::render_to_html_file(&ast, filename, &renderer_config)
    }

    /// Save the HTML output to a file using custom renderer config
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if parsing fails, or `Box<dyn Error>` if file operations fail
    pub fn to_html_file_with_config(
        &mut self,
        filename: &str,
        renderer_config: &RendererConfig,
    ) -> Result<(), Box<dyn Error>> {
        let ast = self.parse()?;
        renderer::render_to_html_file(&ast, filename, renderer_config)
    }
}
