use super::callback_line_suppression::{
    BRANCH_IGNORE, has_callback_parser_candidate, rewrite_ranges,
};
use super::{
    callback_parameters_on_ignored_line, contains_typescript_suppression_comment,
    expression_source_for_typecheck,
};

#[test]
fn ordinary_callbacks_keep_the_allocation_free_result() {
    assert_eq!(
        callback_parameters_on_ignored_line("(item) => item.id"),
        None
    );
    assert_eq!(
        callback_parameters_on_ignored_line("(\n  item\n) => item.id").as_deref(),
        Some("(   item ) => item.id")
    );
    assert_eq!(
        callback_parameters_on_ignored_line("((\n value) => value)").as_deref(),
        Some("((  value) => value)")
    );
}

#[test]
fn nested_function_type_keeps_the_existing_lexical_outer_arrow() {
    let source = "(fn: (s: string) => string,\n value) => value";
    assert!(!has_callback_parser_candidate(source));
    assert_eq!(
        callback_parameters_on_ignored_line(source).as_deref(),
        Some("(fn: (s: string) => string,  value) => value")
    );
}

#[test]
fn generic_constraints_use_ast_bounds_and_preserve_nested_types() {
    let cases = [
        (
            "<T extends () => unknown>(\n  value: T\n) => value",
            "<T extends () => unknown>(   value: T ) => value",
        ),
        (
            r#"<T extends Promise<() => { value: ["=>", `${string}-${`${number}`}`] }>>(
  value: T
): (() => T) /* => */ => value"#,
            r#"<T extends Promise<() => { value: ["=>", `${string}-${`${number}`}`] }>>(   value: T ): (() => T) /* => */ => value"#,
        ),
        (
            "<T extends { render: () => unknown }>(\n value: T) => <Comp value={value} />",
            "<T extends { render: () => unknown }>(  value: T) => <Comp value={value} />",
        ),
    ];
    for (source, collapsed) in cases {
        assert!(has_callback_parser_candidate(source), "{source}");
        let expected = format!("{BRANCH_IGNORE}{collapsed}");
        assert_eq!(
            callback_parameters_on_ignored_line(source).as_deref(),
            Some(expected.as_str()),
            "{source}"
        );
    }
}

#[test]
fn sequence_and_wrapper_newlines_get_branch_local_suppression() {
    assert_eq!(
        callback_parameters_on_ignored_line("(void 0, <T>(\n value\n) => value)").as_deref(),
        Some(concat!(
            "(void 0, ",
            "// @ts-ignore Inference-only callback branch.\n",
            "<T>(  value ) => value)",
        ))
    );
    assert_eq!(
        callback_parameters_on_ignored_line("(\n<T>(\n value\n) => value\n)").as_deref(),
        Some(concat!(
            "(\n",
            "// @ts-ignore Inference-only callback branch.\n",
            "<T>(  value ) => value\n)",
        ))
    );
}

#[test]
fn ordinary_sequence_siblings_parse_only_when_the_last_branch_needs_it() {
    let siblings = "((left) => left,\n (right) => right)";
    assert!(has_callback_parser_candidate(siblings));
    assert_eq!(
        callback_parameters_on_ignored_line(siblings).as_deref(),
        Some(concat!(
            "((left) => left,\n ",
            "// @ts-ignore Inference-only callback branch.\n",
            "(right) => right)",
        ))
    );

    let single = "(void 0,\n (value) => value)";
    assert!(!has_callback_parser_candidate(single));
    assert_eq!(
        callback_parameters_on_ignored_line(single).as_deref(),
        Some("(void 0,  (value) => value)")
    );
}

#[test]
fn conditional_branches_preserve_bodies_comments_and_tsx() {
    let source = r#"condition
 ? <T,>(
    left
  ) => {
    const body = `keep
body`
    return <Comp value={left} />
  }
 : <U,>(
    right
  ) => right"#;
    let rewritten = callback_parameters_on_ignored_line(source).expect("conditional");
    assert_eq!(
        rewritten.as_str(),
        concat!(
            "condition\n ? ",
            "// @ts-ignore Inference-only callback branch.\n",
            "<T,>(     left   ) => {\n",
            "    const body = `keep\nbody`\n",
            "    return <Comp value={left} />\n",
            "  }\n : ",
            "// @ts-ignore Inference-only callback branch.\n",
            "<U,>(     right   ) => right",
        )
    );
    assert_eq!(rewritten.matches(BRANCH_IGNORE).count(), 2);
}

#[test]
fn ordinary_conditional_siblings_use_branch_local_directives() {
    let source = "condition\n ? (\n left) => left\n : (\n right) => right";
    assert!(has_callback_parser_candidate(source));
    assert_eq!(
        callback_parameters_on_ignored_line(source).as_deref(),
        Some(concat!(
            "condition\n ? ",
            "// @ts-ignore Inference-only callback branch.\n",
            "(  left) => left\n : ",
            "// @ts-ignore Inference-only callback branch.\n",
            "(  right) => right",
        ))
    );
}

#[test]
fn function_leaves_get_branch_local_suppression() {
    let source = "ok ? function named(\n left\n) { return left } : <T>(\n right\n) => right";
    assert_eq!(
        callback_parameters_on_ignored_line(source).as_deref(),
        Some(concat!(
            "ok ? ",
            "// @ts-ignore Inference-only callback branch.\n",
            "function named(  left ) { return left } : ",
            "// @ts-ignore Inference-only callback branch.\n",
            "<T>(  right ) => right",
        ))
    );
}

#[test]
fn body_only_line_breaks_never_inject_or_fall_back_to_body_arrows() {
    let function = "function(value) {\n return value\n}";
    assert!(!has_callback_parser_candidate(function));
    assert_eq!(callback_parameters_on_ignored_line(function), None);

    let nested_arrow = "function(value) {\n return (item) => item ?? value\n}";
    assert!(!has_callback_parser_candidate(nested_arrow));
    assert_eq!(callback_parameters_on_ignored_line(nested_arrow), None);

    let direct_arrow_body = "(value) =>\n  (item) => item ?? value";
    assert!(!has_callback_parser_candidate(direct_arrow_body));
    assert_eq!(callback_parameters_on_ignored_line(direct_arrow_body), None);

    let generic = "<T>(value: T) => {\n return value\n}";
    assert!(has_callback_parser_candidate(generic));
    assert_eq!(callback_parameters_on_ignored_line(generic), None);
}

#[test]
fn type_arguments_comparisons_assertions_and_literals_stay_on_the_fast_path() {
    let cases = [
        (
            "(value = factory<Foo<Bar>>(seed),\n next = value) => next",
            "(value = factory<Foo<Bar>>(seed),  next = value) => next",
        ),
        (
            "(node = <Comp value={seed} />,\n value = node) => value",
            "(node = <Comp value={seed} />,  value = node) => value",
        ),
        (
            "(value = <Type>seed,\n next = value) => next",
            "(value = <Type>seed,  next = value) => next",
        ),
        (
            "(value = left < right ? left : right,\n next = value) => next",
            "(value = left < right ? left : right,  next = value) => next",
        ),
        (
            "(value = \"=> <T>\", /* => <T> */\n next = value) => next",
            "(value = \"=> <T>\", /* => <T> */  next = value) => next",
        ),
    ];
    for (source, expected) in cases {
        assert!(!has_callback_parser_candidate(source), "{source}");
        assert_eq!(
            callback_parameters_on_ignored_line(source).as_deref(),
            Some(expected),
            "{source}"
        );
    }
}

#[test]
fn unsafe_or_unterminated_prefixes_fail_closed() {
    let line_comment = "<T extends // keep the terminator\n() => unknown>(\n value: T) => value";
    assert!(has_callback_parser_candidate(line_comment));
    assert_eq!(callback_parameters_on_ignored_line(line_comment), None);

    let template_literal = "<T extends `left\nright`>(\n value: T) => value";
    assert!(has_callback_parser_candidate(template_literal));
    assert_eq!(callback_parameters_on_ignored_line(template_literal), None);

    let unterminated = "/* never closes <T extends () => unknown>(\n value) => value";
    assert!(!has_callback_parser_candidate(unterminated));
    assert_eq!(callback_parameters_on_ignored_line(unterminated), None);
}

#[test]
fn parser_shape_and_range_failures_preserve_exact_input() {
    let incomplete = "<T extends () => unknown>(\n value";
    assert!(has_callback_parser_candidate(incomplete));
    assert_eq!(callback_parameters_on_ignored_line(incomplete), None);

    let nested_only = "factory(<T>(\n value) => value)";
    assert!(has_callback_parser_candidate(nested_only));
    assert_eq!(callback_parameters_on_ignored_line(nested_only), None);

    assert_eq!(rewrite_ranges("abcd", &[0..3, 2..4]), None);
    assert_eq!(rewrite_ranges("abc", &[0..4]), None);
}

#[test]
fn preserves_only_leading_typescript_suppression_comments() {
    assert_eq!(
        expression_source_for_typecheck("// note\nvalue").as_ref(),
        "\nvalue"
    );
    assert_eq!(
        expression_source_for_typecheck("// @ts-ignore\nvalue").as_ref(),
        "// @ts-ignore\nvalue"
    );
    assert_eq!(
        expression_source_for_typecheck("/* @ts-expect-error */\nvalue").as_ref(),
        "/* @ts-expect-error */\nvalue"
    );
    assert!(!contains_typescript_suppression_comment(
        r#""// @ts-ignore" + value"#
    ));
    assert!(!contains_typescript_suppression_comment(
        r#"`/* @ts-expect-error */` + value"#
    ));
    let object = "{ name: 'route',\n// @ts-expect-error upstream note\nparams }";
    assert!(!contains_typescript_suppression_comment(object));
    assert_eq!(
        expression_source_for_typecheck(object).as_ref(),
        "{ name: 'route',\n\nparams }"
    );
}
