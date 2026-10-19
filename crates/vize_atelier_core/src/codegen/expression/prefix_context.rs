//! Context-aware identifier prefixing for codegen (split from
//! `helpers.rs`; the P1-7 retained-AST entry lives here).
//!
//! The legacy entry parses `(content)` (identifier spans shifted by the
//! wrapper, hence `offset: 1`); the retained entry walks the parse-once AST
//! with content-relative spans (`offset: 0`). The visitor and the rewrite
//! application are shared, so the two paths differ only in where the AST
//! comes from.

use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_l0::FxHashSet;
use vize_l0::String;
use vize_l0::ToCompactString;
use vize_relief::{ExpressionScope, SimpleExpressionNode};

use super::super::context::CodegenContext;
use super::helpers::rewrite_props_aliases;
use super::prefix_visitor::{IdentifierVisitor, apply_rewrites};

/// Walk one parsed expression and apply the collected rewrites to `content`.
/// `offset` is the span shift of the parse text relative to `content` (1 for
/// the legacy `(content)` wrapper, 0 for the retained AST).
fn prefix_via_expr(
    expr: &oxc_ast::ast::Expression<'_>,
    offset: u32,
    content: &str,
    ctx: &CodegenContext,
    implicit_event: bool,
) -> String {
    let mut rewrites: Vec<(usize, usize, String)> = Vec::new();
    let mut assignment_targets: FxHashSet<usize> = FxHashSet::default();

    let mut visitor = IdentifierVisitor {
        rewrites: &mut rewrites,
        local_scopes: vec![FxHashSet::default()],
        assignment_targets: &mut assignment_targets,
        ctx,
        offset,
    };
    if implicit_event {
        visitor.add_local("$event");
    }
    visitor.visit_expression(expr);

    rewrite_props_aliases(apply_rewrites(content, rewrites), ctx)
}

pub(super) fn prefix_identifiers_in_scope(
    content: &str,
    ctx: &CodegenContext,
    implicit_event: bool,
) -> String {
    let allocator = crate::expr_parse_probe::parse_arena();
    let source_type = SourceType::default().with_module(true);

    // First try: wrap in parentheses to parse as a single expression
    let mut wrapped = String::with_capacity(content.len() + 2);
    wrapped.push('(');
    wrapped.push_str(content);
    wrapped.push(')');
    let parser = Parser::new(&allocator, &wrapped, source_type);
    let parse_result = parser.parse_expression();

    match parse_result {
        Ok(expr) => prefix_via_expr(&expr, 1, content, ctx, implicit_event),
        Err(_) => {
            // Expression parsing failed -- try parsing as a program
            let allocator2 = crate::expr_parse_probe::parse_arena();
            let parser2 = Parser::new(&allocator2, content, source_type);
            let parse_result2 = parser2.parse();
            if parse_result2.diagnostics.is_empty() {
                let mut rewrites: Vec<(usize, usize, String)> = Vec::new();
                let mut assignment_targets: FxHashSet<usize> = FxHashSet::default();

                let mut visitor = IdentifierVisitor {
                    rewrites: &mut rewrites,
                    local_scopes: vec![FxHashSet::default()],
                    assignment_targets: &mut assignment_targets,
                    ctx,
                    offset: 0,
                };
                if implicit_event {
                    visitor.add_local("$event");
                }
                visitor.visit_program(&parse_result2.program);

                rewrite_props_aliases(apply_rewrites(content, rewrites), ctx)
            } else {
                content.to_compact_string()
            }
        }
    }
}

/// Node-aware [`prefix_identifiers_with_context`] (P1-7): reads the retained
/// AST when it still describes the node's bytes and the dialect gate holds;
/// the legacy wrapped re-parse otherwise.
pub(crate) fn prefix_identifiers_with_context_node(
    node: &SimpleExpressionNode<'_>,
    ctx: &CodegenContext,
) -> String {
    prefix_node_in_scope(node, ctx, false)
}

pub(super) fn prefix_node_in_scope(
    node: &SimpleExpressionNode<'_>,
    ctx: &CodegenContext,
    implicit_event: bool,
) -> String {
    if let Some(js) = crate::retained::retained_whole_expression(node)
        && crate::retained::js_module_compatible(&js)
    {
        let result = prefix_via_expr(js.ast, 0, js.raw, ctx, implicit_event);
        #[cfg(any(test, feature = "legacy-differential"))]
        #[expect(
            clippy::panic,
            reason = "differential oracle: a divergence must abort the run"
        )]
        {
            // Dual-run against the legacy wrapped parse in an uncounted
            // arena; divergence panics, never averages.
            let allocator = oxc_allocator::Allocator::default();
            let mut wrapped = String::with_capacity(js.raw.len() + 2);
            wrapped.push('(');
            wrapped.push_str(js.raw);
            wrapped.push(')');
            let legacy = Parser::new(&allocator, &wrapped, SourceType::default().with_module(true))
                .parse_expression()
                .map(|expr| prefix_via_expr(&expr, 1, js.raw, ctx, implicit_event))
                .unwrap_or_else(|_| {
                    panic!(
                        "davinci-differential (P1-7): js_module_compatible admitted an expression the legacy JS-module parse rejects: {:?}",
                        js.raw
                    )
                });
            assert_eq!(
                result.as_str(),
                legacy.as_str(),
                "davinci-differential (P1-7): retained codegen prefix walk diverged from the legacy re-parse for expression {:?}",
                js.raw
            );
            crate::retained::differential::record_codegen_rewrite_comparison();
        }
        return result;
    }
    prefix_identifiers_in_scope(node.content, ctx, implicit_event)
}

#[cfg(test)]
mod tests {
    use super::prefix_identifiers_with_context_node;
    use crate::codegen::context::CodegenContext;
    use crate::codegen::expression::generate_event_handler;
    use crate::options::CodegenOptions;
    use crate::{ExpressionNode, SimpleExpressionNode, SourceLocation};
    use oxc_span::{GetSpan, SourceType};
    use vize_l0::{Allocator, Box};
    use vize_relief::{RetainedJsAst, RetainedJsAstKind};

    const DECORATOR_CASES: &[(&str, &str)] = &[
        (
            "[(@C class C { value = C }), C]",
            "[(@_ctx.C class C { value = C }), _ctx.C]",
        ),
        (
            "[(@decorate(C) class C { @decorate(C) value = C }), C]",
            "[(@_ctx.decorate(_ctx.C) class C { @_ctx.decorate(C) value = C }), _ctx.C]",
        ),
    ];

    #[test]
    fn codegen_callback_parameters_keep_their_lexical_bindings() {
        let ctx = CodegenContext::new(CodegenOptions::default());
        let allocator = Allocator::new();
        for (source, expected) in [
        (
            "() => { read(); function read() { return $event } }",
            "() => { read(); function read() { return _ctx.$event } }",
        ),
        (
            "() => { const read = () => value; const value = 1; return read() }",
            "() => { const read = () => value; const value = 1; return read() }",
        ),
        (
            "() => { const read = () => C; class C {} return read() }",
            "() => { const read = () => C; class C {} return read() }",
        ),
        (
            "() => { for (let value = 0; value < 1; value++) {} return value }",
            "() => { for (let value = 0; value < 1; value++) {} return _ctx.value }",
        ),
        (
            "() => { for (const value in source) {} return value }",
            "() => { for (const value in _ctx.source) {} return _ctx.value }",
        ),
        (
            "() => { for (const value of source) {} return value }",
            "() => { for (const value of _ctx.source) {} return _ctx.value }",
        ),
        (
            "() => { for (var value of source) {} return value }",
            "() => { for (var value of _ctx.source) {} return value }",
        ),
        (
            "const read = () => value; { var value = $event } read()",
            "const read = () => value; { var value = _ctx.$event } read()",
        ),
        (
            "const read = () => value; { const value = 42 } read()",
            "const read = () => _ctx.value; { const value = 42 } read()",
        ),
        (
            "(value = helper) => { function helper() {} return value }",
            "(value = _ctx.helper) => { function helper() {} return value }",
        ),
        (
            "() => { switch (value) { case 0: const value = 1; break } return value }",
            "() => { switch (_ctx.value) { case 0: const value = 1; break } return _ctx.value }",
        ),
        (
            "[class $event { read() { return $event } }, $event]",
            "[class $event { read() { return $event } }, _ctx.$event]",
        ),
        (
            "() => { class C { static { var value = 1 } } return value }",
            "() => { class C { static { var value = 1 } } return _ctx.value }",
        ),
        (
            "() => { save(value); let value = 1 }",
            "() => { _ctx.save(value); let value = 1 }",
        ),
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
    ]
    .into_iter()
    .chain(DECORATOR_CASES.iter().copied())
    {
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
    fn transform_class_decorators_keep_the_outer_class_binding() {
        let allocator = Allocator::new();
        for &(source, expected) in DECORATOR_CASES {
            for retained in [false, true] {
                let node = ExpressionNode::Simple(Box::new_in(
                    expression_node(source, &allocator, retained),
                    &&allocator,
                ));
                let mut ctx = crate::lane::TransformContext::new(
                    &allocator,
                    source,
                    crate::options::TransformOptions {
                        prefix_identifiers: true,
                        ..Default::default()
                    },
                );
                let ExpressionNode::Simple(result) =
                    crate::steps::expression::process_expression(&mut ctx, &node, false)
                else {
                    panic!("expected a simple expression");
                };
                assert_eq!(result.content, expected, "retained={retained}: {source}");
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
            && let Ok(ast) = oxc_parser::Parser::new(allocator.as_oxc(), source, SourceType::ts())
                .parse_expression()
            && ast.span().end as usize == source.len()
        {
            node.js_ast = Some(RetainedJsAst {
                ast: allocator.as_oxc().alloc(RetainedJsAstKind::Expression(ast)),
                raw: source,
            });
        }
        node
    }
}
