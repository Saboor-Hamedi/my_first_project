//! Unit tests for inline markdown classification, tokenization, and layout geometry.

use super::parser::classify_line;
use super::types::InlineLineKind;

#[test]
fn test_classify_headings() {
    let h1: Vec<char> = "# Hello World".chars().collect();
    let (kind, prefix_len) = classify_line(&h1);
    assert_eq!(kind, InlineLineKind::Heading(1));
    assert_eq!(prefix_len, 2);

    let h2: Vec<char> = "## Subtitle".chars().collect();
    let (kind, prefix_len) = classify_line(&h2);
    assert_eq!(kind, InlineLineKind::Heading(2));
    assert_eq!(prefix_len, 3);

    let h3: Vec<char> = "### Deep Topic".chars().collect();
    let (kind, prefix_len) = classify_line(&h3);
    assert_eq!(kind, InlineLineKind::Heading(3));
    assert_eq!(prefix_len, 4);

    let h4: Vec<char> = "#### Micro Topic".chars().collect();
    let (kind, prefix_len) = classify_line(&h4);
    assert_eq!(kind, InlineLineKind::Heading(4));
    assert_eq!(prefix_len, 5);
}

#[test]
fn test_classify_tasks() {
    let uncheck: Vec<char> = "- [ ] Buy groceries".chars().collect();
    let (kind, prefix_len) = classify_line(&uncheck);
    assert_eq!(
        kind,
        InlineLineKind::TaskItem {
            checked: false,
            check_char_idx: 3,
        }
    );
    assert_eq!(prefix_len, 6);

    let check: Vec<char> = "- [x] Fix compiler errors".chars().collect();
    let (kind, prefix_len) = classify_line(&check);
    assert_eq!(
        kind,
        InlineLineKind::TaskItem {
            checked: true,
            check_char_idx: 3,
        }
    );
    assert_eq!(prefix_len, 6);
}

#[test]
fn test_classify_bullets_and_quotes() {
    let bullet: Vec<char> = "- List item".chars().collect();
    let (kind, prefix_len) = classify_line(&bullet);
    assert_eq!(kind, InlineLineKind::BulletItem);
    assert_eq!(prefix_len, 2);

    let quote: Vec<char> = "> Inspirational quote".chars().collect();
    let (kind, prefix_len) = classify_line(&quote);
    assert_eq!(kind, InlineLineKind::Quote);
    assert_eq!(prefix_len, 2);
}

#[test]
fn test_classify_rules_and_code() {
    let rule: Vec<char> = "---".chars().collect();
    let (kind, prefix_len) = classify_line(&rule);
    assert_eq!(kind, InlineLineKind::Rule);
    assert_eq!(prefix_len, 3);

    let code_fence: Vec<char> = "```rust".chars().collect();
    let (kind, prefix_len) = classify_line(&code_fence);
    assert_eq!(kind, InlineLineKind::CodeFence("rust".to_string()));
    assert_eq!(prefix_len, 7);
}

#[test]
fn test_classify_tables() {
    let header: Vec<char> = "| Title | Status | Date |".chars().collect();
    let (kind, prefix_len) = classify_line(&header);
    assert_eq!(kind, InlineLineKind::TableRow { is_header: false, is_separator: false });
    assert_eq!(prefix_len, 0);

    let sep: Vec<char> = "|:---|:---:|---:|".chars().collect();
    let (kind, prefix_len) = classify_line(&sep);
    assert_eq!(kind, InlineLineKind::TableRow { is_header: false, is_separator: true });
    assert_eq!(prefix_len, 0);
}
