use md_parser::Parser;

fn main() {
    let markdown = r#"# Phase 2 Demo

This is a paragraph with some text.

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

## Subheading

Another paragraph after the diagram."#;

    let parser = Parser::new(markdown.to_string());
    let ast = parser.parse();

    println!("Parsed AST (Phase 2):");
    println!("=====================\n");
    for (i, node) in ast.iter().enumerate() {
        println!("  {}: {:?}\n", i, node);
    }
}
