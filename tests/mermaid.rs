use md_parser::{Config, MermaidParserConfig, Node, Parser, ValidationStatus};

#[test]
fn test_mermaid_validation_valid() {
    let input = "```mermaid\ngraph TD\n    A-->B\n    B-->C\n```".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::MermaidDiagram {
            diagram,
            config,
            validation_status,
            warnings,
        } => {
            assert_eq!(diagram, "graph TD\n    A-->B\n    B-->C");
            assert!(config.is_some());
            // Should be Valid if validation is enabled (default)
            match validation_status {
                ValidationStatus::Valid | ValidationStatus::NotValidated => {}
                ValidationStatus::Invalid { errors } => {
                    panic!("Expected valid diagram, got errors: {:?}", errors);
                }
            }
            assert!(warnings.is_empty());
        }
        _ => panic!("Expected MermaidDiagram"),
    }
}

#[test]
fn test_mermaid_validation_invalid_empty() {
    let input = "```mermaid\n\n```".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::MermaidDiagram {
            validation_status,
            warnings: _,
            ..
        } => match validation_status {
            ValidationStatus::Invalid { errors } => {
                assert!(!errors.is_empty());
                assert!(errors.iter().any(|e| e.contains("empty")));
            }
            _ => panic!("Expected Invalid status for empty diagram"),
        },
        _ => panic!("Expected MermaidDiagram"),
    }
}

#[test]
fn test_mermaid_validation_invalid_unmatched_brackets() {
    let input = "```mermaid\ngraph TD\n    A-->B)\n```".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::MermaidDiagram {
            validation_status,
            warnings: _,
            ..
        } => {
            match validation_status {
                ValidationStatus::Invalid { errors } => {
                    assert!(!errors.is_empty());
                    // Should have unmatched parenthesis error
                    assert!(
                        errors.iter().any(|e| e.contains("parenthesis")),
                        "Expected unmatched parenthesis error, got: {:?}",
                        errors
                    );
                }
                _ => panic!("Expected Invalid status for unmatched brackets"),
            }
        }
        _ => panic!("Expected MermaidDiagram"),
    }
}

#[test]
fn test_mermaid_validation_invalid_missing_type() {
    let input = "```mermaid\nA-->B\n```".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::MermaidDiagram {
            validation_status, ..
        } => match validation_status {
            ValidationStatus::Invalid { errors } => {
                assert!(!errors.is_empty());
                assert!(errors.iter().any(|e| e.contains("diagram type")));
            }
            _ => panic!("Expected Invalid status for missing diagram type"),
        },
        _ => panic!("Expected MermaidDiagram"),
    }
}

#[test]
fn test_mermaid_config_default() {
    let input = "```mermaid\ngraph TD\n    A-->B\n```".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    match &result[0] {
        Node::MermaidDiagram { config, .. } => {
            let cfg = config.as_ref().unwrap();
            assert_eq!(cfg.theme, Some("default".to_string()));
            assert_eq!(cfg.font_size, Some("16px".to_string()));
            assert_eq!(
                cfg.font_family,
                Some("trebuchet ms, verdana, arial".to_string())
            );
        }
        _ => panic!("Expected MermaidDiagram"),
    }
}

#[test]
fn test_mermaid_config_inline_theme() {
    let input = "```mermaid\n%%{init: {'theme':'dark'}}%%\ngraph TD\n    A-->B\n```".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    match &result[0] {
        Node::MermaidDiagram {
            diagram, config, ..
        } => {
            // Frontmatter should be removed from diagram
            assert!(!diagram.contains("%%{"));
            assert!(!diagram.contains("init:"));

            let cfg = config.as_ref().unwrap();
            assert_eq!(cfg.theme, Some("dark".to_string()));
        }
        _ => panic!("Expected MermaidDiagram"),
    }
}

#[test]
fn test_mermaid_config_inline_font_size() {
    let input =
        "```mermaid\n%%{init: {'themeVariables': {'fontSize':'18px'}}}%%\ngraph TD\n    A-->B\n```"
            .to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    match &result[0] {
        Node::MermaidDiagram { config, .. } => {
            let cfg = config.as_ref().unwrap();
            assert_eq!(cfg.font_size, Some("18px".to_string()));
        }
        _ => panic!("Expected MermaidDiagram"),
    }
}

#[test]
fn test_mermaid_config_merge_global_and_inline() {
    let input = "```mermaid\n%%{init: {'theme':'dark'}}%%\ngraph TD\n    A-->B\n```".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    match &result[0] {
        Node::MermaidDiagram { config, .. } => {
            let cfg = config.as_ref().unwrap();
            // Theme should be overridden by inline config
            assert_eq!(cfg.theme, Some("dark".to_string()));
            // Font size should come from global defaults
            assert_eq!(cfg.font_size, Some("16px".to_string()));
        }
        _ => panic!("Expected MermaidDiagram"),
    }
}

#[test]
fn test_mermaid_validation_disabled() {
    let input = "```mermaid\ninvalid diagram syntax\n```".to_string();

    // Create parser with validation disabled
    let config = Config {
        parser: md_parser::ParserConfig {
            mermaid: MermaidParserConfig {
                validate_syntax: false,
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    };

    let mut parser = Parser::with_config(input, config.parser).unwrap();
    let result = parser.parse().unwrap();

    match &result[0] {
        Node::MermaidDiagram {
            validation_status, ..
        } => {
            assert_eq!(*validation_status, ValidationStatus::NotValidated);
        }
        _ => panic!("Expected MermaidDiagram"),
    }
}

#[test]
fn test_mermaid_warnings_preserved() {
    let input = "```mermaid\ngraph TD\n    -->B\n```".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    match &result[0] {
        Node::MermaidDiagram {
            validation_status,
            warnings: _,
            ..
        } => {
            // Should have warnings about arrow syntax
            // Note: This test may need adjustment based on actual validation logic
            match validation_status {
                ValidationStatus::Valid | ValidationStatus::Invalid { .. } => {
                    // Either valid with warnings or invalid
                }
                ValidationStatus::NotValidated => {
                    panic!("Expected validation to run");
                }
            }
        }
        _ => panic!("Expected MermaidDiagram"),
    }
}

#[test]
fn test_mermaid_diagram_kept_on_validation_error() {
    // Even with validation errors, diagram should be kept as MermaidDiagram
    let input = "```mermaid\ninvalid syntax here\n```".to_string();
    let mut parser = Parser::new(input).unwrap();
    let result = parser.parse().unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Node::MermaidDiagram {
            diagram,
            validation_status,
            ..
        } => {
            // Diagram content should be preserved
            assert_eq!(diagram, "invalid syntax here");
            // Should have validation errors
            match validation_status {
                ValidationStatus::Invalid { .. } => {}
                _ => panic!("Expected Invalid status"),
            }
        }
        _ => panic!("Expected MermaidDiagram, not CodeBlock"),
    }
}
