use md_parser::Parser;

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
    parser.to_html_file("output.html")?;
    println!("✓ HTML file generated successfully: output/output.html");

    Ok(())
}
