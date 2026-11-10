//! OXC visitor for the compatibility core-rule family.

use super::{
    CoreKind,
    constant::is_constant,
    precision::numeric_literal_may_lose_precision,
    predicates::{
        addition_has_string_and_non_string, can_unquote_property, collect_nested_addition_spans,
        dot_notation_key, is_empty_string_like, is_implicit_numeric_coercion, is_index_of_call,
        is_known_string, is_negated, is_numeric, is_string_like, is_underscored_identifier,
        leftmost_addition_operand, multiplies_by_fraction_of_one, property_name,
        rightmost_addition_operand,
    },
};
use crate::{context::LintContext, rules::vue::compatibility_template::origin::Origin};
use oxc_ast::ast::{
    ArrayExpression, ArrayExpressionElement, ArrayPattern, AssignmentExpression, AssignmentPattern,
    AssignmentTarget, AssignmentTargetPropertyIdentifier, AssignmentTargetWithDefault,
    BinaryExpression, BindingIdentifier, BindingProperty, CallExpression, ComputedMemberExpression,
    ConditionalExpression, DoWhileStatement, Expression, ForInStatement, ForOfStatement,
    ForStatement, FormalParameter, Function, IdentifierReference, IfStatement, ImportExpression,
    LabelIdentifier, MethodDefinition, NewExpression, NumericLiteral, ObjectExpression,
    ObjectPattern, ObjectPropertyKind, PropertyDefinition, PropertyKey, PropertyKind, Statement,
    StaticMemberExpression, UnaryExpression, WhileStatement, YieldExpression,
};
use oxc_ast_visit::{Visit, walk};
use oxc_span::GetSpan;
use oxc_syntax::operator::{AssignmentOperator, BinaryOperator, UnaryOperator};
use oxc_syntax::scope::ScopeFlags;
use vize_l0::{CompactString, FxHashSet};

pub(super) struct CoreVisitor<'ctx, 'source> {
    pub(super) ctx: &'ctx mut LintContext<'source>,
    pub(super) origin: Origin,
    pub(super) kind: CoreKind,
    /// The parsed text; AST offsets plus `parse_offset` index into it.
    pub(super) source: &'source str,
    pub(super) parse_offset: u32,
    /// Spans an enclosing node already settled: nested `prefer-template`
    /// additions, or the `a * 1` of `a * 1 / b`.
    pub(super) covered_spans: FxHashSet<(u32, u32)>,
    pub(super) camelcase_bindings: bool,
    camelcase_declarations: FxHashSet<CompactString>,
    /// A collection pass that reports nothing.
    pub(super) silent: bool,
    /// Constant loop tests of the current function, cleared by `yield`.
    pub(super) pending_loops: Vec<bool>,
}

impl CoreVisitor<'_, '_> {
    pub(super) fn new<'ctx, 'source>(
        ctx: &'ctx mut LintContext<'source>,
        origin: Origin,
        source: &'source str,
        kind: CoreKind,
        camelcase_bindings: bool,
    ) -> CoreVisitor<'ctx, 'source> {
        CoreVisitor {
            ctx,
            origin,
            source,
            kind,
            parse_offset: 0,
            covered_spans: FxHashSet::default(),
            camelcase_bindings,
            camelcase_declarations: FxHashSet::default(),
            silent: false,
            pending_loops: Vec::new(),
        }
    }
}

impl<'a> Visit<'a> for CoreVisitor<'_, '_> {
    fn visit_identifier_reference(&mut self, identifier: &IdentifierReference<'a>) {
        if self.kind == CoreKind::Camelcase
            && self
                .camelcase_declarations
                .contains(identifier.name.as_str())
            && !self
                .covered_spans
                .contains(&(identifier.span.start, identifier.span.end))
        {
            self.report(identifier.span);
        }
        walk::walk_identifier_reference(self, identifier);
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if self.kind == CoreKind::Camelcase {
            self.camelcase_skip_call_operands(&call.callee, &call.arguments);
        }
        walk::walk_call_expression(self, call);
    }

    fn visit_new_expression(&mut self, call: &NewExpression<'a>) {
        if self.kind == CoreKind::Camelcase {
            self.camelcase_skip_call_operands(&call.callee, &call.arguments);
        }
        walk::walk_new_expression(self, call);
    }

    fn visit_assignment_pattern(&mut self, pattern: &AssignmentPattern<'a>) {
        if self.kind == CoreKind::Camelcase {
            self.camelcase_skip_reference(&pattern.right);
        }
        walk::walk_assignment_pattern(self, pattern);
    }

    fn visit_formal_parameter(&mut self, parameter: &FormalParameter<'a>) {
        if self.kind == CoreKind::Camelcase
            && let Some(initializer) = &parameter.initializer
        {
            self.camelcase_skip_reference(initializer);
        }
        walk::walk_formal_parameter(self, parameter);
    }

    fn visit_assignment_target_with_default(&mut self, target: &AssignmentTargetWithDefault<'a>) {
        if self.kind == CoreKind::Camelcase {
            self.camelcase_skip_reference(&target.init);
        }
        walk::walk_assignment_target_with_default(self, target);
    }

    fn visit_assignment_target_property_identifier(
        &mut self,
        target: &AssignmentTargetPropertyIdentifier<'a>,
    ) {
        if self.kind == CoreKind::Camelcase
            && let Some(init) = &target.init
        {
            self.camelcase_skip_reference(init);
        }
        walk::walk_assignment_target_property_identifier(self, target);
    }

    fn visit_import_expression(&mut self, import: &ImportExpression<'a>) {
        if self.kind == CoreKind::Camelcase {
            self.camelcase_import_options(import.options.as_ref());
        }
        walk::walk_import_expression(self, import);
    }

    fn visit_method_definition(&mut self, method: &MethodDefinition<'a>) {
        if self.kind == CoreKind::Camelcase {
            self.camelcase_class_key(&method.key, method.computed);
        }
        walk::walk_method_definition(self, method);
    }

    fn visit_property_definition(&mut self, property: &PropertyDefinition<'a>) {
        if self.kind == CoreKind::Camelcase {
            self.camelcase_class_key(&property.key, property.computed);
        }
        walk::walk_property_definition(self, property);
    }

    fn visit_label_identifier(&mut self, label: &LabelIdentifier<'a>) {
        if self.kind == CoreKind::Camelcase && is_underscored_identifier(&label.name) {
            self.report(label.span);
        }
        walk::walk_label_identifier(self, label);
    }

    fn visit_for_of_statement(&mut self, statement: &ForOfStatement<'a>) {
        if self.kind == CoreKind::Camelcase
            && let Some(target) = statement.left.as_assignment_target()
        {
            self.camelcase_assignment_target(target, false);
        }
        walk::walk_for_of_statement(self, statement);
    }

    fn visit_for_in_statement(&mut self, statement: &ForInStatement<'a>) {
        if self.kind == CoreKind::Camelcase
            && let Some(target) = statement.left.as_assignment_target()
        {
            self.camelcase_assignment_target(target, false);
        }
        walk::walk_for_in_statement(self, statement);
    }

    fn visit_function(&mut self, function: &Function<'a>, flags: ScopeFlags) {
        let outer_loops = std::mem::take(&mut self.pending_loops);
        walk::walk_function(self, function, flags);
        self.pending_loops = outer_loops;
    }

    fn visit_yield_expression(&mut self, expression: &YieldExpression<'a>) {
        self.pending_loops.fill(false);
        walk::walk_yield_expression(self, expression);
    }

    fn visit_computed_member_expression(&mut self, member: &ComputedMemberExpression<'a>) {
        if self.kind == CoreKind::DotNotation
            && let Some(key) = dot_notation_key(&member.expression)
        {
            self.report(key.span());
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
        if self.kind == CoreKind::NoImplicitCoercion
            && let Some(dividend) = multiplies_by_fraction_of_one(binary)
        {
            self.covered_spans
                .insert((dividend.span.start, dividend.span.end));
        }
        match self.kind {
            CoreKind::Eqeqeq
                if matches!(
                    binary.operator,
                    BinaryOperator::Equality | BinaryOperator::Inequality
                ) =>
            {
                self.report_operator(
                    binary.left.span(),
                    binary.right.span(),
                    binary.operator.as_str(),
                );
            }
            CoreKind::NoImplicitCoercion
                if is_implicit_numeric_coercion(binary)
                    && !self
                        .covered_spans
                        .contains(&(binary.span.start, binary.span.end)) =>
            {
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
                    self.report_operator(left.span(), right.span(), "+");
                }
            }
            CoreKind::PreferTemplate
                if binary.operator == BinaryOperator::Addition
                    && !self
                        .covered_spans
                        .contains(&(binary.span.start, binary.span.end))
                    && addition_has_string_and_non_string(binary) =>
            {
                self.report(binary.span);
                collect_nested_addition_spans(&binary.left, &mut self.covered_spans);
                collect_nested_addition_spans(&binary.right, &mut self.covered_spans);
            }
            _ => {}
        }
        walk::walk_binary_expression(self, binary);
    }

    fn visit_conditional_expression(&mut self, conditional: &ConditionalExpression<'a>) {
        match self.kind {
            CoreKind::NoConstantCondition if is_constant(&conditional.test, true) => {
                self.report(conditional.test.without_parentheses().span());
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
            CoreKind::NoConstantCondition if is_constant(&statement.test, true) => {
                self.report(statement.test.without_parentheses().span());
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
        // The default `checkLoops: "allExceptWhileTrue"` leaves `while (true)` alone.
        let while_true = matches!(
            statement.test.without_parentheses(),
            Expression::BooleanLiteral(literal) if literal.value
        );
        let test = (!while_true).then_some(&statement.test);
        self.enter_loop(test);
        walk::walk_while_statement(self, statement);
        self.exit_loop(test);
    }

    fn visit_do_while_statement(&mut self, statement: &DoWhileStatement<'a>) {
        self.enter_loop(Some(&statement.test));
        walk::walk_do_while_statement(self, statement);
        self.exit_loop(Some(&statement.test));
    }

    fn visit_for_statement(&mut self, statement: &ForStatement<'a>) {
        self.enter_loop(statement.test.as_ref());
        walk::walk_for_statement(self, statement);
        self.exit_loop(statement.test.as_ref());
    }

    fn visit_array_expression(&mut self, array: &ArrayExpression<'a>) {
        if self.kind == CoreKind::NoSparseArrays
            && array
                .elements
                .iter()
                .any(|element| matches!(element, ArrayExpressionElement::Elision(_)))
        {
            self.report_sparse_array_holes(array);
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
            self.camelcase_assignment_target(&assignment.left, true);
        }
        walk::walk_assignment_expression(self, assignment);
    }

    fn visit_unary_expression(&mut self, unary: &UnaryExpression<'a>) {
        if self.kind == CoreKind::NoImplicitCoercion
            && ((unary.operator == UnaryOperator::UnaryPlus && !is_numeric(&unary.argument))
                || (unary.operator == UnaryOperator::LogicalNot
                    && matches!(
                        unary.argument.without_parentheses(),
                        Expression::UnaryExpression(inner)
                            if inner.operator == UnaryOperator::LogicalNot
                    ))
                || (unary.operator == UnaryOperator::BitwiseNot
                    && is_index_of_call(&unary.argument))
                || (unary.operator == UnaryOperator::UnaryNegation
                    && matches!(unary.argument.without_parentheses(),
                        Expression::UnaryExpression(inner)
                            if inner.operator == UnaryOperator::UnaryNegation
                                && !is_numeric(&inner.argument))))
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
        if self.kind == CoreKind::Camelcase {
            self.camelcase_object_keys(object);
        }
        for entry in &object.properties {
            let ObjectPropertyKind::ObjectProperty(property) = entry else {
                continue;
            };
            // `{ __proto__: value }` sets the prototype; the shorthand forms define an own property.
            let proto_key = !property.computed && property_name(&property.key) == Some("__proto__");
            let value = property.value.without_parentheses();
            match self.kind {
                CoreKind::ObjectShorthand
                    if !property.shorthand
                        && !property.method
                        && !property.computed
                        && !proto_key
                        && property_name(&property.key).is_some_and(|key| {
                            matches!(value, Expression::Identifier(value) if value.name == key)
                        }) =>
                {
                    self.report(property.span);
                }
                CoreKind::ObjectShorthand
                    if !property.method
                        && !proto_key
                        && property.kind == PropertyKind::Init
                        && matches!(value, Expression::FunctionExpression(function)
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
