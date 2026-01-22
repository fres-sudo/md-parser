/// Markdown Parser with Mermaid Diagram Support
///
/// This library parses Markdown text into a structured Abstract Syntax Tree (AST).
/// It provides special handling for Mermaid diagrams, distinguishing them from
/// standard code blocks.

use serde::{Deserialize, Serialize};

/// Represents inline elements within text (bold, italic, links, plain text)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Inline {
    /// Plain text content
    #[serde(rename = "text")]
    Text {
        content: String,
    },
    /// Bold text (**text**)
    #[serde(rename = "bold")]
    Bold {
        content: Vec<Inline>,
    },
    /// Italic text (*text*)
    #[serde(rename = "italic")]
    Italic {
        content: Vec<Inline>,
    },
    /// Link [text](url)
    #[serde(rename = "link")]
    Link {
        text: Vec<Inline>,
        url: String,
    },
}

/// Represents a node in the Markdown Abstract Syntax Tree
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Node {
    /// A heading with level (1-6) and content
    #[serde(rename = "heading")]
    Heading {
        level: u8,
        content: Vec<Inline>,
    },
    /// A paragraph of text
    #[serde(rename = "paragraph")]
    Paragraph {
        content: Vec<Inline>,
    },
    /// An unordered list item
    #[serde(rename = "list_item")]
    ListItem {
        content: Vec<Inline>,
    },
    /// A fenced code block with optional language identifier
    #[serde(rename = "code_block")]
    CodeBlock {
        lang: Option<String>,
        code: String,
    },
    /// A Mermaid diagram (distinct from CodeBlock)
    #[serde(rename = "mermaid_diagram")]
    MermaidDiagram {
        diagram: String,
    },
}

/// Parser for converting Markdown text into an AST
pub struct Parser {
    input: String,
}

impl Parser {
    /// Create a new parser from a Markdown string
    pub fn new(input: String) -> Self {
        Self { input }
    }

    /// Parse inline elements from a text string
    fn parse_inline(&self, text: &str) -> Vec<Inline> {
        use regex::Regex;
        let mut inlines = Vec::new();
        let mut remaining = text;

        // Regex patterns for inline elements
        // Note: Process bold before italic to avoid conflicts
        let link_re = Regex::new(r"\[([^\]]+)\]\(([^)]+)\)").unwrap();
        // Bold: **...** - match content that doesn't contain **
        // We'll use a pattern that matches **, then content (allowing single *), then **
        // Simple approach: match ** then any chars until **, but we need to be careful
        let bold_re = Regex::new(r"\*\*((?:[^*]|\*[^*\n])+?)\*\*").unwrap();
        // Italic: single * not preceded or followed by *
        // We'll check manually that it's not part of bold
        let italic_re = Regex::new(r"\*([^*\n]+?)\*").unwrap();

        while !remaining.is_empty() {
            // Find the earliest match among all patterns
            let mut earliest_pos = remaining.len();
            let mut match_type = None;
            let mut match_range = (0, 0);

            // Check for links
            if let Some(m) = link_re.find(remaining) {
                if m.start() < earliest_pos {
                    earliest_pos = m.start();
                    match_type = Some("link");
                    match_range = (m.start(), m.end());
                }
            }

            // Check for bold (must check before italic to avoid conflicts)
            if let Some(m) = bold_re.find(remaining) {
                if m.start() < earliest_pos {
                    earliest_pos = m.start();
                    match_type = Some("bold");
                    match_range = (m.start(), m.end());
                }
            }

            // Check for italic (only if not part of bold - check that it's not **)
            if let Some(m) = italic_re.find(remaining) {
                let start = m.start();
                let end = m.end();
                // Make sure it's not part of bold (check for ** before or after)
                let is_bold = (start > 0 && remaining.as_bytes()[start - 1] == b'*')
                    || (end < remaining.len() && remaining.as_bytes()[end] == b'*');

                if !is_bold && start < earliest_pos {
                    match_type = Some("italic");
                    match_range = (start, end);
                    earliest_pos = start;
                }
            }

            // Process the match
            match match_type {
                Some("link") => {
                    // Add text before the link
                    if match_range.0 > 0 {
                        let text_before = &remaining[..match_range.0];
                        if !text_before.is_empty() {
                            inlines.push(Inline::Text { content: text_before.to_string() });
                        }
                    }
                    if let Some(caps) = link_re.captures(&remaining[match_range.0..match_range.1]) {
                        let link_text = caps.get(1).unwrap().as_str();
                        let link_url = caps.get(2).unwrap().as_str();
                        let text_inlines = self.parse_inline(link_text);
                        inlines.push(Inline::Link {
                            text: text_inlines,
                            url: link_url.to_string(),
                        });
                    }
                    remaining = &remaining[match_range.1..];
                }
                Some("bold") => {
                    // Add text before the bold
                    if match_range.0 > 0 {
                        let text_before = &remaining[..match_range.0];
                        if !text_before.is_empty() {
                            inlines.push(Inline::Text { content: text_before.to_string() });
                        }
                    }
                    if let Some(caps) = bold_re.captures(&remaining[match_range.0..match_range.1]) {
                        let bold_text = caps.get(1).unwrap().as_str();
                        let bold_inlines = self.parse_inline(bold_text);
                        inlines.push(Inline::Bold { content: bold_inlines });
                    }
                    remaining = &remaining[match_range.1..];
                }
                Some("italic") => {
                    // Add text before the italic
                    if match_range.0 > 0 {
                        let text_before = &remaining[..match_range.0];
                        if !text_before.is_empty() {
                            inlines.push(Inline::Text { content: text_before.to_string() });
                        }
                    }
                    if let Some(caps) = italic_re.captures(&remaining[match_range.0..match_range.1]) {
                        let italic_text = caps.get(1).unwrap().as_str();
                        let italic_inlines = self.parse_inline(italic_text);
                        inlines.push(Inline::Italic { content: italic_inlines });
                    }
                    remaining = &remaining[match_range.1..];
                }
                Some(_) => {
                    // Unexpected match type (should not happen)
                    remaining = &remaining[match_range.1..];
                }
                None => {
                    // No more matches, add remaining text
                    if !remaining.is_empty() {
                        inlines.push(Inline::Text { content: remaining.to_string() });
                    }
                    break;
                }
            }
        }

        // If no inline elements were found, return a single text node
        if inlines.is_empty() && !text.is_empty() {
            inlines.push(Inline::Text { content: text.to_string() });
        }

        inlines
    }

    /// Parse the input Markdown into a vector of AST nodes
    pub fn parse(&self) -> Vec<Node> {
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

            // Check for fenced code blocks (```)
            if line.starts_with("```") {
                let lang_tag = line[3..].trim();
                let lang = if lang_tag.is_empty() {
                    None
                } else {
                    Some(lang_tag.to_string())
                };

                // Collect code block content until closing fence
                let mut code_lines = Vec::new();
                i += 1;
                while i < lines.len() {
                    if lines[i].trim() == "```" {
                        break;
                    }
                    code_lines.push(lines[i]);
                    i += 1;
                }

                let code = code_lines.join("\n");

                // Special handling for Mermaid diagrams
                if lang.as_ref().map(|s| s.to_lowercase()) == Some("mermaid".to_string()) {
                    nodes.push(Node::MermaidDiagram { diagram: code });
                } else {
                    nodes.push(Node::CodeBlock { lang, code });
                }
                i += 1;
                continue;
            }

            // Check for headings (# syntax)
            if line.starts_with('#') {
                let mut level = 0;
                let mut chars = line.chars();
                while chars.next() == Some('#') && level < 6 {
                    level += 1;
                }

                if level > 0 && level <= 6 {
                    let content = line[level..].trim();
                    if !content.is_empty() {
                        let inline_content = self.parse_inline(content);
                        nodes.push(Node::Heading {
                            level: level as u8,
                            content: inline_content,
                        });
                        i += 1;
                        continue;
                    }
                }
            }

            // Collect paragraph lines (until empty line or block element)
            let mut para_lines = Vec::new();
            while i < lines.len() {
                let current_line = lines[i].trim();

                // Stop at empty line or block elements
                if current_line.is_empty() {
                    break;
                }
                if current_line.starts_with('#') || current_line.starts_with("```") {
                    break;
                }

                para_lines.push(current_line);
                i += 1;
            }

            if !para_lines.is_empty() {
                let para_text = para_lines.join(" ");
                let inline_content = self.parse_inline(&para_text);
                nodes.push(Node::Paragraph { content: inline_content });
            }
        }

        nodes
    }

    /// Serialize the AST to JSON string
    pub fn to_json(&self) -> String {
        let ast = self.parse();
        serde_json::to_string_pretty(&ast).unwrap_or_else(|e| format!("Error serializing: {}", e))
    }

    /// Render inline elements to HTML
    fn render_inline(&self, inline: &Inline) -> String {
        match inline {
            Inline::Text { content } => self.escape_html(content),
            Inline::Bold { content } => {
                let inner: String = content.iter().map(|i| self.render_inline(i)).collect();
                format!("<strong>{}</strong>", inner)
            }
            Inline::Italic { content } => {
                let inner: String = content.iter().map(|i| self.render_inline(i)).collect();
                format!("<em>{}</em>", inner)
            }
            Inline::Link { text, url } => {
                let link_text: String = text.iter().map(|i| self.render_inline(i)).collect();
                format!("<a href=\"{}\">{}</a>", self.escape_html(url), link_text)
            }
        }
    }

    /// Escape HTML special characters
    fn escape_html(&self, text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;")
    }

    /// Render a single node to HTML
    fn render_node(&self, node: &Node) -> String {
        match node {
            Node::Heading { level, content } => {
                let inner: String = content.iter().map(|i| self.render_inline(i)).collect();
                format!("<h{}>{}</h{}>", level, inner, level)
            }
            Node::Paragraph { content } => {
                let inner: String = content.iter().map(|i| self.render_inline(i)).collect();
                format!("<p>{}</p>", inner)
            }
            Node::ListItem { content } => {
                let inner: String = content.iter().map(|i| self.render_inline(i)).collect();
                format!("<li>{}</li>", inner)
            }
            Node::CodeBlock { lang, code } => {
                let lang_class = lang.as_ref()
                    .map(|l| format!(" class=\"language-{}\"", self.escape_html(l)))
                    .unwrap_or_default();
                let escaped_code = self.escape_html(code);
                format!("<pre><code{}>{}</code></pre>", lang_class, escaped_code)
            }
            Node::MermaidDiagram { diagram } => {
                // Use a div with class "mermaid" for Mermaid.js to process
                let escaped_diagram = self.escape_html(diagram);
                format!("<div class=\"mermaid\">{}</div>", escaped_diagram)
            }
        }
    }

    /// Generate a complete HTML document from the AST
    pub fn to_html(&self) -> String {
        // Load static assets at compile time
        const HTML_HEADER: &str = include_str!("../assets/html_header.html");
        const STYLES_CSS: &str = include_str!("../assets/styles.css");
        const HTML_BODY_START: &str = include_str!("../assets/html_body_start.html");
        const HTML_FOOTER: &str = include_str!("../assets/html_footer.html");

        let ast = self.parse();
        let mut html = String::new();

        // Build HTML document from assets
        html.push_str(HTML_HEADER);
        html.push_str(STYLES_CSS);
        html.push_str(HTML_BODY_START);

        // Add rendered nodes
        for node in &ast {
            html.push_str(&self.render_node(node));
            html.push('\n');
        }

        // Add footer
        html.push_str(HTML_FOOTER);

        html
    }

    /// Save the HTML output to a file in the output/ directory
    pub fn to_html_file(&self, filename: &str) -> std::io::Result<()> {
        use std::fs::{create_dir_all, File};
        use std::io::Write;
        use std::path::PathBuf;

        // Create output directory if it doesn't exist
        let output_dir = PathBuf::from("output");
        create_dir_all(&output_dir)?;

        // Create the full path to the file
        let file_path = output_dir.join(filename);

        let html = self.to_html();
        let mut file = File::create(&file_path)?;
        file.write_all(html.as_bytes())?;
        Ok(())
    }
}

