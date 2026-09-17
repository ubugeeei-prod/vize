use super::{PatternBinding, PatternExpression, PatternKind, parse_match_pattern};
use oxc_span::Span;

#[test]
fn parses_long_form_bindings_with_authored_byte_ranges() {
    let source =
        "{ const value, data: [const head, ...const tail], ...const rest } as result if (head > 0)";
    let arm = parse_match_pattern(source).unwrap();
    let expected = ["value", "head", "tail", "rest", "result"].map(|name| {
        let start = source.find(name).unwrap() as u32;
        PatternBinding {
            name: name.into(),
            span: Span::new(start, start + name.len() as u32),
        }
    });
    assert_eq!(arm.bindings, expected);
    let start = source.find("head >").unwrap() as u32;
    assert_eq!(
        arm.guard,
        Some(PatternExpression {
            text: "head > 0".into(),
            span: Span::new(start, start + 8)
        })
    );
    assert!(matches!(arm.pattern.kind, PatternKind::As { .. }));
}

#[test]
fn accepts_literal_value_structural_and_guard_forms() {
    for source in [
        "_",
        "(_) ",
        "const value",
        "true",
        "false",
        "null",
        "undefined",
        "NaN",
        "Infinity",
        "0",
        "-0",
        "+1",
        ".5",
        "1.",
        "1e-3",
        "-0xFF",
        "0o77",
        "0b10",
        "1n",
        "-1n",
        "0xFFn",
        "'text'",
        r"'a\'b'",
        r"'\u{1f642}'",
        r"'\x61'",
        r"'\u0061'",
        r"'\ud800'",
        "Status.ready",
        "Status[key]",
        "Status['ready']",
        "Status[0]",
        "Status[0n]",
        "Status.default",
        "{}",
        "[]",
        "[1,]",
        "[const first, ...const rest]",
        "[...]",
        "{ a: 1, ... }",
        "{ const a, }",
        "'ready' | 'done' as status",
        "(_ | null) as value",
        "{ const row } if (row.id > 0)",
        "_ if (value /* guard */)",
        "_ if (value as string)",
    ] {
        assert!(
            parse_match_pattern(source).is_ok(),
            "{source}: {:?}",
            parse_match_pattern(source)
        );
    }
}

#[test]
fn rejects_invalid_and_unsupported_pattern_grammar() {
    for source in [
        "",
        "_ trailing",
        "value()",
        "value + 1",
        "value?.key",
        "new Value",
        "this.key",
        "{ field }",
        "{ [key]: _ }",
        "const { value }",
        "const [value]",
        "const",
        "const await",
        "const eval",
        "const arguments",
        "let value",
        "var value",
        "_ as const value",
        "[const value, const value]",
        "const value as value",
        "const a | const b",
        "{ const a } | _",
        "(_ as a) | _",
        "[,,]",
        "[1,,2]",
        "[...rest]",
        "[...const rest,]",
        "[...const rest, _]",
        "{ ...rest }",
        "{ ..., a: _ }",
        "{ ...const rest, }",
        "{ 1n: _ }",
        "{ a: _, a: _ }",
        r"{ a: _, '\x61': _ }",
        r"{ '\u0061': _, 'a': _ }",
        "{ 1: _, '1': _ }",
        "{ 1e21: _, '1e+21': _ }",
        "+1n",
        "1.2n",
        "1_000",
        "01n",
        "0x",
        "1e+",
        "- 1",
        "-Infinity",
        r"'\1'",
        r"'\08'",
        r"'\u{}'",
        r"'\u{110000}'",
        "'unterminated",
        "'line\nfeed'",
        "'line\\\nfeed'",
        "_ if ()",
        "_ if (",
        "_ if (value",
        "_ if (value; other)",
        "_ if (value) trailing",
    ] {
        let error = parse_match_pattern(source).expect_err(source);
        assert!(error.offset as usize <= source.len(), "{source}: {error:?}");
    }
}

#[test]
fn canonical_property_keys_preserve_lone_surrogates() {
    let arm = parse_match_pattern(r"{ '\ud800': _, '\ufffd': _, '\ud801': _ }").unwrap();
    let PatternKind::Object { properties, .. } = arm.pattern.kind else {
        panic!()
    };
    assert_eq!(
        properties
            .iter()
            .map(|p| p.key_value.clone())
            .collect::<Vec<_>>(),
        [vec![0xd800], vec![0xfffd], vec![0xd801]]
    );
    assert!(parse_match_pattern(r"{ '\ud800': _, '\uD800': _ }").is_err());
}

#[test]
fn unicode_binding_spans_and_keyword_boundaries_are_exact() {
    let source = "{ const \u{65e5}\u{672c}, const constx }";
    let arm = parse_match_pattern(source).unwrap();
    assert_eq!(
        arm.bindings,
        [
            PatternBinding {
                name: "\u{65e5}\u{672c}".into(),
                span: Span::new(8, 14)
            },
            PatternBinding {
                name: "constx".into(),
                span: Span::new(22, 28)
            },
        ]
    );
    for source in [
        "constx",
        "constant",
        "ifValue",
        "letValue",
        "asValue",
        "\u{feff}const \u{3b1}",
    ] {
        assert!(parse_match_pattern(source).is_ok(), "{source}");
    }
}

#[test]
fn nesting_is_bounded_without_limiting_wide_patterns() {
    let deep = vize_s0::cstr!("{}0{}", "[".repeat(10000), "]".repeat(10000));
    assert_eq!(
        parse_match_pattern(&deep).unwrap_err().message,
        "Pattern nesting exceeds the supported limit of 128."
    );
    let wide = vize_s0::cstr!("[{}]", vec!["_"; 10000].join(","));
    let PatternKind::Array { elements, rest } = parse_match_pattern(&wide).unwrap().pattern.kind
    else {
        panic!()
    };
    assert_eq!(elements.len(), 10000);
    assert_eq!(rest, None);
}

#[test]
fn incomplete_edits_are_bounded_and_never_panic() {
    for source in [
        "{ kind: 'ready', data: [const item, ...const rest] } as whole if (item > 0)",
        "([1, 2] | [3, 4]) as pair",
        "{ '\\ud800': const value, ...const rest }",
        "{ const \u{65e5}\u{672c} } if (\u{65e5}\u{672c}.length > 0)",
    ] {
        for end in source
            .char_indices()
            .map(|(offset, _)| offset)
            .chain([source.len()])
        {
            if let Err(error) = parse_match_pattern(&source[..end]) {
                assert!(error.offset as usize <= end, "{source}: {error:?}");
            }
        }
    }
}
