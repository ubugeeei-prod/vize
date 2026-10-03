//! Reporter #7600: retained Vapor style bindings keep their static value.

#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    reason = "regression assertions use std strings and format"
)]

use super::{VaporCompilerOptions, compile_vapor, ir::OperationNode, transform_to_ir};
use vize_atelier_core::{TransformOptions, WhitespaceStrategy, lane::transform, parser::parse};
use vize_carton::Allocator;

const REPORTED: &str = include_str!("../tests/fixtures/static-style-merged-binding.input.txt");

fn assert_valid_module(code: &str) {
    let allocator = oxc_allocator::Allocator::default();
    let parsed = oxc_parser::Parser::new(&allocator, code, oxc_span::SourceType::mjs()).parse();
    assert!(
        parsed.diagnostics.is_empty(),
        "{:?}\n{code}",
        parsed.diagnostics
    );
}

#[test]
fn public_transformed_ir_preserves_static_style_before_dynamic_style() {
    let allocator = Allocator::new();
    let source = REPORTED.trim();
    let (mut root, errors) = parse(&allocator, source);
    assert!(errors.is_empty());
    assert!(
        transform(
            &allocator,
            &mut root,
            TransformOptions {
                vapor: true,
                ..Default::default()
            },
            None,
        )
        .is_empty()
    );
    let ir = transform_to_ir(&allocator, &root, source);
    let style = ir
        .block
        .effect
        .iter()
        .flat_map(|effect| effect.operations.iter())
        .find_map(|operation| match operation {
            OperationNode::SetProp(node) if node.prop.key.content == "style" => Some(node),
            _ => None,
        })
        .expect("reactive style operation");
    assert_eq!(style.prop.values.len(), 2);
    assert!(style.prop.values[0].is_static);
    assert_eq!(style.prop.values[0].content, "color: red");
    assert!(!style.prop.values[1].is_static);
    assert_eq!(style.prop.values[1].content, "s");
}

#[test]
fn reported_style_merge_survives_preserve_and_parser_recovery() {
    let recovered = format!("{}<p>", REPORTED.trim());
    for source in [REPORTED.trim(), recovered.as_str()] {
        for strategy in [WhitespaceStrategy::Condense, WhitespaceStrategy::Preserve] {
            let allocator = Allocator::new();
            let result = vize_atelier_core::parser::with_whitespace_strategy(strategy, || {
                compile_vapor(
                    &allocator,
                    source,
                    VaporCompilerOptions {
                        prefix_identifiers: true,
                        ..Default::default()
                    },
                )
            });
            assert!(
                result.error_messages.is_empty(),
                "{:?}",
                result.error_messages
            );
            assert!(
                result
                    .code
                    .contains("_setStyle(n0, [\"color: red\", _ctx.s])"),
                "{strategy:?}: {}",
                result.code
            );
            assert_valid_module(&result.code);
        }
    }
}

#[test]
fn retained_style_merge_preserves_order_and_existing_class_behavior() {
    for (source, expected) in [
        (
            r#"<div :style="s" style="color: red">x</div>"#,
            r#"_setStyle(n0, ["color: red", _ctx.s])"#,
        ),
        (
            r#"<div style="" :style="s">x</div>"#,
            r#"_setStyle(n0, ["", _ctx.s])"#,
        ),
        (
            r#"<div style='--label: &quot;x&quot;' :style="s">x</div>"#,
            r#"_setStyle(n0, ["--label: \"x\"", _ctx.s])"#,
        ),
        (r#"<div :style="s">x</div>"#, "_setStyle(n0, _ctx.s)"),
        (
            r#"<div class="a" :class="c" style="color: red" :style="s">x</div>"#,
            r#"_setClass(n0, ["a", _ctx.c])"#,
        ),
        (
            r#"<div class="a" :class="c" style="color: red" :style="s">x</div>"#,
            r#"_setStyle(n0, ["color: red", _ctx.s])"#,
        ),
    ] {
        let allocator = Allocator::new();
        let result = compile_vapor(
            &allocator,
            source,
            VaporCompilerOptions {
                prefix_identifiers: true,
                davinci_retained_lane: true,
                ..Default::default()
            },
        );
        assert!(
            result.error_messages.is_empty(),
            "{:?}",
            result.error_messages
        );
        assert!(result.code.contains(expected), "{source}: {}", result.code);
        assert_valid_module(&result.code);
    }
}
