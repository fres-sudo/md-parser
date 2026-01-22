use md_parser::{Alignment, Inline, Node, Parser};

#[test]
fn test_simple_table() {
    let input =
        "| Header 1 | Header 2 |\n|----------|----------|\n| Cell 1   | Cell 2   |".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::Table {
            headers,
            rows,
            alignments,
        } => {
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
    let input = "| Left | Center | Right |\n|:-----|:------:|------:|\n| L    | C      | R     |"
        .to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::Table {
            headers,
            rows,
            alignments,
        } => {
            assert_eq!(headers.len(), 3);
            assert_eq!(rows.len(), 1);
            assert_eq!(alignments.len(), 3);
            // Check alignments: left, center, right
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
        Node::Table {
            headers,
            rows: _,
            alignments: _,
        } => {
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
        Node::Table {
            headers,
            rows,
            alignments: _,
        } => {
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
    let input = "| Name | Age |\n|------|-----|\n| Alice | 30 |\n| Bob   | 25 |\n| Carol | 35 |"
        .to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::Table {
            headers,
            rows,
            alignments: _,
        } => {
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
    let input =
        "| Header 1 | Header 2 |\n|----------|----------|\n| Cell 1   | Cell 2   |".to_string();
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
