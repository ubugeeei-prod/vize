//! Coverage for the string/template-literal aware delimiter scanner.
//!
//! Kept separate from `tests.rs` so that already large file does not grow past
//! the source-file-length limit.

use super::super::string_tracking::{
    StringTrackState, count_braces_outside_strings, count_braces_with_state,
    count_delims_with_state,
};

#[test]
fn test_count_braces_normal() {
    assert_eq!(count_braces_outside_strings("{ a: 1 }"), 0);
    assert_eq!(count_braces_outside_strings("{"), 1);
    assert_eq!(count_braces_outside_strings("}"), -1);
    assert_eq!(count_braces_outside_strings("{ { }"), 1);
}

#[test]
fn test_count_braces_ignores_string_braces() {
    assert_eq!(
        count_braces_outside_strings("_toDisplayString(isArray.value ? ']' : '}')"),
        0
    );
    assert_eq!(count_braces_outside_strings(r#"var x = "{";"#), 0);
    assert_eq!(count_braces_outside_strings("var x = `{`;"), 0);
}

#[test]
fn test_count_braces_mixed_string_and_code() {
    assert_eq!(count_braces_outside_strings("if (x) { var s = '}'"), 1);
}

#[test]
fn test_count_braces_escaped_quotes() {
    assert_eq!(count_braces_outside_strings(r"var x = '\'' + '}'"), 0);
}

#[test]
fn test_count_braces_multiline_template_literal() {
    let mut state = StringTrackState::default();

    let line1 = r#"}, _toDisplayString(`${t("key")}: v${ver.major}.${"#;
    let count1 = count_braces_with_state(line1, &mut state);
    assert_eq!(count1, -1, "Line 1 brace count");
    assert!(
        !state.template_expr_brace_stack.is_empty(),
        "Should be inside template expression after line 1"
    );

    let line2 = "            ver.minor";
    let count2 = count_braces_with_state(line2, &mut state);
    assert_eq!(count2, 0, "Line 2 brace count");

    let line3 = r##"          }`) + "\n      ", 1 /* TEXT */)))"##;
    let count3 = count_braces_with_state(line3, &mut state);
    assert_eq!(count3, 0, "Line 3 brace count");
    assert!(!state.in_string, "Should be outside string after line 3");
    assert!(
        state.template_expr_brace_stack.is_empty(),
        "Template expression stack should be empty"
    );

    assert_eq!(count1 + count2 + count3, -1);
}

#[test]
fn test_count_braces_template_literal_with_nested_object() {
    let mut state = StringTrackState::default();
    let line = r#"x = `result: ${fn({a: 1, b: {c: 2}})}`"#;
    let count = count_braces_with_state(line, &mut state);
    assert_eq!(
        count, 0,
        "Braces inside template expression should be balanced"
    );
    assert!(!state.in_string, "Template literal should be closed");
}

#[test]
fn test_count_braces_nested_template_literals() {
    let mut state = StringTrackState::default();
    let line = r#"x = `outer ${`inner ${x}`} end`"#;
    let count = count_braces_with_state(line, &mut state);
    assert_eq!(
        count, 0,
        "Nested template literals should not affect brace count"
    );
    assert!(!state.in_string, "All template literals should be closed");
}

#[test]
fn test_count_braces_multiline_template_expr_with_object() {
    let mut state = StringTrackState::default();

    let line1 = r#"x = `value: ${fn({"#;
    let c1 = count_braces_with_state(line1, &mut state);
    assert_eq!(
        c1, 1,
        "Line 1: object literal brace inside template expression"
    );

    let line2 = r#"  key: val"#;
    let c2 = count_braces_with_state(line2, &mut state);
    assert_eq!(c2, 0, "Line 2: no braces");

    let line3 = r#"})}`"#;
    let c3 = count_braces_with_state(line3, &mut state);
    assert_eq!(c3, -1, "Line 3: closing object brace");
    assert!(!state.in_string, "Template literal should be closed");
    assert_eq!(c1 + c2 + c3, 0, "Total should be balanced");
}

#[test]
fn test_count_braces_template_literal_with_arrow_function() {
    let mut state = StringTrackState::default();
    let line = r#"x = `${items.map(x => ({ name: x })).join()}`"#;
    let count = count_braces_with_state(line, &mut state);
    assert_eq!(count, 0);
    assert!(!state.in_string);
}

#[test]
fn test_count_braces_state_across_many_lines() {
    let mut state = StringTrackState::default();

    let c1 = count_braces_with_state("function render() {", &mut state);
    assert_eq!(c1, 1);

    let c2 = count_braces_with_state(r#"  return _toDisplayString(`${fn({"#, &mut state);
    assert_eq!(c2, 1, "Object literal brace inside template expression");

    let c3 = count_braces_with_state("    key: val,", &mut state);
    assert_eq!(c3, 0);

    let c4 = count_braces_with_state("    nested: {", &mut state);
    assert_eq!(c4, 1, "Nested brace inside template expression");

    let c5 = count_braces_with_state("      deep: true", &mut state);
    assert_eq!(c5, 0);

    let c6 = count_braces_with_state("    }", &mut state);
    assert_eq!(c6, -1, "Closing nested brace inside template expression");

    let c7 = count_braces_with_state(r#"  })}`)"#, &mut state);
    assert_eq!(c7, -1, "Closing outer object brace");

    let c8 = count_braces_with_state("}", &mut state);
    assert_eq!(c8, -1);

    assert_eq!(
        c1 + c2 + c3 + c4 + c5 + c6 + c7 + c8,
        0,
        "Total: function opens and closes"
    );
    assert!(!state.in_string);
    assert!(state.template_expr_brace_stack.is_empty());
}

#[test]
fn test_count_braces_regular_strings_with_braces() {
    let mut state = StringTrackState::default();

    let line = r#"if (x) { var s = "}" + '{' }"#;
    let count = count_braces_with_state(line, &mut state);
    assert_eq!(count, 0, "Braces inside regular strings should be ignored");
}

/// Net delimiter depth must equal zero for a balanced object literal that spans lines,
/// treating `{} [] ()` uniformly and ignoring delimiters inside strings.
#[test]
fn test_count_delims_with_state_multiline_object() {
    let mut state = StringTrackState::default();
    let mut depth = 0;
    depth += count_delims_with_state("const _hoisted_1 = { style: {", &mut state);
    assert_eq!(depth, 2);
    depth += count_delims_with_state("  position: 'absolute',", &mut state);
    assert_eq!(depth, 2);
    depth += count_delims_with_state("  content: '({[',", &mut state);
    assert_eq!(depth, 2, "delimiters inside strings must not affect depth");
    depth += count_delims_with_state("} }", &mut state);
    assert_eq!(depth, 0, "declaration is balanced after the closing line");
}
