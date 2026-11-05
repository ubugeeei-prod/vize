//! Keep the legacy JavaScript-delimiter AST and native Vue attribute-head
//! syntax as separate contracts over the same complete authored inputs.
//!
//! The native goldens were checked with compiler-core 3.5.35 and 3.5.44:
//! https://github.com/vuejs/core/blob/v3.5.35/packages/compiler-core/src/tokenizer.ts
//! (`stateInDirArg` / `stateInDynamicDirArg`) and `parser.ts` (`ondirarg`).
//! A `]` leaves Vue's dynamic state even inside a JavaScript string; a later
//! `.` starts modifiers. The legacy lexer instead balances JS delimiters and
//! quotes. SFC mode does not make those two grammars equivalent.

use vize_armature::{
    ExpressionNode, ParseMode, ParserOptions, PropNode, TemplateChildNode, parse_with_options,
};
use vize_l0::{Allocator, ErrorCode, Span, cstr};
use vize_l1::markup::{ArgSyntax, DirectiveName, DirectivePrefix, DirectiveSyntax, VueDirectives};

fn shifted(span: Span, offset: u32) -> Span {
    Span::new(span.start + offset, span.end + offset)
}

fn shifted_arg(arg: ArgSyntax, offset: u32) -> ArgSyntax {
    match arg {
        ArgSyntax::Static(span) => ArgSyntax::Static(shifted(span, offset)),
        ArgSyntax::Dynamic(span) => ArgSyntax::Dynamic(shifted(span, offset)),
    }
}

fn argument_parts(arg: ArgSyntax) -> (Span, bool) {
    match arg {
        ArgSyntax::Static(span) => (span, true),
        ArgSyntax::Dynamic(span) => (span, false),
    }
}

#[test]
fn directive_heads_preserve_independent_vue_goldens_and_legacy_parser_ast() {
    use ArgSyntax::{Dynamic, Static};
    use DirectivePrefix::{Bind, Full, On, Prop, Slot};
    // Each span/modifier start is relative to the argument's authored start,
    // independent of both implementations. All 14 original inputs remain.
    let arguments = [
        ("name", Static(Span::new(0, 4)), 4),
        ("名", Static(Span::new(0, 3)), 3),
        ("[state.names[state.index]]", Dynamic(Span::new(1, 25)), 26),
        (
            "[state.names[indices[state.index]]]",
            Dynamic(Span::new(1, 34)),
            35,
        ),
        (
            "[[state.first,state.second][state.index]]",
            Dynamic(Span::new(1, 40)),
            41,
        ),
        ("[state.keys[']']]", Dynamic(Span::new(1, 16)), 17),
        (r#"[state.keys["]"]]"#, Dynamic(Span::new(1, 16)), 17),
        (r#"[state.keys["\"]"]]"#, Dynamic(Span::new(1, 18)), 19),
        (r#"[state.keys['\']']]"#, Dynamic(Span::new(1, 18)), 19),
        (
            r#"[state.keys['\\'][state.index]]"#,
            Dynamic(Span::new(1, 30)),
            31,
        ),
        (
            "[({'key]':state.field})['key]']]",
            Dynamic(Span::new(1, 14)),
            15,
        ),
        (
            "[`key${state.names[state.index]}`]",
            Dynamic(Span::new(1, 33)),
            34,
        ),
        (
            "[`key${`nested${state.keys[']']}`}`]",
            Dynamic(Span::new(1, 35)),
            36,
        ),
        ("[鍵[索引['値.']]]", Dynamic(Span::new(1, 20)), 21),
    ];
    for (prefix, prefix_kind, logical_name) in [
        (":", Bind, "bind"),
        ("v-bind:", Full, "bind"),
        (".", Prop, "bind"),
        ("@", On, "on"),
        ("v-on:", Full, "on"),
        ("#", Slot, "slot"),
        ("v-slot:", Full, "slot"),
        ("v-custom:", Full, "custom"),
    ] {
        for (argument, native_arg, native_modifier_start) in arguments {
            for modifiers in ["", ".foo.bar", ".prop"] {
                let raw = cstr!("{prefix}{argument}{modifiers}");
                let source = cstr!("<!--中🍣--><Child {raw}=\"value\" id=\"tail\"/>");
                let offset = source.find(raw.as_str()).unwrap() as u32;
                let argument_offset = offset + prefix.len() as u32;
                let expected_native = DirectiveName {
                    prefix: prefix_kind,
                    name: if prefix_kind == Full {
                        Span::new(offset + 2, argument_offset - 1)
                    } else {
                        Span::new(offset, offset)
                    },
                    arg: Some(shifted_arg(native_arg, argument_offset)),
                    modifiers: Span::new(
                        argument_offset + native_modifier_start,
                        offset + raw.len() as u32,
                    ),
                };
                let native = VueDirectives.decompose(&raw, offset).unwrap().unwrap();
                assert_eq!(native, expected_native, "{source}");
                let allocator = Allocator::new();
                let (root, errors) = parse_with_options(
                    &allocator,
                    &source,
                    ParserOptions {
                        mode: ParseMode::Sfc,
                        ..Default::default()
                    },
                );
                assert!(errors.is_empty(), "{source}: {errors:?}");
                let TemplateChildNode::Element(element) = root.children.last().unwrap() else {
                    panic!("expected component: {source}");
                };
                let PropNode::Directive(legacy) = &element.props[0] else {
                    panic!("expected directive: {source}");
                };
                assert_eq!(legacy.name, logical_name, "{source}");
                let Some(ExpressionNode::Simple(legacy_arg)) = &legacy.arg else {
                    panic!("expected argument: {source}");
                };
                let (legacy_argument, legacy_span, is_static) = match argument
                    .strip_prefix('[')
                    .and_then(|body| body.strip_suffix(']'))
                {
                    Some(body) => (
                        body,
                        Span::new(
                            argument_offset + 1,
                            argument_offset + argument.len() as u32 - 1,
                        ),
                        false,
                    ),
                    None => (
                        argument,
                        Span::new(argument_offset, argument_offset + argument.len() as u32),
                        true,
                    ),
                };
                assert_eq!(legacy_arg.content, legacy_argument, "{source}");
                assert_eq!(legacy_arg.is_static, is_static, "{source}");
                assert_eq!(legacy_arg.loc.span, legacy_span, "{source}");
                let mut expected: Vec<&str> = modifiers
                    .split('.')
                    .filter(|modifier| !modifier.is_empty())
                    .collect();
                if prefix_kind == Prop && !expected.contains(&"prop") {
                    expected.insert(0, "prop");
                }
                let actual: Vec<&str> = legacy
                    .modifiers
                    .iter()
                    .map(|modifier| modifier.content)
                    .collect();
                assert_eq!(actual, expected, "{source}");
                if native_modifier_start != argument.len() as u32 {
                    // Reintroducing the old JS-aware native scan must fail:
                    // the whole legacy expression is not Vue's lexical arg.
                    assert_ne!(
                        argument_parts(native.arg.unwrap()).0.slice(&source),
                        legacy_argument,
                        "{source}",
                    );
                }
                assert!(
                    matches!(&element.props[1], PropNode::Attribute(attr) if attr.name == "id")
                );
            }
        }
    }
}

#[test]
fn malformed_argument_recovery_preserves_the_legacy_argument_evidence() {
    use ArgSyntax::{Dynamic, Static};
    use DirectivePrefix::{Bind, Full};
    // Native coordinates are for a complete raw head, excluding brackets and
    // the markup terminator. Vue's full-markup AST `arg.loc` includes brackets
    // and, for an unclosed dynamic state, the following `=` during recovery.
    // In particular `v-on:a[` recovers to static `a[=` in that AST; the raw-only
    // provider retains static `a[` without consuming a byte outside its input.
    // The legacy lexer separately retains its JS-delimiter recovery evidence.
    for (raw, native_prefix, native_name, native_arg, native_modifiers, legacy_arg, missing_end) in [
        (
            ":[state.names[state.index]",
            Bind,
            Span::new(0, 0),
            Dynamic(Span::new(2, 25)),
            Span::new(26, 26),
            Dynamic(Span::new(2, 26)),
            true,
        ),
        (
            ":[state.names[state.index",
            Bind,
            Span::new(0, 0),
            Dynamic(Span::new(2, 25)),
            Span::new(25, 25),
            Dynamic(Span::new(2, 25)),
            true,
        ),
        (
            ":[state.keys['broken",
            Bind,
            Span::new(0, 0),
            Dynamic(Span::new(2, 20)),
            Span::new(20, 20),
            Dynamic(Span::new(2, 20)),
            true,
        ),
        (
            ":[(]]",
            Bind,
            Span::new(0, 0),
            Dynamic(Span::new(2, 4)),
            Span::new(5, 5),
            Dynamic(Span::new(2, 5)),
            true,
        ),
        (
            "v-on:a[",
            Full,
            Span::new(2, 4),
            Static(Span::new(5, 7)),
            Span::new(7, 7),
            Static(Span::new(5, 6)),
            true,
        ),
        (
            ":[key]tail.mod",
            Bind,
            Span::new(0, 0),
            Dynamic(Span::new(2, 9)),
            Span::new(10, 14),
            Static(Span::new(6, 10)),
            false,
        ),
    ] {
        let source = cstr!("<Child {raw}=\"value\" id=\"tail\"/>");
        let expected_native = DirectiveName {
            prefix: native_prefix,
            name: shifted(native_name, 7),
            arg: Some(shifted_arg(native_arg, 7)),
            modifiers: shifted(native_modifiers, 7),
        };
        let native = VueDirectives.decompose(raw, 7).unwrap().unwrap();
        assert_eq!(native, expected_native, "{source}");
        let allocator = Allocator::new();
        let (root, errors) = parse_with_options(
            &allocator,
            &source,
            ParserOptions {
                mode: ParseMode::Sfc,
                ..Default::default()
            },
        );
        let TemplateChildNode::Element(element) = &root.children[0] else {
            panic!("expected recovered component: {source}");
        };
        let PropNode::Directive(legacy) = &element.props[0] else {
            panic!("expected recovered directive: {source}");
        };
        let Some(ExpressionNode::Simple(actual_arg)) = &legacy.arg else {
            panic!("expected recovered argument: {source}");
        };
        let (span, is_static) = argument_parts(shifted_arg(legacy_arg, 7));
        assert_eq!(actual_arg.content, span.slice(&source), "{source}");
        assert_eq!(actual_arg.is_static, is_static, "{source}");
        assert_eq!(actual_arg.loc.span, span, "{source}");
        let actual_modifiers: Vec<_> = legacy
            .modifiers
            .iter()
            .map(|modifier| modifier.content)
            .collect();
        let expected_modifiers: &[&str] = if missing_end { &[] } else { &["mod"] };
        assert_eq!(actual_modifiers, expected_modifiers, "{source}");
        let actual_errors: Vec<_> = errors
            .iter()
            .map(|error| {
                (
                    error.code,
                    error.message.as_str(),
                    error.loc.as_ref().map(|loc| loc.span),
                )
            })
            .collect();
        let expected_errors = if missing_end {
            let boundary = 7 + raw.len() as u32;
            vec![(
                ErrorCode::MissingDynamicDirectiveArgumentEnd,
                "Dynamic directive argument is missing its closing `]`; inferred the argument end at the next tag boundary.",
                Some(Span::new(boundary, boundary + 1)),
            )]
        } else {
            vec![]
        };
        assert_eq!(actual_errors, expected_errors, "{source}");
        if native_arg != legacy_arg {
            assert_ne!(native.arg, Some(shifted_arg(legacy_arg, 7)), "{source}");
        }
    }
}

#[test]
fn parsed_pre_name_matches_legacy_verbatim_policy_for_recovered_heads() {
    for raw in [
        "v-pre",
        "v-pre.foo",
        "v-pre:arg",
        "v-pre:[broken",
        "v-pre:.",
        "v-pre..",
        "v-pre:[",
        "v-pre:",
    ] {
        let source = cstr!(
            "<div {raw} :title=\"value\">{{{{ msg }}}}<span @click=\"handler\">{{{{ nested }}}}</span></div>"
        );
        let native = VueDirectives.decompose(raw, 5).unwrap().unwrap();
        assert_eq!(native.prefix, DirectivePrefix::Full, "{source}");
        assert_eq!(native.name.slice(&source), "pre", "{source}");
        let allocator = Allocator::new();
        let (root, _) = parse_with_options(
            &allocator,
            &source,
            ParserOptions {
                mode: ParseMode::Sfc,
                ..Default::default()
            },
        );
        let TemplateChildNode::Element(div) = &root.children[0] else {
            panic!("expected verbatim div: {source}");
        };
        assert_eq!(div.props.len(), 1, "{source}");
        assert!(matches!(&div.props[0], PropNode::Attribute(attr) if attr.name == ":title"));
        assert!(
            matches!(&div.children[0], TemplateChildNode::Text(text) if text.content == "{{ msg }}")
        );
        let TemplateChildNode::Element(span) = &div.children[1] else {
            panic!("expected verbatim span: {source}");
        };
        assert!(matches!(&span.props[0], PropNode::Attribute(attr) if attr.name == "@click"));
        assert!(
            matches!(&span.children[0], TemplateChildNode::Text(text) if text.content == "{{ nested }}")
        );
    }
}
