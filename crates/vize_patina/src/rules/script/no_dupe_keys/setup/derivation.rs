//! Provenance of a setup binding that shares a prop or model name.
//!
//! Mirrors eslint-plugin-vue: a binding whose initializer reads the props
//! object (or the model) is derived, unless it directly aliases another prop
//! (`const foo = props.bar`, `toRef(props, 'bar')`, `{ bar: foo } = props`).

use oxc_ast::ast::{Argument, BindingPattern, Expression, IdentifierReference, VariableDeclarator};
use oxc_ast_visit::Visit;
use oxc_semantic::Semantic;
use oxc_span::{GetSpan, Span};
use oxc_syntax::symbol::SymbolId;

/// Whether `initializer` references one of `symbols` or contains one of `calls`.
pub(super) fn initializer_references(
    initializer: &Expression<'_>,
    semantic: &Semantic<'_>,
    symbols: &[SymbolId],
    calls: &[Span],
) -> bool {
    let span = initializer.span();
    if calls.iter().any(|call| span.contains_inclusive(*call)) {
        return true;
    }
    let mut visitor = References {
        semantic,
        symbols,
        found: false,
    };
    visitor.visit_expression(initializer);
    visitor.found
}

struct References<'s> {
    semantic: &'s Semantic<'s>,
    symbols: &'s [SymbolId],
    found: bool,
}

impl<'a> Visit<'a> for References<'_> {
    fn visit_identifier_reference(&mut self, identifier: &IdentifierReference<'a>) {
        self.found |= references(identifier, self.semantic, self.symbols);
    }
}

fn references(
    identifier: &IdentifierReference<'_>,
    semantic: &Semantic<'_>,
    symbols: &[SymbolId],
) -> bool {
    identifier
        .reference_id
        .get()
        .and_then(|reference| semantic.scoping().get_reference(reference).symbol_id())
        .is_some_and(|symbol| symbols.contains(&symbol))
}

fn is_props(expression: &Expression<'_>, semantic: &Semantic<'_>, props: &[SymbolId]) -> bool {
    matches!(expression.without_parentheses(), Expression::Identifier(identifier)
        if references(identifier, semantic, props))
}

/// ESTree `skipTSAsExpression` (parentheses are not nodes there).
fn skip_as<'b, 'a>(expression: &'b Expression<'a>) -> &'b Expression<'a> {
    match expression.without_parentheses() {
        Expression::TSAsExpression(expression) => skip_as(&expression.expression),
        expression => expression,
    }
}

/// Whether the declarator binds `name` to a different prop of the props object.
pub(super) fn aliases_other_prop(
    declarator: &VariableDeclarator<'_>,
    semantic: &Semantic<'_>,
    props: &[SymbolId],
    name: &str,
) -> bool {
    let Some(initializer) = &declarator.init else {
        return false;
    };
    match &declarator.id {
        BindingPattern::BindingIdentifier(_) => {
            aliased_prop(initializer, semantic, props).is_some_and(|aliased| aliased != name)
        }
        BindingPattern::ObjectPattern(pattern) if props_object(initializer, semantic, props) => {
            pattern
                .properties
                .iter()
                .find(|property| {
                    let value = match &property.value {
                        BindingPattern::AssignmentPattern(assignment) => &assignment.left,
                        value => value,
                    };
                    matches!(value, BindingPattern::BindingIdentifier(id) if id.name == name)
                })
                .and_then(|property| property.key.static_name())
                .is_some_and(|key| key != name)
        }
        _ => false,
    }
}

/// The prop an initializer reads directly: `props.bar` or `toRef(props, 'bar')`.
fn aliased_prop<'a>(
    initializer: &'a Expression<'a>,
    semantic: &Semantic<'_>,
    props: &[SymbolId],
) -> Option<std::borrow::Cow<'a, str>> {
    let expression = match skip_as(initializer) {
        Expression::ChainExpression(chain) => chain.expression.as_member_expression()?,
        expression => {
            if let Expression::CallExpression(call) = expression
                && matches!(&call.callee, Expression::Identifier(callee) if callee.name == "toRef")
                && call
                    .arguments
                    .first()
                    .and_then(Argument::as_expression)
                    .is_some_and(|argument| is_props(argument, semantic, props))
            {
                return match call.arguments.get(1) {
                    Some(Argument::StringLiteral(key)) => Some(key.value.as_str().into()),
                    _ => None,
                };
            }
            expression.as_member_expression()?
        }
    };
    if !is_props(skip_as(expression.object()), semantic, props) {
        return None;
    }
    expression.static_property_name().map(Into::into)
}

/// `props` itself or `toRefs(props)`.
fn props_object(expression: &Expression<'_>, semantic: &Semantic<'_>, props: &[SymbolId]) -> bool {
    match expression.without_parentheses() {
        Expression::CallExpression(call) => {
            matches!(&call.callee, Expression::Identifier(callee) if callee.name == "toRefs")
                && call
                    .arguments
                    .first()
                    .and_then(Argument::as_expression)
                    .is_some_and(|argument| is_props(argument, semantic, props))
        }
        expression => is_props(expression, semantic, props),
    }
}
