use super::{format_markdown, format_yaml};

// -- YAML ---------------------------------------------------------------

#[test]
fn yaml_normalizes_crlf_and_adds_final_newline() {
    let result = format_yaml("a: 1\r\nb: 2");
    assert_eq!(result.code.as_str(), "a: 1\nb: 2\n");
    assert!(result.changed);
}

#[test]
fn yaml_preserves_block_scalar_trailing_whitespace() {
    // Trailing spaces inside a literal block scalar are significant; YAML must
    // never touch line content.
    let source = "key: |\n  line with spaces   \n  next\n";
    let result = format_yaml(source);
    assert_eq!(result.code.as_str(), source);
    assert!(!result.changed);
}

#[test]
fn yaml_is_idempotent_and_noop_when_clean() {
    let source = "# comment\nname: vize\nlist:\n  - a\n  - b\n";
    let first = format_yaml(source);
    assert_eq!(first.code.as_str(), source);
    let second = format_yaml(first.code.as_str());
    assert_eq!(first.code, second.code);
}

#[test]
fn yaml_empty_stays_empty() {
    assert_eq!(format_yaml("").code.as_str(), "");
}

// -- Markdown -----------------------------------------------------------

#[test]
fn markdown_trims_trailing_whitespace() {
    let result = format_markdown("# Title   \n\nText with space \n");
    assert_eq!(result.code.as_str(), "# Title\n\nText with space\n");
}

#[test]
fn markdown_preserves_two_space_hard_break() {
    let source = "line one  \nline two\n";
    let result = format_markdown(source);
    assert_eq!(result.code.as_str(), source);
    assert!(!result.changed);
}

#[test]
fn markdown_preserves_fenced_code_block_verbatim() {
    let source = "```sh\necho hi   \n```\ntext   \n";
    let result = format_markdown(source);
    // Trailing spaces inside the fence are kept; outside is trimmed.
    assert_eq!(result.code.as_str(), "```sh\necho hi   \n```\ntext\n");
}

#[test]
fn markdown_preserves_indented_code_block() {
    let source = "para\n\n    code with spaces   \n";
    let result = format_markdown(source);
    assert_eq!(result.code.as_str(), source);
}

#[test]
fn markdown_adds_final_newline_and_is_idempotent() {
    let first = format_markdown("# Title");
    assert_eq!(first.code.as_str(), "# Title\n");
    let second = format_markdown(first.code.as_str());
    assert_eq!(first.code, second.code);
}
