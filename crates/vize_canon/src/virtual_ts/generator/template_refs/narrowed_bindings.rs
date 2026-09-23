//! Setup bindings a template reads in a narrowing position.
//!
//! `vue-tsc` reads such a binding through `.value` at every position, asserted
//! as `NonNullable<T> & Ref & { value: Unwrapped<T> }`. For almost every type
//! that equals the distributive unwrap; the observable difference is a union
//! of a ref and a nullish member (`inject<Ref<number>>(key)`), whose ref
//! `value` absorbs the nullish member: `number`, not `number | undefined`.
//! Only maybe-ref bindings can carry such a union, so only those are analysed.

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Argument, AssignmentTarget, BinaryOperator, ChainElement, Expression, SimpleAssignmentTarget,
    UnaryOperator,
};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::{FxHashSet, String};
use vize_croquis::{Croquis, ScopeData, TemplateExpressionKind};

/// The subset of `candidates` some template expression narrows.
pub(super) fn collect(summary: &Croquis, candidates: &[&str]) -> FxHashSet<String> {
    let mut narrowed = FxHashSet::default();
    if candidates.is_empty() {
        return narrowed;
    }
    let mut visit = |source: &str, narrowing: bool| {
        if candidates.iter().any(|name| source.contains(name)) {
            collect_expression(source, narrowing, candidates, &mut narrowed);
        }
    };
    for expression in &summary.template_expressions {
        visit(
            expression.content.as_str(),
            expression.kind == TemplateExpressionKind::VIf,
        );
        if let Some(guard) = expression.vif_guard.as_ref() {
            visit(guard.as_str(), true);
        }
    }
    for usage in vize_croquis::facts::component_usage_list(summary) {
        if let Some(guard) = usage.vif_guard.as_ref() {
            visit(guard.as_str(), true);
        }
        for prop in &usage.props {
            if prop.is_dynamic
                && let Some(value) = prop.value.as_ref()
            {
                visit(value.as_str(), false);
            }
        }
    }
    for scope in summary.scopes.iter() {
        match scope.data() {
            ScopeData::VFor(data) => visit(data.source.as_str(), false),
            ScopeData::EventHandler(data) => {
                if let Some(handler) = data.handler_expression.as_ref() {
                    visit(handler.as_str(), false);
                }
            }
            _ => {}
        }
    }
    narrowed
}

fn collect_expression(
    source: &str,
    narrowing: bool,
    candidates: &[&str],
    narrowed: &mut FxHashSet<String>,
) {
    let allocator = Allocator::default();
    let Ok(expression) = Parser::new(&allocator, source, SourceType::ts()).parse_expression()
    else {
        return;
    };
    walk(&expression, narrowing, &mut |name| {
        if candidates.contains(&name) {
            narrowed.insert(name.into());
        }
    });
}

/// `vue-tsc`'s narrowing positions: a member-access root, a condition, an
/// equality operand, an assignment target and the `!`/`typeof`/`delete`/update
/// operands. Every other position is a plain read of its children.
fn walk(expression: &Expression<'_>, narrowing: bool, record: &mut impl FnMut(&str)) {
    match expression {
        Expression::Identifier(identifier) => {
            if narrowing {
                record(identifier.name.as_str());
            }
        }
        Expression::StaticMemberExpression(member) => walk(&member.object, true, record),
        Expression::PrivateFieldExpression(member) => walk(&member.object, true, record),
        Expression::ComputedMemberExpression(member) => {
            walk(&member.object, true, record);
            walk(&member.expression, false, record);
        }
        Expression::ChainExpression(chain) => match &chain.expression {
            ChainElement::StaticMemberExpression(member) => walk(&member.object, true, record),
            ChainElement::PrivateFieldExpression(member) => walk(&member.object, true, record),
            ChainElement::ComputedMemberExpression(member) => {
                walk(&member.object, true, record);
                walk(&member.expression, false, record);
            }
            ChainElement::CallExpression(call) => {
                walk(&call.callee, false, record);
                walk_arguments(&call.arguments, narrowing, record);
            }
            ChainElement::TSNonNullExpression(inner) => walk(&inner.expression, narrowing, record),
        },
        Expression::CallExpression(call) => {
            walk(&call.callee, false, record);
            walk_arguments(&call.arguments, narrowing, record);
        }
        Expression::NewExpression(new) => {
            walk(&new.callee, false, record);
            walk_arguments(&new.arguments, narrowing, record);
        }
        Expression::ParenthesizedExpression(inner) => walk(&inner.expression, narrowing, record),
        Expression::TSNonNullExpression(inner) => walk(&inner.expression, narrowing, record),
        Expression::TSAsExpression(inner) => walk(&inner.expression, narrowing, record),
        Expression::TSSatisfiesExpression(inner) => walk(&inner.expression, narrowing, record),
        Expression::TSTypeAssertion(inner) => walk(&inner.expression, narrowing, record),
        Expression::LogicalExpression(logical) => {
            walk(&logical.left, true, record);
            walk(&logical.right, narrowing, record);
        }
        Expression::ConditionalExpression(conditional) => {
            walk(&conditional.test, true, record);
            walk(&conditional.consequent, false, record);
            walk(&conditional.alternate, false, record);
        }
        Expression::SequenceExpression(sequence) => {
            let last = sequence.expressions.len().saturating_sub(1);
            for (index, expression) in sequence.expressions.iter().enumerate() {
                walk(expression, narrowing && index == last, record);
            }
        }
        Expression::BinaryExpression(binary) => {
            let equality = matches!(
                binary.operator,
                BinaryOperator::Equality
                    | BinaryOperator::Inequality
                    | BinaryOperator::StrictEquality
                    | BinaryOperator::StrictInequality
            );
            walk(
                &binary.left,
                equality || binary.operator == BinaryOperator::Instanceof,
                record,
            );
            walk(
                &binary.right,
                equality || binary.operator == BinaryOperator::In,
                record,
            );
        }
        Expression::UnaryExpression(unary) => walk(
            &unary.argument,
            matches!(
                unary.operator,
                UnaryOperator::LogicalNot | UnaryOperator::Typeof | UnaryOperator::Delete
            ),
            record,
        ),
        Expression::UpdateExpression(update) => {
            if let SimpleAssignmentTarget::AssignmentTargetIdentifier(identifier) = &update.argument
            {
                record(identifier.name.as_str());
            }
        }
        Expression::AssignmentExpression(assignment) => {
            if let AssignmentTarget::AssignmentTargetIdentifier(identifier) = &assignment.left {
                record(identifier.name.as_str());
            }
            walk(&assignment.right, false, record);
        }
        Expression::ArrowFunctionExpression(arrow) if arrow.expression => {
            if let Some(oxc_ast::ast::Statement::ExpressionStatement(body)) =
                arrow.body.statements.first()
            {
                walk(&body.expression, false, record);
            }
        }
        Expression::ArrayExpression(array) => {
            for element in &array.elements {
                if let Some(expression) = element.as_expression() {
                    walk(expression, false, record);
                }
            }
        }
        Expression::ObjectExpression(object) => {
            for property in &object.properties {
                if let oxc_ast::ast::ObjectPropertyKind::ObjectProperty(property) = property {
                    walk(&property.value, false, record);
                }
            }
        }
        Expression::TemplateLiteral(template) => {
            for expression in &template.expressions {
                walk(expression, false, record);
            }
        }
        _ => {}
    }
}

fn walk_arguments(arguments: &[Argument<'_>], narrowing: bool, record: &mut impl FnMut(&str)) {
    for argument in arguments {
        if let Some(expression) = argument.as_expression() {
            walk(expression, narrowing, record);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::collect_expression;
    use vize_carton::{FxHashSet, String};

    fn narrowed(source: &str, narrowing: bool) -> Vec<String> {
        let mut names = FxHashSet::default();
        collect_expression(source, narrowing, &["a", "b", "c"], &mut names);
        let mut names: Vec<String> = names.into_iter().collect();
        names.sort_unstable();
        names
    }

    #[test]
    fn narrowing_positions_follow_the_vue_tsc_access_rules() {
        let cases: [(&str, bool, &[&str]); 9] = [
            ("a", false, &[]),
            ("a", true, &["a"]),
            ("a.x + b[c]", false, &["a", "b"]),
            ("a && b", false, &["a"]),
            ("a ? b : c", false, &["a"]),
            ("!a || typeof b === 'string'", false, &["a", "b"]),
            ("f(a, b?.x)", true, &["a", "b"]),
            ("a = b", false, &["a"]),
            ("[a, { k: b }, `${c}`]", true, &[]),
        ];
        for (source, narrowing, expected) in cases {
            let expected: Vec<String> = expected.iter().map(|name| (*name).into()).collect();
            assert_eq!(narrowed(source, narrowing), expected, "{source}");
        }
    }
}
