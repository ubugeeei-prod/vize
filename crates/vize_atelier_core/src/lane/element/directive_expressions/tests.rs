#[cfg(test)]
#[expect(
    clippy::expect_used,
    clippy::disallowed_types,
    reason = "original parser/argument custody regression laws"
)]
mod original_argument_ownership {
    use super::super::process_directive_expressions;
    use crate::{
        ErrorCode, ExpressionNode, PropNode, RuntimeHelper, TemplateChildNode,
        lane::TransformContext,
        options::{BindingMetadata, BindingType, TransformOptions},
        parser::parse,
    };
    use vize_l0::{Allocator, FxHashMap, Span};

    const APP: &str = include_str!(
        "../../../../../../tests/_fixtures/differential/compiler/setup-dynamic-args-7881/App.vue.txt"
    );

    fn original_template() -> &'static str {
        APP.split_once("<template>\n")
            .expect("original template header")
            .1
            .split_once("</template>")
            .expect("original template end")
            .0
    }

    #[test]
    fn original_setup_arguments_keep_their_authored_spans_and_required_helpers() {
        for (kind, inline_content, separate_content) in [
            (
                BindingType::SetupRef,
                ["evt.value", "attr.value"],
                ["$setup.evt", "$setup.attr"],
            ),
            (
                BindingType::SetupMaybeRef,
                ["_unref(evt)", "_unref(attr)"],
                ["$setup.evt", "$setup.attr"],
            ),
            (
                BindingType::SetupConst,
                ["evt", "attr"],
                ["$setup.evt", "$setup.attr"],
            ),
        ] {
            for inline in [true, false] {
                let allocator = Allocator::new();
                let source = original_template();
                let (mut root, errors) = parse(&allocator, source);
                assert!(errors.is_empty(), "{errors:?}");
                let el = root
                    .children
                    .iter_mut()
                    .find_map(|node| match node {
                        TemplateChildNode::Element(el) => Some(el),
                        _ => None,
                    })
                    .expect("original button owner");
                let owner = &**el as *const _;
                let original: std::vec::Vec<_> = el
                    .props
                    .iter()
                    .filter_map(|prop| match prop {
                        PropNode::Directive(dir) => match &dir.arg {
                            Some(ExpressionNode::Simple(arg)) => {
                                Some((dir.name, arg.loc.span, arg.content))
                            }
                            _ => None,
                        },
                        _ => None,
                    })
                    .collect();
                assert_eq!(original.len(), 2);
                let mut bindings = FxHashMap::default();
                bindings.insert("evt".into(), kind);
                bindings.insert("attr".into(), kind);
                bindings.insert("n".into(), BindingType::SetupRef);
                let mut ctx = TransformContext::new(
                    &allocator,
                    source,
                    TransformOptions {
                        prefix_identifiers: true,
                        inline,
                        binding_metadata: Some(BindingMetadata {
                            bindings,
                            props_aliases: FxHashMap::default(),
                            is_script_setup: true,
                        }),
                        ..Default::default()
                    },
                );
                process_directive_expressions(&mut ctx, el);
                assert!(std::ptr::eq(owner, &**el));
                let processed: std::vec::Vec<_> = el
                    .props
                    .iter()
                    .filter_map(|prop| match prop {
                        PropNode::Directive(dir) => match &dir.arg {
                            Some(ExpressionNode::Simple(arg)) => Some((dir.name, arg)),
                            _ => None,
                        },
                        _ => None,
                    })
                    .collect();
                let expected = if inline {
                    inline_content
                } else {
                    separate_content
                };
                assert_eq!(processed.len(), original.len());
                for (((original_name, span, raw), (name, arg)), content) in
                    original.iter().zip(processed.iter()).zip(expected)
                {
                    assert_eq!(original_name, name);
                    assert_eq!(*span, arg.loc.span);
                    assert_eq!(span.slice(source), *raw);
                    assert_eq!(arg.content, content);
                    assert!(arg.is_ref_transformed);
                }
                assert_eq!(
                    ctx.has_helper(RuntimeHelper::Unref),
                    inline && kind == BindingType::SetupMaybeRef
                );
                assert!(ctx.errors.is_empty(), "{:?}", ctx.errors);
            }
        }
    }

    #[test]
    fn ordinary_argument_namespace_and_static_keys_keep_the_original_carriers() {
        let allocator = Allocator::new();
        for (source, expected_errors) in [
            (original_template(), std::vec::Vec::new()),
            (
                "<button @click=\"n++\" :title=\"'tip'\" />",
                std::vec![(
                    ErrorCode::ExtendPoint,
                    "Invalid self-closing syntax on non-void HTML element was rewritten as an empty element with an explicit end tag.",
                    Some(Span::new(0, 38))
                )],
            ),
            (
                "<button @click=\"n++\" :title=\"'tip'\"></button>",
                std::vec::Vec::new(),
            ),
        ] {
            let (mut root, errors) = parse(&allocator, source);
            assert_eq!(
                errors
                    .iter()
                    .map(|error| (
                        error.code,
                        error.message.as_str(),
                        error.loc.as_ref().map(|loc| loc.span)
                    ))
                    .collect::<std::vec::Vec<_>>(),
                expected_errors,
                "direct HTML parser boundary; self-closing is valid Vue syntax"
            );
            let el = root
                .children
                .iter_mut()
                .find_map(|node| match node {
                    TemplateChildNode::Element(el) => Some(el),
                    _ => None,
                })
                .expect("original button");
            let original: std::vec::Vec<_> = el
                .props
                .iter()
                .filter_map(|prop| match prop {
                    PropNode::Directive(dir) => match &dir.arg {
                        Some(ExpressionNode::Simple(arg)) => {
                            Some((arg.content, arg.loc.span, arg.is_static))
                        }
                        _ => None,
                    },
                    _ => None,
                })
                .collect();
            let mut ctx = TransformContext::new(
                &allocator,
                source,
                TransformOptions {
                    prefix_identifiers: true,
                    ..Default::default()
                },
            );
            process_directive_expressions(&mut ctx, el);
            let after: std::vec::Vec<_> = el
                .props
                .iter()
                .filter_map(|prop| match prop {
                    PropNode::Directive(dir) => match &dir.arg {
                        Some(ExpressionNode::Simple(arg)) => {
                            assert!(!arg.is_ref_transformed);
                            Some((arg.content, arg.loc.span, arg.is_static))
                        }
                        _ => None,
                    },
                    _ => None,
                })
                .collect();
            assert_eq!(after, original);
            assert_eq!(original.len(), 2);
        }
    }

    #[test]
    fn original_server_and_vapor_argument_carriers_keep_their_existing_processing() {
        for (ssr, vapor) in [(true, false), (false, true)] {
            let allocator = Allocator::new();
            let source = original_template();
            let (mut root, errors) = parse(&allocator, source);
            assert!(errors.is_empty(), "{errors:?}");
            let el = root
                .children
                .iter_mut()
                .find_map(|node| match node {
                    TemplateChildNode::Element(el) => Some(el),
                    _ => None,
                })
                .expect("original button owner");
            let arguments = |el: &crate::ElementNode<'_>| {
                el.props
                    .iter()
                    .filter_map(|prop| match prop {
                        PropNode::Directive(dir) => match &dir.arg {
                            Some(ExpressionNode::Simple(arg)) => Some((
                                arg.content.to_owned(),
                                arg.loc.span,
                                arg.is_static,
                                arg.is_ref_transformed,
                            )),
                            _ => None,
                        },
                        _ => None,
                    })
                    .collect::<std::vec::Vec<_>>()
            };
            let original = arguments(el);
            let mut bindings = FxHashMap::default();
            for name in ["evt", "attr", "n"] {
                bindings.insert(name.into(), BindingType::SetupRef);
            }
            let mut ctx = TransformContext::new(
                &allocator,
                source,
                TransformOptions {
                    prefix_identifiers: true,
                    inline: true,
                    ssr,
                    vapor,
                    binding_metadata: Some(BindingMetadata {
                        bindings,
                        props_aliases: FxHashMap::default(),
                        is_script_setup: true,
                    }),
                    ..Default::default()
                },
            );
            process_directive_expressions(&mut ctx, el);
            assert_eq!(arguments(el), original);
            assert_eq!(original.len(), 2);
            assert!(ctx.errors.is_empty(), "{:?}", ctx.errors);
        }
    }
}
