// Generated from OXC's pinned immutable walk; see mod.rs for provenance.
use super::*;

#[inline]
pub(crate) fn walk_assignment_target_maybe_default<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a AssignmentTargetMaybeDefault<'a>,
) {
    // No `AstKind` for this type
    match it {
        AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(it) => {
            walk_assignment_target_with_default(visitor, it)
        }
        match_assignment_target!(AssignmentTargetMaybeDefault) => {
            walk_assignment_target(visitor, it.to_assignment_target())
        }
    }
}

#[inline]
pub(crate) fn walk_assignment_target_with_default<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a AssignmentTargetWithDefault<'a>,
) {
    walk_span(visitor, &it.span);
    walk_assignment_target(visitor, &it.binding);
    walk_expression(visitor, &it.init);
}

#[inline]
pub(crate) fn walk_assignment_target_property<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a AssignmentTargetProperty<'a>,
) {
    // No `AstKind` for this type
    match it {
        AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(it) => {
            walk_assignment_target_property_identifier(visitor, it)
        }
        AssignmentTargetProperty::AssignmentTargetPropertyProperty(it) => {
            walk_assignment_target_property_property(visitor, it)
        }
    }
}

#[inline]
pub(crate) fn walk_assignment_target_property_identifier<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a AssignmentTargetPropertyIdentifier<'a>,
) {
    walk_span(visitor, &it.span);
    walk_identifier_reference(visitor, &it.binding);
    if let Some(init) = &it.init {
        walk_expression(visitor, init);
    }
}

#[inline]
pub(crate) fn walk_assignment_target_property_property<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a AssignmentTargetPropertyProperty<'a>,
) {
    walk_span(visitor, &it.span);
    walk_property_key(visitor, &it.name);
    walk_assignment_target_maybe_default(visitor, &it.binding);
}

#[inline]
pub(crate) fn walk_sequence_expression<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a SequenceExpression<'a>,
) {
    walk_span(visitor, &it.span);
    walk_expressions(visitor, &it.expressions);
}

#[inline]
pub(crate) fn walk_super<'a, V: LentNodes<'a>>(visitor: &mut V, it: &'a Super) {
    walk_span(visitor, &it.span);
}

#[inline]
pub(crate) fn walk_await_expression<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a AwaitExpression<'a>,
) {
    walk_span(visitor, &it.span);
    walk_expression(visitor, &it.argument);
}

#[inline]
pub(crate) fn walk_chain_expression<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a ChainExpression<'a>,
) {
    walk_span(visitor, &it.span);
    walk_chain_element(visitor, &it.expression);
}

#[inline]
pub(crate) fn walk_chain_element<'a, V: LentNodes<'a>>(visitor: &mut V, it: &'a ChainElement<'a>) {
    // No `AstKind` for this type
    match it {
        ChainElement::CallExpression(it) => walk_call_expression(visitor, it),
        ChainElement::TSNonNullExpression(it) => walk_ts_non_null_expression(visitor, it),
        match_member_expression!(ChainElement) => {
            walk_member_expression(visitor, it.to_member_expression())
        }
    }
}

#[inline]
pub(crate) fn walk_parenthesized_expression<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a ParenthesizedExpression<'a>,
) {
    walk_span(visitor, &it.span);
    walk_expression(visitor, &it.expression);
}

pub(crate) fn walk_statement<'a, V: LentNodes<'a>>(visitor: &mut V, it: &'a Statement<'a>) {
    // No `AstKind` for this type
    match it {
        Statement::BlockStatement(it) => walk_block_statement(visitor, it),
        Statement::BreakStatement(it) => walk_break_statement(visitor, it),
        Statement::ContinueStatement(it) => walk_continue_statement(visitor, it),
        Statement::DebuggerStatement(it) => walk_debugger_statement(visitor, it),
        Statement::DoWhileStatement(it) => walk_do_while_statement(visitor, it),
        Statement::EmptyStatement(it) => walk_empty_statement(visitor, it),
        Statement::ExpressionStatement(it) => walk_expression_statement(visitor, it),
        Statement::ForInStatement(it) => walk_for_in_statement(visitor, it),
        Statement::ForOfStatement(it) => walk_for_of_statement(visitor, it),
        Statement::ForStatement(it) => walk_for_statement(visitor, it),
        Statement::IfStatement(it) => walk_if_statement(visitor, it),
        Statement::LabeledStatement(it) => walk_labeled_statement(visitor, it),
        Statement::ReturnStatement(it) => walk_return_statement(visitor, it),
        Statement::SwitchStatement(it) => walk_switch_statement(visitor, it),
        Statement::ThrowStatement(it) => walk_throw_statement(visitor, it),
        Statement::TryStatement(it) => walk_try_statement(visitor, it),
        Statement::WhileStatement(it) => walk_while_statement(visitor, it),
        Statement::WithStatement(it) => walk_with_statement(visitor, it),
        match_declaration!(Statement) => walk_declaration(visitor, it.to_declaration()),
        match_module_declaration!(Statement) => {
            walk_module_declaration(visitor, it.to_module_declaration())
        }
    }
}

#[inline]
pub(crate) fn walk_directive<'a, V: LentNodes<'a>>(visitor: &mut V, it: &'a Directive<'a>) {
    walk_span(visitor, &it.span);
    walk_string_literal(visitor, &it.expression);
}

#[inline]
pub(crate) fn walk_hashbang<'a, V: LentNodes<'a>>(visitor: &mut V, it: &'a Hashbang<'a>) {
    walk_span(visitor, &it.span);
}

#[inline]
pub(crate) fn walk_block_statement<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a BlockStatement<'a>,
) {
    walk_span(visitor, &it.span);
    walk_statements(visitor, &it.body);
}

pub(crate) fn walk_declaration<'a, V: LentNodes<'a>>(visitor: &mut V, it: &'a Declaration<'a>) {
    // No `AstKind` for this type
    match it {
        Declaration::VariableDeclaration(it) => walk_variable_declaration(visitor, it),
        Declaration::FunctionDeclaration(it) => {
            let flags = ScopeFlags::Function;
            walk_function(visitor, it, flags)
        }
        Declaration::ClassDeclaration(it) => walk_class(visitor, it),
        Declaration::TSTypeAliasDeclaration(it) => walk_ts_type_alias_declaration(visitor, it),
        Declaration::TSInterfaceDeclaration(it) => walk_ts_interface_declaration(visitor, it),
        Declaration::TSEnumDeclaration(it) => walk_ts_enum_declaration(visitor, it),
        Declaration::TSModuleDeclaration(it) => walk_ts_module_declaration(visitor, it),
        Declaration::TSGlobalDeclaration(it) => walk_ts_global_declaration(visitor, it),
        Declaration::TSImportEqualsDeclaration(it) => {
            walk_ts_import_equals_declaration(visitor, it)
        }
    }
}

#[inline]
pub(crate) fn walk_variable_declaration<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a VariableDeclaration<'a>,
) {
    walk_span(visitor, &it.span);
    walk_variable_declarators(visitor, &it.declarations);
}

#[inline]
pub(crate) fn walk_variable_declarator<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a VariableDeclarator<'a>,
) {
    walk_span(visitor, &it.span);
    walk_binding_pattern(visitor, &it.id);
    if let Some(type_annotation) = &it.type_annotation {
        walk_ts_type_annotation(visitor, type_annotation);
    }
    if let Some(init) = &it.init {
        walk_expression(visitor, init);
    }
}

#[inline]
pub(crate) fn walk_empty_statement<'a, V: LentNodes<'a>>(visitor: &mut V, it: &'a EmptyStatement) {
    walk_span(visitor, &it.span);
}

#[inline]
pub(crate) fn walk_expression_statement<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a ExpressionStatement<'a>,
) {
    walk_span(visitor, &it.span);
    walk_expression(visitor, &it.expression);
}

#[inline]
pub(crate) fn walk_if_statement<'a, V: LentNodes<'a>>(visitor: &mut V, it: &'a IfStatement<'a>) {
    walk_span(visitor, &it.span);
    walk_expression(visitor, &it.test);
    walk_statement(visitor, &it.consequent);
    if let Some(alternate) = &it.alternate {
        walk_statement(visitor, alternate);
    }
}

#[inline]
pub(crate) fn walk_do_while_statement<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a DoWhileStatement<'a>,
) {
    walk_span(visitor, &it.span);
    walk_statement(visitor, &it.body);
    walk_expression(visitor, &it.test);
}

#[inline]
pub(crate) fn walk_while_statement<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a WhileStatement<'a>,
) {
    walk_span(visitor, &it.span);
    walk_expression(visitor, &it.test);
    walk_statement(visitor, &it.body);
}

#[inline]
pub(crate) fn walk_for_statement<'a, V: LentNodes<'a>>(visitor: &mut V, it: &'a ForStatement<'a>) {
    walk_span(visitor, &it.span);
    if let Some(init) = &it.init {
        walk_for_statement_init(visitor, init);
    }
    if let Some(test) = &it.test {
        walk_expression(visitor, test);
    }
    if let Some(update) = &it.update {
        walk_expression(visitor, update);
    }
    walk_statement(visitor, &it.body);
}

#[inline]
pub(crate) fn walk_for_statement_init<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a ForStatementInit<'a>,
) {
    // No `AstKind` for this type
    match it {
        ForStatementInit::VariableDeclaration(it) => walk_variable_declaration(visitor, it),
        match_expression!(ForStatementInit) => walk_expression(visitor, it.to_expression()),
    }
}

#[inline]
pub(crate) fn walk_for_in_statement<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a ForInStatement<'a>,
) {
    walk_span(visitor, &it.span);
    walk_for_statement_left(visitor, &it.left);
    walk_expression(visitor, &it.right);
    walk_statement(visitor, &it.body);
}

#[inline]
pub(crate) fn walk_for_statement_left<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a ForStatementLeft<'a>,
) {
    // No `AstKind` for this type
    match it {
        ForStatementLeft::VariableDeclaration(it) => walk_variable_declaration(visitor, it),
        match_assignment_target!(ForStatementLeft) => {
            walk_assignment_target(visitor, it.to_assignment_target())
        }
    }
}

#[inline]
pub(crate) fn walk_for_of_statement<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a ForOfStatement<'a>,
) {
    walk_span(visitor, &it.span);
    walk_for_statement_left(visitor, &it.left);
    walk_expression(visitor, &it.right);
    walk_statement(visitor, &it.body);
}

#[inline]
pub(crate) fn walk_continue_statement<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a ContinueStatement<'a>,
) {
    walk_span(visitor, &it.span);
    if let Some(label) = &it.label {
        walk_label_identifier(visitor, label);
    }
}

#[inline]
pub(crate) fn walk_break_statement<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a BreakStatement<'a>,
) {
    walk_span(visitor, &it.span);
    if let Some(label) = &it.label {
        walk_label_identifier(visitor, label);
    }
}

#[inline]
pub(crate) fn walk_return_statement<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a ReturnStatement<'a>,
) {
    walk_span(visitor, &it.span);
    if let Some(argument) = &it.argument {
        walk_expression(visitor, argument);
    }
}

#[inline]
pub(crate) fn walk_with_statement<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a WithStatement<'a>,
) {
    walk_span(visitor, &it.span);
    walk_expression(visitor, &it.object);
    walk_statement(visitor, &it.body);
}

#[inline]
pub(crate) fn walk_switch_statement<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a SwitchStatement<'a>,
) {
    walk_span(visitor, &it.span);
    walk_expression(visitor, &it.discriminant);
    walk_switch_cases(visitor, &it.cases);
}

#[inline]
pub(crate) fn walk_switch_case<'a, V: LentNodes<'a>>(visitor: &mut V, it: &'a SwitchCase<'a>) {
    walk_span(visitor, &it.span);
    if let Some(test) = &it.test {
        walk_expression(visitor, test);
    }
    walk_statements(visitor, &it.consequent);
}

#[inline]
pub(crate) fn walk_labeled_statement<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a LabeledStatement<'a>,
) {
    walk_span(visitor, &it.span);
    walk_label_identifier(visitor, &it.label);
    walk_statement(visitor, &it.body);
}

#[inline]
pub(crate) fn walk_throw_statement<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a ThrowStatement<'a>,
) {
    walk_span(visitor, &it.span);
    walk_expression(visitor, &it.argument);
}

#[inline]
pub(crate) fn walk_try_statement<'a, V: LentNodes<'a>>(visitor: &mut V, it: &'a TryStatement<'a>) {
    walk_span(visitor, &it.span);
    walk_block_statement(visitor, &it.block);
    if let Some(handler) = &it.handler {
        walk_catch_clause(visitor, handler);
    }
    if let Some(finalizer) = &it.finalizer {
        walk_block_statement(visitor, finalizer);
    }
}

#[inline]
pub(crate) fn walk_catch_clause<'a, V: LentNodes<'a>>(visitor: &mut V, it: &'a CatchClause<'a>) {
    walk_span(visitor, &it.span);
    if let Some(param) = &it.param {
        walk_catch_parameter(visitor, param);
    }
    walk_block_statement(visitor, &it.body);
}

#[inline]
pub(crate) fn walk_catch_parameter<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a CatchParameter<'a>,
) {
    walk_span(visitor, &it.span);
    walk_binding_pattern(visitor, &it.pattern);
    if let Some(type_annotation) = &it.type_annotation {
        walk_ts_type_annotation(visitor, type_annotation);
    }
}

#[inline]
pub(crate) fn walk_debugger_statement<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a DebuggerStatement,
) {
    walk_span(visitor, &it.span);
}

#[inline]
pub(crate) fn walk_binding_pattern<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a BindingPattern<'a>,
) {
    // No `AstKind` for this type
    match it {
        BindingPattern::BindingIdentifier(it) => walk_binding_identifier(visitor, it),
        BindingPattern::ObjectPattern(it) => walk_object_pattern(visitor, it),
        BindingPattern::ArrayPattern(it) => walk_array_pattern(visitor, it),
        BindingPattern::AssignmentPattern(it) => walk_assignment_pattern(visitor, it),
    }
}

#[inline]
pub(crate) fn walk_assignment_pattern<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a AssignmentPattern<'a>,
) {
    walk_span(visitor, &it.span);
    walk_binding_pattern(visitor, &it.left);
    walk_expression(visitor, &it.right);
}

#[inline]
pub(crate) fn walk_object_pattern<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a ObjectPattern<'a>,
) {
    walk_span(visitor, &it.span);
    walk_binding_properties(visitor, &it.properties);
    if let Some(rest) = &it.rest {
        walk_binding_rest_element(visitor, rest);
    }
}
