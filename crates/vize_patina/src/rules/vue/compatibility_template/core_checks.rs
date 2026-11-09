//! Shared retained-AST walk for the template core-extension rules.

use super::origin::Origin;
use crate::context::LintContext;
use oxc_ast::ast::{
    ArrayExpression, ArrayExpressionElement, ArrayPattern, BinaryExpression,
    ComputedMemberExpression, ConditionalExpression, Expression, IdentifierReference,
    NumericLiteral, ObjectExpression, ObjectPattern, ObjectPropertyKind, PropertyKey,
    StaticMemberExpression, UnaryExpression,
};
use oxc_ast_visit::{Visit, walk};
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::{BinaryOperator, UnaryOperator};
use vize_l1::markup::entity::EntityContext;
use vize_relief::ExpressionNode;

#[derive(Clone, Copy)]
pub(super) enum CoreKind {
    Camelcase,
    DotNotation,
    Eqeqeq,
    NoConsole,
    NoConstantCondition,
    NoEmptyPattern,
    NoImplicitCoercion,
    NoIrregularWhitespace,
    NoLossOfPrecision,
    NoNegatedCondition,
    NoSparseArrays,
    NoUselessConcat,
    ObjectShorthand,
    PreferTemplate,
    QuoteProps,
}

impl CoreKind {
    fn message(self) -> &'static str {
        match self {
            Self::Camelcase => "Identifier is not in camel case",
            Self::DotNotation => "Use dot notation instead of a computed property",
            Self::Eqeqeq => "Use strict equality operators",
            Self::NoConsole => "Unexpected console access in a template expression",
            Self::NoConstantCondition => "Unexpected constant condition",
            Self::NoEmptyPattern => "Unexpected empty destructuring pattern",
            Self::NoImplicitCoercion => "Use an explicit type conversion",
            Self::NoIrregularWhitespace => "Unexpected irregular whitespace",
            Self::NoLossOfPrecision => "This numeric literal loses precision",
            Self::NoNegatedCondition => "Unexpected negated ternary condition",
            Self::NoSparseArrays => "Unexpected sparse array",
            Self::NoUselessConcat => "Unexpected concatenation of literal strings",
            Self::ObjectShorthand => "Use object property shorthand",
            Self::PreferTemplate => "Use a template literal instead of string concatenation",
            Self::QuoteProps => "Unnecessarily quoted object property",
        }
    }
}

pub(super) fn check_expression(
    ctx: &mut LintContext<'_>,
    expression: &ExpressionNode<'_>,
    kind: CoreKind,
    entity_context: EntityContext,
) {
    let ExpressionNode::Simple(simple) = expression else {
        return;
    };
    let Some(origin) = Origin::new(ctx.source, simple.content, simple.loc.span, entity_context)
    else {
        return;
    };

    if matches!(kind, CoreKind::NoIrregularWhitespace) {
        for (index, character) in simple.content.char_indices() {
            if is_irregular_whitespace(character)
                && let Some(range) = origin.range(Span::new(
                    index as u32,
                    (index + character.len_utf8()) as u32,
                ))
            {
                ctx.warn_at_with_help(kind.message(), range, compatibility_help());
            }
        }
        return;
    }

    let Some(retained) = simple
        .js_ast
        .filter(|retained| retained.raw == simple.content)
    else {
        return;
    };
    let Some(retained) = retained.as_expression() else {
        return;
    };
    let mut visitor = CoreVisitor { ctx, origin, kind };
    visitor.visit_expression(retained.ast);
}

fn compatibility_help() -> &'static str {
    "Move this JavaScript expression to a script binding or rewrite it using the recommended syntax"
}

struct CoreVisitor<'ctx, 'source> {
    ctx: &'ctx mut LintContext<'source>,
    origin: Origin,
    kind: CoreKind,
}

impl CoreVisitor<'_, '_> {
    fn report(&mut self, span: Span) {
        if let Some(range) = self.origin.range(span) {
            self.ctx
                .warn_at_with_help(self.kind.message(), range, compatibility_help());
        }
    }
}

impl<'a> Visit<'a> for CoreVisitor<'_, '_> {
    fn visit_identifier_reference(&mut self, identifier: &IdentifierReference<'a>) {
        if matches!(self.kind, CoreKind::Camelcase) && has_internal_underscore(&identifier.name) {
            self.report(identifier.span);
        }
        walk::walk_identifier_reference(self, identifier);
    }

    fn visit_computed_member_expression(&mut self, member: &ComputedMemberExpression<'a>) {
        if matches!(self.kind, CoreKind::DotNotation)
            && member
                .static_property_name()
                .is_some_and(|property| is_identifier_name(property.as_str()))
        {
            self.report(member.expression.span());
        }
        walk::walk_computed_member_expression(self, member);
    }

    fn visit_static_member_expression(&mut self, member: &StaticMemberExpression<'a>) {
        if matches!(self.kind, CoreKind::NoConsole)
            && matches!(
                member.object.get_inner_expression(),
                Expression::Identifier(identifier) if identifier.name == "console"
            )
        {
            self.report(member.span);
        }
        walk::walk_static_member_expression(self, member);
    }

    fn visit_binary_expression(&mut self, binary: &BinaryExpression<'a>) {
        match self.kind {
            CoreKind::Eqeqeq
                if matches!(
                    binary.operator,
                    BinaryOperator::Equality | BinaryOperator::Inequality
                ) =>
            {
                self.report(binary.span);
            }
            CoreKind::NoImplicitCoercion
                if binary.operator == BinaryOperator::Addition
                    && (is_empty_string(&binary.left) || is_empty_string(&binary.right)) =>
            {
                self.report(binary.span);
            }
            CoreKind::NoUselessConcat
                if binary.operator == BinaryOperator::Addition
                    && is_string_literal(&binary.left)
                    && is_string_literal(&binary.right) =>
            {
                self.report(binary.span);
            }
            CoreKind::PreferTemplate
                if binary.operator == BinaryOperator::Addition
                    && (is_string_literal(&binary.left) || is_string_literal(&binary.right)) =>
            {
                self.report(binary.span);
            }
            _ => {}
        }
        walk::walk_binary_expression(self, binary);
    }

    fn visit_conditional_expression(&mut self, conditional: &ConditionalExpression<'a>) {
        match self.kind {
            CoreKind::NoConstantCondition if is_constant(&conditional.test) => {
                self.report(conditional.test.span());
            }
            CoreKind::NoNegatedCondition
                if matches!(
                    conditional.test.get_inner_expression(),
                    Expression::UnaryExpression(unary)
                        if unary.operator == UnaryOperator::LogicalNot
                ) =>
            {
                self.report(conditional.test.span());
            }
            _ => {}
        }
        walk::walk_conditional_expression(self, conditional);
    }

    fn visit_array_expression(&mut self, array: &ArrayExpression<'a>) {
        if matches!(self.kind, CoreKind::NoSparseArrays)
            && array
                .elements
                .iter()
                .any(|element| matches!(element, ArrayExpressionElement::Elision(_)))
        {
            self.report(array.span);
        }
        walk::walk_array_expression(self, array);
    }

    fn visit_object_pattern(&mut self, object: &ObjectPattern<'a>) {
        if matches!(self.kind, CoreKind::NoEmptyPattern) && object.is_empty() {
            self.report(object.span);
        }
        walk::walk_object_pattern(self, object);
    }

    fn visit_array_pattern(&mut self, array: &ArrayPattern<'a>) {
        if matches!(self.kind, CoreKind::NoEmptyPattern) && array.is_empty() {
            self.report(array.span);
        }
        walk::walk_array_pattern(self, array);
    }

    fn visit_unary_expression(&mut self, unary: &UnaryExpression<'a>) {
        if matches!(self.kind, CoreKind::NoImplicitCoercion)
            && (unary.operator == UnaryOperator::UnaryPlus
                || (unary.operator == UnaryOperator::LogicalNot
                    && matches!(
                        unary.argument.get_inner_expression(),
                        Expression::UnaryExpression(inner)
                            if inner.operator == UnaryOperator::LogicalNot
                    )))
        {
            self.report(unary.span);
        }
        walk::walk_unary_expression(self, unary);
    }

    fn visit_numeric_literal(&mut self, literal: &NumericLiteral<'a>) {
        if matches!(self.kind, CoreKind::NoLossOfPrecision)
            && numeric_literal_may_lose_precision(literal)
        {
            self.report(literal.span);
        }
        walk::walk_numeric_literal(self, literal);
    }

    fn visit_object_expression(&mut self, object: &ObjectExpression<'a>) {
        for entry in &object.properties {
            let ObjectPropertyKind::ObjectProperty(property) = entry else {
                continue;
            };
            match self.kind {
                CoreKind::ObjectShorthand
                    if !property.shorthand
                        && !property.method
                        && property_name(&property.key).is_some_and(|key| {
                            matches!(
                                property.value.get_inner_expression(),
                                Expression::Identifier(value) if value.name == key
                            )
                        }) =>
                {
                    self.report(property.span);
                }
                CoreKind::QuoteProps
                    if matches!(&property.key, PropertyKey::StringLiteral(literal)
                        if is_identifier_name(literal.value.as_str())) =>
                {
                    self.report(property.key.span());
                }
                _ => {}
            }
        }
        walk::walk_object_expression(self, object);
    }
}

fn has_internal_underscore(name: &str) -> bool {
    name.trim_matches('_').contains('_')
}

fn is_identifier_name(name: &str) -> bool {
    let mut chars = name.chars();
    chars
        .next()
        .is_some_and(|character| character == '_' || character == '$' || character.is_alphabetic())
        && chars
            .all(|character| character == '_' || character == '$' || character.is_alphanumeric())
        && !oxc_syntax::keyword::is_reserved_keyword(name)
}

fn is_empty_string(expression: &Expression<'_>) -> bool {
    matches!(expression.get_inner_expression(), Expression::StringLiteral(literal) if literal.value.is_empty())
}

fn is_string_literal(expression: &Expression<'_>) -> bool {
    match expression.get_inner_expression() {
        Expression::StringLiteral(_) => true,
        Expression::TemplateLiteral(template) => template.expressions.is_empty(),
        _ => false,
    }
}

fn is_constant(expression: &Expression<'_>) -> bool {
    matches!(
        expression.get_inner_expression(),
        Expression::BooleanLiteral(_)
            | Expression::NullLiteral(_)
            | Expression::NumericLiteral(_)
            | Expression::StringLiteral(_)
    )
}

fn numeric_literal_may_lose_precision(literal: &NumericLiteral<'_>) -> bool {
    let Some(raw) = literal.raw.as_ref().map(|raw| raw.as_str()) else {
        return false;
    };
    let significant_digits = raw
        .bytes()
        .filter(|byte| byte.is_ascii_digit())
        .skip_while(|byte| *byte == b'0')
        .count();
    significant_digits > 15
}

fn property_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.as_str()),
        PropertyKey::Identifier(identifier) => Some(identifier.name.as_str()),
        PropertyKey::StringLiteral(literal) => Some(literal.value.as_str()),
        _ => None,
    }
}

fn is_irregular_whitespace(character: char) -> bool {
    matches!(
        character,
        '\u{000B}' | '\u{000C}' | '\u{00A0}' | '\u{0085}' | '\u{1680}' | '\u{180E}' | '\u{2000}'
            ..='\u{200B}'
                | '\u{2028}'
                | '\u{2029}'
                | '\u{202F}'
                | '\u{205F}'
                | '\u{3000}'
                | '\u{FEFF}'
    )
}
