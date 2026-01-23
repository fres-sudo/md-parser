use md_parser::{Node, Parser};

#[test]
fn test_simple_horizontal_rule_dashes() {
    let input = "---".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::HorizontalRule => {}
        _ => panic!("Expected HorizontalRule"),
    }
}

#[test]
fn test_simple_horizontal_rule_asterisks() {
    let input = "***".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::HorizontalRule => {}
        _ => panic!("Expected HorizontalRule"),
    }
}

#[test]
fn test_horizontal_rule_more_than_three() {
    let input = "----".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::HorizontalRule => {}
        _ => panic!("Expected HorizontalRule"),
    }
}

#[test]
fn test_horizontal_rule_many_asterisks() {
    let input = "*****".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::HorizontalRule => {}
        _ => panic!("Expected HorizontalRule"),
    }
}

#[test]
fn test_horizontal_rule_with_spaces() {
    let input = "  ---  ".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::HorizontalRule => {}
        _ => panic!("Expected HorizontalRule"),
    }
}

#[test]
fn test_horizontal_rule_asterisks_with_spaces() {
    let input = "  ***  ".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::HorizontalRule => {}
        _ => panic!("Expected HorizontalRule"),
    }
}

#[test]
fn test_horizontal_rule_between_paragraphs() {
    let input = "First paragraph.\n\n---\n\nSecond paragraph.".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 3);
    match &result[0] {
        Node::Paragraph { .. } => {}
        _ => panic!("Expected Paragraph as first element"),
    }
    match &result[1] {
        Node::HorizontalRule => {}
        _ => panic!("Expected HorizontalRule as second element"),
    }
    match &result[2] {
        Node::Paragraph { .. } => {}
        _ => panic!("Expected Paragraph as third element"),
    }
}

#[test]
fn test_multiple_horizontal_rules() {
    let input = "---\n\n***\n\n----".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 3);
    for node in &result {
        match node {
            Node::HorizontalRule => {}
            _ => panic!("Expected all HorizontalRule elements"),
        }
    }
}

#[test]
fn test_horizontal_rule_after_heading() {
    let input = "# Heading\n\n---".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 2);
    match &result[0] {
        Node::Heading { .. } => {}
        _ => panic!("Expected Heading as first element"),
    }
    match &result[1] {
        Node::HorizontalRule => {}
        _ => panic!("Expected HorizontalRule as second element"),
    }
}

#[test]
fn test_horizontal_rule_before_list() {
    let input = "---\n\n- List item".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 2);
    match &result[0] {
        Node::HorizontalRule => {}
        _ => panic!("Expected HorizontalRule as first element"),
    }
    match &result[1] {
        Node::UnorderedList { .. } => {}
        _ => panic!("Expected UnorderedList as second element"),
    }
}

#[test]
fn test_horizontal_rule_after_list() {
    let input = "- List item\n\n---".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 2);
    match &result[0] {
        Node::UnorderedList { .. } => {}
        _ => panic!("Expected UnorderedList as first element"),
    }
    match &result[1] {
        Node::HorizontalRule => {}
        _ => panic!("Expected HorizontalRule as second element"),
    }
}

#[test]
fn test_horizontal_rule_with_paragraph_before_and_after() {
    let input = "Paragraph one.\n\n---\n\nParagraph two.".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 3);
    match &result[0] {
        Node::Paragraph { .. } => {}
        _ => panic!("Expected Paragraph as first element"),
    }
    match &result[1] {
        Node::HorizontalRule => {}
        _ => panic!("Expected HorizontalRule as second element"),
    }
    match &result[2] {
        Node::Paragraph { .. } => {}
        _ => panic!("Expected Paragraph as third element"),
    }
}

#[test]
fn test_reject_less_than_three_dashes() {
    let input = "--".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::Paragraph { .. } => {}
        _ => panic!("Expected Paragraph, not HorizontalRule"),
    }
}

#[test]
fn test_reject_less_than_three_asterisks() {
    let input = "**".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::Paragraph { .. } => {}
        _ => panic!("Expected Paragraph, not HorizontalRule"),
    }
}

#[test]
fn test_reject_mixed_characters() {
    let input = "---text---".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::Paragraph { .. } => {}
        _ => panic!("Expected Paragraph, not HorizontalRule"),
    }
}

#[test]
fn test_reject_mixed_dashes_and_asterisks() {
    let input = "---***".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::Paragraph { .. } => {}
        _ => panic!("Expected Paragraph, not HorizontalRule"),
    }
}

#[test]
fn test_horizontal_rule_in_complex_document() {
    let input = "# Title\n\nFirst paragraph.\n\n---\n\n## Subtitle\n\nSecond paragraph.\n\n***\n\n- List item 1\n- List item 2".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    // Should have: Heading, Paragraph, HorizontalRule, Heading, Paragraph, HorizontalRule, UnorderedList
    assert_eq!(result.len(), 7);
    
    // Check first horizontal rule
    match &result[2] {
        Node::HorizontalRule => {}
        _ => panic!("Expected HorizontalRule at position 2"),
    }
    
    // Check second horizontal rule
    match &result[5] {
        Node::HorizontalRule => {}
        _ => panic!("Expected HorizontalRule at position 5"),
    }
}

#[test]
fn test_horizontal_rule_rendering() {
    let input = "---".to_string();
    let mut parser = Parser::new(input).unwrap();
    let html = parser.to_html().unwrap();

    assert!(html.contains("<hr>"));
}

#[test]
fn test_horizontal_rule_rendering_asterisks() {
    let input = "***".to_string();
    let mut parser = Parser::new(input).unwrap();
    let html = parser.to_html().unwrap();

    assert!(html.contains("<hr>"));
}
