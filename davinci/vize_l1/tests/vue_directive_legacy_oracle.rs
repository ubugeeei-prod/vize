//! Dev-only legacy AST oracle; explicit coordinate goldens live separately.

use vize_armature::{ExpressionNode, PropNode, TemplateChildNode, parse};
use vize_l0::{Allocator, cstr};
use vize_l1::markup::{ArgSyntax, DirectivePrefix, DirectiveSyntax, VueDirectives};

#[test]
fn admitted_directive_heads_match_the_legacy_parser_ast() {
    let arguments = [
        "name",
        "名",
        "[state.names[state.index]]",
        "[state.names[indices[state.index]]]",
        "[[state.first,state.second][state.index]]",
        "[state.keys[']']]",
        r#"[state.keys["]"]]"#,
        r#"[state.keys["\"]"]]"#,
        r#"[state.keys['\']']]"#,
        r#"[state.keys['\\'][state.index]]"#,
        "[({'key]':state.field})['key]']]",
        "[`key${state.names[state.index]}`]",
        "[`key${`nested${state.keys[']']}`}`]",
        "[鍵[索引['値.']]]",
    ];
    for prefix in [
        ":",
        "v-bind:",
        ".",
        "@",
        "v-on:",
        "#",
        "v-slot:",
        "v-custom:",
    ] {
        for argument in arguments {
            for modifiers in ["", ".foo.bar", ".prop"] {
                let raw = cstr!("{prefix}{argument}{modifiers}");
                let source = cstr!("<!--中🍣--><Child {raw}=\"value\" id=\"tail\"/>");
                let offset = source.find(raw.as_str()).unwrap() as u32;
                let native = VueDirectives.decompose(&raw, offset).unwrap().unwrap();
                let allocator = Allocator::new();
                let (root, errors) = parse(&allocator, &source);
                assert!(errors.is_empty(), "{source}: {errors:?}");
                let TemplateChildNode::Element(element) = root.children.last().unwrap() else {
                    panic!("expected component: {source}");
                };
                let PropNode::Directive(legacy) = &element.props[0] else {
                    panic!("expected directive: {source}");
                };
                let logical_name = match native.prefix {
                    DirectivePrefix::Full => native.name.slice(&source),
                    DirectivePrefix::Bind | DirectivePrefix::Prop => "bind",
                    DirectivePrefix::On => "on",
                    DirectivePrefix::Slot => "slot",
                };
                assert_eq!(legacy.name, logical_name, "{source}");
                let Some(ExpressionNode::Simple(legacy_arg)) = &legacy.arg else {
                    panic!("expected argument: {source}");
                };
                let (native_span, is_static) = match native.arg.unwrap() {
                    ArgSyntax::Static(span) => (span, true),
                    ArgSyntax::Dynamic(span) => (span, false),
                };
                assert_eq!(legacy_arg.content, native_span.slice(&source), "{source}");
                assert_eq!(legacy_arg.is_static, is_static, "{source}");
                assert_eq!(legacy_arg.loc.span, native_span, "{source}");
                let mut expected: Vec<&str> = native
                    .modifiers
                    .slice(&source)
                    .split('.')
                    .filter(|modifier| !modifier.is_empty())
                    .collect();
                if native.prefix == DirectivePrefix::Prop && !expected.contains(&"prop") {
                    expected.insert(0, "prop");
                }
                let actual: Vec<&str> = legacy
                    .modifiers
                    .iter()
                    .map(|modifier| modifier.content)
                    .collect();
                assert_eq!(actual, expected, "{source}");
                assert!(
                    matches!(&element.props[1], PropNode::Attribute(attr) if attr.name == "id")
                );
            }
        }
    }
}

#[test]
fn malformed_argument_recovery_preserves_the_legacy_argument_evidence() {
    for raw in [
        ":[state.names[state.index]",
        ":[state.names[state.index",
        ":[state.keys['broken",
        ":[(]]",
        "v-on:a[",
        ":[key]tail.mod",
    ] {
        let source = cstr!("<Child {raw}=\"value\" id=\"tail\"/>");
        let native = VueDirectives.decompose(raw, 7).unwrap().unwrap();
        let allocator = Allocator::new();
        let (root, _) = parse(&allocator, &source);
        let TemplateChildNode::Element(element) = &root.children[0] else {
            panic!("expected recovered component: {source}");
        };
        let PropNode::Directive(legacy) = &element.props[0] else {
            panic!("expected recovered directive: {source}");
        };
        let Some(ExpressionNode::Simple(legacy_arg)) = &legacy.arg else {
            panic!("expected recovered argument: {source}");
        };
        let (span, is_static) = match native.arg.unwrap() {
            ArgSyntax::Static(span) => (span, true),
            ArgSyntax::Dynamic(span) => (span, false),
        };
        assert_eq!(legacy_arg.content, span.slice(&source), "{source}");
        assert_eq!(legacy_arg.is_static, is_static, "{source}");
        assert_eq!(legacy_arg.loc.span, span, "{source}");
    }
}

#[test]
fn parsed_pre_name_matches_legacy_verbatim_policy_for_recovered_heads() {
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
        let source = cstr!(
            "<div {raw} :title=\"value\">{{{{ msg }}}}<span @click=\"handler\">{{{{ nested }}}}</span></div>"
        );
        let native = VueDirectives.decompose(raw, 5).unwrap().unwrap();
        assert_eq!(native.prefix, DirectivePrefix::Full, "{source}");
        assert_eq!(native.name.slice(&source), "pre", "{source}");
        let allocator = Allocator::new();
        let (root, _) = parse(&allocator, &source);
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
