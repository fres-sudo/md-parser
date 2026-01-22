use md_parser::Parser;
use std::fs;

const OUTPUT_DIR: &str = "output";

fn ensure_output_dir() -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(OUTPUT_DIR)
        .map_err(|e| format!("Error creating output dir '{}': {}", OUTPUT_DIR, e).into())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let markdown = r#"# Phase 3 Demo

This is a paragraph with **bold text** and *italic text*.

Here's a [link to Rust](https://rust-lang.org) in the text.

```rust
fn main() {
    println!("Hello, Rust!");
}
```

Here's a Mermaid diagram:

```mermaid
graph TD
    A[Start] --> B{Decision}
    B -->|Yes| C[Action 1]
    B -->|No| D[Action 2]
```

## Subheading with **Bold** and *Italic*

Another paragraph with mixed formatting: **bold** and *italic* and a [link](https://example.com)."#;

    let parser = Parser::new(markdown.to_string())?;
    let ast = parser.parse()?;

    println!("Parsed AST (Phase 3 - Debug Format):");
    println!("====================================\n");
    for (i, node) in ast.iter().enumerate() {
        println!("  {}: {:?}\n", i, node);
    }

    println!("\n\nParsed AST (JSON Format):");
    println!("==========================\n");
    let json = parser.to_json()?;
    println!("{}", json);

    println!("\n\nGenerating HTML file...");
    ensure_output_dir()?;
    parser.to_html_file("output.html")?;
    println!("✓ HTML file generated successfully: {}/output.html", OUTPUT_DIR);

    Ok(())
}
