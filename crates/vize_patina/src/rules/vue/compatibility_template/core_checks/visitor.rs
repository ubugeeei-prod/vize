//! OXC visitor for the compatibility core-rule family.

use super::{
    CoreKind, compatibility_help,
    predicates::{
        addition_has_string_and_non_string, can_unquote_property, is_constant,
        is_empty_string_like, is_identifier_name, is_implicit_numeric_coercion, is_index_of_call,
        is_known_string, is_negated, is_numeric, is_string_like, is_underscored_identifier,
        leftmost_addition_operand, numeric_literal_may_lose_precision, property_name,
        rightmost_addition_operand,
    },
};
use crate::{context::LintContext, rules::vue::compatibility_template::origin::Origin};
use oxc_ast::ast::{
    ArrayExpression, ArrayExpressionElement, ArrayPattern, AssignmentExpression, AssignmentTarget,
    BinaryExpression, BindingIdentifier, BindingProperty, ComputedMemberExpression,
    ConditionalExpression, DoWhileStatement, Expression, ForStatement, IdentifierReference,
    IfStatement, NumericLiteral, ObjectExpression, ObjectPattern, ObjectProperty,
    ObjectPropertyKind, PropertyKey, PropertyKind, Statement, StaticMemberExpression,
    UnaryExpression, WhileStatement,
};
use oxc_ast_visit::{Visit, walk};
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::{AssignmentOperator, BinaryOperator, UnaryOperator};
use vize_l0::{CompactString, FxHashSet};

pub(super) struct CoreVisitor<'ctx, 'source> {
    ctx: &'ctx mut LintContext<'source>,
    origin: Origin,
    kind: CoreKind,
    pub(super) parse_offset: u32,
    prefer_template_covered_additions: FxHashSet<(u32, u32)>,
    pub(super) camelcase_bindings: bool,
    camelcase_declarations: FxHashSet<CompactString>,
}

impl CoreVisitor<'_, '_> {
    pub(super) fn new<'ctx, 'source>(
        ctx: &'ctx mut LintContext<'source>,
        origin: Origin,
        kind: CoreKind,
        camelcase_bindings: bool,
    ) -> CoreVisitor<'ctx, 'source> {
        CoreVisitor {
            ctx,
            origin,
            kind,
            parse_offset: 0,
            prefer_template_covered_additions: FxHashSet::default(),
            camelcase_bindings,
            camelcase_declarations: FxHashSet::default(),
        }
    }

    pub(super) fn report(&mut self, span: Span) {
        let span = Span::new(
            span.start.saturating_add(self.parse_offset),
            span.end.saturating_add(self.parse_offset),
        );
        if let Some(range) = self.origin.range(span) {
            self.ctx
                .warn_at_with_help(self.kind.message(), range, compatibility_help());
        }
    }

    pub(super) fn set_parse_offset(&mut self, parse_offset: u32) {
        if self.parse_offset != parse_offset {
            self.parse_offset = parse_offset;
            self.prefer_template_covered_additions.clear();
        }
    }

    fn expressions_share_line(&self, left: Span, right: Span) -> bool {
        let adjusted = |span: Span| {
            Span::new(
                span.start.saturating_add(self.parse_offset),
                span.end.saturating_add(self.parse_offset),
            )
        };
        let Some(left) = self.origin.range(adjusted(left)) else {
            return false;
        };
        let Some(right) = self.origin.range(adjusted(right)) else {
            return false;
        };
        self.ctx
            .source
            .get(left.end as usize..right.start as usize)
            .is_some_and(|between| !between.contains(['\n', '\r']))
    }
}

impl<'a> Visit<'a> for CoreVisitor<'_, '_> {
    fn visit_identifier_reference(&mut self, identifier: &IdentifierReference<'a>) {
        if self.kind == CoreKind::Camelcase
            && self
                .camelcase_declarations
                .contains(identifier.name.as_str())
        {
            self.report(identifier.span);
        }
        walk::walk_identifier_reference(self, identifier);
    }

    fn visit_computed_member_expression(&mut self, member: &ComputedMemberExpression<'a>) {
        if self.kind == CoreKind::DotNotation
            && member
                .static_property_name()
                .is_some_and(|property| is_identifier_name(property.as_str()))
        {
            self.report(member.expression.span());
        }
        if self.kind == CoreKind::NoConsole
            && matches!(member.object.get_inner_expression(),
                Expression::Identifier(identifier) if identifier.name == "console")
        {
            self.report(member.span);
        }
        walk::walk_computed_member_expression(self, member);
    }

    fn visit_static_member_expression(&mut self, member: &StaticMemberExpression<'a>) {
        if self.kind == CoreKind::NoConsole
            && matches!(
                member.object.get_inner_expression(),
                Expression::Identifier(identifier) if identifier.name == "console"
            )
        {
            self.report(member.span);
        }
        walk::walk_static_member_expression(self, member);
    }

    fn visit_binding_identifier(&mut self, identifier: &BindingIdentifier<'a>) {
        if self.kind == CoreKind::Camelcase
            && self.camelcase_bindings
            && is_underscored_identifier(&identifier.name)
        {
            self.report(identifier.span);
            self.camelcase_declarations
                .insert(CompactString::from(identifier.name.as_str()));
        }
        walk::walk_binding_identifier(self, identifier);
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
            CoreKind::NoImplicitCoercion if is_implicit_numeric_coercion(binary) => {
                self.report(binary.span);
            }
            CoreKind::NoImplicitCoercion
                if binary.operator == BinaryOperator::Addition
                    && is_empty_string_like(&binary.left)
                    && !is_known_string(&binary.right)
                    || binary.operator == BinaryOperator::Addition
                        && is_empty_string_like(&binary.right)
                        && !is_known_string(&binary.left) =>
            {
                self.report(binary.span);
            }
            CoreKind::NoUselessConcat if binary.operator == BinaryOperator::Addition => {
                let left = rightmost_addition_operand(&binary.left);
                let right = leftmost_addition_operand(&binary.right);
                if is_string_like(left)
                    && is_string_like(right)
                    && self.expressions_share_line(left.span(), right.span())
                {
                    self.report(binary.span);
                }
            }
            CoreKind::PreferTemplate
                if binary.operator == BinaryOperator::Addition
                    && !self
                        .prefer_template_covered_additions
                        .contains(&(binary.span.start, binary.span.end))
                    && addition_has_string_and_non_string(binary) =>
            {
                self.report(binary.span);
                collect_nested_addition_spans(
                    &binary.left,
                    &mut self.prefer_template_covered_additions,
                );
                collect_nested_addition_spans(
                    &binary.right,
                    &mut self.prefer_template_covered_additions,
                );
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
            CoreKind::NoNegatedCondition if is_negated(&conditional.test) => {
                self.report(conditional.test.span());
            }
            _ => {}
        }
        walk::walk_conditional_expression(self, conditional);
    }

    fn visit_if_statement(&mut self, statement: &IfStatement<'a>) {
        match self.kind {
            CoreKind::NoConstantCondition if is_constant(&statement.test) => {
                self.report(statement.test.span());
            }
            CoreKind::NoNegatedCondition
                if statement
                    .alternate
                    .as_ref()
                    .is_some_and(|alternate| !matches!(alternate, Statement::IfStatement(_)))
                    && is_negated(&statement.test) =>
            {
                self.report(statement.test.span());
            }
            _ => {}
        }
        walk::walk_if_statement(self, statement);
    }

    fn visit_while_statement(&mut self, statement: &WhileStatement<'a>) {
        if self.kind == CoreKind::NoConstantCondition && is_constant(&statement.test) {
            self.report(statement.test.span());
        }
        walk::walk_while_statement(self, statement);
    }

    fn visit_do_while_statement(&mut self, statement: &DoWhileStatement<'a>) {
        if self.kind == CoreKind::NoConstantCondition && is_constant(&statement.test) {
            self.report(statement.test.span());
        }
        walk::walk_do_while_statement(self, statement);
    }

    fn visit_for_statement(&mut self, statement: &ForStatement<'a>) {
        if self.kind == CoreKind::NoConstantCondition
            && let Some(test) = &statement.test
            && is_constant(test)
        {
            self.report(test.span());
        }
        walk::walk_for_statement(self, statement);
    }

    fn visit_array_expression(&mut self, array: &ArrayExpression<'a>) {
        if self.kind == CoreKind::NoSparseArrays
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
        if self.kind == CoreKind::NoEmptyPattern && object.is_empty() {
            self.report(object.span);
        }
        walk::walk_object_pattern(self, object);
    }

    fn visit_array_pattern(&mut self, array: &ArrayPattern<'a>) {
        if self.kind == CoreKind::NoEmptyPattern && array.is_empty() {
            self.report(array.span);
        }
        walk::walk_array_pattern(self, array);
    }

    fn visit_assignment_expression(&mut self, assignment: &AssignmentExpression<'a>) {
        if self.kind == CoreKind::NoEmptyPattern {
            match &assignment.left {
                AssignmentTarget::ObjectAssignmentTarget(object)
                    if object.properties.is_empty() =>
                {
                    self.report(object.span);
                }
                AssignmentTarget::ArrayAssignmentTarget(array) if array.elements.is_empty() => {
                    self.report(array.span);
                }
                _ => {}
            }
        }
        if self.kind == CoreKind::NoImplicitCoercion
            && assignment.operator == AssignmentOperator::Addition
            && is_empty_string_like(&assignment.right)
        {
            self.report(assignment.span);
        }
        if self.kind == CoreKind::Camelcase {
            match &assignment.left {
                AssignmentTarget::StaticMemberExpression(member)
                    if is_underscored_identifier(&member.property.name) =>
                {
                    self.report(member.property.span);
                }
                AssignmentTarget::ComputedMemberExpression(member)
                    if member
                        .static_property_name()
                        .is_some_and(|property| is_underscored_identifier(property.as_str())) =>
                {
                    self.report(member.expression.span());
                }
                _ => {}
            }
        }
        walk::walk_assignment_expression(self, assignment);
    }

    fn visit_unary_expression(&mut self, unary: &UnaryExpression<'a>) {
        if self.kind == CoreKind::NoImplicitCoercion
            && ((unary.operator == UnaryOperator::UnaryPlus && !is_numeric(&unary.argument))
                || (unary.operator == UnaryOperator::LogicalNot
                    && matches!(
                        unary.argument.get_inner_expression(),
                        Expression::UnaryExpression(inner)
                            if inner.operator == UnaryOperator::LogicalNot
                    ))
                || (unary.operator == UnaryOperator::BitwiseNot
                    && is_index_of_call(&unary.argument))
                || (unary.operator == UnaryOperator::UnaryNegation
                    && matches!(unary.argument.get_inner_expression(),
                        Expression::UnaryExpression(inner)
                            if inner.operator == UnaryOperator::UnaryNegation)))
        {
            self.report(unary.span);
        }
        walk::walk_unary_expression(self, unary);
    }

    fn visit_numeric_literal(&mut self, literal: &NumericLiteral<'a>) {
        if self.kind == CoreKind::NoLossOfPrecision && numeric_literal_may_lose_precision(literal) {
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
                        && !property.computed
                        && property_name(&property.key) != Some("__proto__")
                        && property_name(&property.key).is_some_and(|key| {
                            matches!(
                                &property.value,
                                Expression::Identifier(value) if value.name == key
                            )
                        }) =>
                {
                    self.report(property.span);
                }
                CoreKind::ObjectShorthand
                    if !property.method
                        && property.kind == PropertyKind::Init
                        && matches!(&property.value, Expression::FunctionExpression(function)
                            if function.id.is_none()) =>
                {
                    self.report(property.span);
                }
                CoreKind::QuoteProps
                    if !property.computed
                        && !property.method
                        && matches!(&property.key, PropertyKey::StringLiteral(literal)
                            if can_unquote_property(literal.value.as_str())) =>
                {
                    self.report(property.key.span());
                }
                _ => {}
            }
        }
        walk::walk_object_expression(self, object);
    }

    fn visit_object_property(&mut self, property: &ObjectProperty<'a>) {
        if self.kind == CoreKind::Camelcase
            && !property.computed
            && property_name(&property.key).is_some_and(is_underscored_identifier)
        {
            self.report(property.key.span());
        }
        walk::walk_object_property(self, property);
    }

    fn visit_binding_property(&mut self, property: &BindingProperty<'a>) {
        if self.kind == CoreKind::QuoteProps
            && !property.computed
            && matches!(&property.key, PropertyKey::StringLiteral(literal)
                if can_unquote_property(literal.value.as_str()))
        {
            self.report(property.key.span());
        }
        walk::walk_binding_property(self, property);
    }
}

fn collect_nested_addition_spans(expression: &Expression<'_>, spans: &mut FxHashSet<(u32, u32)>) {
    let Expression::BinaryExpression(binary) = expression.get_inner_expression() else {
        return;
    };
    if binary.operator != BinaryOperator::Addition {
        return;
    }
    spans.insert((binary.span.start, binary.span.end));
    collect_nested_addition_spans(&binary.left, spans);
    collect_nested_addition_spans(&binary.right, spans);
}
