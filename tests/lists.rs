use md_parser::{Inline, Node, Parser};

#[test]
fn test_unordered_list_simple() {
    let input = "- one\n- two".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::UnorderedList { items } => {
            assert_eq!(items.len(), 2);
            assert_eq!(items[0].content.len(), 1);
            assert_eq!(
                items[0].content[0],
                Inline::Text {
                    content: "one".to_string()
                }
            );
            assert_eq!(items[1].content.len(), 1);
            assert_eq!(
                items[1].content[0],
                Inline::Text {
                    content: "two".to_string()
                }
            );
            assert!(items[0].children.is_empty());
            assert!(items[1].children.is_empty());
        }
        _ => panic!("Expected UnorderedList, got {:?}", result[0]),
    }
}

#[test]
fn test_unordered_list_markers() {
    // Test * marker
    let input = "* a\n* b".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::UnorderedList { items } => {
            assert_eq!(items.len(), 2);
        }
        _ => panic!("Expected UnorderedList"),
    }

    // Test + marker
    let input = "+ x\n+ y".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::UnorderedList { items } => {
            assert_eq!(items.len(), 2);
        }
        _ => panic!("Expected UnorderedList"),
    }
}

#[test]
fn test_nested_list_two_levels() {
    let input = "- a\n  - b\n  - c".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::UnorderedList { items } => {
            assert_eq!(items.len(), 1);
            assert_eq!(items[0].content.len(), 1);
            assert_eq!(
                items[0].content[0],
                Inline::Text {
                    content: "a".to_string()
                }
            );
            assert_eq!(items[0].children.len(), 2);
            assert_eq!(items[0].children[0].content.len(), 1);
            assert_eq!(
                items[0].children[0].content[0],
                Inline::Text {
                    content: "b".to_string()
                }
            );
            assert_eq!(items[0].children[1].content.len(), 1);
            assert_eq!(
                items[0].children[1].content[0],
                Inline::Text {
                    content: "c".to_string()
                }
            );
        }
        _ => panic!("Expected UnorderedList, got {:?}", result[0]),
    }
}

#[test]
fn test_nested_list_three_levels() {
    let input = "- a\n  - b\n    - c".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::UnorderedList { items } => {
            assert_eq!(items.len(), 1);
            assert_eq!(items[0].children.len(), 1);
            assert_eq!(items[0].children[0].children.len(), 1);
            assert_eq!(
                items[0].children[0].children[0].content[0],
                Inline::Text {
                    content: "c".to_string()
                }
            );
        }
        _ => panic!("Expected UnorderedList"),
    }
}

#[test]
fn test_list_then_paragraph() {
    let input = "- one\n- two\n\nSome paragraph.".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 2);
    match &result[0] {
        Node::UnorderedList { items } => {
            assert_eq!(items.len(), 2);
        }
        _ => panic!("Expected UnorderedList first"),
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
        _ => panic!("Expected Paragraph second"),
    }
}

#[test]
fn test_list_then_heading() {
    let input = "- item\n\n# Heading".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 2);
    match &result[0] {
        Node::UnorderedList { .. } => {}
        _ => panic!("Expected UnorderedList first"),
    }
    match &result[1] {
        Node::Heading { level, .. } => {
            assert_eq!(*level, 1);
        }
        _ => panic!("Expected Heading second"),
    }
}

#[test]
fn test_list_then_code_block() {
    let input = "- item\n\n```rust\nfn main() {}\n```".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 2);
    match &result[0] {
        Node::UnorderedList { .. } => {}
        _ => panic!("Expected UnorderedList first"),
    }
    match &result[1] {
        Node::CodeBlock { lang, .. } => {
            assert_eq!(lang.as_ref(), Some(&"rust".to_string()));
        }
        _ => panic!("Expected CodeBlock second"),
    }
}

#[test]
fn test_list_item_inline_formatting() {
    let input = "- **bold**\n- [text](url)".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::UnorderedList { items } => {
            assert_eq!(items.len(), 2);
            // First item should have bold
            match &items[0].content[0] {
                Inline::Bold { .. } => {}
                _ => panic!("Expected Bold in first item"),
            }
            // Second item should have link
            match &items[1].content[0] {
                Inline::Link { .. } => {}
                _ => panic!("Expected Link in second item"),
            }
        }
        _ => panic!("Expected UnorderedList"),
    }
}

#[test]
fn test_empty_list_item() {
    let input = "- ".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::UnorderedList { items } => {
            assert_eq!(items.len(), 1);
            assert!(items[0].content.is_empty());
        }
        _ => panic!("Expected UnorderedList"),
    }
}

#[test]
fn test_list_continuation() {
    let input = "- one\n  two".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::UnorderedList { items } => {
            assert_eq!(items.len(), 1);
            // Content should include both "one" and "two"
            let content_text: String = items[0]
                .content
                .iter()
                .filter_map(|inline| {
                    if let Inline::Text { content } = inline {
                        Some(content.as_str())
                    } else {
                        None
                    }
                })
                .collect();
            assert!(content_text.contains("one"));
            assert!(content_text.contains("two"));
        }
        _ => panic!("Expected UnorderedList"),
    }
}

#[test]
fn test_ordered_list_simple() {
    let input = "1. one\n2. two\n3. three".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::OrderedList { items } => {
            assert_eq!(items.len(), 3);
            assert_eq!(items[0].content.len(), 1);
            assert_eq!(
                items[0].content[0],
                Inline::Text {
                    content: "one".to_string()
                }
            );
            assert_eq!(items[1].content.len(), 1);
            assert_eq!(
                items[1].content[0],
                Inline::Text {
                    content: "two".to_string()
                }
            );
            assert_eq!(items[2].content.len(), 1);
            assert_eq!(
                items[2].content[0],
                Inline::Text {
                    content: "three".to_string()
                }
            );
            assert!(items[0].children.is_empty());
            assert!(items[1].children.is_empty());
            assert!(items[2].children.is_empty());
        }
        _ => panic!("Expected OrderedList, got {:?}", result[0]),
    }
}

#[test]
fn test_ordered_list_numbers() {
    // Test with different starting numbers
    let input = "5. five\n6. six\n7. seven".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::OrderedList { items } => {
            assert_eq!(items.len(), 3);
        }
        _ => panic!("Expected OrderedList"),
    }

    // Test with non-sequential numbers (should still work)
    let input = "1. first\n5. fifth\n3. third".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::OrderedList { items } => {
            assert_eq!(items.len(), 3);
        }
        _ => panic!("Expected OrderedList"),
    }
}

#[test]
fn test_ordered_list_empty_item() {
    let input = "1. ".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::OrderedList { items } => {
            assert_eq!(items.len(), 1);
            assert!(items[0].content.is_empty());
        }
        _ => panic!("Expected OrderedList"),
    }
}

#[test]
fn test_ordered_list_nested_two_levels() {
    let input = "1. a\n  1. b\n  2. c".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::OrderedList { items } => {
            assert_eq!(items.len(), 1);
            assert_eq!(items[0].content.len(), 1);
            assert_eq!(
                items[0].content[0],
                Inline::Text {
                    content: "a".to_string()
                }
            );
            assert_eq!(items[0].children.len(), 2);
            assert_eq!(items[0].children[0].content.len(), 1);
            assert_eq!(
                items[0].children[0].content[0],
                Inline::Text {
                    content: "b".to_string()
                }
            );
            assert_eq!(items[0].children[1].content.len(), 1);
            assert_eq!(
                items[0].children[1].content[0],
                Inline::Text {
                    content: "c".to_string()
                }
            );
        }
        _ => panic!("Expected OrderedList, got {:?}", result[0]),
    }
}

#[test]
fn test_ordered_list_nested_three_levels() {
    let input = "1. a\n  1. b\n    1. c".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::OrderedList { items } => {
            assert_eq!(items.len(), 1);
            assert_eq!(items[0].children.len(), 1);
            assert_eq!(items[0].children[0].children.len(), 1);
            assert_eq!(
                items[0].children[0].children[0].content[0],
                Inline::Text {
                    content: "c".to_string()
                }
            );
        }
        _ => panic!("Expected OrderedList"),
    }
}

#[test]
fn test_ordered_list_mixed_with_unordered() {
    let input = "1. first\n   - nested unordered\n   - another".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::OrderedList { items } => {
            assert_eq!(items.len(), 1);
            assert_eq!(items[0].children.len(), 2);
            // Children should be parsed as list items (they'll be in the children vector)
        }
        _ => panic!("Expected OrderedList"),
    }
}

#[test]
fn test_ordered_list_continuation() {
    let input = "1. one\n   two".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::OrderedList { items } => {
            assert_eq!(items.len(), 1);
            // Content should include both "one" and "two"
            let content_text: String = items[0]
                .content
                .iter()
                .filter_map(|inline| {
                    if let Inline::Text { content } = inline {
                        Some(content.as_str())
                    } else {
                        None
                    }
                })
                .collect();
            assert!(content_text.contains("one"));
            assert!(content_text.contains("two"));
        }
        _ => panic!("Expected OrderedList"),
    }
}

#[test]
fn test_ordered_list_inline_formatting() {
    let input = "1. **bold**\n2. [text](url)".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::OrderedList { items } => {
            assert_eq!(items.len(), 2);
            // First item should have bold
            match &items[0].content[0] {
                Inline::Bold { .. } => {}
                _ => panic!("Expected Bold in first item"),
            }
            // Second item should have link
            match &items[1].content[0] {
                Inline::Link { .. } => {}
                _ => panic!("Expected Link in second item"),
            }
        }
        _ => panic!("Expected OrderedList"),
    }
}

#[test]
fn test_ordered_list_then_paragraph() {
    let input = "1. one\n2. two\n\nSome paragraph.".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 2);
    match &result[0] {
        Node::OrderedList { items } => {
            assert_eq!(items.len(), 2);
        }
        _ => panic!("Expected OrderedList first"),
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
        _ => panic!("Expected Paragraph second"),
    }
}

#[test]
fn test_ordered_list_then_heading() {
    let input = "1. item\n\n# Heading".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 2);
    match &result[0] {
        Node::OrderedList { .. } => {}
        _ => panic!("Expected OrderedList first"),
    }
    match &result[1] {
        Node::Heading { level, .. } => {
            assert_eq!(*level, 1);
        }
        _ => panic!("Expected Heading second"),
    }
}

#[test]
fn test_ordered_list_then_code_block() {
    let input = "1. item\n\n```rust\nfn main() {}\n```".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 2);
    match &result[0] {
        Node::OrderedList { .. } => {}
        _ => panic!("Expected OrderedList first"),
    }
    match &result[1] {
        Node::CodeBlock { lang, .. } => {
            assert_eq!(lang.as_ref(), Some(&"rust".to_string()));
        }
        _ => panic!("Expected CodeBlock second"),
    }
}

#[test]
fn test_ordered_list_then_table() {
    let input = "1. item\n\n| Header |\n|--------|\n| Cell   |".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 2);
    match &result[0] {
        Node::OrderedList { .. } => {}
        _ => panic!("Expected OrderedList first"),
    }
    match &result[1] {
        Node::Table { .. } => {}
        _ => panic!("Expected Table second"),
    }
}

#[test]
fn test_ordered_list_then_blockquote() {
    let input = "1. item\n\n> Quote".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 2);
    match &result[0] {
        Node::OrderedList { .. } => {}
        _ => panic!("Expected OrderedList first"),
    }
    match &result[1] {
        Node::Blockquote { .. } => {}
        _ => panic!("Expected Blockquote second"),
    }
}
