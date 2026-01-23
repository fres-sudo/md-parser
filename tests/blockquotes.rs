use md_parser::{Inline, Node, Parser};

#[test]
fn test_simple_blockquote() {
    let input = "> This is a simple blockquote.".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::Blockquote { level, content } => {
            assert_eq!(*level, 1);
            assert_eq!(content.len(), 1);
            assert_eq!(
                content[0],
                Inline::Text {
                    content: "This is a simple blockquote.".to_string()
                }
            );
        }
        _ => panic!("Expected Blockquote"),
    }
}

#[test]
fn test_multiline_blockquote() {
    let input = "> First line of blockquote.\n> Second line of blockquote.\n> Third line.".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::Blockquote { level, content } => {
            assert_eq!(*level, 1);
            // Content should be joined with spaces
            assert!(!content.is_empty());
            let text_content: String = content
                .iter()
                .map(|inline| match inline {
                    Inline::Text { content } => content.clone(),
                    _ => String::new(),
                })
                .collect();
            assert!(text_content.contains("First line"));
            assert!(text_content.contains("Second line"));
            assert!(text_content.contains("Third line"));
        }
        _ => panic!("Expected Blockquote"),
    }
}

#[test]
fn test_blockquote_with_inline_formatting() {
    let input = "> This is **bold** and *italic* text.".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::Blockquote { level, content } => {
            assert_eq!(*level, 1);
            // Should have bold and italic elements
            let has_bold = content
                .iter()
                .any(|inline| matches!(inline, Inline::Bold { .. }));
            let has_italic = content
                .iter()
                .any(|inline| matches!(inline, Inline::Italic { .. }));
            assert!(has_bold, "Expected Bold element in blockquote");
            assert!(has_italic, "Expected Italic element in blockquote");
        }
        _ => panic!("Expected Blockquote"),
    }
}

#[test]
fn test_blockquote_with_link() {
    let input = "> Visit [Rust](https://rust-lang.org) for more info.".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::Blockquote { level, content } => {
            assert_eq!(*level, 1);
            let has_link = content
                .iter()
                .any(|inline| matches!(inline, Inline::Link { .. }));
            assert!(has_link, "Expected Link element in blockquote");
        }
        _ => panic!("Expected Blockquote"),
    }
}

#[test]
fn test_nested_blockquote_level_2() {
    let input = ">> This is a nested blockquote.".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::Blockquote { level, content } => {
            assert_eq!(*level, 2);
            assert_eq!(content.len(), 1);
            assert_eq!(
                content[0],
                Inline::Text {
                    content: "This is a nested blockquote.".to_string()
                }
            );
        }
        _ => panic!("Expected Blockquote with level 2"),
    }
}

#[test]
fn test_nested_blockquote_level_3() {
    let input = ">>> This is a deeply nested blockquote.".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::Blockquote { level, content } => {
            assert_eq!(*level, 3);
            assert_eq!(content.len(), 1);
            assert_eq!(
                content[0],
                Inline::Text {
                    content: "This is a deeply nested blockquote.".to_string()
                }
            );
        }
        _ => panic!("Expected Blockquote with level 3"),
    }
}

#[test]
fn test_blockquote_followed_by_paragraph() {
    let input = "> This is a blockquote.\n\nThis is a paragraph.".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 2);
    match &result[0] {
        Node::Blockquote { level, .. } => {
            assert_eq!(*level, 1);
        }
        _ => panic!("Expected Blockquote as first element"),
    }
    match &result[1] {
        Node::Paragraph { .. } => {}
        _ => panic!("Expected Paragraph as second element"),
    }
}

#[test]
fn test_blockquote_stops_at_empty_line() {
    let input = "> First line.\n> Second line.\n\n> Another blockquote.".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 2);
    match &result[0] {
        Node::Blockquote { level, .. } => {
            assert_eq!(*level, 1);
        }
        _ => panic!("Expected first Blockquote"),
    }
    match &result[1] {
        Node::Blockquote { level, .. } => {
            assert_eq!(*level, 1);
        }
        _ => panic!("Expected second Blockquote"),
    }
}

#[test]
fn test_multiple_separate_blockquotes() {
    let input = "> First blockquote.\n\n> Second blockquote.\n\n> Third blockquote.".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 3);
    for node in &result {
        match node {
            Node::Blockquote { level, .. } => {
                assert_eq!(*level, 1);
            }
            _ => panic!("Expected all Blockquote elements"),
        }
    }
}

#[test]
fn test_blockquote_stops_at_different_nesting_level() {
    let input = "> First level.\n>> Second level.".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 2);
    match &result[0] {
        Node::Blockquote { level, .. } => {
            assert_eq!(*level, 1);
        }
        _ => panic!("Expected first Blockquote with level 1"),
    }
    match &result[1] {
        Node::Blockquote { level, .. } => {
            assert_eq!(*level, 2);
        }
        _ => panic!("Expected second Blockquote with level 2"),
    }
}

#[test]
fn test_blockquote_stops_at_heading() {
    let input = "> This is a blockquote.\n# This is a heading".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 2);
    match &result[0] {
        Node::Blockquote { .. } => {}
        _ => panic!("Expected Blockquote"),
    }
    match &result[1] {
        Node::Heading { .. } => {}
        _ => panic!("Expected Heading"),
    }
}

#[test]
fn test_blockquote_stops_at_list() {
    let input = "> This is a blockquote.\n- This is a list item".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 2);
    match &result[0] {
        Node::Blockquote { .. } => {}
        _ => panic!("Expected Blockquote"),
    }
    match &result[1] {
        Node::UnorderedList { .. } => {}
        _ => panic!("Expected UnorderedList"),
    }
}

#[test]
fn test_blockquote_stops_at_code_block() {
    let input = "> This is a blockquote.\n```rust\nfn main() {}\n```".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 2);
    match &result[0] {
        Node::Blockquote { .. } => {}
        _ => panic!("Expected Blockquote"),
    }
    match &result[1] {
        Node::CodeBlock { .. } => {}
        _ => panic!("Expected CodeBlock"),
    }
}

#[test]
fn test_blockquote_with_space_after_gt() {
    let input = ">  This is a blockquote with space after >.".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::Blockquote { level, content } => {
            assert_eq!(*level, 1);
            assert!(!content.is_empty());
        }
        _ => panic!("Expected Blockquote"),
    }
}

#[test]
fn test_blockquote_without_space_after_gt() {
    let input = ">This is a blockquote without space after >.".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::Blockquote { level, content } => {
            assert_eq!(*level, 1);
            assert!(!content.is_empty());
        }
        _ => panic!("Expected Blockquote"),
    }
}

#[test]
fn test_empty_blockquote() {
    let input = ">".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::Blockquote { level, content } => {
            assert_eq!(*level, 1);
            assert!(content.is_empty());
        }
        _ => panic!("Expected Blockquote"),
    }
}

#[test]
fn test_blockquote_with_only_whitespace() {
    let input = ">   ".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::Blockquote { level, content } => {
            assert_eq!(*level, 1);
            // Whitespace-only content should result in empty or minimal content
            assert!(content.is_empty() || content.iter().all(|inline| {
                matches!(inline, Inline::Text { content } if content.trim().is_empty())
            }));
        }
        _ => panic!("Expected Blockquote"),
    }
}

#[test]
fn test_blockquote_mixed_with_other_elements() {
    let input = "# Heading\n\n> Blockquote here.\n\nParagraph text.".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 3);
    match &result[0] {
        Node::Heading { .. } => {}
        _ => panic!("Expected Heading"),
    }
    match &result[1] {
        Node::Blockquote { .. } => {}
        _ => panic!("Expected Blockquote"),
    }
    match &result[2] {
        Node::Paragraph { .. } => {}
        _ => panic!("Expected Paragraph"),
    }
}
