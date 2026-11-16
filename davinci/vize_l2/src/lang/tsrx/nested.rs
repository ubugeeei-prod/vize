//! Markup regions nested in a conditional expression child.
//!
//! `{a ? <A /> : <B />}`, its `a ? … : b ? … : …` chains, a last alternate
//! `null`, and `{a && <A />}` with a provably boolean `a` lower to one `ui.if`
//! whose branches are the nested regions' markup, the shapes JSX control flow
//! lowers (`framework/vue/jsx/control_flow.rs`). An arm may itself be such a
//! conditional. The nested regions become part of the enclosing artifact.
//!
//! Any other expression that holds markup stays a value child, and its regions
//! stay standalone: `{n && <A />}` renders `0` for a zero `n`, which a `ui.if`
//! cannot, and `.map()` callbacks wait for their loop lowering.

use core::cell::RefCell;

use oxc_ast::ast::{
    ConditionalExpression, Expression, LogicalExpression, LogicalOperator, UnaryOperator,
};
use oxc_span::GetSpan;
use vize_l0::Span;
use vize_l1::markup::tsrx::{Region, TsrxModule};

use super::emit::{self, Context};
use super::{TsrxIssue, TsrxIssueKind, expression};
use crate::artifact::{BranchBuilder, RegionBuilder};
use crate::expr::ExprRef;
use crate::op::Namespace;

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
        _ => false,
    }
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
        failure = match cx.module.expression_region(unwrap(arm)) {
            Some(nested) => fold(cx, nested, namespace, region).err(),
            None => lower(cx, arm, self::span(arm.span()), namespace, region).err(),
        };
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

fn artifact(error: Option<crate::artifact::ArtifactError>, span: Span) -> Option<TsrxIssue> {
    error.map(|error| TsrxIssue {
        kind: TsrxIssueKind::Artifact(error),
        span,
    })
}

/// [`structural`] accepted a shape [`lower`] does not build: a compiler bug.
fn unstructured(span: Span) -> TsrxIssue {
    TsrxIssue {
        kind: TsrxIssueKind::Artifact(crate::artifact::ArtifactError::InvalidSpan {
            node: None,
            span,
        }),
        span,
    }
}
