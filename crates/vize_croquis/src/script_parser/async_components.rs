//! Async-component policy over source-owned binding facts. Module lookup stays in CF.

use super::value_binding_origins::{
    ValueBindingKey, ValueBindingOrigins, ValueImportExport, ValueReference,
};
use crate::scope::ScopeId;
use oxc_ast::ast::{Expression, Function, ObjectPropertyKind, PropertyKind, Statement};
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
        let Some(argument) = call.arguments[0].as_expression() else {
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
            if import.source != "vue" || !import.value {
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
    if import.options.is_some() {
        return None;
    }
    let Expression::StringLiteral(literal) = import.source.get_inner_expression() else {
        return None;
    };
    Some(CompactString::new(literal.value.as_str()))
}
