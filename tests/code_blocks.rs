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
        Node::MermaidDiagram { diagram } => {
            assert_eq!(diagram, "graph TD\n    A-->B");
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
