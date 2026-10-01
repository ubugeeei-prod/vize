//! Independent source-coordinate goldens for the ordinary native library build.

use vize_l0::{Span, String, cstr};
use vize_l1::dialect::vue3::directive::MAX_DIRECTIVE_DELIMITER_RUNS;
use vize_l1::markup::directive::DirectiveNameError;
use vize_l1::markup::{ArgSyntax, DirectiveName, DirectivePrefix, DirectiveSyntax, VueDirectives};

fn shifted(span: Span, offset: u32) -> Span {
    Span::new(span.start + offset, span.end + offset)
}

fn shifted_arg(arg: Option<ArgSyntax>, offset: u32) -> Option<ArgSyntax> {
    arg.map(|arg| match arg {
        ArgSyntax::Static(span) => ArgSyntax::Static(shifted(span, offset)),
        ArgSyntax::Dynamic(span) => ArgSyntax::Dynamic(shifted(span, offset)),
    })
}

#[test]
fn explicit_goldens_pin_names_arguments_modifiers_and_absolute_utf8_offsets() {
    use DirectivePrefix::{Bind, Full, On, Prop, Slot};
    let cases = [
        ("v-on", Full, (2, 4), None, (4, 4)),
        (
            "v-on:click.stop.prevent",
            Full,
            (2, 4),
            Some(ArgSyntax::Static(Span::new(5, 10))),
            (10, 23),
        ),
        (
            "v-my-dir:名.修飾",
            Full,
            (2, 8),
            Some(ArgSyntax::Static(Span::new(9, 12))),
            (12, 19),
        ),
        (
            "@click.stop",
            On,
            (0, 0),
            Some(ArgSyntax::Static(Span::new(1, 6))),
            (6, 11),
        ),
        (
            ":title",
            Bind,
            (0, 0),
            Some(ArgSyntax::Static(Span::new(1, 6))),
            (6, 6),
        ),
        (
            ".value",
            Prop,
            (0, 0),
            Some(ArgSyntax::Static(Span::new(1, 6))),
            (6, 6),
        ),
        (
            "#default.foo",
            Slot,
            (0, 0),
            Some(ArgSyntax::Static(Span::new(1, 8))),
            (8, 12),
        ),
        (
            "v-bind:[keys['a.b']].camel",
            Full,
            (2, 6),
            Some(ArgSyntax::Dynamic(Span::new(8, 19))),
            (20, 26),
        ),
        (
            ":[keys[indices[0]]].camel",
            Bind,
            (0, 0),
            Some(ArgSyntax::Dynamic(Span::new(2, 18))),
            (19, 25),
        ),
        (
            ":[keys[']']].x",
            Bind,
            (0, 0),
            Some(ArgSyntax::Dynamic(Span::new(2, 11))),
            (12, 14),
        ),
        ("v-", Full, (2, 2), None, (2, 2)),
        ("v-:", Full, (2, 2), None, (3, 3)),
        ("v-.x", Full, (2, 2), None, (2, 4)),
        (":", Bind, (0, 0), None, (1, 1)),
        (":[", Bind, (0, 0), None, (2, 2)),
        (
            ":[foo",
            Bind,
            (0, 0),
            Some(ArgSyntax::Dynamic(Span::new(2, 5))),
            (5, 5),
        ),
        (
            ":[(]]",
            Bind,
            (0, 0),
            Some(ArgSyntax::Dynamic(Span::new(2, 5))),
            (5, 5),
        ),
        (
            "v-on:a[",
            Full,
            (2, 4),
            Some(ArgSyntax::Static(Span::new(5, 6))),
            (7, 7),
        ),
        (
            ":[key]tail.mod",
            Bind,
            (0, 0),
            Some(ArgSyntax::Static(Span::new(6, 10))),
            (10, 14),
        ),
        (
            ":[foo]..",
            Bind,
            (0, 0),
            Some(ArgSyntax::Dynamic(Span::new(2, 5))),
            (6, 8),
        ),
        (
            ":[]",
            Bind,
            (0, 0),
            Some(ArgSyntax::Dynamic(Span::new(2, 2))),
            (3, 3),
        ),
    ];
    let offset = 17;
    let prefix = "表紙🍣-------";
    assert_eq!(prefix.len(), offset as usize);
    for (raw, prefix_kind, name, arg, modifiers) in cases {
        let expected = DirectiveName {
            prefix: prefix_kind,
            name: shifted(Span::new(name.0, name.1), offset),
            arg: shifted_arg(arg, offset),
            modifiers: shifted(Span::new(modifiers.0, modifiers.1), offset),
        };
        let actual = VueDirectives.decompose(raw, offset).unwrap().unwrap();
        assert_eq!(actual, expected, "{raw}");
        let authored = cstr!("{prefix}{raw}");
        let mut spans = vec![actual.name, actual.modifiers];
        if let Some(ArgSyntax::Static(span) | ArgSyntax::Dynamic(span)) = actual.arg {
            spans.push(span);
        }
        for span in spans {
            assert!(
                authored
                    .get(span.start as usize..span.end as usize)
                    .is_some(),
                "{raw}: {span:?}"
            );
        }
    }
}

#[test]
fn ordinary_attributes_are_distinct_from_source_admission_errors() {
    for raw in ["", "class", "data-v-on", "V-on", "v", "v_foo"] {
        assert_eq!(VueDirectives.decompose(raw, 0), Ok(None), "{raw}");
    }
    assert_eq!(VueDirectives.decompose("", u32::MAX), Ok(None));
    for raw in ["class", "@", "v-pre", "v-名"] {
        assert_eq!(
            VueDirectives.decompose(raw, u32::MAX),
            Err(DirectiveNameError::OffsetOverflow),
            "{raw}"
        );
    }
    assert_eq!(
        VueDirectives
            .decompose("v-pre", u32::MAX - 5)
            .unwrap()
            .unwrap()
            .name,
        Span::new(u32::MAX - 3, u32::MAX),
    );
}

#[test]
fn pre_is_the_parsed_full_name_including_modified_and_malformed_heads() {
    for raw in [
        "v-pre",
        "v-pre.foo",
        "v-pre:arg",
        "v-pre:[broken",
        "v-pre[broken",
        "v-pre:.",
        "v-pre..",
        "v-pre:[",
        "v-pre:",
    ] {
        let authored = cstr!("中{raw}");
        let shape = VueDirectives.decompose(raw, 3).unwrap().unwrap();
        assert_eq!(shape.prefix, DirectivePrefix::Full, "{raw}");
        assert_eq!(shape.name, Span::new(5, 8), "{raw}");
        assert_eq!(shape.name.slice(&authored), "pre", "{raw}");
    }
    for raw in [
        "v-pretty", "v-prefix", "v-PRE", ":pre", "@pre", "#pre", ".pre",
    ] {
        let shape = VueDirectives.decompose(raw, 0).unwrap().unwrap();
        assert!(
            shape.prefix != DirectivePrefix::Full || shape.name.slice(raw) != "pre",
            "{raw}"
        );
    }
}

#[test]
fn dynamic_argument_quotes_escapes_and_nested_templates_preserve_the_whole_expression() {
    for expression in [
        "keys['a.b']",
        "keys['[']",
        "keys[']']",
        r#"keys["\"]"]"#,
        "`prefix${keys['[']}`",
        "`prefix${`nested${keys[']']}`}`",
        "`escaped\\`][${keys[index]}`",
        "({a:`outer${{b:`inner${keys[']']}`}.b}`}).a",
        "鍵[索引['値.']]",
    ] {
        let raw = cstr!("v-on:[{expression}].stop.prevent");
        let source = cstr!("中{raw}");
        let shape = VueDirectives.decompose(&raw, 3).unwrap().unwrap();
        let Some(ArgSyntax::Dynamic(span)) = shape.arg else {
            panic!("{raw}: {shape:?}")
        };
        assert_eq!(span.start, 9);
        assert_eq!(span.slice(&source), expression, "{raw}");
        assert_eq!(shape.modifiers.slice(&source), ".stop.prevent", "{raw}");
    }
    for raw in [":[keys['broken.more", ":[`broken.more", ":[keys['escaped\\"] {
        let shape = VueDirectives.decompose(raw, 0).unwrap().unwrap();
        assert!(
            matches!(shape.arg, Some(ArgSyntax::Dynamic(span)) if span == Span::new(2, raw.len() as u32))
        );
        assert_eq!(
            shape.modifiers,
            Span::new(raw.len() as u32, raw.len() as u32)
        );
    }
}

#[test]
fn repeated_brackets_use_scalar_counts_and_excess_distinct_runs_are_explicitly_unadmitted() {
    let expression = cstr!("{}key{}", "[".repeat(20_000), "]".repeat(20_000));
    let raw = cstr!(":[{expression}].prop");
    let shape = VueDirectives.decompose(&raw, 0).unwrap().unwrap();
    assert_eq!(shape.arg, Some(ArgSyntax::Dynamic(Span::new(2, 40_005))));
    assert_eq!(shape.modifiers, Span::new(40_006, 40_011));

    let alternating = |count: usize| {
        let opens: String = (0..count)
            .map(|i| if i % 2 == 0 { '(' } else { '[' })
            .collect();
        let closes: String = opens
            .chars()
            .rev()
            .map(|c| if c == '(' { ')' } else { ']' })
            .collect();
        cstr!(":[{opens}key{closes}]")
    };
    assert!(
        VueDirectives
            .decompose(&alternating(MAX_DIRECTIVE_DELIMITER_RUNS - 1), 0)
            .unwrap()
            .is_some()
    );
    assert_eq!(
        VueDirectives.decompose(&alternating(MAX_DIRECTIVE_DELIMITER_RUNS), 0),
        Err(DirectiveNameError::NestingLimit)
    );
}

#[test]
fn transitional_public_paths_resolve_the_same_native_provider() {
    let provider: vize_l1::dialect::vue3::VueDirectives = VueDirectives;
    let alias: vize_l1::markup::directive::VueDirectives = provider;
    assert_eq!(
        alias.decompose("v-pre.foo", 0),
        VueDirectives.decompose("v-pre.foo", 0)
    );
}
