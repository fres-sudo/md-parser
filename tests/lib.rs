use md_parser::{Inline, Node, ParseError, Parser};

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

// Phase 2 Tests

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

// Phase 3 Tests

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

// Phase 4 Tests - Lists

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

// Error handling tests: invalid heading level, unclosed code block, line info in errors

#[test]
fn test_invalid_heading_level() {
    let input = "####### foo".to_string();
    let mut parser = Parser::new(input).unwrap();
    let err = parser.parse().unwrap_err();

    match &err {
        ParseError::InvalidHeadingLevel { level, span } => {
            assert!(*level > 6, "expected level > 6, got {}", level);
            assert_eq!(span.line, 1);
            assert_eq!(span.column, None);
        }
        _ => panic!("expected InvalidHeadingLevel, got {:?}", err),
    }

    let msg = format!("{}", err);
    assert!(
        msg.contains("line 1"),
        "error message should include line: {}",
        msg
    );
    assert!(
        msg.contains("invalid heading level"),
        "error message should describe error: {}",
        msg
    );
}

#[test]
fn test_invalid_heading_level_line_number() {
    let input = "para\n\n####### bar".to_string();
    let mut parser = Parser::new(input).unwrap();
    let err = parser.parse().unwrap_err();

    match &err {
        ParseError::InvalidHeadingLevel { level, span } => {
            assert!(*level > 6);
            assert_eq!(span.line, 3, "heading is on line 3");
        }
        _ => panic!("expected InvalidHeadingLevel, got {:?}", err),
    }
}

#[test]
fn test_unclosed_code_block() {
    let input = "```\ncode\n".to_string();
    let mut parser = Parser::new(input).unwrap();
    let err = parser.parse().unwrap_err();

    match &err {
        ParseError::UnclosedCodeBlock { span } => {
            assert_eq!(span.line, 1);
            assert_eq!(span.column, None);
        }
        _ => panic!("expected UnclosedCodeBlock, got {:?}", err),
    }

    let msg = format!("{}", err);
    assert!(
        msg.contains("line 1"),
        "error message should include line: {}",
        msg
    );
    assert!(
        msg.contains("unclosed code block"),
        "error message should describe error: {}",
        msg
    );
}

#[test]
fn test_unclosed_code_block_line_number() {
    let input = "text\n\n```rust\nfn main() {}\n".to_string();
    let mut parser = Parser::new(input).unwrap();
    let err = parser.parse().unwrap_err();

    match &err {
        ParseError::UnclosedCodeBlock { span } => {
            assert_eq!(span.line, 3, "opening ``` is on line 3");
        }
        _ => panic!("expected UnclosedCodeBlock, got {:?}", err),
    }
}

// Task list tests

#[test]
fn test_task_list_unchecked() {
    let input = "- [ ] task 1\n- [ ] task 2".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::UnorderedList { items } => {
            assert_eq!(items.len(), 2);
            assert_eq!(items[0].checked, Some(false));
            assert_eq!(items[1].checked, Some(false));
            assert_eq!(
                items[0].content[0],
                Inline::Text {
                    content: "task 1".to_string()
                }
            );
            assert_eq!(
                items[1].content[0],
                Inline::Text {
                    content: "task 2".to_string()
                }
            );
        }
        _ => panic!("Expected UnorderedList, got {:?}", result[0]),
    }
}

#[test]
fn test_task_list_checked() {
    let input = "- [x] completed task".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::UnorderedList { items } => {
            assert_eq!(items.len(), 1);
            assert_eq!(items[0].checked, Some(true));
            assert_eq!(
                items[0].content[0],
                Inline::Text {
                    content: "completed task".to_string()
                }
            );
        }
        _ => panic!("Expected UnorderedList, got {:?}", result[0]),
    }
}

#[test]
fn test_task_list_case_insensitive() {
    let input = "- [X] uppercase task".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::UnorderedList { items } => {
            assert_eq!(items.len(), 1);
            assert_eq!(items[0].checked, Some(true));
            assert_eq!(
                items[0].content[0],
                Inline::Text {
                    content: "uppercase task".to_string()
                }
            );
        }
        _ => panic!("Expected UnorderedList, got {:?}", result[0]),
    }
}

#[test]
fn test_task_list_mixed() {
    let input = "- [x] done\n- [ ] todo\n- [x] also done".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::UnorderedList { items } => {
            assert_eq!(items.len(), 3);
            assert_eq!(items[0].checked, Some(true));
            assert_eq!(items[1].checked, Some(false));
            assert_eq!(items[2].checked, Some(true));
        }
        _ => panic!("Expected UnorderedList, got {:?}", result[0]),
    }
}

#[test]
fn test_task_list_with_regular_items() {
    let input = "- [x] task item\n- regular item\n- [ ] another task".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::UnorderedList { items } => {
            assert_eq!(items.len(), 3);
            assert_eq!(items[0].checked, Some(true));
            assert_eq!(items[1].checked, None); // Regular list item
            assert_eq!(items[2].checked, Some(false));
        }
        _ => panic!("Expected UnorderedList, got {:?}", result[0]),
    }
}

#[test]
fn test_task_list_nested_regular() {
    let input = "- [x] parent task\n  - child item\n  - [ ] child task".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::UnorderedList { items } => {
            assert_eq!(items.len(), 1);
            assert_eq!(items[0].checked, Some(true));
            assert_eq!(items[0].children.len(), 2);
            assert_eq!(items[0].children[0].checked, None); // Regular child
            assert_eq!(items[0].children[1].checked, Some(false)); // Task child
        }
        _ => panic!("Expected UnorderedList, got {:?}", result[0]),
    }
}

#[test]
fn test_task_list_continuation() {
    let input = "- [x] task with\n  continuation text".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::UnorderedList { items } => {
            assert_eq!(items.len(), 1);
            assert_eq!(items[0].checked, Some(true));
            // Content should include both lines
            let content_text: String = items[0]
                .content
                .iter()
                .map(|i| match i {
                    Inline::Text { content } => content.clone(),
                    _ => String::new(),
                })
                .collect();
            assert!(content_text.contains("task with"));
            assert!(content_text.contains("continuation text"));
        }
        _ => panic!("Expected UnorderedList, got {:?}", result[0]),
    }
}

#[test]
fn test_task_list_empty() {
    let input = "- [ ]\n- [x]".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::UnorderedList { items } => {
            assert_eq!(items.len(), 2);
            assert_eq!(items[0].checked, Some(false));
            assert_eq!(items[1].checked, Some(true));
            assert!(items[0].content.is_empty());
            assert!(items[1].content.is_empty());
        }
        _ => panic!("Expected UnorderedList, got {:?}", result[0]),
    }
}

#[test]
fn test_task_list_nested_tasks() {
    let input = "- [ ] parent\n  - [x] child task\n  - [ ] another child".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::UnorderedList { items } => {
            assert_eq!(items.len(), 1);
            assert_eq!(items[0].checked, Some(false));
            assert_eq!(items[0].children.len(), 2);
            assert_eq!(items[0].children[0].checked, Some(true));
            assert_eq!(items[0].children[1].checked, Some(false));
        }
        _ => panic!("Expected UnorderedList, got {:?}", result[0]),
    }
}

// Phase 5 Tests - Tables

#[test]
fn test_simple_table() {
    let input = "| Header 1 | Header 2 |\n|----------|----------|\n| Cell 1   | Cell 2   |".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::Table { headers, rows, alignments } => {
            assert_eq!(headers.len(), 2);
            assert_eq!(rows.len(), 1);
            assert_eq!(alignments.len(), 2);
            // Check headers
            assert_eq!(headers[0].len(), 1);
            assert_eq!(
                headers[0][0],
                Inline::Text {
                    content: "Header 1".to_string()
                }
            );
            assert_eq!(headers[1].len(), 1);
            assert_eq!(
                headers[1][0],
                Inline::Text {
                    content: "Header 2".to_string()
                }
            );
            // Check rows
            assert_eq!(rows[0].len(), 2);
            assert_eq!(
                rows[0][0][0],
                Inline::Text {
                    content: "Cell 1".to_string()
                }
            );
            assert_eq!(
                rows[0][1][0],
                Inline::Text {
                    content: "Cell 2".to_string()
                }
            );
        }
        _ => panic!("Expected Table, got {:?}", result[0]),
    }
}

#[test]
fn test_table_with_alignment() {
    let input = "| Left | Center | Right |\n|:-----|:------:|------:|\n| L    | C      | R     |".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::Table { headers, rows, alignments } => {
            assert_eq!(headers.len(), 3);
            assert_eq!(rows.len(), 1);
            assert_eq!(alignments.len(), 3);
            // Check alignments: left, center, right
            use md_parser::Alignment;
            assert_eq!(alignments[0], Some(Alignment::Left));
            assert_eq!(alignments[1], Some(Alignment::Center));
            assert_eq!(alignments[2], Some(Alignment::Right));
        }
        _ => panic!("Expected Table, got {:?}", result[0]),
    }
}

#[test]
fn test_table_with_inline_formatting() {
    let input = "| **Bold** | *Italic* | [Link](url) |\n|----------|----------|-------------|\n| Text | More | Here |".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::Table { headers, rows: _, alignments: _ } => {
            assert_eq!(headers.len(), 3);
            // First header should have bold
            match &headers[0][0] {
                Inline::Bold { .. } => {}
                _ => panic!("Expected Bold in first header"),
            }
            // Second header should have italic
            match &headers[1][0] {
                Inline::Italic { .. } => {}
                _ => panic!("Expected Italic in second header"),
            }
            // Third header should have link
            match &headers[2][0] {
                Inline::Link { .. } => {}
                _ => panic!("Expected Link in third header"),
            }
        }
        _ => panic!("Expected Table, got {:?}", result[0]),
    }
}

#[test]
fn test_table_with_empty_cells() {
    let input = "| Col 1 | Col 2 | Col 3 |\n|-------|-------|-------|\n| A     |       | C     |\n|       | B     |       |".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::Table { headers, rows, alignments: _ } => {
            assert_eq!(headers.len(), 3);
            assert_eq!(rows.len(), 2);
            // First row: A, empty, C
            assert_eq!(rows[0].len(), 3);
            assert!(!rows[0][0].is_empty());
            assert!(rows[0][1].is_empty()); // Empty cell
            assert!(!rows[0][2].is_empty());
            // Second row: empty, B, empty
            assert_eq!(rows[1].len(), 3);
            assert!(rows[1][0].is_empty());
            assert!(!rows[1][1].is_empty());
            assert!(rows[1][2].is_empty());
        }
        _ => panic!("Expected Table, got {:?}", result[0]),
    }
}

#[test]
fn test_table_followed_by_paragraph() {
    let input = "| A | B |\n|---|---|\n| 1 | 2 |\n\nSome paragraph text.".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 2);
    match &result[0] {
        Node::Table { .. } => {}
        _ => panic!("Expected Table first"),
    }
    match &result[1] {
        Node::Paragraph { content } => {
            assert_eq!(content.len(), 1);
            assert_eq!(
                content[0],
                Inline::Text {
                    content: "Some paragraph text.".to_string()
                }
            );
        }
        _ => panic!("Expected Paragraph second"),
    }
}

#[test]
fn test_table_preceded_by_heading() {
    let input = "# Title\n\n| A | B |\n|---|---|\n| 1 | 2 |".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 2);
    match &result[0] {
        Node::Heading { level, .. } => {
            assert_eq!(*level, 1);
        }
        _ => panic!("Expected Heading first"),
    }
    match &result[1] {
        Node::Table { .. } => {}
        _ => panic!("Expected Table second"),
    }
}

#[test]
fn test_table_multiple_rows() {
    let input = "| Name | Age |\n|------|-----|\n| Alice | 30 |\n| Bob   | 25 |\n| Carol | 35 |".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::Table { headers, rows, alignments: _ } => {
            assert_eq!(headers.len(), 2);
            assert_eq!(rows.len(), 3);
            // Check first data row
            assert_eq!(rows[0].len(), 2);
            let name_text: String = rows[0][0]
                .iter()
                .filter_map(|i| {
                    if let Inline::Text { content } = i {
                        Some(content.as_str())
                    } else {
                        None
                    }
                })
                .collect();
            assert!(name_text.contains("Alice"));
        }
        _ => panic!("Expected Table, got {:?}", result[0]),
    }
}

#[test]
fn test_table_default_alignment() {
    let input = "| Col 1 | Col 2 |\n|-------|-------|\n| A     | B     |".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::Table { alignments, .. } => {
            assert_eq!(alignments.len(), 2);
            // Default alignment should be None (left-aligned by default)
            assert_eq!(alignments[0], None);
            assert_eq!(alignments[1], None);
        }
        _ => panic!("Expected Table, got {:?}", result[0]),
    }
}

#[test]
fn test_table_with_trailing_pipe() {
    let input = "| Header 1 | Header 2 |\n|----------|----------|\n| Cell 1   | Cell 2   |".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::Table { headers, rows, .. } => {
            assert_eq!(headers.len(), 2);
            assert_eq!(rows.len(), 1);
        }
        _ => panic!("Expected Table, got {:?}", result[0]),
    }
}

#[test]
fn test_table_without_trailing_pipe() {
    let input = "| Header 1 | Header 2\n|----------|----------\n| Cell 1   | Cell 2".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::Table { headers, rows, .. } => {
            assert_eq!(headers.len(), 2);
            assert_eq!(rows.len(), 1);
        }
        _ => panic!("Expected Table, got {:?}", result[0]),
    }
}
