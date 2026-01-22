use md_parser::{Inline, Node, Parser};

#[test]
fn test_simple_paragraph() {
    let input = "This is a simple paragraph.".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::Paragraph { content: inlines } => {
            assert_eq!(inlines.len(), 1);
            assert_eq!(
                inlines[0],
                Inline::Text {
                    content: "This is a simple paragraph.".to_string()
                }
            );
        }
        _ => panic!("Expected Paragraph"),
    }
}

#[test]
fn test_multiple_paragraphs() {
    let input = "First paragraph.\n\nSecond paragraph.".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 2);
    match &result[0] {
        Node::Paragraph { content: inlines } => {
            assert_eq!(inlines.len(), 1);
            assert_eq!(
                inlines[0],
                Inline::Text {
                    content: "First paragraph.".to_string()
                }
            );
        }
        _ => panic!("Expected Paragraph"),
    }
    match &result[1] {
        Node::Paragraph { content: inlines } => {
            assert_eq!(inlines.len(), 1);
            assert_eq!(
                inlines[0],
                Inline::Text {
                    content: "Second paragraph.".to_string()
                }
            );
        }
        _ => panic!("Expected Paragraph"),
    }
}

#[test]
fn test_empty_input() {
    let input = String::new();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 0);
}

#[test]
fn test_whitespace_only() {
    let input = "   \n\n   ".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 0);
}
