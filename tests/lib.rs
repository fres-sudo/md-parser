use md_parser::{Inline, Node, Parser};

#[test]
fn test_simple_paragraph() {
    let input = "This is a simple paragraph.".to_string();
    let parser = Parser::new(input).unwrap();
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
    let parser = Parser::new(input).unwrap();
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
    let parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 0);
}

#[test]
fn test_whitespace_only() {
    let input = "   \n\n   ".to_string();
    let parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 0);
}

// Phase 2 Tests

#[test]
fn test_standard_code_block() {
    let input = "```rust\nfn main() {\n    println!(\"Hello\");\n}\n```".to_string();
    let parser = Parser::new(input).unwrap();
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
    let parser = Parser::new(input).unwrap();
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
    let parser = Parser::new(input).unwrap();
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
    let parser = Parser::new(input).unwrap();
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
    let parser = Parser::new(input).unwrap();
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
    let parser = Parser::new(input).unwrap();
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
    let parser = Parser::new(input).unwrap();
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
    let parser = Parser::new(input).unwrap();
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
    let parser = Parser::new(input).unwrap();
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
    let parser = Parser::new(input).unwrap();
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
    let parser = Parser::new(input).unwrap();
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
    let parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::Paragraph { content: inlines } => {
            // Should have at least "This is " text and a Bold element
            assert!(inlines.len() >= 1);
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
    let parser = Parser::new(input).unwrap();
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
    let parser = Parser::new(input).unwrap();
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
