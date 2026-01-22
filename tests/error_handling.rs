use md_parser::{ParseError, Parser};

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
