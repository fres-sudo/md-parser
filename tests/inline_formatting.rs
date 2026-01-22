use md_parser::{Inline, Node, Parser};

#[test]
fn test_bold_text() {
    let input = "This is **bold** text.".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::Paragraph { content: inlines } => {
            assert_eq!(inlines.len(), 3);
            assert_eq!(
                inlines[0],
                Inline::Text {
                    content: "This is ".to_string()
                }
            );
            match &inlines[1] {
                Inline::Bold {
                    content: bold_inlines,
                } => {
                    assert_eq!(bold_inlines.len(), 1);
                    assert_eq!(
                        bold_inlines[0],
                        Inline::Text {
                            content: "bold".to_string()
                        }
                    );
                }
                _ => panic!("Expected Bold"),
            }
            assert_eq!(
                inlines[2],
                Inline::Text {
                    content: " text.".to_string()
                }
            );
        }
        _ => panic!("Expected Paragraph"),
    }
}

#[test]
fn test_italic_text() {
    let input = "This is *italic* text.".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::Paragraph { content: inlines } => {
            assert_eq!(inlines.len(), 3);
            assert_eq!(
                inlines[0],
                Inline::Text {
                    content: "This is ".to_string()
                }
            );
            match &inlines[1] {
                Inline::Italic {
                    content: italic_inlines,
                } => {
                    assert_eq!(italic_inlines.len(), 1);
                    assert_eq!(
                        italic_inlines[0],
                        Inline::Text {
                            content: "italic".to_string()
                        }
                    );
                }
                _ => panic!("Expected Italic"),
            }
            assert_eq!(
                inlines[2],
                Inline::Text {
                    content: " text.".to_string()
                }
            );
        }
        _ => panic!("Expected Paragraph"),
    }
}

#[test]
fn test_link() {
    let input = "Visit [Rust](https://rust-lang.org) today!".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::Paragraph { content: inlines } => {
            assert_eq!(inlines.len(), 3);
            assert_eq!(
                inlines[0],
                Inline::Text {
                    content: "Visit ".to_string()
                }
            );
            match &inlines[1] {
                Inline::Link { text, url } => {
                    assert_eq!(text.len(), 1);
                    assert_eq!(
                        text[0],
                        Inline::Text {
                            content: "Rust".to_string()
                        }
                    );
                    assert_eq!(url, "https://rust-lang.org");
                }
                _ => panic!("Expected Link"),
            }
            assert_eq!(
                inlines[2],
                Inline::Text {
                    content: " today!".to_string()
                }
            );
        }
        _ => panic!("Expected Paragraph"),
    }
}

#[test]
fn test_nested_bold_italic() {
    let input = "This is **bold with *italic* inside**.".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::Paragraph { content: inlines } => {
            // Should have at least "This is " text and a Bold element
            assert!(!inlines.is_empty());
            // Check that we have a Bold element somewhere
            let has_bold = inlines
                .iter()
                .any(|inline| matches!(inline, Inline::Bold { .. }));
            assert!(has_bold, "Expected at least one Bold element");
            // If we have text before bold, verify it
            if let Some(Inline::Text { content }) = inlines.first() {
                assert!(content.contains("This is") || content.is_empty());
            }
        }
        _ => panic!("Expected Paragraph"),
    }
}

#[test]
fn test_heading_with_inline() {
    let input = "# This is a **bold** heading".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::Heading { level, content } => {
            assert_eq!(*level, 1);
            assert!(content.len() >= 2);
        }
        _ => panic!("Expected Heading"),
    }
}

#[test]
fn test_mixed_inline_elements() {
    let input = "Check out [Rust](https://rust-lang.org) and **bold** and *italic*.".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::Paragraph { content: inlines } => {
            // Should have multiple inline elements
            assert!(inlines.len() >= 3);
        }
        _ => panic!("Expected Paragraph"),
    }
}
