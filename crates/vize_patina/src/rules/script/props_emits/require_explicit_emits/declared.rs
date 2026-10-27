//! Declared-emit resolution for `script/require-explicit-emits`.
//!
//! Mirrors eslint-plugin-vue's `getComponentEmitsFromOptions` and
//! `getComponentEmitsFromDefineEmits`: one entry that cannot be named (a
//! spread, a computed key, an unresolved type) makes the whole set unknown, and
//! an unknown set declares every event.

use oxc_ast::ast::{
    Argument, ArrayExpressionElement, BindingPattern, CallExpression, Expression, ObjectExpression,
    ObjectPropertyKind, PropertyKey, Statement,
};
use oxc_syntax::symbol::SymbolId;
use vize_l0::{CompactString, FxHashSet, ToCompactString};

use super::types::{Block, Resolver};

/// The events a component declares.
pub(super) enum Emits {
    Known(FxHashSet<CompactString>),
    Unknown,
}

impl Emits {
    pub(super) fn declares(&self, name: &str) -> bool {
        match self {
            Emits::Known(names) => names.contains(name),
            Emits::Unknown => true,
        }
    }
}

#[derive(Default)]
pub(super) struct Collect {
    names: FxHashSet<CompactString>,
    pub(super) unknown: bool,
}

impl Collect {
    pub(super) fn push(&mut self, name: Option<CompactString>) {
        match name {
            Some(name) => {
                self.names.insert(name);
            }
            None => self.unknown = true,
        }
    }

    pub(super) fn finish(self) -> Emits {
        if self.unknown {
            Emits::Unknown
        } else {
            Emits::Known(self.names)
        }
    }
}

/// The `emits` option of a Vue object. No option declares nothing.
pub(super) fn options_emits(object: &ObjectExpression<'_>) -> Emits {
    let option = object
        .properties
        .iter()
        .find_map(|property| match property {
            ObjectPropertyKind::ObjectProperty(property)
                if property_name(&property.key, property.computed).as_deref() == Some("emits") =>
            {
                Some(property)
            }
            _ => None,
        });
    match option {
        None => Emits::Known(FxHashSet::default()),
        Some(option) => define_value(&option.value).unwrap_or(Emits::Unknown),
    }
}

/// A top-level `defineEmits(...)` macro of `<script setup>`.
pub(super) struct DefineEmits<'a> {
    /// Calls before the macro are not checked, as upstream visits in order.
    pub(super) start: u32,
    /// `const emit = defineEmits(...)`: the template-visible name and symbol.
    pub(super) binding: Option<(&'a str, Option<SymbolId>)>,
    pub(super) emits: Emits,
}

pub(super) fn define_emits<'a>(resolver: &Resolver<'_, 'a, '_>) -> Option<DefineEmits<'a>> {
    for statement in &resolver.program.body {
        match statement {
            Statement::VariableDeclaration(declaration) => {
                for declarator in &declaration.declarations {
                    let Some(call) = declarator.init.as_ref().and_then(macro_call) else {
                        continue;
                    };
                    let binding = match &declarator.id {
                        BindingPattern::BindingIdentifier(id) => {
                            Some((id.name.as_str(), id.symbol_id.get()))
                        }
                        _ => None,
                    };
                    return Some(DefineEmits {
                        start: call.span.start,
                        binding,
                        emits: call_emits(call, resolver),
                    });
                }
            }
            Statement::ExpressionStatement(statement) => {
                if let Some(call) = macro_call(&statement.expression) {
                    return Some(DefineEmits {
                        start: call.span.start,
                        binding: None,
                        emits: call_emits(call, resolver),
                    });
                }
            }
            _ => {}
        }
    }
    None
}

fn macro_call<'b, 'a>(expression: &'b Expression<'a>) -> Option<&'b CallExpression<'a>> {
    match expression.without_parentheses() {
        Expression::CallExpression(call) if matches!(&call.callee, Expression::Identifier(id) if id.name == "defineEmits") => {
            Some(call)
        }
        _ => None,
    }
}

fn call_emits(call: &CallExpression<'_>, resolver: &Resolver<'_, '_, '_>) -> Emits {
    if let Some(argument) = call.arguments.first() {
        return match argument {
            Argument::SpreadElement(_) => Emits::Unknown,
            argument => argument
                .as_expression()
                .and_then(|expression| resolver.object_or_array(expression, Block::Own, 0))
                .unwrap_or(Emits::Unknown),
        };
    }
    call.type_arguments
        .as_ref()
        .and_then(|arguments| arguments.params.first())
        .map_or(Emits::Unknown, |declared| resolver.type_emits(declared))
}

/// Upstream's `getComponentEmitsFromDefine` for an object or array literal.
pub(super) fn define_value(expression: &Expression<'_>) -> Option<Emits> {
    let mut collect = Collect::default();
    match expression.without_parentheses() {
        Expression::ObjectExpression(object) => {
            for property in &object.properties {
                collect.push(match property {
                    ObjectPropertyKind::ObjectProperty(property) => {
                        property_name(&property.key, property.computed)
                    }
                    ObjectPropertyKind::SpreadProperty(_) => None,
                });
            }
        }
        Expression::ArrayExpression(array) => {
            for element in &array.elements {
                match element {
                    ArrayExpressionElement::Elision(_) => {}
                    ArrayExpressionElement::SpreadElement(_) => collect.push(None),
                    element => collect.push(element.as_expression().and_then(literal_value)),
                }
            }
        }
        _ => return None,
    }
    Some(collect.finish())
}

/// Upstream's `getStaticPropertyName` for a property.
fn property_name(key: &PropertyKey<'_>, computed: bool) -> Option<CompactString> {
    match key {
        PropertyKey::StaticIdentifier(identifier) if !computed => {
            Some(identifier.name.to_compact_string())
        }
        key => literal_value(key.as_expression()?),
    }
}

/// Upstream's `getStringLiteralValue`, which also stringifies other literals.
fn literal_value(expression: &Expression<'_>) -> Option<CompactString> {
    match expression.without_parentheses() {
        Expression::StringLiteral(literal) => Some(literal.value.to_compact_string()),
        Expression::TemplateLiteral(literal) if literal.expressions.is_empty() => literal
            .single_quasi()
            .map(|quasi| quasi.to_compact_string()),
        Expression::NumericLiteral(literal) => Some(literal.value.to_compact_string()),
        Expression::BooleanLiteral(literal) => Some(literal.value.to_compact_string()),
        Expression::BigIntLiteral(literal) => Some(literal.value.to_compact_string()),
        Expression::RegExpLiteral(literal) => Some(literal.regex.to_compact_string()),
        _ => None,
    }
}
