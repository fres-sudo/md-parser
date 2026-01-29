//! Horizontal rule parsing.

/// Check if a line is a horizontal rule
///
/// A horizontal rule is a line containing at least 3 consecutive `-` or `*` characters,
/// with optional leading and trailing spaces. The line may contain only spaces and
/// the rule characters.
///
/// Examples:
/// - `---` (valid)
/// - `***` (valid)
/// - `  ---  ` (valid, with spaces)
/// - `----` (valid, more than 3)
/// - `--` (invalid, less than 3)
/// - `**` (invalid, less than 3)
/// - `---text---` (invalid, contains other characters)
pub(super) fn detect_horizontal_rule(line: &str) -> bool {
    let trimmed = line.trim();

    // Must have at least 3 characters
    if trimmed.len() < 3 {
        return false;
    }

    // Check if all characters are either `-` or `*`
    // Must have at least 3 consecutive characters of the same type
    let first_char = trimmed.chars().next().unwrap();

    // Only `-` and `*` are valid for horizontal rules
    if first_char != '-' && first_char != '*' {
        return false;
    }

    // Check that all characters are the same as the first character
    // and there are at least 3 of them
    let count = trimmed.chars().take_while(|&c| c == first_char).count();

    count >= 3
        && trimmed
            .chars()
            .all(|c| c == first_char || c.is_whitespace())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_three_dashes() {
        assert!(detect_horizontal_rule("---"));
    }

    #[test]
    fn test_detect_three_asterisks() {
        assert!(detect_horizontal_rule("***"));
    }

    #[test]
    fn test_detect_more_than_three() {
        assert!(detect_horizontal_rule("----"));
        assert!(detect_horizontal_rule("****"));
        assert!(detect_horizontal_rule("-----"));
    }

    #[test]
    fn test_detect_with_spaces() {
        assert!(detect_horizontal_rule("  ---  "));
        assert!(detect_horizontal_rule("  ***  "));
        assert!(detect_horizontal_rule(" --- "));
    }

    #[test]
    fn test_reject_less_than_three() {
        assert!(!detect_horizontal_rule("--"));
        assert!(!detect_horizontal_rule("**"));
        assert!(!detect_horizontal_rule("-"));
        assert!(!detect_horizontal_rule("*"));
    }

    #[test]
    fn test_reject_mixed_characters() {
        assert!(!detect_horizontal_rule("---text---"));
        assert!(!detect_horizontal_rule("***text***"));
        assert!(!detect_horizontal_rule("---***"));
    }

    #[test]
    fn test_reject_other_characters() {
        assert!(!detect_horizontal_rule("___"));
        assert!(!detect_horizontal_rule("==="));
        assert!(!detect_horizontal_rule("###"));
    }

    #[test]
    fn test_reject_empty() {
        assert!(!detect_horizontal_rule(""));
        assert!(!detect_horizontal_rule("   "));
    }
}
