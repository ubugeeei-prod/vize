use super::{IGNORE, rewrite};

#[test]
fn original_nested_parameter_gets_its_own_inference_directive() {
    let source = "(foo) => {\n  foo.bar((baz) => console.log(baz));\n}";
    assert_eq!(
        rewrite(source).as_deref(),
        Some(format!("(foo) => {{\n  foo.bar(({IGNORE}baz) => console.log(baz));\n}}").as_str())
    );
}

#[test]
fn lone_parameters_and_return_line_breaks_keep_their_grammar() {
    let cases = [
        (
            "(foo) => {\n return baz => baz;\n}",
            format!("(foo) => {{\n return ({IGNORE}baz) => baz;\n}}"),
        ),
        (
            "(foo) => {\n return\n baz => baz;\n}",
            format!("(foo) => {{\n return\n ({IGNORE}baz) => baz;\n}}"),
        ),
        (
            "(foo) => {\n return async baz => baz;\n}",
            format!("(foo) => {{\n return async ({IGNORE}baz) => baz;\n}}"),
        ),
    ];
    for (source, expected) in cases {
        assert_eq!(rewrite(source).as_deref(), Some(expected.as_str()));
    }
}

#[test]
fn multiple_parameters_and_function_expressions_preserve_comments() {
    let source = "(foo) => {\n foo.bar(function named(\n a, /* keep */ b\n) {\n return a + b // keep LF\n });\n}";
    let expected = format!(
        "(foo) => {{\n foo.bar(function named(\n {IGNORE}a, /* keep */ {IGNORE}b\n) {{\n return a + b // keep LF\n }});\n}}"
    );
    assert_eq!(rewrite(source).as_deref(), Some(expected.as_str()));
}

#[test]
fn templates_and_nested_callback_bodies_keep_authored_bytes() {
    let source = "(foo) => {\n const text = `keep\n raw ${foo.bar(baz => baz)}`;\n return (baz) => {\n return text // next line\n };\n}";
    let expected = format!(
        "(foo) => {{\n const text = `keep\n raw ${{foo.bar(({IGNORE}baz) => baz)}}`;\n return ({IGNORE}baz) => {{\n return text // next line\n }};\n}}"
    );
    assert_eq!(rewrite(source).as_deref(), Some(expected.as_str()));
}

#[test]
fn annotated_defaults_rest_and_destructuring_are_not_retyped() {
    assert_eq!(
        rewrite(
            "(foo) => {\n foo.bar((baz: number) => baz);\n foo.bar((baz = 1) => baz);\n foo.bar((...baz) => baz);\n foo.bar(({baz}) => baz);\n}"
        ),
        None
    );
    let source = "(foo) => {\n foo.bar((baz: number, other) => baz + other);\n}";
    assert_eq!(
        rewrite(source).as_deref(),
        Some(
            format!("(foo) => {{\n foo.bar((baz: number, {IGNORE}other) => baz + other);\n}}")
                .as_str()
        )
    );
}

#[test]
fn ordinary_and_incomplete_inputs_preserve_the_old_result() {
    for source in [
        "(foo) => foo.bar((baz) => baz)",
        "(foo) => foo.bar((baz) => baz)\n",
        "(foo) => {\n return foo;\n}",
        "(foo) => {\n foo.bar((baz) =>",
    ] {
        assert_eq!(rewrite(source), None);
    }
}

#[test]
fn jsx_unicode_and_crlf_preserve_the_complete_body() {
    let source = "(foo) => {\n return <Comp render={baz => <Other value={baz} />} />;\n}";
    let expected = format!(
        "(foo) => {{\n return <Comp render={{({IGNORE}baz) => <Other value={{baz}} />}} />;\n}}"
    );
    assert_eq!(rewrite(source).as_deref(), Some(expected.as_str()));
    let source = "(foo) => {\r\n foo.bar((baz) => baz);\r\n}";
    let expected = format!("(foo) => {{\r\n foo.bar(({IGNORE}baz) => baz);\r\n}}");
    assert_eq!(rewrite(source).as_deref(), Some(expected.as_str()));
    let source = r"(foo) => {
 foo.bar(\u0062az => baz);
}";
    let expected = format!("(foo) => {{\n foo.bar(({IGNORE}\\u0062az) => baz);\n}}");
    assert_eq!(rewrite(source).as_deref(), Some(expected.as_str()));
}

#[test]
fn authored_expect_error_is_not_reassigned_to_a_new_directive() {
    assert_eq!(
        rewrite("(foo) => {\n // @ts-expect-error authored\n foo.bar((baz) => baz);\n}"),
        None
    );
}

#[test]
fn declarations_and_methods_do_not_gain_callback_suppression() {
    for source in [
        "(foo) => {\n function local(baz) { return baz; }\n}",
        "(foo) => {\n class Local { method(baz) { return baz; } }\n}",
    ] {
        assert_eq!(rewrite(source), None);
    }
}
