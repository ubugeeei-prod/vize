//! Conservative derived-state proof; ambiguity suppresses advice.
use super::unwrap_expression;
use oxc_ast::ast::{
    AssignmentTargetPropertyIdentifier, BindingIdentifier, BindingPattern, CallExpression,
    Expression, IdentifierReference, Program, SimpleAssignmentTarget, Statement, UnaryExpression,
};
use oxc_ast_visit::{Visit, walk};
use oxc_syntax::operator::{AssignmentOperator, UnaryOperator};
use vize_l0::{CompactString, FxHashMap, FxHashSet};

#[derive(Default)]
pub(super) struct Inventory {
    aliases: FxHashMap<CompactString, CompactString>,
    bindings: FxHashMap<CompactString, usize>,
    refs: FxHashSet<CompactString>,
    writes: FxHashMap<CompactString, usize>,
}

impl Inventory {
    pub(super) fn collect(program: &Program<'_>) -> Self {
        let mut inventory = Self::default();
        for statement in &program.body {
            if let Statement::ImportDeclaration(import) = statement
                && import.source.value == "vue"
                && let Some(specifiers) = &import.specifiers
            {
                for specifier in specifiers {
                    if let oxc_ast::ast::ImportDeclarationSpecifier::ImportSpecifier(specifier) =
                        specifier
                    {
                        inventory.aliases.insert(
                            CompactString::new(specifier.local.name.as_str()),
                            CompactString::new(specifier.imported.name().as_str()),
                        );
                    }
                }
            }
        }
        inventory.visit_program(program);
        // Limit suggestions to component/module declarations. Any same-named
        // binding elsewhere makes name resolution ambiguous and suppresses it.
        for statement in &program.body {
            if let Statement::VariableDeclaration(declaration) = statement {
                for declarator in &declaration.declarations {
                    if let BindingPattern::BindingIdentifier(binding) = &declarator.id
                        && let Some(Expression::CallExpression(call)) =
                            declarator.init.as_ref().map(unwrap_expression)
                        && let Expression::Identifier(factory) = unwrap_expression(&call.callee)
                        && (inventory.is_vue_factory(factory.name.as_str(), "ref")
                            || inventory.is_vue_factory(factory.name.as_str(), "shallowRef"))
                        && inventory.bindings.get(binding.name.as_str()) == Some(&1)
                    {
                        inventory
                            .refs
                            .insert(CompactString::new(binding.name.as_str()));
                    }
                }
            }
        }
        inventory
    }

    pub(super) fn is_vue_factory(&self, local: &str, factory: &str) -> bool {
        match self.aliases.get(local) {
            Some(imported) => imported == factory && self.bindings.get(local) == Some(&1),
            None => local == factory && !self.bindings.contains_key(local),
        }
    }

    pub(super) fn written_names(self) -> impl Iterator<Item = CompactString> {
        self.writes.into_keys()
    }

    fn write(&mut self, name: &str) {
        *self.writes.entry(CompactString::new(name)).or_default() += 1;
    }
}

impl<'a> Visit<'a> for Inventory {
    fn visit_binding_identifier(&mut self, binding: &BindingIdentifier<'a>) {
        *self
            .bindings
            .entry(CompactString::new(binding.name.as_str()))
            .or_default() += 1;
    }

    fn visit_simple_assignment_target(&mut self, target: &SimpleAssignmentTarget<'a>) {
        match target {
            SimpleAssignmentTarget::AssignmentTargetIdentifier(identifier) => {
                self.write(identifier.name.as_str())
            }
            SimpleAssignmentTarget::StaticMemberExpression(member) => {
                if let Some(name) = root_name(&member.object) {
                    self.write(name);
                }
            }
            SimpleAssignmentTarget::ComputedMemberExpression(member) => {
                if let Some(name) = root_name(&member.object) {
                    self.write(name);
                }
            }
            SimpleAssignmentTarget::TSAsExpression(expression) => {
                if let Some(name) = root_name(&expression.expression) {
                    self.write(name);
                }
            }
            SimpleAssignmentTarget::TSSatisfiesExpression(expression) => {
                if let Some(name) = root_name(&expression.expression) {
                    self.write(name);
                }
            }
            SimpleAssignmentTarget::TSNonNullExpression(expression) => {
                if let Some(name) = root_name(&expression.expression) {
                    self.write(name);
                }
            }
            SimpleAssignmentTarget::TSTypeAssertion(expression) => {
                if let Some(name) = root_name(&expression.expression) {
                    self.write(name);
                }
            }
            _ => {}
        }
        walk::walk_simple_assignment_target(self, target);
    }

    fn visit_assignment_target_property_identifier(
        &mut self,
        target: &AssignmentTargetPropertyIdentifier<'a>,
    ) {
        self.write(target.binding.name.as_str());
        walk::walk_assignment_target_property_identifier(self, target);
    }

    fn visit_unary_expression(&mut self, expression: &UnaryExpression<'a>) {
        if expression.operator == UnaryOperator::Delete
            && let Some(name) = root_name(&expression.argument)
        {
            self.write(name);
        }
        walk::walk_unary_expression(self, expression);
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if let Expression::StaticMemberExpression(member) = unwrap_expression(&call.callee)
            && matches!(
                member.property.name.as_str(),
                "push"
                    | "pop"
                    | "shift"
                    | "unshift"
                    | "splice"
                    | "sort"
                    | "reverse"
                    | "fill"
                    | "copyWithin"
            )
            && let Some(name) = root_name(&member.object)
        {
            self.write(name);
        }
        walk::walk_call_expression(self, call);
    }
}

fn root_name<'a, 'b>(expression: &'b Expression<'a>) -> Option<&'b str> {
    match unwrap_expression(expression) {
        Expression::Identifier(identifier) => Some(identifier.name.as_str()),
        Expression::StaticMemberExpression(member) => root_name(&member.object),
        Expression::ComputedMemberExpression(member) => root_name(&member.object),
        _ => None,
    }
}

/// Only a synchronous, unconditional, single assignment can be derived state.
pub(super) fn derived_assignment<'a, 'b>(
    call: &'b CallExpression<'a>,
    inventory: &Inventory,
) -> Option<(&'b str, oxc_span::Span)> {
    // Watch options such as `once` retain event-driven state rather than a
    // continuously derived value. Unknown options make the advice unsafe.
    if call.arguments.len() != 2 {
        return None;
    }
    let callback = unwrap_expression(call.arguments.get(1)?.as_expression()?);
    let (params, body) = match callback {
        Expression::ArrowFunctionExpression(arrow) if !arrow.r#async => {
            (&arrow.params, &arrow.body)
        }
        Expression::FunctionExpression(function) if !function.r#async && !function.generator => {
            (&function.params, function.body.as_ref()?)
        }
        _ => return None,
    };
    if body.statements.len() != 1
        || !body.directives.is_empty()
        || params.items.len() > 1
        || params.rest.is_some()
    {
        return None;
    }
    let Statement::ExpressionStatement(statement) = body.statements.first()? else {
        return None;
    };
    let Expression::AssignmentExpression(assignment) = unwrap_expression(&statement.expression)
    else {
        return None;
    };
    let oxc_ast::ast::AssignmentTarget::StaticMemberExpression(member) = &assignment.left else {
        return None;
    };
    let Expression::Identifier(target) = unwrap_expression(&member.object) else {
        return None;
    };
    if assignment.operator != AssignmentOperator::Assign
        || member.property.name != "value"
        || !inventory.refs.contains(target.name.as_str())
        || inventory.writes.get(target.name.as_str()) != Some(&1)
    {
        return None;
    }
    let parameter = match params.items.first() {
        Some(parameter) => {
            let BindingPattern::BindingIdentifier(identifier) = &parameter.pattern else {
                return None;
            };
            Some(identifier.name.as_str())
        }
        None => None,
    };
    let source = source_path(call.arguments.first()?.as_expression()?);
    let mut proof = PureDerived {
        parameter,
        source: source.as_deref(),
        target: target.name.as_str(),
        pure: true,
        depends: false,
    };
    proof.visit_expression(&assignment.right);
    (proof.pure && proof.depends).then_some((target.name.as_str(), assignment.span))
}

fn member_path(expression: &Expression<'_>) -> Option<CompactString> {
    match unwrap_expression(expression) {
        Expression::Identifier(identifier) => Some(CompactString::new(identifier.name.as_str())),
        Expression::StaticMemberExpression(member) => {
            let mut path = member_path(&member.object)?;
            path.push('.');
            path.push_str(member.property.name.as_str());
            Some(path)
        }
        _ => None,
    }
}

fn source_path(expression: &Expression<'_>) -> Option<CompactString> {
    match unwrap_expression(expression) {
        Expression::Identifier(identifier) => Some(vize_l0::cstr!("{}.value", identifier.name)),
        Expression::ArrowFunctionExpression(arrow) if !arrow.r#async && arrow.expression => {
            let Statement::ExpressionStatement(statement) = arrow.body.statements.first()? else {
                return None;
            };
            member_path(&statement.expression)
        }
        _ => None,
    }
}

struct PureDerived<'s> {
    parameter: Option<&'s str>,
    source: Option<&'s str>,
    target: &'s str,
    pure: bool,
    depends: bool,
}
impl<'a> Visit<'a> for PureDerived<'_> {
    fn visit_expression(&mut self, expression: &Expression<'a>) {
        if matches!(
            expression,
            Expression::CallExpression(_)
                | Expression::NewExpression(_)
                | Expression::AwaitExpression(_)
                | Expression::YieldExpression(_)
                | Expression::AssignmentExpression(_)
                | Expression::UpdateExpression(_)
                | Expression::ArrowFunctionExpression(_)
                | Expression::FunctionExpression(_)
                | Expression::TaggedTemplateExpression(_)
                | Expression::SequenceExpression(_)
                | Expression::ClassExpression(_)
        ) {
            self.pure = false;
            return;
        }
        if let Expression::UnaryExpression(unary) = expression
            && unary.operator == UnaryOperator::Delete
        {
            self.pure = false;
            return;
        }
        if let Some(source) = self.source
            && member_path(expression).is_some_and(|path| path == source)
        {
            self.depends = true;
        }
        walk::walk_expression(self, expression);
    }
    fn visit_identifier_reference(&mut self, identifier: &IdentifierReference<'a>) {
        if identifier.name == self.target {
            self.pure = false;
        }
        if self.parameter == Some(identifier.name.as_str()) {
            self.depends = true;
        }
    }
}
