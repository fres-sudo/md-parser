//! HTML rendering logic.

use crate::ast::{Alignment, Inline, ListItem, Node, ValidationStatus};
use crate::config::RendererConfig;
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
        Inline::Strikethrough { content } => {
            let inner: String = content.iter().map(render_inline).collect();
            format!("<del>{}</del>", inner)
        }
        Inline::Link { text, url } => {
            let link_text: String = text.iter().map(render_inline).collect();
            format!("<a href=\"{}\">{}</a>", escape_html(url), link_text)
        }
        Inline::Image { alt, url } => {
            format!(
                "<img src=\"{}\" alt=\"{}\" />",
                escape_html(url),
                escape_html(alt)
            )
        }
    }
}

/// Render a list item and its nested children recursively
fn render_list_item(item: &ListItem) -> String {
    let content: String = item.content.iter().map(render_inline).collect();

    // Render checkbox for task list items
    let checkbox = if let Some(checked) = item.checked {
        if checked {
            "<input type=\"checkbox\" disabled checked> "
        } else {
            "<input type=\"checkbox\" disabled> "
        }
    } else {
        ""
    };

    let mut html = format!("<li>{}{}", checkbox, content);

    // Render nested children if any
    if !item.children.is_empty() {
        html.push_str("<ul>");
        for child in &item.children {
            html.push_str(&render_list_item(child));
        }
        html.push_str("</ul>");
    }

    html.push_str("</li>");
    html
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
        Node::UnorderedList { items } => {
            let mut html = String::from("<ul>");
            for item in items {
                html.push_str(&render_list_item(item));
            }
            html.push_str("</ul>");
            html
        }
        Node::OrderedList { items } => {
            let mut html = String::from("<ol>");
            for item in items {
                html.push_str(&render_list_item(item));
            }
            html.push_str("</ol>");
            html
        }
        Node::CodeBlock { lang, code } => {
            let lang_class = lang
                .as_ref()
                .map(|l| format!(" class=\"language-{}\"", escape_html(l)))
                .unwrap_or_default();
            let escaped_code = escape_html(code);
            format!("<pre><code{}>{}</code></pre>", lang_class, escaped_code)
        }
        Node::MermaidDiagram {
            diagram,
            config,
            validation_status,
            warnings,
        } => {
            let escaped_diagram = escape_html(diagram);

            // Build data attributes for configuration
            let mut data_attrs = String::new();
            if let Some(cfg) = config {
                // Serialize config to JSON for data attribute
                if let Ok(config_json) = serde_json::to_string(cfg) {
                    data_attrs.push_str(&format!(
                        " data-mermaid-config=\"{}\"",
                        escape_html(&config_json)
                    ));
                }

                // Also add individual attributes for easier access
                if let Some(ref theme) = cfg.theme {
                    data_attrs.push_str(&format!(" data-mermaid-theme=\"{}\"", escape_html(theme)));
                }
                if let Some(ref font_size) = cfg.font_size {
                    data_attrs.push_str(&format!(
                        " data-mermaid-font-size=\"{}\"",
                        escape_html(font_size)
                    ));
                }
                if let Some(ref font_family) = cfg.font_family {
                    data_attrs.push_str(&format!(
                        " data-mermaid-font-family=\"{}\"",
                        escape_html(font_family)
                    ));
                }
            }

            // Add validation status as data attribute
            let validation_attr = match validation_status {
                ValidationStatus::Valid => " data-mermaid-valid=\"true\"",
                ValidationStatus::Invalid { .. } => " data-mermaid-valid=\"false\"",
                ValidationStatus::NotValidated => "",
            };

            // Build HTML with validation warnings as comments
            let mut html = String::new();

            // Add validation warning comments if present
            if let ValidationStatus::Invalid { ref errors } = validation_status {
                html.push_str("<!-- Mermaid validation errors:\n");
                for error in errors {
                    html.push_str(&format!("  - {}\n", escape_html(error)));
                }
                html.push_str("-->\n");
            }

            if !warnings.is_empty() {
                html.push_str("<!-- Mermaid validation warnings:\n");
                for warning in warnings {
                    html.push_str(&format!("  - {}\n", escape_html(warning)));
                }
                html.push_str("-->\n");
            }

            html.push_str(&format!(
                "<div class=\"mermaid\"{}{}>{}</div>",
                data_attrs, validation_attr, escaped_diagram
            ));

            html
        }
        Node::Table {
            headers,
            rows,
            alignments,
        } => {
            let mut html = String::from("<table>\n<thead>\n<tr>");
            for (i, header_cell) in headers.iter().enumerate() {
                let alignment = alignments
                    .get(i)
                    .and_then(|a| a.as_ref())
                    .map(|a| match a {
                        Alignment::Left => " style=\"text-align: left;\"",
                        Alignment::Center => " style=\"text-align: center;\"",
                        Alignment::Right => " style=\"text-align: right;\"",
                    })
                    .unwrap_or_default();
                let cell_content: String = header_cell.iter().map(render_inline).collect();
                html.push_str(&format!("<th{}>{}</th>", alignment, cell_content));
            }
            html.push_str("</tr>\n</thead>\n<tbody>");
            for row in rows {
                html.push_str("<tr>");
                for (i, cell) in row.iter().enumerate() {
                    let alignment = alignments
                        .get(i)
                        .and_then(|a| a.as_ref())
                        .map(|a| match a {
                            Alignment::Left => " style=\"text-align: left;\"",
                            Alignment::Center => " style=\"text-align: center;\"",
                            Alignment::Right => " style=\"text-align: right;\"",
                        })
                        .unwrap_or_default();
                    let cell_content: String = cell.iter().map(render_inline).collect();
                    html.push_str(&format!("<td{}>{}</td>", alignment, cell_content));
                }
                html.push_str("</tr>");
            }
            html.push_str("</tbody>\n</table>");
            html
        }
        Node::Blockquote { level, content } => {
            let inner: String = content.iter().map(render_inline).collect();
            // For nested blockquotes, nest multiple <blockquote> elements
            let mut html = String::new();
            for _ in 0..*level {
                html.push_str("<blockquote>");
            }
            html.push_str(&inner);
            for _ in 0..*level {
                html.push_str("</blockquote>");
            }
            html
        }
        Node::HorizontalRule => String::from("<hr>"),
    }
}

/// Generate a complete HTML document from the AST.
///
/// Loads header, styles, body start, and footer from configured paths, then renders each node.
///
/// # Errors
///
/// Returns an error if template files cannot be read
pub(crate) fn render_to_html(
    ast: &[Node],
    config: &RendererConfig,
) -> Result<String, Box<dyn Error>> {
    // Try to load from configured paths, fallback to include_str! if files don't exist
    let html_header = if std::path::Path::new(&config.html_header_path).exists() {
        std::fs::read_to_string(&config.html_header_path)?
    } else {
        include_str!("../assets/html_header.html").to_string()
    };

    let styles_css = if std::path::Path::new(&config.styles_css_path).exists() {
        std::fs::read_to_string(&config.styles_css_path)?
    } else {
        include_str!("../assets/styles.css").to_string()
    };

    let html_body_start = if std::path::Path::new(&config.html_body_start_path).exists() {
        std::fs::read_to_string(&config.html_body_start_path)?
    } else {
        include_str!("../assets/html_body_start.html").to_string()
    };

    let html_footer = if std::path::Path::new(&config.html_footer_path).exists() {
        std::fs::read_to_string(&config.html_footer_path)?
    } else {
        include_str!("../assets/html_footer.html").to_string()
    };

    let mut html = String::new();
    html.push_str(&html_header);
    html.push_str(&format!("<style>\n{}\n</style>", styles_css));
    html.push_str(&html_body_start);

    for node in ast {
        html.push_str(&render_node(node));
        html.push('\n');
    }

    html.push_str(&html_footer);
    Ok(html)
}

/// Write the AST as a full HTML document to the configured output directory.
///
/// Creates the output directory if it does not exist.
///
/// # Errors
///
/// Returns `Box<dyn Error>` if directory creation, template loading, or file writing fails.
pub(crate) fn render_to_html_file(
    ast: &[Node],
    filename: &str,
    config: &RendererConfig,
) -> Result<(), Box<dyn Error>> {
    let output_dir = PathBuf::from(&config.output_directory);
    create_dir_all(&output_dir)?;

    let file_path = output_dir.join(filename);
    let html = render_to_html(ast, config)?;
    let mut file = File::create(&file_path)?;
    file.write_all(html.as_bytes())?;
    Ok(())
}
