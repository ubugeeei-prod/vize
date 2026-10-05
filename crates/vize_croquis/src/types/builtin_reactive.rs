//! Primitive Vue builtin facts recorded by the existing declarator AST walk.
//! These are source-owned editor facts, independent of native checker answers.

use super::TypeResolver;
use crate::reactivity::ReactiveKind;
use vize_carton::CompactString;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Primitive {
    Boolean,
    Number,
    String,
}

/// A proven builtin's primitive value and exact authored identifier span.
/// Only the script AST producer can construct or insert these facts.
#[derive(Debug, Clone)]
pub struct BuiltinReactiveType {
    pub(crate) name: CompactString,
    pub(crate) span: (u32, u32),
    pub(crate) kind: ReactiveKind,
    pub(crate) value: Primitive,
}

impl BuiltinReactiveType {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn span(&self) -> (u32, u32) {
        self.span
    }

    pub fn kind(&self) -> ReactiveKind {
        self.kind
    }

    pub fn value_type(&self) -> &'static str {
        match self.value {
            Primitive::Boolean => "boolean",
            Primitive::Number => "number",
            Primitive::String => "string",
        }
    }
}

impl TypeResolver {
    pub fn builtin_reactive_types(&self) -> impl Iterator<Item = &BuiltinReactiveType> {
        self.builtin_reactive.values()
    }

    pub(crate) fn builtin_reactive_type(&self, name: &str) -> Option<&BuiltinReactiveType> {
        self.builtin_reactive.get(name)
    }

    pub(crate) fn clear_builtin_reactive_type(&mut self, name: &str) {
        self.builtin_reactive.remove(name);
    }

    pub(crate) fn clear_builtin_reactive_types(&mut self) {
        self.builtin_reactive.clear();
    }

    pub(crate) fn record_builtin_reactive_type(&mut self, fact: BuiltinReactiveType) {
        self.builtin_reactive.insert(fact.name.clone(), fact);
    }

    pub(crate) fn shift_builtin_reactive_types(&mut self, delta: u32) {
        for fact in self.builtin_reactive.values_mut() {
            fact.span.0 = fact.span.0.saturating_add(delta);
            fact.span.1 = fact.span.1.saturating_add(delta);
        }
    }
}
