use md_parser::{Inline, Node, Parser};

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
