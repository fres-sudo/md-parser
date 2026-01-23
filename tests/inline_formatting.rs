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
            // Should have: "This is ", bold, "."
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
                    // Should have: "bold with ", italic, " inside"
                    assert_eq!(bold_inlines.len(), 3);
                    assert_eq!(
                        bold_inlines[0],
                        Inline::Text {
                            content: "bold with ".to_string()
                        }
                    );
                    // Verify italic is nested inside bold
                    match &bold_inlines[1] {
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
                        _ => panic!("Expected Italic nested inside Bold"),
                    }
                    assert_eq!(
                        bold_inlines[2],
                        Inline::Text {
                            content: " inside".to_string()
                        }
                    );
                }
                _ => panic!("Expected Bold"),
            }
            assert_eq!(
                inlines[2],
                Inline::Text {
                    content: ".".to_string()
                }
            );
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

#[test]
fn test_image() {
    let input = "Here's an image ![alt text](https://example.com/image.png).".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::Paragraph { content: inlines } => {
            assert_eq!(inlines.len(), 3);
            assert_eq!(
                inlines[0],
                Inline::Text {
                    content: "Here's an image ".to_string()
                }
            );
            match &inlines[1] {
                Inline::Image { alt, url } => {
                    assert_eq!(alt, "alt text");
                    assert_eq!(url, "https://example.com/image.png");
                }
                _ => panic!("Expected Image"),
            }
            assert_eq!(
                inlines[2],
                Inline::Text {
                    content: ".".to_string()
                }
            );
        }
        _ => panic!("Expected Paragraph"),
    }
}

#[test]
fn test_image_empty_alt() {
    let input = "![ ](https://example.com/image.png)".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::Paragraph { content: inlines } => {
            assert_eq!(inlines.len(), 1);
            match &inlines[0] {
                Inline::Image { alt, url } => {
                    assert_eq!(alt, " ");
                    assert_eq!(url, "https://example.com/image.png");
                }
                _ => panic!("Expected Image"),
            }
        }
        _ => panic!("Expected Paragraph"),
    }
}

#[test]
fn test_image_vs_link() {
    let input = "![image](url.png) and [link](url.html)".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::Paragraph { content: inlines } => {
            assert_eq!(inlines.len(), 3);
            // First should be image
            match &inlines[0] {
                Inline::Image { alt, url } => {
                    assert_eq!(alt, "image");
                    assert_eq!(url, "url.png");
                }
                _ => panic!("Expected Image"),
            }
            // Second should be text " and "
            assert_eq!(
                inlines[1],
                Inline::Text {
                    content: " and ".to_string()
                }
            );
            // Third should be link
            match &inlines[2] {
                Inline::Link { text, url } => {
                    assert_eq!(text.len(), 1);
                    assert_eq!(
                        text[0],
                        Inline::Text {
                            content: "link".to_string()
                        }
                    );
                    assert_eq!(url, "url.html");
                }
                _ => panic!("Expected Link"),
            }
        }
        _ => panic!("Expected Paragraph"),
    }
}

#[test]
fn test_image_with_mixed_inline() {
    let input = "See ![logo](logo.png) and visit [site](https://example.com) for **more** info."
        .to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::Paragraph { content: inlines } => {
            // Should have image, link, and bold elements
            let has_image = inlines
                .iter()
                .any(|inline| matches!(inline, Inline::Image { .. }));
            let has_link = inlines
                .iter()
                .any(|inline| matches!(inline, Inline::Link { .. }));
            let has_bold = inlines
                .iter()
                .any(|inline| matches!(inline, Inline::Bold { .. }));
            assert!(has_image, "Expected Image element");
            assert!(has_link, "Expected Link element");
            assert!(has_bold, "Expected Bold element");
        }
        _ => panic!("Expected Paragraph"),
    }
}

#[test]
fn test_image_in_heading() {
    let input = "# Header with ![icon](icon.png)".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::Heading { level, content } => {
            assert_eq!(*level, 1);
            let has_image = content
                .iter()
                .any(|inline| matches!(inline, Inline::Image { .. }));
            assert!(has_image, "Expected Image element in heading");
        }
        _ => panic!("Expected Heading"),
    }
}

#[test]
fn test_bold_with_italic_inside() {
    let input = "**bold *italic* text**".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::Paragraph { content: inlines } => {
            assert_eq!(inlines.len(), 1);
            match &inlines[0] {
                Inline::Bold {
                    content: bold_inlines,
                } => {
                    assert_eq!(bold_inlines.len(), 3);
                    assert_eq!(
                        bold_inlines[0],
                        Inline::Text {
                            content: "bold ".to_string()
                        }
                    );
                    match &bold_inlines[1] {
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
                        _ => panic!("Expected Italic nested inside Bold"),
                    }
                    assert_eq!(
                        bold_inlines[2],
                        Inline::Text {
                            content: " text".to_string()
                        }
                    );
                }
                _ => panic!("Expected Bold"),
            }
        }
        _ => panic!("Expected Paragraph"),
    }
}

#[test]
fn test_italic_with_bold_inside() {
    let input = "*italic **bold** text*".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::Paragraph { content: inlines } => {
            assert_eq!(inlines.len(), 1);
            match &inlines[0] {
                Inline::Italic {
                    content: italic_inlines,
                } => {
                    assert_eq!(italic_inlines.len(), 3);
                    assert_eq!(
                        italic_inlines[0],
                        Inline::Text {
                            content: "italic ".to_string()
                        }
                    );
                    match &italic_inlines[1] {
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
                        _ => panic!("Expected Bold nested inside Italic"),
                    }
                    assert_eq!(
                        italic_inlines[2],
                        Inline::Text {
                            content: " text".to_string()
                        }
                    );
                }
                _ => panic!("Expected Italic"),
            }
        }
        _ => panic!("Expected Paragraph"),
    }
}

#[test]
fn test_bold_with_multiple_italic_inside() {
    let input = "**bold *italic* and *more italic* text**".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::Paragraph { content: inlines } => {
            assert_eq!(inlines.len(), 1);
            match &inlines[0] {
                Inline::Bold {
                    content: bold_inlines,
                } => {
                    // Should have: "bold ", italic, " and ", italic, " text"
                    assert_eq!(bold_inlines.len(), 5);
                    assert_eq!(
                        bold_inlines[0],
                        Inline::Text {
                            content: "bold ".to_string()
                        }
                    );
                    // First italic
                    match &bold_inlines[1] {
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
                        _ => panic!("Expected Italic nested inside Bold"),
                    }
                    assert_eq!(
                        bold_inlines[2],
                        Inline::Text {
                            content: " and ".to_string()
                        }
                    );
                    // Second italic
                    match &bold_inlines[3] {
                        Inline::Italic {
                            content: italic_inlines,
                        } => {
                            assert_eq!(italic_inlines.len(), 1);
                            assert_eq!(
                                italic_inlines[0],
                                Inline::Text {
                                    content: "more italic".to_string()
                                }
                            );
                        }
                        _ => panic!("Expected Italic nested inside Bold"),
                    }
                    assert_eq!(
                        bold_inlines[4],
                        Inline::Text {
                            content: " text".to_string()
                        }
                    );
                }
                _ => panic!("Expected Bold"),
            }
        }
        _ => panic!("Expected Paragraph"),
    }
}

#[test]
fn test_italic_with_bold_inside_complex() {
    let input = "*italic **bold** and **more bold** text*".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::Paragraph { content: inlines } => {
            assert_eq!(inlines.len(), 1);
            match &inlines[0] {
                Inline::Italic {
                    content: italic_inlines,
                } => {
                    // Should have: "italic ", bold, " and ", bold, " text"
                    assert_eq!(italic_inlines.len(), 5);
                    assert_eq!(
                        italic_inlines[0],
                        Inline::Text {
                            content: "italic ".to_string()
                        }
                    );
                    // First bold
                    match &italic_inlines[1] {
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
                        _ => panic!("Expected Bold nested inside Italic"),
                    }
                    assert_eq!(
                        italic_inlines[2],
                        Inline::Text {
                            content: " and ".to_string()
                        }
                    );
                    // Second bold
                    match &italic_inlines[3] {
                        Inline::Bold {
                            content: bold_inlines,
                        } => {
                            assert_eq!(bold_inlines.len(), 1);
                            assert_eq!(
                                bold_inlines[0],
                                Inline::Text {
                                    content: "more bold".to_string()
                                }
                            );
                        }
                        _ => panic!("Expected Bold nested inside Italic"),
                    }
                    assert_eq!(
                        italic_inlines[4],
                        Inline::Text {
                            content: " text".to_string()
                        }
                    );
                }
                _ => panic!("Expected Italic"),
            }
        }
        _ => panic!("Expected Paragraph"),
    }
}

#[test]
fn test_multiple_nested_formats() {
    let input = "Start **bold *italic* text** and *italic **bold** text* end.".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::Paragraph { content: inlines } => {
            // Should have: "Start ", bold (with italic), " and ", italic (with bold), " end."
            assert!(inlines.len() >= 5);
            assert_eq!(
                inlines[0],
                Inline::Text {
                    content: "Start ".to_string()
                }
            );
            // First bold with italic
            match &inlines[1] {
                Inline::Bold {
                    content: bold_inlines,
                } => {
                    let has_italic = bold_inlines
                        .iter()
                        .any(|inline| matches!(inline, Inline::Italic { .. }));
                    assert!(has_italic, "Expected Italic nested inside first Bold");
                }
                _ => panic!("Expected Bold"),
            }
            assert_eq!(
                inlines[2],
                Inline::Text {
                    content: " and ".to_string()
                }
            );
            // Italic with bold
            match &inlines[3] {
                Inline::Italic {
                    content: italic_inlines,
                } => {
                    let has_bold = italic_inlines
                        .iter()
                        .any(|inline| matches!(inline, Inline::Bold { .. }));
                    assert!(has_bold, "Expected Bold nested inside Italic");
                }
                _ => panic!("Expected Italic"),
            }
            assert_eq!(
                inlines[4],
                Inline::Text {
                    content: " end.".to_string()
                }
            );
        }
        _ => panic!("Expected Paragraph"),
    }
}

#[test]
fn test_simple_bold_no_nesting() {
    let input = "This is **bold** text.".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::Paragraph { content: inlines } => {
            assert_eq!(inlines.len(), 3);
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
        }
        _ => panic!("Expected Paragraph"),
    }
}

#[test]
fn test_simple_italic_no_nesting() {
    let input = "This is *italic* text.".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::Paragraph { content: inlines } => {
            assert_eq!(inlines.len(), 3);
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
        }
        _ => panic!("Expected Paragraph"),
    }
}

// ========== Inline Code Tests ==========

#[test]
fn test_inline_code_simple() {
    let input = "This is `code` text.".to_string();
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
                Inline::Code { content } => {
                    assert_eq!(content, "code");
                }
                _ => panic!("Expected Code"),
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
fn test_inline_code_in_paragraph() {
    let input = "Use the `println!` macro to print.".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::Paragraph { content: inlines } => {
            assert!(inlines.len() >= 3);
            let has_code = inlines
                .iter()
                .any(|inline| matches!(inline, Inline::Code { content: _ }));
            assert!(has_code, "Expected Code element");
        }
        _ => panic!("Expected Paragraph"),
    }
}

#[test]
fn test_inline_code_at_start() {
    let input = "`code` at the start".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::Paragraph { content: inlines } => {
            assert!(inlines.len() >= 2);
            match &inlines[0] {
                Inline::Code { content } => {
                    assert_eq!(content, "code");
                }
                _ => panic!("Expected Code at start"),
            }
        }
        _ => panic!("Expected Paragraph"),
    }
}

#[test]
fn test_inline_code_at_end() {
    let input = "Text ends with `code`".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::Paragraph { content: inlines } => {
            assert!(inlines.len() >= 2);
            let last_idx = inlines.len() - 1;
            match &inlines[last_idx] {
                Inline::Code { content } => {
                    assert_eq!(content, "code");
                }
                _ => panic!("Expected Code at end"),
            }
        }
        _ => panic!("Expected Paragraph"),
    }
}

#[test]
fn test_inline_code_with_spaces() {
    let input = "Code with ` spaces inside ` works".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::Paragraph { content: inlines } => {
            let code_inline = inlines
                .iter()
                .find(|inline| matches!(inline, Inline::Code { .. }));
            match code_inline {
                Some(Inline::Code { content }) => {
                    assert_eq!(content, " spaces inside ");
                }
                _ => panic!("Expected Code element"),
            }
        }
        _ => panic!("Expected Paragraph"),
    }
}

#[test]
fn test_inline_code_special_chars() {
    let input = "HTML: `<div>&amp;</div>`".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::Paragraph { content: inlines } => {
            let code_inline = inlines
                .iter()
                .find(|inline| matches!(inline, Inline::Code { .. }));
            match code_inline {
                Some(Inline::Code { content }) => {
                    assert_eq!(content, "<div>&amp;</div>");
                }
                _ => panic!("Expected Code element"),
            }
        }
        _ => panic!("Expected Paragraph"),
    }
}

#[test]
fn test_multiple_inline_code() {
    let input = "Use `fn` and `let` keywords in Rust.".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::Paragraph { content: inlines } => {
            let code_count = inlines
                .iter()
                .filter(|inline| matches!(inline, Inline::Code { .. }))
                .count();
            assert_eq!(code_count, 2, "Expected 2 Code elements");
        }
        _ => panic!("Expected Paragraph"),
    }
}

#[test]
fn test_bold_with_inline_code() {
    let input = "**Bold with `code` inside**".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::Paragraph { content: inlines } => {
            assert_eq!(inlines.len(), 1);
            match &inlines[0] {
                Inline::Bold { content: bold_inlines } => {
                    let has_code = bold_inlines
                        .iter()
                        .any(|inline| matches!(inline, Inline::Code { .. }));
                    assert!(has_code, "Expected Code inside Bold");
                }
                _ => panic!("Expected Bold"),
            }
        }
        _ => panic!("Expected Paragraph"),
    }
}

#[test]
fn test_italic_with_inline_code() {
    let input = "*Italic with `code` inside*".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::Paragraph { content: inlines } => {
            assert_eq!(inlines.len(), 1);
            match &inlines[0] {
                Inline::Italic { content: italic_inlines } => {
                    let has_code = italic_inlines
                        .iter()
                        .any(|inline| matches!(inline, Inline::Code { .. }));
                    assert!(has_code, "Expected Code inside Italic");
                }
                _ => panic!("Expected Italic"),
            }
        }
        _ => panic!("Expected Paragraph"),
    }
}

#[test]
fn test_inline_code_in_heading() {
    let input = "# Heading with `code`".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::Heading { level, content } => {
            assert_eq!(*level, 1);
            let has_code = content
                .iter()
                .any(|inline| matches!(inline, Inline::Code { .. }));
            assert!(has_code, "Expected Code element in heading");
        }
        _ => panic!("Expected Heading"),
    }
}

#[test]
fn test_inline_code_in_link() {
    let input = "Link with [`code`](https://example.com)".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::Paragraph { content: inlines } => {
            let link_inline = inlines
                .iter()
                .find(|inline| matches!(inline, Inline::Link { .. }));
            match link_inline {
                Some(Inline::Link { text, .. }) => {
                    let has_code = text
                        .iter()
                        .any(|inline| matches!(inline, Inline::Code { .. }));
                    assert!(has_code, "Expected Code inside Link text");
                }
                _ => panic!("Expected Link element"),
            }
        }
        _ => panic!("Expected Paragraph"),
    }
}

#[test]
fn test_inline_code_in_list() {
    let input = "- Item with `code` inside".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::UnorderedList { items } => {
            assert_eq!(items.len(), 1);
            let has_code = items[0]
                .content
                .iter()
                .any(|inline| matches!(inline, Inline::Code { .. }));
            assert!(has_code, "Expected Code element in list item");
        }
        _ => panic!("Expected UnorderedList"),
    }
}
