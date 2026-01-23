use md_parser::{Node, Parser};

#[test]
fn test_standard_code_block() {
    let input = "```rust\nfn main() {\n    println!(\"Hello\");\n}\n```".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::CodeBlock { lang, code } => {
            assert_eq!(lang.as_ref(), Some(&"rust".to_string()));
            assert_eq!(code, "fn main() {\n    println!(\"Hello\");\n}");
        }
        _ => panic!("Expected CodeBlock, got {:?}", result[0]),
    }
}

#[test]
fn test_code_block_without_language() {
    let input = "```\nSome code here\n```".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::CodeBlock { lang, code } => {
            assert_eq!(lang, &None);
            assert_eq!(code, "Some code here");
        }
        _ => panic!("Expected CodeBlock, got {:?}", result[0]),
    }
}

#[test]
fn test_mermaid_diagram() {
    let input = "```mermaid\ngraph TD\n    A-->B\n```".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::MermaidDiagram {
            diagram,
            config,
            validation_status,
            warnings,
        } => {
            assert_eq!(diagram, "graph TD\n    A-->B");
            assert!(config.is_some(), "Config should be present");
            // Validation should be Valid or NotValidated depending on config
            match validation_status {
                md_parser::ValidationStatus::Valid | md_parser::ValidationStatus::NotValidated => {}
                _ => panic!("Expected Valid or NotValidated status"),
            }
            assert!(
                warnings.is_empty(),
                "No warnings expected for valid diagram"
            );
        }
        _ => panic!("Expected MermaidDiagram, got {:?}", result[0]),
    }
}

#[test]
fn test_mermaid_vs_codeblock_distinction() {
    let input = "```rust\nfn main() {}\n```\n\n```mermaid\ngraph TD\n    A-->B\n```".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 2);
    // First should be CodeBlock
    match &result[0] {
        Node::CodeBlock { lang, .. } => {
            assert_eq!(lang.as_ref(), Some(&"rust".to_string()));
        }
        _ => panic!("First block should be CodeBlock, got {:?}", result[0]),
    }
    // Second should be MermaidDiagram
    match &result[1] {
        Node::MermaidDiagram { .. } => {}
        _ => panic!("Second block should be MermaidDiagram, got {:?}", result[1]),
    }
}

#[test]
fn test_python_code_block() {
    let input = "```python\ndef hello():\n    print(\"Hello, World!\")\n```".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::CodeBlock { lang, code } => {
            assert_eq!(lang.as_ref(), Some(&"python".to_string()));
            assert_eq!(code, "def hello():\n    print(\"Hello, World!\")");
        }
        _ => panic!("Expected CodeBlock, got {:?}", result[0]),
    }
}

#[test]
fn test_javascript_code_block() {
    let input = "```javascript\nfunction greet() {\n    console.log('Hello');\n}\n```".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::CodeBlock { lang, code } => {
            assert_eq!(lang.as_ref(), Some(&"javascript".to_string()));
            assert_eq!(code, "function greet() {\n    console.log('Hello');\n}");
        }
        _ => panic!("Expected CodeBlock, got {:?}", result[0]),
    }
}

#[test]
fn test_typescript_code_block() {
    let input =
        "```typescript\ninterface User {\n    name: string;\n    age: number;\n}\n```".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::CodeBlock { lang, code } => {
            assert_eq!(lang.as_ref(), Some(&"typescript".to_string()));
            assert_eq!(
                code,
                "interface User {\n    name: string;\n    age: number;\n}"
            );
        }
        _ => panic!("Expected CodeBlock, got {:?}", result[0]),
    }
}

#[test]
fn test_multiple_language_code_blocks() {
    let input = "```rust\nfn main() {}\n```\n\n```python\ndef main():\n    pass\n```\n\n```javascript\nfunction main() {}\n```".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 3);

    // First block: Rust
    match &result[0] {
        Node::CodeBlock { lang, code } => {
            assert_eq!(lang.as_ref(), Some(&"rust".to_string()));
            assert_eq!(code, "fn main() {}");
        }
        _ => panic!("First block should be Rust CodeBlock, got {:?}", result[0]),
    }

    // Second block: Python
    match &result[1] {
        Node::CodeBlock { lang, code } => {
            assert_eq!(lang.as_ref(), Some(&"python".to_string()));
            assert_eq!(code, "def main():\n    pass");
        }
        _ => panic!(
            "Second block should be Python CodeBlock, got {:?}",
            result[1]
        ),
    }

    // Third block: JavaScript
    match &result[2] {
        Node::CodeBlock { lang, code } => {
            assert_eq!(lang.as_ref(), Some(&"javascript".to_string()));
            assert_eq!(code, "function main() {}");
        }
        _ => panic!(
            "Third block should be JavaScript CodeBlock, got {:?}",
            result[2]
        ),
    }
}
