//! Structural control flow (`Depth::Structural` only): `&&`, `?:` and
//! `.map()` children as `ui.if` / `ui.for`.
//!
//! [`is_structural`] recognizes the shapes; a recognized shape either lowers
//! completely or refuses, so an unrecognized one is always the caller's value
//! child. `&&` over a value that is not provably boolean waits for `ui.and`:
//! the shipped output spells its falsy `left` as text or a comment, which the
//! level emitter's `left && …` does not reproduce. A ternary arm that is a
//! value, not markup, renders as text, as the template's interpolation does.

use oxc_ast::ast::{
    CallExpression, ConditionalExpression, Expression, FormalParameters, LogicalExpression,
    LogicalOperator, Statement,
};
use oxc_span::GetSpan;
use vize_l0::{Box, Span, String, Vec};
use vize_l2::artifact::OpFamily;
use vize_l2::expr::ExprRef;
use vize_l2::op::{ForBinding, ForOp, IfBranch, IfOp, InterpolationOp, Op, Region};
use vize_l2::scope::{ScopeBinding, ScopeFacts, ScopeOrigin};

use super::children::{contains_jsx, element};
use super::{Cx, Depth, JsxRefusal, span};

/// Lower an expression child at `at` (its container, or the expression itself
/// inside a branch) as control flow. `None` when it is not structural.
pub(super) fn control_flow<'a>(
    cx: &mut Cx<'a, '_>,
    expression: &Expression<'_>,
    at: oxc_span::Span,
) -> Result<Option<Op<'a>>, JsxRefusal> {
    if cx.dialect.depth != Depth::Structural || !is_structural(expression) {
        return Ok(None);
    }
    match unwrap_parens(expression) {
        Expression::LogicalExpression(logical) => logical_and(cx, logical, at).map(Some),
        Expression::ConditionalExpression(conditional) => {
            conditional_chain(cx, conditional, at).map(Some)
        }
        Expression::CallExpression(call) => map_call(cx, call, at).map(Some),
        _ => Err(JsxRefusal::Child),
    }
}

/// `c && <A/>`: one `ui.if` branch when `c` is provably boolean.
fn logical_and<'a>(
    cx: &mut Cx<'a, '_>,
    logical: &LogicalExpression<'_>,
    at: oxc_span::Span,
) -> Result<Op<'a>, JsxRefusal> {
    if !cx.file.booleans.proves(&logical.left, cx.file.scoping) {
        return Err(JsxRefusal::Child);
    }
    cx.mint();
    let mut branches = Vec::new_in(&cx.allocator);
    branches.push(IfBranch {
        condition: Some(cx.expr(logical.left.span())?),
        region: render_region(cx, &logical.right)?,
        span: span(logical.span),
    });
    Ok(if_op(cx, branches, at))
}

/// `a ? <A/> : b ? <B/> : <C/>`: one `ui.if`, a nested alternate ternary
/// adding an else-if branch.
fn conditional_chain<'a>(
    cx: &mut Cx<'a, '_>,
    conditional: &ConditionalExpression<'_>,
    at: oxc_span::Span,
) -> Result<Op<'a>, JsxRefusal> {
    cx.mint();
    let mut branches = Vec::new_in(&cx.allocator);
    let mut current = conditional;
    loop {
        branches.push(IfBranch {
            condition: Some(cx.expr(current.test.span())?),
            region: branch_region(cx, &current.consequent)?,
            span: span(current.consequent.span()),
        });
        if let Expression::ConditionalExpression(nested) = unwrap_parens(&current.alternate) {
            current = nested;
            continue;
        }
        branches.push(IfBranch {
            condition: None,
            region: branch_region(cx, &current.alternate)?,
            span: span(current.alternate.span()),
        });
        break;
    }
    Ok(if_op(cx, branches, at))
}

fn if_op<'a>(cx: &mut Cx<'a, '_>, branches: Vec<'a, IfBranch<'a>>, at: oxc_span::Span) -> Op<'a> {
    cx.observe(OpFamily::If);
    Op::If(Box::new_in(
        IfOp {
            branches,
            span: span(at),
        },
        &cx.allocator,
    ))
}

/// `xs.map((x, i) => <A/>)`: `ui.for` over `xs` with value `x` and key `i`,
/// iterated as `v-for` does.
fn map_call<'a>(
    cx: &mut Cx<'a, '_>,
    call: &CallExpression<'_>,
    at: oxc_span::Span,
) -> Result<Op<'a>, JsxRefusal> {
    let Some((list, params, body)) = map_parts(call) else {
        return Err(JsxRefusal::Child);
    };
    // `xs.map?.(…)` renders nothing when `map` is missing; `ui.for` cannot.
    if call.optional {
        return Err(JsxRefusal::Child);
    }
    let node = cx.mint();
    let position = |index: usize| {
        params
            .items
            .get(index)
            .map(|param| cx.expr(param.pattern.span()))
            .transpose()
    };
    let value = position(0)?;
    let key = position(1)?;
    let list_span = list.span();
    let binding = ForBinding {
        source: cx.expr(list_span)?,
        // A parameterless callback binds nothing: an empty value position at
        // the list.
        value: match value {
            Some(value) => value,
            None => ExprRef::parse_js_in(
                cx.allocator,
                "",
                Span::new(list_span.start, list_span.start),
            ),
        },
        key,
        index: None,
    };
    let tag = cx.mint_scope();
    let mut bindings = alloc::vec::Vec::new();
    for bound in [Some(&binding.value), binding.key.as_ref()]
        .into_iter()
        .flatten()
    {
        if let ExprRef::Js(js) = bound
            && matches!(js.ast, Expression::Identifier(_))
        {
            bindings.push(ScopeBinding {
                name: String::from(js.source),
                origin: ScopeOrigin::Authored { span: bound.span() },
            });
        }
    }
    cx.attach_scope(node, ScopeFacts { tag, bindings });
    let region = render_region(cx, body)?;
    cx.observe(OpFamily::For);
    Ok(Op::For(Box::new_in(
        ForOp {
            binding,
            region,
            host: None,
            span: span(at),
        },
        &cx.allocator,
    )))
}

/// A ternary arm: an element, nested control flow at the arm itself, or a
/// value rendered as text. A fragment, or a value holding markup, waits for
/// its region op.
fn branch_region<'a>(cx: &mut Cx<'a, '_>, arm: &Expression<'_>) -> Result<Region<'a>, JsxRefusal> {
    let inner = unwrap_parens(arm);
    if let Expression::JSXElement(_) = inner {
        return render_region(cx, inner);
    }
    if let Some(op) = control_flow(cx, inner, inner.span())? {
        return Ok(single(cx, op));
    }
    value_arm(cx, inner)
}

/// A value arm as text. Kept out of line: element arms, the common case,
/// never reach it.
#[inline(never)]
fn value_arm<'a>(cx: &mut Cx<'a, '_>, value: &Expression<'_>) -> Result<Region<'a>, JsxRefusal> {
    if contains_jsx(value) {
        return Err(JsxRefusal::Child);
    }
    cx.mint();
    let text = Op::Interpolation(Box::new_in(
        InterpolationOp {
            expression: cx.expr(value.span())?,
            raw_html: false,
            span: span(value.span()),
        },
        &cx.allocator,
    ));
    Ok(single(cx, text))
}

/// What an `&&` right side or a `.map` callback returns: an element or
/// nested control flow. A fragment waits for its region op.
fn render_region<'a>(
    cx: &mut Cx<'a, '_>,
    rendered: &Expression<'_>,
) -> Result<Region<'a>, JsxRefusal> {
    let op = match unwrap_parens(rendered) {
        Expression::JSXElement(nested) => element(cx, nested)?,
        Expression::JSXFragment(_) => return Err(JsxRefusal::Child),
        inner => control_flow(cx, inner, inner.span())?.ok_or(JsxRefusal::Child)?,
    };
    Ok(single(cx, op))
}

fn single<'a>(cx: &Cx<'a, '_>, op: Op<'a>) -> Region<'a> {
    let mut ops = Vec::new_in(&cx.allocator);
    ops.push(op);
    Region { ops }
}

/// Whether `expression` is a structural `&&`, `?:` or `.map()` child.
fn is_structural(expression: &Expression<'_>) -> bool {
    match unwrap_parens(expression) {
        Expression::LogicalExpression(logical) => {
            logical.operator == LogicalOperator::And && is_jsx(&logical.right)
        }
        Expression::ConditionalExpression(conditional) => has_jsx_arm(conditional),
        Expression::CallExpression(call) => map_parts(call).is_some(),
        _ => false,
    }
}

/// Whether a ternary chain renders JSX in some arm, rather than choosing a
/// value.
fn has_jsx_arm(conditional: &ConditionalExpression<'_>) -> bool {
    is_jsx(&conditional.consequent)
        || match unwrap_parens(&conditional.alternate) {
            Expression::ConditionalExpression(nested) => has_jsx_arm(nested),
            other => is_jsx(other),
        }
}

/// The list, callback parameters and returned render expression of
/// `xs.map(cb)`. `renderList` runs only the returned JSX per item and passes
/// `(item, index)`, so the callback must be an expression body or one
/// `return`, with at most two plain parameters.
fn map_parts<'e, 'x>(
    call: &'e CallExpression<'x>,
) -> Option<(
    &'e Expression<'x>,
    &'e FormalParameters<'x>,
    &'e Expression<'x>,
)> {
    let Expression::StaticMemberExpression(member) = unwrap_parens(&call.callee) else {
        return None;
    };
    if member.property.name.as_str() != "map" || member.optional {
        return None;
    }
    let [argument] = call.arguments.as_slice() else {
        return None;
    };
    let (params, statements, expression_body) = match unwrap_parens(argument.as_expression()?) {
        Expression::ArrowFunctionExpression(arrow) => {
            (&*arrow.params, &arrow.body.statements, arrow.expression)
        }
        Expression::FunctionExpression(function) => (
            &*function.params,
            &function.body.as_ref()?.statements,
            false,
        ),
        _ => return None,
    };
    if params.items.len() > 2 || params.rest.is_some() {
        return None;
    }
    let returned = match statements.as_slice() {
        [Statement::ExpressionStatement(statement)] if expression_body => &statement.expression,
        [Statement::ReturnStatement(statement)] if !expression_body => {
            statement.argument.as_ref()?
        }
        _ => return None,
    };
    let renders = match unwrap_parens(returned) {
        Expression::JSXElement(_) | Expression::JSXFragment(_) => true,
        inner @ (Expression::LogicalExpression(_)
        | Expression::ConditionalExpression(_)
        | Expression::CallExpression(_)) => is_structural(inner),
        _ => false,
    };
    renders.then_some((&member.object, params, returned))
}

fn is_jsx(expression: &Expression<'_>) -> bool {
    matches!(
        unwrap_parens(expression),
        Expression::JSXElement(_) | Expression::JSXFragment(_)
    )
}

fn unwrap_parens<'e, 'x>(mut expression: &'e Expression<'x>) -> &'e Expression<'x> {
    while let Expression::ParenthesizedExpression(inner) = expression {
        expression = &inner.expression;
    }
    expression
}
