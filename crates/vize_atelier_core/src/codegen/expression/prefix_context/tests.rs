use super::prefix_identifiers_with_context_node;
use crate::codegen::context::CodegenContext;
use crate::codegen::expression::generate_event_handler;
use crate::options::CodegenOptions;
use crate::{ExpressionNode, JsExpression, SimpleExpressionNode, SourceLocation};
use oxc_span::{GetSpan, SourceType};
use vize_s0::{Allocator, Box};

#[test]
fn codegen_callback_parameters_keep_their_lexical_bindings() {
    let ctx = CodegenContext::new(CodegenOptions::default());
    let allocator = Allocator::new();
    for (source, expected) in [
        ("(...$event) => $event[0]", "(...$event) => $event[0]"),
        (
            "({ type: $event }) => $event",
            "({ type: $event }) => $event",
        ),
        (
            "([[$event], ...rest]) => [$event, rest, outside]",
            "([[$event], ...rest]) => [$event, rest, _ctx.outside]",
        ),
        (
            "({ event: $event = fallback } = source) => $event",
            "({ event: $event = _ctx.fallback } = _ctx.source) => $event",
        ),
        (
            "({ [key]: $event }) => $event",
            "({ [_ctx.key]: $event }) => $event",
        ),
        (
            "(event = $event) => { var $event; return event }",
            "(event = _ctx.$event) => { var $event; return event }",
        ),
        (
            "[($event) => $event, $event]",
            "[($event) => $event, _ctx.$event]",
        ),
        (
            "[({ type: $event }) => ({ $event }), () => $event]",
            "[({ type: $event }) => ({ $event }), () => _ctx.$event]",
        ),
        (
            "function $event(...args) { return [$event, args, outside] }",
            "function $event(...args) { return [$event, args, _ctx.outside] }",
        ),
        (
            "event => { const read = () => $event; { var $event = event } return read() }",
            "event => { const read = () => $event; { var $event = event } return read() }",
        ),
        (
            "() => { const $event = $event; return $event }",
            "() => { const $event = $event; return $event }",
        ),
        (
            "event => { { const $event = event; save($event) } save($event) }",
            "event => { { const $event = event; _ctx.save($event) } _ctx.save(_ctx.$event) }",
        ),
        (
            "event => { try { throw event } catch ($event) { save($event) } save($event) }",
            "event => { try { throw event } catch ($event) { _ctx.save($event) } _ctx.save(_ctx.$event) }",
        ),
    ] {
        for retained in [false, true] {
            let node = expression_node(source, &allocator, retained);
            assert_eq!(
                prefix_identifiers_with_context_node(&node, &ctx),
                expected,
                "retained={retained}: {source}"
            );
        }
    }
}

#[test]
fn codegen_handler_owns_an_implicit_event_only_for_inline_bodies() {
    let allocator = Allocator::new();
    for (source, expected) in [
        ("$event", "_ctx.$event"),
        ("$event.target", "_ctx.$event.target"),
        ("() => save($event)", "() => _ctx.save(_ctx.$event)"),
        (
            "(...$event) => save($event[0])",
            "(...$event) => _ctx.save($event[0])",
        ),
        ("save($event.type)", "$event => (_ctx.save($event.type))"),
        (
            "save($event.type); count++",
            "$event => {_ctx.save($event.type); _ctx.count++}",
        ),
    ] {
        for retained in [false, true] {
            let mut ctx = CodegenContext::new(CodegenOptions {
                prefix_identifiers: true,
                ..Default::default()
            });
            let node = ExpressionNode::Simple(Box::new_in(
                expression_node(source, &allocator, retained),
                &&allocator,
            ));
            generate_event_handler(&mut ctx, &node, false);
            assert_eq!(ctx.into_code(), expected, "retained={retained}: {source}");
        }
    }
}

fn expression_node<'a>(
    source: &'a str,
    allocator: &'a Allocator,
    retained: bool,
) -> SimpleExpressionNode<'a> {
    let mut node = SimpleExpressionNode::new(source, false, SourceLocation::STUB);
    if retained
        && let Ok(ast) =
            oxc_parser::Parser::new(allocator.as_oxc(), source, SourceType::ts()).parse_expression()
        && ast.span().end as usize == source.len()
    {
        node.js_ast = Some(JsExpression {
            ast: allocator.as_oxc().alloc(ast),
            raw: source,
        });
    }
    node
}
