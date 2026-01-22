use md_parser::{Inline, Node, Parser};

#[test]
fn test_heading_h1() {
    let input = "# Heading 1".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::Heading { level, content } => {
            assert_eq!(*level, 1);
            assert_eq!(content.len(), 1);
            assert_eq!(
                content[0],
                Inline::Text {
                    content: "Heading 1".to_string()
                }
            );
        }
        _ => panic!("Expected Heading"),
    }
}

#[test]
fn test_heading_h2() {
    let input = "## Heading 2".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::Heading { level, content } => {
            assert_eq!(*level, 2);
            assert_eq!(content.len(), 1);
            assert_eq!(
                content[0],
                Inline::Text {
                    content: "Heading 2".to_string()
                }
            );
        }
        _ => panic!("Expected Heading"),
    }
}

#[test]
fn test_heading_h6() {
    let input = "###### Heading 6".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::Heading { level, content } => {
            assert_eq!(*level, 6);
            assert_eq!(content.len(), 1);
            assert_eq!(
                content[0],
                Inline::Text {
                    content: "Heading 6".to_string()
                }
            );
        }
        _ => panic!("Expected Heading"),
    }
}

#[test]
fn test_mixed_content() {
    let input = "# Title\n\nSome paragraph.\n\n```rust\nfn main() {}\n```\n\n```mermaid\ngraph TD\n    A-->B\n```".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 4);
    match &result[0] {
        Node::Heading { level, .. } => {
            assert_eq!(*level, 1);
        }
        _ => panic!("Expected Heading"),
    }
    match &result[1] {
        Node::Paragraph { content: inlines } => {
            assert_eq!(inlines.len(), 1);
            assert_eq!(
                inlines[0],
                Inline::Text {
                    content: "Some paragraph.".to_string()
                }
            );
        }
        _ => panic!("Expected Paragraph"),
    }
    match &result[2] {
        Node::CodeBlock { lang, .. } => {
            assert_eq!(lang.as_ref(), Some(&"rust".to_string()));
        }
        _ => panic!("Expected CodeBlock"),
    }
    match &result[3] {
        Node::MermaidDiagram { .. } => {}
        _ => panic!("Expected MermaidDiagram"),
    }
}
