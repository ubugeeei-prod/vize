//! Async-component policy over source-owned binding facts. Module lookup stays in CF.

use super::value_binding_origins::{
    ValueBindingKey, ValueBindingOrigins, ValueImportExport, ValueReference,
};
use crate::scope::ScopeId;
use oxc_ast::ast::{
    BindingPattern, Expression, Function, ObjectPropertyKind, Program, PropertyKey, PropertyKind,
    Statement,
};
use vize_carton::{CompactString, FxHashMap, FxHashSet};

#[derive(Debug)]
enum Factory {
    Reference(ValueReference),
    Member(ValueReference, CompactString),
}

#[derive(Debug)]
enum Loader {
    Import(CompactString),
    Reference(ValueReference),
}

#[derive(Debug)]
struct ComponentCall {
    factory: Factory,
    loader: Loader,
}

#[derive(Debug, Default)]
pub(crate) struct AsyncComponentBindings {
    calls: FxHashMap<ValueBindingKey, ComponentCall>,
    loaders: FxHashMap<ValueBindingKey, CompactString>,
    factories: FxHashMap<ValueBindingKey, Factory>,
}

/// Admit the provenance store only when the existing collector can retain a
/// root async-component call. This predicate deliberately borrows the AST and
/// performs no hashing or string allocation: once admitted, the normal whole-
/// program walk still owns every alias, TDZ and mutation fence.
pub(crate) fn has_top_level_component_call(program: &Program<'_>) -> bool {
    program.body.iter().any(|statement| {
        let Statement::VariableDeclaration(declaration) = statement else {
            return false;
        };
        if !declaration.kind.is_const() {
            return false;
        }
        declaration.declarations.iter().any(|declarator| {
            if !matches!(&declarator.id, BindingPattern::BindingIdentifier(_)) {
                return false;
            }
            let Some(Expression::CallExpression(call)) = declarator
                .init
                .as_ref()
                .map(Expression::get_inner_expression)
            else {
                return false;
            };
            !call.optional
                && call.arguments.len() == 1
                && is_factory_reference(&call.callee)
                && call
                    .arguments
                    .first()
                    .and_then(|argument| argument.as_expression())
                    .is_some_and(is_component_loader)
        })
    })
}

fn is_factory_reference(expression: &Expression<'_>) -> bool {
    match expression.get_inner_expression() {
        Expression::Identifier(_) => true,
        Expression::StaticMemberExpression(member) => {
            !member.optional
                && matches!(
                    member.object.get_inner_expression(),
                    Expression::Identifier(_)
                )
        }
        _ => false,
    }
}

fn is_component_loader(expression: &Expression<'_>) -> bool {
    let expression = expression.get_inner_expression();
    let Expression::ObjectExpression(object) = expression else {
        return is_loader_value(expression);
    };
    let mut loader = false;
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return false;
        };
        if property.computed || property.method || property.kind != PropertyKind::Init {
            return false;
        }
        let Some(is_loader) = static_option_key(&property.key) else {
            return false;
        };
        if is_loader {
            if loader || !is_loader_value(&property.value) {
                return false;
            }
            loader = true;
        }
    }
    loader
}

/// The collector accepts every statically named option but only reads
/// `loader`. Mirror that surface without `PropertyKey::static_name`, which
/// formats numeric and regexp keys into an owned string.
fn static_option_key(key: &PropertyKey<'_>) -> Option<bool> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name == "loader"),
        PropertyKey::StringLiteral(literal) => Some(literal.value == "loader"),
        PropertyKey::TemplateLiteral(template) => template
            .single_quasi()
            .map(|quasi| quasi.as_str() == "loader"),
        PropertyKey::RegExpLiteral(_)
        | PropertyKey::NumericLiteral(_)
        | PropertyKey::BigIntLiteral(_)
        | PropertyKey::NullLiteral(_) => Some(false),
        _ => None,
    }
}

fn is_loader_value(expression: &Expression<'_>) -> bool {
    let expression = expression.get_inner_expression();
    matches!(expression, Expression::Identifier(_)) || is_function_loader(expression)
}

fn is_function_loader(expression: &Expression<'_>) -> bool {
    match expression {
        Expression::ArrowFunctionExpression(arrow)
            if arrow.params.items.is_empty() && arrow.params.rest.is_none() =>
        {
            match arrow.body.statements.as_slice() {
                [Statement::ExpressionStatement(statement)] if arrow.expression => {
                    is_import_path(&statement.expression)
                }
                [Statement::ReturnStatement(statement)] if !arrow.expression => {
                    statement.argument.as_ref().is_some_and(is_import_path)
                }
                _ => false,
            }
        }
        Expression::FunctionExpression(function)
            if !function.generator
                && function.params.items.is_empty()
                && function.params.rest.is_none() =>
        {
            matches!(
                function.body.as_ref().map(|body| body.statements.as_slice()),
                Some([Statement::ReturnStatement(statement)])
                    if statement.argument.as_ref().is_some_and(is_import_path)
            )
        }
        _ => false,
    }
}

fn is_import_path(expression: &Expression<'_>) -> bool {
    let Expression::ImportExpression(import) = expression.get_inner_expression() else {
        return false;
    };
    import.options.is_none()
        && import.phase.is_none()
        && matches!(
            import.source.get_inner_expression(),
            Expression::StringLiteral(_)
        )
}

impl AsyncComponentBindings {
    pub(crate) fn note_initializer(
        &mut self,
        binding: ValueBindingKey,
        scope: ScopeId,
        expression: &Expression<'_>,
    ) {
        let expression = expression.get_inner_expression();
        if let Some(path) = function_loader(expression) {
            self.loaders.insert(binding, path);
        }
        if let Some(factory) = factory_reference(scope, expression) {
            self.factories.insert(binding, factory);
        }
        let Expression::CallExpression(call) = expression else {
            return;
        };
        if call.optional || call.arguments.len() != 1 {
            return;
        }
        let Some(factory) = factory_reference(scope, &call.callee) else {
            return;
        };
        let Some(argument) = call
            .arguments
            .first()
            .and_then(|argument| argument.as_expression())
        else {
            return;
        };
        let Some(loader) = component_loader(scope, argument) else {
            return;
        };
        self.calls
            .insert(binding, ComponentCall { factory, loader });
    }

    pub(crate) fn note_function(&mut self, binding: ValueBindingKey, function: &Function<'_>) {
        if let Some(path) = function_body_loader(function) {
            self.loaders.insert(binding, path);
        }
    }

    pub(crate) fn module_for<'a>(
        &'a self,
        origins: &ValueBindingOrigins,
        tag: &str,
    ) -> Option<&'a str> {
        if !origins.is_complete() {
            return None;
        }
        let reference = origins.root_reference(tag)?;
        let resolved = origins.resolve(&reference)?;
        if !origins.by_key_is_stable(resolved.binding) {
            return None;
        }
        let binding = resolved.canonical?;
        let call = self.calls.get(&binding)?;
        if !self.is_vue_factory(origins, &call.factory, &mut FxHashSet::default()) {
            return None;
        }
        match &call.loader {
            Loader::Import(path) => Some(path.as_str()),
            Loader::Reference(reference) => {
                let key = origins.resolve(reference)?.canonical?;
                self.loaders.get(&key).map(CompactString::as_str)
            }
        }
    }

    fn is_vue_factory(
        &self,
        origins: &ValueBindingOrigins,
        factory: &Factory,
        seen: &mut FxHashSet<ValueBindingKey>,
    ) -> bool {
        let reference = match factory {
            Factory::Reference(reference) | Factory::Member(reference, _) => reference,
        };
        let Some(resolved) = origins.resolve(reference) else {
            return false;
        };
        let Some(key) = resolved.canonical else {
            return false;
        };
        if !seen.insert(key) || origins.has_member_write(key) {
            return false;
        }
        if let Some(import) = resolved.import {
            if import.source != "vue" || !import.value || !import.ordinary {
                return false;
            }
            return match (factory, import.export) {
                (Factory::Reference(_), ValueImportExport::Named(export)) => {
                    export == "defineAsyncComponent"
                }
                (Factory::Member(_, member), ValueImportExport::Namespace) => {
                    member == "defineAsyncComponent"
                }
                _ => false,
            };
        }
        match factory {
            Factory::Reference(_) => self
                .factories
                .get(&key)
                .is_some_and(|next| self.is_vue_factory(origins, next, seen)),
            Factory::Member(_, _) => false,
        }
    }
}

fn factory_reference(scope: ScopeId, expression: &Expression<'_>) -> Option<Factory> {
    match expression.get_inner_expression() {
        Expression::Identifier(id) => Some(Factory::Reference(ValueReference::new(
            scope,
            id.name.as_str(),
            id.span,
        ))),
        Expression::StaticMemberExpression(member) if !member.optional => {
            let Expression::Identifier(id) = member.object.get_inner_expression() else {
                return None;
            };
            Some(Factory::Member(
                ValueReference::new(scope, id.name.as_str(), id.span),
                CompactString::new(member.property.name.as_str()),
            ))
        }
        _ => None,
    }
}

fn component_loader(scope: ScopeId, expression: &Expression<'_>) -> Option<Loader> {
    let expression = expression.get_inner_expression();
    if let Expression::ObjectExpression(object) = expression {
        let mut loader = None;
        for property in &object.properties {
            let ObjectPropertyKind::ObjectProperty(property) = property else {
                return None;
            };
            if property.computed || property.method || property.kind != PropertyKind::Init {
                return None;
            }
            let name = property.key.static_name()?;
            if name == "loader" {
                if loader.is_some() {
                    return None;
                }
                loader = loader_value(scope, &property.value);
                // An opaque duplicate must not be replaced by a later loader.
                loader.as_ref()?;
            }
        }
        loader
    } else {
        loader_value(scope, expression)
    }
}

fn loader_value(scope: ScopeId, expression: &Expression<'_>) -> Option<Loader> {
    if let Some(path) = function_loader(expression.get_inner_expression()) {
        return Some(Loader::Import(path));
    }
    match expression.get_inner_expression() {
        Expression::Identifier(id) => Some(Loader::Reference(ValueReference::new(
            scope,
            id.name.as_str(),
            id.span,
        ))),
        _ => None,
    }
}

/// Exactly the returned import, never an arbitrary import somewhere in a body.
fn function_loader(expression: &Expression<'_>) -> Option<CompactString> {
    match expression {
        Expression::ArrowFunctionExpression(arrow)
            if arrow.params.items.is_empty() && arrow.params.rest.is_none() =>
        {
            match arrow.body.statements.as_slice() {
                [Statement::ExpressionStatement(statement)] if arrow.expression => {
                    import_path(&statement.expression)
                }
                [Statement::ReturnStatement(statement)] if !arrow.expression => {
                    import_path(statement.argument.as_ref()?)
                }
                _ => None,
            }
        }
        Expression::FunctionExpression(function) => function_body_loader(function),
        _ => None,
    }
}

fn function_body_loader(function: &Function<'_>) -> Option<CompactString> {
    if function.generator || !function.params.items.is_empty() || function.params.rest.is_some() {
        return None;
    }
    match function.body.as_ref()?.statements.as_slice() {
        [Statement::ReturnStatement(statement)] => import_path(statement.argument.as_ref()?),
        _ => None,
    }
}

fn import_path(expression: &Expression<'_>) -> Option<CompactString> {
    let Expression::ImportExpression(import) = expression.get_inner_expression() else {
        return None;
    };
    if import.options.is_some() || import.phase.is_some() {
        return None;
    }
    let Expression::StringLiteral(literal) = import.source.get_inner_expression() else {
        return None;
    };
    Some(CompactString::new(literal.value.as_str()))
}
