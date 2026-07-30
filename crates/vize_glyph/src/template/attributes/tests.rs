use super::{ParsedAttribute, render_attribute, write_rendered_attribute};

fn write(attr: &str, continuation_depth: usize, indent_continuation: bool) -> String {
    let mut output = Vec::new();
    write_rendered_attribute(
        &mut output,
        attr,
        b"\n",
        b"  ",
        continuation_depth,
        indent_continuation,
    );
    String::from_utf8(output).unwrap()
}

#[test]
fn render_attribute_uses_single_quotes_when_value_contains_double_quotes() {
    let attr = ParsedAttribute {
        name: "title".into(),
        value: Some(r#"say "hello""#.into()),
        priority: 0,
        original_index: 0,
        indent_multiline_value: false,
    };

    assert_eq!(render_attribute(&attr).as_str(), r#"title='say "hello"'"#);
}

#[test]
fn render_attribute_escapes_double_quotes_when_value_contains_both_quote_styles() {
    let attr = ParsedAttribute {
        name: "title".into(),
        value: Some(r#"say "hello" and 'bye'"#.into()),
        priority: 0,
        original_index: 0,
        indent_multiline_value: false,
    };

    assert_eq!(
        render_attribute(&attr).as_str(),
        r#"title="say &quot;hello&quot; and 'bye'""#
    );
}

#[test]
fn write_rendered_attribute_indents_multiline_value_lines() {
    assert_eq!(
        write(":class='[\n  active\n]'", 2, true),
        ":class='[\n      active\n    ]'"
    );
}

#[test]
fn write_rendered_attribute_leaves_literal_multiline_values_verbatim() {
    assert_eq!(
        write("class=\"\n  active\n\"", 2, false),
        "class=\"\n  active\n\""
    );
}

#[test]
fn write_rendered_attribute_leaves_template_literal_lines_verbatim() {
    // The `  w-full` line begins inside the literal, so its leading spaces are
    // part of the string's runtime value: no attribute indent may go in front
    // of them and none of them may be stripped. The line closing the literal
    // begins inside it too, so its indentation is string content as well.
    // Only the `]'` line, which begins outside, is re-anchored. (#3379)
    assert_eq!(
        write(":class='[`\n  w-full\n`]'", 2, true),
        ":class='[`\n  w-full\n`]'"
    );
    assert_eq!(
        write(":class='[`\n  w-full\n  `,\n  x]'", 2, true),
        ":class='[`\n  w-full\n  `,\n      x]'"
    );
}
