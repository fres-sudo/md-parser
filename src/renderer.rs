//! HTML rendering logic.

use crate::ast::{Inline, Node};
use std::error::Error;
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::PathBuf;

/// Escape HTML special characters
fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Render inline elements to HTML
fn render_inline(inline: &Inline) -> String {
    match inline {
        Inline::Text { content } => escape_html(content),
        Inline::Bold { content } => {
            let inner: String = content.iter().map(render_inline).collect();
            format!("<strong>{}</strong>", inner)
        }
        Inline::Italic { content } => {
            let inner: String = content.iter().map(render_inline).collect();
            format!("<em>{}</em>", inner)
        }
        Inline::Link { text, url } => {
            let link_text: String = text.iter().map(render_inline).collect();
            format!("<a href=\"{}\">{}</a>", escape_html(url), link_text)
        }
    }
}

/// Render a single node to HTML
fn render_node(node: &Node) -> String {
    match node {
        Node::Heading { level, content } => {
            let inner: String = content.iter().map(render_inline).collect();
            format!("<h{}>{}</h{}>", level, inner, level)
        }
        Node::Paragraph { content } => {
            let inner: String = content.iter().map(render_inline).collect();
            format!("<p>{}</p>", inner)
        }
        Node::ListItem { content } => {
            let inner: String = content.iter().map(render_inline).collect();
            format!("<li>{}</li>", inner)
        }
        Node::CodeBlock { lang, code } => {
            let lang_class = lang
                .as_ref()
                .map(|l| format!(" class=\"language-{}\"", escape_html(l)))
                .unwrap_or_default();
            let escaped_code = escape_html(code);
            format!("<pre><code{}>{}</code></pre>", lang_class, escaped_code)
        }
        Node::MermaidDiagram { diagram } => {
            let escaped_diagram = escape_html(diagram);
            format!("<div class=\"mermaid\">{}</div>", escaped_diagram)
        }
    }
}

/// Generate a complete HTML document from the AST.
///
/// Loads header, styles, body start, and footer from assets, then renders each node.
pub(crate) fn render_to_html(ast: &[Node]) -> String {
    const HTML_HEADER: &str = include_str!("../assets/html_header.html");
    const STYLES_CSS: &str = include_str!("../assets/styles.css");
    const HTML_BODY_START: &str = include_str!("../assets/html_body_start.html");
    const HTML_FOOTER: &str = include_str!("../assets/html_footer.html");

    let mut html = String::new();
    html.push_str(HTML_HEADER);
    html.push_str(STYLES_CSS);
    html.push_str(HTML_BODY_START);

    for node in ast {
        html.push_str(&render_node(node));
        html.push('\n');
    }

    html.push_str(HTML_FOOTER);
    html
}

/// Write the AST as a full HTML document to `output/<filename>`.
///
/// Creates the output directory if it does not exist.
///
/// # Errors
///
/// Returns `Box<dyn Error>` if directory creation or file writing fails.
pub(crate) fn render_to_html_file(ast: &[Node], filename: &str) -> Result<(), Box<dyn Error>> {
    let output_dir = PathBuf::from("output");
    create_dir_all(&output_dir)?;

    let file_path = output_dir.join(filename);
    let html = render_to_html(ast);
    let mut file = File::create(&file_path)?;
    file.write_all(html.as_bytes())?;
    Ok(())
}
