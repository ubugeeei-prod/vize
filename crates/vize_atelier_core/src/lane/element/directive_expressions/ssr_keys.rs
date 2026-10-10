use super::process_directive_expressions;
use crate::lane::TransformContext;
use crate::{
    ExpressionNode, PropNode, RuntimeHelper, TemplateChildNode,
    options::{BindingMetadata, BindingType, TransformOptions},
    parser::Parser,
};
use vize_l0::{Allocator, FxHashMap};

#[test]
fn nonidentifier_arguments_use_existing_context_setup_global_and_scope_rules() {
    for (argument, expected, local, setup) in [
        ("keys['name]']", "_ctx.keys['name]']", false, false),
        (
            "keys[indices[index]]",
            "_ctx.keys[_ctx.indices[_ctx.index]]",
            false,
            false,
        ),
        ("Math.max(0,index)", "Math.max(0,_ctx.index)", false, false),
        ("row.key+suffix", "row.key+_ctx.suffix", true, false),
        ("keys[index]", "$setup.keys[_ctx.index]", false, true),
    ] {
        for spelling in [":", "v-bind:", "@", "v-on:"] {
            let arena = Allocator::new();
            let source = vize_l0::cstr!("<Child {spelling}[{argument}]=\"value\"></Child>");
            let (mut root, errors) = Parser::new(&arena, &source).parse();
            assert!(errors.is_empty(), "{errors:?}");
            let TemplateChildNode::Element(el) = &mut root.children[0] else {
                panic!("component owner")
            };
            let original_span = match &el.props[0] {
                PropNode::Directive(dir) => dir.arg.as_ref().map(|arg| match arg {
                    ExpressionNode::Simple(simple) => simple.loc.span,
                    ExpressionNode::Compound(compound) => compound.loc.span,
                }),
                _ => None,
            };
            let mut bindings = FxHashMap::default();
            bindings.insert("keys".into(), BindingType::SetupRef);
            let mut context = TransformContext::new(
                &arena,
                &source,
                TransformOptions {
                    prefix_identifiers: true,
                    ssr: true,
                    binding_metadata: setup.then_some(BindingMetadata {
                        bindings,
                        props_aliases: FxHashMap::default(),
                        is_script_setup: true,
                    }),
                    ..Default::default()
                },
            );
            if local {
                context.enter_v_for_scope(Some("row"), None, None, "rows");
            }
            process_directive_expressions(&mut context, el);
            assert!(context.errors.is_empty(), "{:?}", context.errors);
            let PropNode::Directive(dir) = &el.props[0] else {
                panic!("original directive")
            };
            let Some(ExpressionNode::Simple(argument)) = &dir.arg else {
                panic!("processed argument")
            };
            assert_eq!(argument.content, expected, "{source}");
            assert_eq!(Some(argument.loc.span), original_span, "{source}");
        }
    }
}

#[test]
fn setup_key_expressions_keep_binding_kinds_helpers_and_authored_spans() {
    for (kind, inline_key) in [
        (BindingType::SetupRef, "keys.value[_ctx.index]"),
        (BindingType::SetupMaybeRef, "_unref(keys)[_ctx.index]"),
        (BindingType::SetupConst, "keys[_ctx.index]"),
    ] {
        for inline in [false, true] {
            for spelling in [":", "v-bind:", "@", "v-on:"] {
                let arena = Allocator::new();
                let source = vize_l0::cstr!("<Child {spelling}[keys[index]]=\"value\"></Child>");
                let (mut root, errors) = Parser::new(&arena, &source).parse();
                assert!(errors.is_empty(), "{errors:?}");
                let TemplateChildNode::Element(el) = &mut root.children[0] else {
                    panic!("component owner")
                };
                let PropNode::Directive(dir) = &el.props[0] else {
                    panic!("original directive")
                };
                let Some(ExpressionNode::Simple(argument)) = &dir.arg else {
                    panic!("original key")
                };
                let span = argument.loc.span;
                let mut bindings = FxHashMap::default();
                bindings.insert("keys".into(), kind);
                let mut context = TransformContext::new(
                    &arena,
                    &source,
                    TransformOptions {
                        prefix_identifiers: true,
                        ssr: true,
                        inline,
                        binding_metadata: Some(BindingMetadata {
                            bindings,
                            props_aliases: FxHashMap::default(),
                            is_script_setup: true,
                        }),
                        ..Default::default()
                    },
                );
                process_directive_expressions(&mut context, el);
                assert!(context.errors.is_empty(), "{:?}", context.errors);
                let PropNode::Directive(dir) = &el.props[0] else {
                    panic!("same directive")
                };
                let Some(ExpressionNode::Simple(argument)) = &dir.arg else {
                    panic!("processed key")
                };
                assert_eq!(
                    argument.content,
                    if inline {
                        inline_key
                    } else {
                        "$setup.keys[_ctx.index]"
                    }
                );
                assert_eq!(argument.loc.span, span);
                assert_eq!(span.slice(&source), "keys[index]");
                assert!(argument.is_ref_transformed);
                assert_eq!(
                    context.has_helper(RuntimeHelper::Unref),
                    inline && kind == BindingType::SetupMaybeRef
                );
            }
        }
    }
}

fn collect_keys(
    children: &[TemplateChildNode<'_>],
    result: &mut Vec<(vize_l0::Span, std::string::String)>,
) {
    for child in children {
        match child {
            TemplateChildNode::Element(el) => {
                for prop in &el.props {
                    if let PropNode::Directive(dir) = prop
                        && matches!(dir.name, "bind" | "on")
                        && let Some(ExpressionNode::Simple(arg)) = &dir.arg
                        && !arg.is_static
                    {
                        result.push((arg.loc.span, arg.content.to_owned()));
                    }
                }
                collect_keys(&el.children, result);
            }
            TemplateChildNode::For(node) => collect_keys(&node.children, result),
            _ => {}
        }
    }
}

#[test]
fn actual_for_and_slot_traversal_keep_locals_and_do_not_leak_them_to_owners_or_siblings() {
    for (source, expected) in [
        (
            "<Child v-for=\"(row,index) in rows\" :[row.keys[index]+suffix]=\"value\"></Child><Child :[row.keys[index]+suffix]=\"value\"></Child>",
            vec![
                "row.keys[index]+_ctx.suffix",
                "_ctx.row.keys[_ctx.index]+_ctx.suffix",
            ],
        ),
        (
            "<Host v-slot=\"{row,index}\" :[row.keys[index]+suffix]=\"value\"><Child :[row.keys[index]+suffix]=\"value\"></Child></Host><Child :[row.keys[index]+suffix]=\"value\"></Child>",
            vec![
                "_ctx.row.keys[_ctx.index]+_ctx.suffix",
                "row.keys[index]+_ctx.suffix",
                "_ctx.row.keys[_ctx.index]+_ctx.suffix",
            ],
        ),
    ] {
        let arena = Allocator::new();
        let (mut root, errors) = Parser::new(&arena, source).parse();
        assert!(errors.is_empty(), "{errors:?}");
        let mut before = Vec::new();
        collect_keys(&root.children, &mut before);
        let errors = crate::lane::transform(
            &arena,
            &mut root,
            TransformOptions {
                prefix_identifiers: true,
                ssr: true,
                ..Default::default()
            },
            None,
        );
        assert!(errors.is_empty(), "{errors:?}");
        let mut after = Vec::new();
        collect_keys(&root.children, &mut after);
        assert_eq!(
            after
                .iter()
                .map(|(_, key)| key.as_str())
                .collect::<Vec<_>>(),
            expected
        );
        assert_eq!(
            after.iter().map(|(span, _)| *span).collect::<Vec<_>>(),
            before.iter().map(|(span, _)| *span).collect::<Vec<_>>()
        );
    }
}

#[test]
fn literal_and_bare_ssr_keys_and_default_dom_vapor_carriers_keep_original_content() {
    for (ssr, vapor, arguments) in [
        (
            true,
            false,
            vec![
                "key",
                "true",
                "false",
                "null",
                "undefined",
                "0",
                "'literal'",
            ],
        ),
        (false, false, vec!["keys[index]", "key", "'literal'"]),
        (false, true, vec!["keys[index]", "key", "'literal'"]),
    ] {
        for argument in arguments {
            for spelling in [":", "v-bind:", "@", "v-on:"] {
                let arena = Allocator::new();
                let source = vize_l0::cstr!("<Child {spelling}[{argument}]=\"value\"></Child>");
                let (mut root, errors) = Parser::new(&arena, &source).parse();
                assert!(errors.is_empty(), "{errors:?}");
                let mut before = Vec::new();
                collect_keys(&root.children, &mut before);
                let TemplateChildNode::Element(el) = &mut root.children[0] else {
                    panic!("component owner")
                };
                let mut context = TransformContext::new(
                    &arena,
                    &source,
                    TransformOptions {
                        prefix_identifiers: true,
                        ssr,
                        vapor,
                        ..Default::default()
                    },
                );
                process_directive_expressions(&mut context, el);
                assert!(context.errors.is_empty(), "{:?}", context.errors);
                let mut after = Vec::new();
                collect_keys(&root.children, &mut after);
                assert_eq!(after, before, "{source} SSR={ssr} Vapor={vapor}");
            }
        }
    }
}
