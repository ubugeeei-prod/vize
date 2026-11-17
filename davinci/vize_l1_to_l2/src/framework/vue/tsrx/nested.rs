//! Markup regions nested in a conditional or loop expression child.
//!
//! `{a ? <A /> : <B />}`, its `a ? … : b ? … : …` chains, a last alternate
//! `null`, and `{a && <A />}` with a provably boolean `a` lower to one `ui.if`
//! whose branches are the nested regions' markup, and `{xs.map((x, i) => <A />)}`
//! lowers to a `ui.for` over `xs` with the callback's parameters as aliases:
//! the shapes JSX control flow lowers (`framework/vue/jsx/control_flow.rs`). An
//! arm or a callback may itself return such a shape. The nested regions become
//! part of the enclosing artifact.
//!
//! Any other expression that holds markup stays a value child, and its regions
//! stay standalone: `{n && <A />}` renders `0` for a zero `n`, which a `ui.if`
//! cannot.

use core::cell::RefCell;

use oxc_ast::ast::{
    CallExpression, ConditionalExpression, Expression, FormalParameters, LogicalExpression,
    LogicalOperator, Statement, UnaryOperator,
};
use oxc_span::GetSpan;
use vize_l0::Span;
use vize_l1::markup::tsrx::{Region, TsrxModule};

use super::emit::{self, Context};
use super::{TsrxIssue, TsrxIssueKind, expression};
use vize_l2::artifact::{BranchBuilder, CallbackForHead, RegionBuilder};
use vize_l2::expr::ExprRef;
use vize_l2::op::Namespace;

/// The regions folded into an enclosing artifact, in the order they were met.
pub(super) type Folded = RefCell<alloc::vec::Vec<Span>>;

/// Lower `value`, authored at `at`, as a `ui.if` when it is a conditional over
/// markup. `false` leaves it to the caller as a value child.
pub(super) fn conditional<'a>(
    cx: &Context<'a>,
    value: &'a Expression<'a>,
    at: Span,
    namespace: Namespace,
    builder: &mut RegionBuilder<'_, 'a>,
) -> Result<bool, TsrxIssue> {
    if !structural(cx.module, value) {
        return Ok(false);
    }
    lower(cx, value, at, namespace, builder)?;
    Ok(true)
}

fn structural<'a>(module: &TsrxModule<'a>, value: &Expression<'a>) -> bool {
    match unwrap(value) {
        Expression::ConditionalExpression(chain) => chain_renders(module, chain),
        Expression::LogicalExpression(logical) => {
            logical.operator == LogicalOperator::And
                && is_boolean(&logical.left)
                && renders(module, &logical.right)
        }
        Expression::CallExpression(call) => {
            map_parts(call).is_some_and(|(_, _, returned)| renders(module, returned))
        }
        _ => false,
    }
}

/// The array, the callback's parameters and its returned value of
/// `xs.map(cb)`: an arrow or function expression of at most two plain
/// parameters whose body is its value or one `return`. `renderList` passes
/// `(item, index)` for an array, as `map` does.
fn map_parts<'e, 'a>(
    call: &'e CallExpression<'a>,
) -> Option<(
    &'e Expression<'a>,
    &'e FormalParameters<'a>,
    &'e Expression<'a>,
)> {
    let Expression::StaticMemberExpression(member) = unwrap(&call.callee) else {
        return None;
    };
    if member.property.name != "map" || member.optional || call.optional {
        return None;
    }
    let [argument] = call.arguments.as_slice() else {
        return None;
    };
    let (params, statements, expression) = match unwrap(argument.as_expression()?) {
        Expression::ArrowFunctionExpression(arrow) if !arrow.r#async => {
            (&*arrow.params, &arrow.body.statements, arrow.expression)
        }
        Expression::FunctionExpression(function) if !function.r#async && !function.generator => (
            &*function.params,
            &function.body.as_ref()?.statements,
            false,
        ),
        _ => return None,
    };
    if params.rest.is_some()
        || params.items.len() > 2
        || params.items.iter().any(|param| param.initializer.is_some())
    {
        return None;
    }
    let returned = match statements.as_slice() {
        [Statement::ExpressionStatement(statement)] if expression => &statement.expression,
        [Statement::ReturnStatement(statement)] if !expression => statement.argument.as_ref()?,
        _ => return None,
    };
    Some((&member.object, params, returned))
}

/// Every consequent renders, and so does the last alternate unless it is `null`.
fn chain_renders<'a>(module: &TsrxModule<'a>, chain: &ConditionalExpression<'a>) -> bool {
    renders(module, &chain.consequent)
        && match unwrap(&chain.alternate) {
            Expression::ConditionalExpression(next) => chain_renders(module, next),
            last => last.is_null() || renders(module, last),
        }
}

fn renders<'a>(module: &TsrxModule<'a>, arm: &Expression<'a>) -> bool {
    module.expression_region(unwrap(arm)).is_some() || structural(module, arm)
}

fn lower<'a>(
    cx: &Context<'a>,
    value: &'a Expression<'a>,
    at: Span,
    namespace: Namespace,
    builder: &mut RegionBuilder<'_, 'a>,
) -> Result<(), TsrxIssue> {
    if let Expression::CallExpression(call) = unwrap(value) {
        return map(cx, call, at, namespace, builder);
    }
    let mut failure = None;
    let built = builder.conditional(at, |branches, _| {
        failure = match unwrap(value) {
            Expression::ConditionalExpression(chain) => {
                self::chain(cx, chain, namespace, branches).err()
            }
            Expression::LogicalExpression(logical) => and(cx, logical, namespace, branches).err(),
            _ => Some(unstructured(at)),
        };
    });
    failure.or(artifact(built.err(), at)).map_or(Ok(()), Err)
}

fn map<'a>(
    cx: &Context<'a>,
    call: &'a CallExpression<'a>,
    at: Span,
    namespace: Namespace,
    builder: &mut RegionBuilder<'_, 'a>,
) -> Result<(), TsrxIssue> {
    let Some((list, params, returned)) = map_parts(call) else {
        return Err(unstructured(at));
    };
    let head = CallbackForHead {
        collection: expression(cx.allocator, cx.source, list)?,
        params,
    };
    let mut failure = None;
    let built = builder.callback_for(head, at, |region, _| {
        failure = arm(cx, returned, namespace, region).err();
    });
    failure.or(artifact(built.err(), at)).map_or(Ok(()), Err)
}

/// What an arm or a callback renders: a nested region's markup, or a
/// conditional or loop.
fn arm<'a>(
    cx: &Context<'a>,
    value: &'a Expression<'a>,
    namespace: Namespace,
    region: &mut RegionBuilder<'_, 'a>,
) -> Result<(), TsrxIssue> {
    match cx.module.expression_region(unwrap(value)) {
        Some(nested) => fold(cx, nested, namespace, region),
        None => lower(cx, value, span(value.span()), namespace, region),
    }
}

fn chain<'a>(
    cx: &Context<'a>,
    mut chain: &'a ConditionalExpression<'a>,
    namespace: Namespace,
    branches: &mut BranchBuilder<'_, 'a>,
) -> Result<(), TsrxIssue> {
    loop {
        branch(
            cx,
            Some(&chain.test),
            &chain.consequent,
            namespace,
            branches,
        )?;
        match unwrap(&chain.alternate) {
            Expression::ConditionalExpression(next) => chain = next,
            last if last.is_null() => return Ok(()),
            _ => return branch(cx, None, &chain.alternate, namespace, branches),
        }
    }
}

fn and<'a>(
    cx: &Context<'a>,
    logical: &'a LogicalExpression<'a>,
    namespace: Namespace,
    branches: &mut BranchBuilder<'_, 'a>,
) -> Result<(), TsrxIssue> {
    branch(cx, Some(&logical.left), &logical.right, namespace, branches)
}

/// One branch from its test through its arm: the nested region's markup, or
/// a conditional arm.
fn branch<'a>(
    cx: &Context<'a>,
    test: Option<&'a Expression<'a>>,
    arm: &'a Expression<'a>,
    namespace: Namespace,
    branches: &mut BranchBuilder<'_, 'a>,
) -> Result<(), TsrxIssue> {
    let mut span = span(arm.span());
    let condition = match test {
        Some(test) => {
            span.start = test.span().start;
            Some(ExprRef::Js(expression(cx.allocator, cx.source, test)?))
        }
        None => None,
    };
    let mut failure = None;
    let built = branches.branch(condition, span, |region, _| {
        failure = self::arm(cx, arm, namespace, region).err();
    });
    failure.or(artifact(built.err(), span)).map_or(Ok(()), Err)
}

fn fold<'a>(
    cx: &Context<'a>,
    nested: &'a Region<'a>,
    namespace: Namespace,
    region: &mut RegionBuilder<'_, 'a>,
) -> Result<(), TsrxIssue> {
    cx.folded.borrow_mut().push(nested.span);
    emit::node(cx, &nested.node, namespace, region)
}

/// Whether `value` always evaluates to a boolean, from its syntax alone.
fn is_boolean(value: &Expression<'_>) -> bool {
    match value {
        Expression::ParenthesizedExpression(inner) => is_boolean(&inner.expression),
        Expression::BooleanLiteral(_) => true,
        Expression::UnaryExpression(unary) => unary.operator == UnaryOperator::LogicalNot,
        Expression::BinaryExpression(binary) => {
            binary.operator.is_equality()
                || binary.operator.is_compare()
                || binary.operator.is_relational()
        }
        Expression::LogicalExpression(logical) => {
            is_boolean(&logical.left) && is_boolean(&logical.right)
        }
        Expression::ConditionalExpression(conditional) => {
            is_boolean(&conditional.consequent) && is_boolean(&conditional.alternate)
        }
        // Type annotations do not prove a runtime value's type.
        _ => false,
    }
}

fn unwrap<'e, 'a>(mut value: &'e Expression<'a>) -> &'e Expression<'a> {
    while let Expression::ParenthesizedExpression(inner) = value {
        value = &inner.expression;
    }
    value
}

fn span(span: oxc_span::Span) -> Span {
    Span::new(span.start, span.end)
}

fn artifact(error: Option<vize_l2::artifact::ArtifactError>, span: Span) -> Option<TsrxIssue> {
    error.map(|error| TsrxIssue {
        kind: TsrxIssueKind::Artifact(error),
        span,
    })
}

/// [`structural`] accepted a shape [`lower`] does not build: a compiler bug.
fn unstructured(span: Span) -> TsrxIssue {
    TsrxIssue {
        kind: TsrxIssueKind::Artifact(vize_l2::artifact::ArtifactError::InvalidSpan {
            node: None,
            span,
        }),
        span,
    }
}
