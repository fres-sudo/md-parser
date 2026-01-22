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
        let ast = self.parse();
        let mut html = String::from(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Markdown Parser Output</title>
    <script src="https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.min.js"></script>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
            line-height: 1.6;
            max-width: 800px;
            margin: 0 auto;
            padding: 20px;
            color: #333;
            background-color: #fff;
        }
        h1, h2, h3, h4, h5, h6 {
            margin-top: 24px;
            margin-bottom: 16px;
            font-weight: 600;
            line-height: 1.25;
        }
        h1 { font-size: 2em; border-bottom: 1px solid #eaecef; padding-bottom: 0.3em; }
        h2 { font-size: 1.5em; border-bottom: 1px solid #eaecef; padding-bottom: 0.3em; }
        h3 { font-size: 1.25em; }
        h4 { font-size: 1em; }
        h5 { font-size: 0.875em; }
        h6 { font-size: 0.85em; color: #6a737d; }
        p {
            margin-bottom: 16px;
        }
        strong {
            font-weight: 600;
        }
        em {
            font-style: italic;
        }
        a {
            color: #0366d6;
            text-decoration: none;
        }
        a:hover {
            text-decoration: underline;
        }
        pre {
            background-color: #f6f8fa;
            border-radius: 6px;
            padding: 16px;
            overflow: auto;
            margin-bottom: 16px;
        }
        code {
            font-family: 'SFMono-Regular', Consolas, 'Liberation Mono', Menlo, monospace;
            font-size: 85%;
        }
        pre code {
            display: block;
            padding: 0;
            margin: 0;
            overflow: visible;
            word-wrap: normal;
            background-color: transparent;
            border: 0;
        }
        .mermaid {
            margin: 24px 0;
            text-align: center;
            background-color: #f6f8fa;
            padding: 20px;
            border-radius: 6px;
        }
        li {
            margin-bottom: 8px;
        }
    </style>
</head>
<body>
"#,
        );

        for node in &ast {
            html.push_str(&self.render_node(node));
            html.push('\n');
        }

        html.push_str(
            r#"
    <script>
        mermaid.initialize({ startOnLoad: true, theme: 'default' });
    </script>
</body>
</html>"#,
        );

        html
    }

    /// Save the HTML output to a file
    pub fn to_html_file(&self, filename: &str) -> std::io::Result<()> {
        use std::fs::File;
        use std::io::Write;
        let html = self.to_html();
        let mut file = File::create(filename)?;
        file.write_all(html.as_bytes())?;
        Ok(())
    }
}

