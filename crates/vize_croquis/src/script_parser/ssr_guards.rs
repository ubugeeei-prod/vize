//! Browser guards recorded while the existing statement walker owns the AST.

use oxc_ast::ast::{BinaryOperator, Expression, LogicalOperator, Statement, UnaryOperator};
use oxc_span::{GetSpan, Span};

use super::ScriptParseResult;

#[derive(Clone, Copy)]
struct Truth {
    server: bool,
    /// A `typeof window` premise needs complete lexical ownership at query time.
    global_probe: Option<Span>,
}

/// Record the client branch and, only within this statement list, a guarded tail.
pub(super) fn note_guard(
    result: &mut ScriptParseResult,
    stmt: &Statement<'_>,
    following_end: Option<u32>,
) {
    if result.skip_diagnostics {
        return;
    }
    let Statement::IfStatement(branch) = stmt else {
        return;
    };
    note_probes(result, &branch.test);
    let Some(truth) = server_truth(&branch.test) else {
        return;
    };
    if truth.server {
        if let Some(alternate) = &branch.alternate {
            note_region(result, alternate.span(), truth.global_probe);
        }
    } else {
        note_region(result, branch.consequent.span(), truth.global_probe);
    }

    let server_branch = if truth.server {
        Some(&branch.consequent)
    } else {
        branch.alternate.as_ref()
    };
    if server_branch.is_some_and(terminates)
        && let Some(end) = following_end
        && end > branch.span.end
    {
        note_region(result, Span::new(branch.span.end, end), truth.global_probe);
    }
}

fn note_region(result: &mut ScriptParseResult, span: Span, probe: Option<Span>) {
    if let Some(probe) = probe {
        result.setup_context.note_global_client_region(
            span.start,
            span.end,
            probe.start,
            probe.end,
        );
    } else {
        result
            .setup_context
            .note_client_region(span.start, span.end);
    }
}

fn terminates(statement: &Statement<'_>) -> bool {
    match statement {
        Statement::ReturnStatement(_) | Statement::ThrowStatement(_) => true,
        Statement::BlockStatement(block) => block.body.last().is_some_and(terminates),
        _ => false,
    }
}

fn server_truth(expression: &Expression<'_>) -> Option<Truth> {
    let expression = expression.get_inner_expression();
    if let Expression::StaticMemberExpression(member) = expression
        && !member.optional
    {
        if member.property.name == "client"
            && matches!(
                member.object.get_inner_expression(),
                Expression::ImportMeta(_)
            )
        {
            return Some(Truth {
                server: false,
                global_probe: None,
            });
        }
        if member.property.name == "SSR"
            && let Expression::StaticMemberExpression(env) = member.object.get_inner_expression()
            && !env.optional
            && env.property.name == "env"
            && matches!(env.object.get_inner_expression(), Expression::ImportMeta(_))
        {
            return Some(Truth {
                server: true,
                global_probe: None,
            });
        }
    }
    match expression {
        Expression::UnaryExpression(unary) if unary.operator == UnaryOperator::LogicalNot => {
            server_truth(&unary.argument).map(|truth| Truth {
                server: !truth.server,
                ..truth
            })
        }
        Expression::BinaryExpression(binary) => {
            let equality = match binary.operator {
                BinaryOperator::Equality | BinaryOperator::StrictEquality => true,
                BinaryOperator::Inequality | BinaryOperator::StrictInequality => false,
                _ => return None,
            };
            for (value, literal) in [(&binary.left, &binary.right), (&binary.right, &binary.left)] {
                if let Expression::StringLiteral(literal) = literal.get_inner_expression()
                    && literal.value == "undefined"
                    && let Some(probe) = window_probe(value)
                {
                    return Some(Truth {
                        server: equality,
                        global_probe: Some(probe),
                    });
                }
                // Boolean comparisons of the known flags are exact; do not coerce
                // arbitrary values in loose comparisons.
                if matches!(
                    binary.operator,
                    BinaryOperator::StrictEquality | BinaryOperator::StrictInequality
                ) && let Expression::BooleanLiteral(literal) = literal.get_inner_expression()
                    && let Some(truth) = server_truth(value)
                {
                    return Some(Truth {
                        server: (truth.server == literal.value) == equality,
                        ..truth
                    });
                }
            }
            None
        }
        Expression::LogicalExpression(logical) => {
            let left = server_truth(&logical.left);
            let right = server_truth(&logical.right);
            match logical.operator {
                LogicalOperator::And => combine(left, right, false),
                LogicalOperator::Or => combine(left, right, true),
                LogicalOperator::Coalesce => None,
            }
        }
        _ => None,
    }
}

fn combine(left: Option<Truth>, right: Option<Truth>, decisive: bool) -> Option<Truth> {
    // A single false premise proves `&&` false; a single true premise proves
    // `||` true. Prefer an unconditional import.meta premise when available.
    for truth in [left, right].into_iter().flatten() {
        if truth.server == decisive && truth.global_probe.is_none() {
            return Some(truth);
        }
    }
    for truth in [left, right].into_iter().flatten() {
        if truth.server == decisive {
            return Some(truth);
        }
    }
    let (left, right) = (left?, right?);
    let global_probe = match (left.global_probe, right.global_probe) {
        (Some(left), Some(right)) if left != right => return None,
        (Some(probe), _) | (_, Some(probe)) => Some(probe),
        (None, None) => None,
    };
    Some(Truth {
        server: !decisive,
        global_probe,
    })
}

fn window_probe(expression: &Expression<'_>) -> Option<Span> {
    let Expression::UnaryExpression(unary) = expression.get_inner_expression() else {
        return None;
    };
    if unary.operator != UnaryOperator::Typeof {
        return None;
    }
    let Expression::Identifier(identifier) = unary.argument.get_inner_expression() else {
        return None;
    };
    (identifier.name == "window").then_some(identifier.span)
}

fn note_probes(result: &mut ScriptParseResult, expression: &Expression<'_>) {
    if let Some(probe) = window_probe(expression) {
        // The safe probe itself is conditional on complete global ownership.
        note_region(result, probe, Some(probe));
        return;
    }
    match expression.get_inner_expression() {
        Expression::UnaryExpression(unary) => note_probes(result, &unary.argument),
        Expression::BinaryExpression(binary) => {
            note_probes(result, &binary.left);
            note_probes(result, &binary.right);
        }
        Expression::LogicalExpression(logical) => {
            note_probes(result, &logical.left);
            note_probes(result, &logical.right);
        }
        _ => {}
    }
}
