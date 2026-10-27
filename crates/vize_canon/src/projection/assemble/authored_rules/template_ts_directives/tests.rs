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
        let source = format!(
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
        "// @ts-expect-error_more\nmissing",
        "// @ts-expect-error-more\nmissing",
        "// @ts-expect-error\n\nmissing",
        "// @ts-expect-error\n// ordinary comment\nmissing",
        "/* @ts-expect-error */ missing",
        "// @ts-expect-error\nmissing(",
        "/* @ts-expect-error\nmissing",
    ] {
        let source = format!("<template>{{{{ {expression} }}}}</template>");
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
