use super::*;

fn diagnostic(source: &str, needle: &str, code: u32) -> AssembledDiagnostic<()> {
    let start = source.find(needle).unwrap();
    AssembledDiagnostic {
        start,
        end: start + needle.len(),
        code: Some(code),
        severity: Some(1),
        message: if code == 2578 {
            "Unused '@ts-expect-error' directive.".into()
        } else {
            "Property 'missing' does not exist on type 'ComponentInstance'.".into()
        },
        origin: AssembledOrigin::Checker(()),
    }
}

#[test]
fn relocated_instance_read_consumes_only_its_own_expectation() {
    for newline in ["\n", "\r\n"] {
        let source = vize_carton::cstr!(
            "<template>日本😀{{{{{newline}// @ts-expect-error{newline}missing{newline}}}}}{{{{ sibling }}}}</template>"
        );
        let mut diagnostics = vec![
            diagnostic(&source, "// @ts-expect-error", 2578),
            diagnostic(&source, "missing", 2339),
            diagnostic(&source, "sibling", 2339),
        ];
        let expected = diagnostics[2].clone();
        filter(&AuthoredSource::vue(&source), &mut diagnostics);
        assert_eq!(diagnostics, [expected]);
    }
}

#[test]
fn valid_and_repaired_expressions_keep_the_unused_expectation() {
    let source = "<template>{{\n// @ts-expect-error\nknown\n}}</template>";
    let original = vec![diagnostic(source, "// @ts-expect-error", 2578)];
    let mut diagnostics = original.clone();
    filter(&AuthoredSource::vue(source), &mut diagnostics);
    assert_eq!(diagnostics, original);
}

#[test]
fn ignore_suppresses_its_target_without_consuming_a_different_expectation() {
    let source =
        "<template>{{\n// @ts-ignore\nmissing\n}}{{\n// @ts-expect-error\nknown\n}}</template>";
    let mut diagnostics = vec![
        diagnostic(source, "missing", 2339),
        diagnostic(source, "// @ts-expect-error", 2578),
    ];
    let expected = diagnostics[1].clone();
    filter(&AuthoredSource::vue(source), &mut diagnostics);
    assert_eq!(diagnostics, [expected]);
}

#[test]
fn literal_nonleading_and_malformed_comments_have_no_authority() {
    for expression in [
        "'// @ts-expect-error' + missing",
        "known +\n// @ts-expect-error\nmissing",
        "// prose @ts-expect-error\nmissing",
        "/* @ts-expect-error */ missing",
        "// @ts-expect-error\nmissing(",
        "/* @ts-expect-error\nmissing",
    ] {
        let source = vize_carton::cstr!("<template>{{{{ {expression} }}}}</template>");
        let original = vec![diagnostic(&source, "missing", 2339)];
        let mut diagnostics = original.clone();
        filter(&AuthoredSource::vue(&source), &mut diagnostics);
        assert_eq!(diagnostics, original, "{expression}");
    }
}

#[test]
fn no_directive_script_and_attribute_diagnostics_are_preserved() {
    for source in [
        "<template>{{ missing }}</template>",
        "<script>// @ts-expect-error\nmissing</script><template/>",
        "<template><div title='// @ts-expect-error\nmissing'/></template>",
    ] {
        let original = vec![diagnostic(source, "missing", 2339)];
        let mut diagnostics = original.clone();
        filter(&AuthoredSource::vue(source), &mut diagnostics);
        assert_eq!(diagnostics, original);
    }
}

#[test]
fn only_actual_native_property_errors_consume_an_expectation() {
    let source = "<template>{{\n// @ts-expect-error\nmissing\n}}</template>";
    for code in [2322, 2345, 2563] {
        let original = vec![
            diagnostic(source, "// @ts-expect-error", 2578),
            diagnostic(source, "missing", code),
        ];
        let mut diagnostics = original.clone();
        filter(&AuthoredSource::vue(source), &mut diagnostics);
        assert_eq!(diagnostics, original);
    }
    let mut error = diagnostic(source, "missing", 2339);
    error.origin = AssembledOrigin::GeneratedGiveUp(());
    let original = vec![diagnostic(source, "// @ts-expect-error", 2578), error];
    let mut diagnostics = original.clone();
    filter(&AuthoredSource::vue(source), &mut diagnostics);
    assert_eq!(diagnostics, original);
}

#[test]
fn complete_block_comment_uses_its_exact_next_line() {
    let source = "<template>{{\n/* @ts-expect-error reason */\nmissing\n}}</template>";
    let mut diagnostics = vec![
        diagnostic(source, "/* @ts-expect-error", 2578),
        diagnostic(source, "missing", 2339),
    ];
    filter(&AuthoredSource::vue(source), &mut diagnostics);
    assert!(diagnostics.is_empty());
}

#[test]
fn blank_and_explanatory_comment_lines_preserve_native_directive_ownership() {
    for gap in ["\n", "// explanation\n"] {
        let source = vize_carton::cstr!(
            "<template>{{{{\n// @ts-expect-error\n{gap}missing\n}}}}</template>"
        );
        let mut diagnostics = vec![
            diagnostic(&source, "// @ts-expect-error", 2578),
            diagnostic(&source, "missing", 2339),
        ];
        filter(&AuthoredSource::vue(&source), &mut diagnostics);
        assert!(diagnostics.is_empty(), "{source}");
        let mut repaired = vec![diagnostic(&source, "// @ts-expect-error", 2578)];
        let expected = repaired.clone();
        filter(&AuthoredSource::vue(&source), &mut repaired);
        assert_eq!(repaired, expected);
    }
}

#[test]
fn a_later_code_line_and_a_repeated_sibling_are_not_owned() {
    let source = "<template>{{\n// @ts-expect-error\nknown +\nmissing\n}}{{ missing }}</template>";
    let mut first = diagnostic(source, "missing", 2339);
    let mut sibling = first.clone();
    sibling.start = source.rfind("missing").unwrap();
    sibling.end = sibling.start + "missing".len();
    first.end = first.start + "missing".len();
    let original = vec![
        diagnostic(source, "// @ts-expect-error", 2578),
        first,
        sibling,
    ];
    let mut diagnostics = original.clone();
    filter(&AuthoredSource::vue(source), &mut diagnostics);
    assert_eq!(diagnostics, original);
}

#[test]
fn javascript_line_terminators_and_unicode_indentation_match_native_directives() {
    for newline in ["\n", "\r\n", "\r", "\u{2028}", "\u{2029}"] {
        for indentation in ["", "\u{00a0}", "\u{feff}", "\u{3000}"] {
            let source = vize_carton::cstr!(
                "<template>{{{{{newline}{indentation}// @ts-expect-error{newline}{indentation}missing{newline}}}}}</template>"
            );
            let mut diagnostics = vec![
                diagnostic(&source, "// @ts-expect-error", 2578),
                diagnostic(&source, "missing", 2339),
            ];
            filter(&AuthoredSource::vue(&source), &mut diagnostics);
            assert!(diagnostics.is_empty(), "{source:?}");
        }
    }
}

#[test]
fn ordinary_block_comments_and_first_line_block_tokens_do_not_suppress() {
    for expression in [
        "// @ts-expect-error\n/* explanation */\nmissing",
        "/* @ts-expect-error\n explanation */\nmissing",
    ] {
        let source = vize_carton::cstr!("<template>{{{{\n{expression}\n}}}}</template>");
        let original = vec![diagnostic(&source, "missing", 2339)];
        let mut diagnostics = original.clone();
        filter(&AuthoredSource::vue(&source), &mut diagnostics);
        assert_eq!(diagnostics, original, "{source}");
    }
}

#[test]
fn native_directive_prefix_and_final_block_line_grammar_are_preserved() {
    for comment in [
        "// @ts-expect-error-more",
        "// @ts-expect-error_more",
        "/// @ts-expect-error",
        "/* explanation\n @ts-expect-error */",
        "/* explanation\n * @ts-expect-error */",
    ] {
        let source = vize_carton::cstr!("<template>{{{{\n{comment}\nmissing\n}}}}</template>");
        let mut diagnostics = vec![
            diagnostic(&source, comment, 2578),
            diagnostic(&source, "missing", 2339),
        ];
        filter(&AuthoredSource::vue(&source), &mut diagnostics);
        assert!(diagnostics.is_empty(), "{source}");
    }
}

#[test]
fn a_block_comment_on_the_target_line_does_not_block_the_directive() {
    for comment in ["/* explanation */ ", "/* 日本 🧭 */ "] {
        let source = vize_carton::cstr!(
            "<template>{{{{\n// @ts-expect-error\n{comment}missing\n}}}}</template>"
        );
        let mut diagnostics = vec![
            diagnostic(&source, "// @ts-expect-error", 2578),
            diagnostic(&source, "missing", 2339),
        ];
        filter(&AuthoredSource::vue(&source), &mut diagnostics);
        assert!(diagnostics.is_empty(), "{source}");
    }
}
