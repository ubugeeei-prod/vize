use super::ValueTokens;

fn slices<'a>(source: &'a str, spans: &[(usize, usize)]) -> Vec<&'a str> {
    spans
        .iter()
        .map(|&(start, end)| &source[start..end])
        .collect()
}

#[test]
fn declaration_tokens_keep_complete_functions_and_terminal_flags() {
    let source =
        ".a { color: v-bind('theme.color'); width: calc(v-bind(size) * 1px) !/**/IMPORTANT; }";
    let tokens = ValueTokens::new(source);
    assert_eq!(
        slices(source, &tokens.bindings),
        ["v-bind('theme.color')", "v-bind(size)"]
    );
    assert_eq!(slices(source, &tokens.important), ["!/**/IMPORTANT"]);
}

#[test]
fn opaque_tokens_and_rule_preludes_do_not_become_declarations() {
    let source = r#"/* v-bind(comment) !important */
@supports (width: v-bind(probe)) {
  .a[data-value="v-bind(selector) !important"] {
    content: "quote\" v-bind(string) !important";
    background: url("v-bind(url) !important");
    --escaped: v\2d bind(escaped);
    --different: my-v-bind(wrong);
    --real: v-bind(value);
  }
}"#;
    let tokens = ValueTokens::new(source);
    assert_eq!(slices(source, &tokens.bindings), ["v-bind(value)"]);
    assert!(tokens.important.is_empty());
}

#[test]
fn nested_rules_and_declaration_at_rules_retain_authored_values() {
    let source = "@font-face { --font: v-bind(font); } @keyframes x { from { width: v-bind(start); } } .a { &:hover { color: red !important; } }";
    let tokens = ValueTokens::new(source);
    assert_eq!(
        slices(source, &tokens.bindings),
        ["v-bind(font)", "v-bind(start)"]
    );
    assert_eq!(slices(source, &tokens.important), ["!important"]);
}

#[test]
fn nested_custom_values_keep_functions_but_not_nested_or_quoted_flags() {
    let source = ".a { --raw: { note: '!important'; actual: v-bind(token) !important }; --last: v-bind(next) ! important; }";
    let tokens = ValueTokens::new(source);
    assert_eq!(
        slices(source, &tokens.bindings),
        ["v-bind(token)", "v-bind(next)"]
    );
    assert_eq!(slices(source, &tokens.important), ["! important"]);
}

#[test]
fn rejected_declaration_attempt_cannot_publish_selector_functions() {
    let source = ".a { color:v-bind(selector) { --actual: v-bind(value); } }";
    let tokens = ValueTokens::new(source);
    assert_eq!(slices(source, &tokens.bindings), ["v-bind(value)"]);
    assert!(tokens.important.is_empty());
}
