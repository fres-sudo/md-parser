//! Markdown Parser with Mermaid Diagram Support
//!
//! This library parses Markdown text into a structured Abstract Syntax Tree (AST).
//! It provides special handling for Mermaid diagrams, distinguishing them from
//! standard code blocks.

mod ast;
mod parser;
mod renderer;

pub use ast::{Inline, Node, ParseError};
pub use parser::Parser;

use std::error::Error;

impl Parser {
    /// Generate a complete HTML document from the AST
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if parsing fails
    pub fn to_html(&mut self) -> Result<String, ParseError> {
        let ast = self.parse()?;
        Ok(renderer::render_to_html(&ast))
    }

    /// Save the HTML output to a file in the output/ directory
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if parsing fails, or `std::io::Error` if file operations fail
    pub fn to_html_file(&mut self, filename: &str) -> Result<(), Box<dyn Error>> {
        let ast = self.parse()?;
        renderer::render_to_html_file(&ast, filename)
    }
}
