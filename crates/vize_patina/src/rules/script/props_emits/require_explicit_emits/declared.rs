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
use oxc_span::{GetSpan, Span};
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

/// One named macro declaration and the node upstream reports it at.
pub(crate) struct MacroEntry {
    pub(crate) name: CompactString,
    pub(crate) span: Span,
    /// Declared in the sibling script block rather than the linted one.
    pub(crate) sibling: bool,
    /// From the second payload value to the last, when the declaration has
    /// more than one: eslint-plugin-vue's `prefer-single-event-payload` range.
    pub(crate) extra_payload: Option<Span>,
}

/// The span from the `from`-th node to the last one, if there is a `from`-th.
pub(super) fn tail_span(mut spans: impl Iterator<Item = Span>, from: usize) -> Option<Span> {
    let first = spans.nth(from)?;
    let last = spans.last().unwrap_or(first);
    Some(Span::new(first.start, last.end))
}

/// The spans of a parameter list, a rest parameter included.
pub(super) fn param_spans<'p>(
    params: &'p oxc_ast::ast::FormalParameters<'_>,
) -> impl Iterator<Item = Span> + 'p {
    params
        .items
        .iter()
        .map(GetSpan::span)
        .chain(params.rest.iter().map(|rest| rest.span))
}

#[derive(Default)]
pub(super) struct Collect {
    names: FxHashSet<CompactString>,
    pub(super) unknown: bool,
    pub(super) entries: Vec<MacroEntry>,
    /// The block the next entries are declared in.
    pub(super) sibling: bool,
}

impl Collect {
    pub(super) fn push(&mut self, name: Option<CompactString>, span: Span) {
        self.push_payload(name, span, None);
    }

    /// [`Self::push`] with the range of the payload values after the first.
    pub(super) fn push_payload(
        &mut self,
        name: Option<CompactString>,
        span: Span,
        extra_payload: Option<Span>,
    ) {
        match name {
            Some(name) => {
                self.entries.push(MacroEntry {
                    name: name.clone(),
                    span,
                    sibling: self.sibling,
                    extra_payload,
                });
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
    define_value_into(expression, &mut collect).then(|| collect.finish())
}

/// [`define_value`] into `collect`; `false` when not an object or array literal.
pub(super) fn define_value_into(expression: &Expression<'_>, collect: &mut Collect) -> bool {
    match expression.without_parentheses() {
        Expression::ObjectExpression(object) => {
            for property in &object.properties {
                let (name, extra_payload) = match property {
                    ObjectPropertyKind::ObjectProperty(property) => (
                        property_name(&property.key, property.computed),
                        validator_payload(&property.value),
                    ),
                    ObjectPropertyKind::SpreadProperty(_) => (None, None),
                };
                collect.push_payload(name, property.span(), extra_payload);
            }
        }
        Expression::ArrayExpression(array) => {
            for element in &array.elements {
                match element {
                    ArrayExpressionElement::Elision(_) => {}
                    element => collect.push(
                        element.as_expression().and_then(literal_value),
                        element.span(),
                    ),
                }
            }
        }
        _ => return false,
    }
    true
}

/// A validator's parameters after the first: `(x, y) => true` declares `y`.
fn validator_payload(value: &Expression<'_>) -> Option<Span> {
    let params = match value.without_parentheses() {
        Expression::ArrowFunctionExpression(function) => &function.params,
        Expression::FunctionExpression(function) => &function.params,
        _ => return None,
    };
    tail_span(param_spans(params), 1)
}

/// Every named event a top-level `defineEmits` call declares, with its node.
///
/// Shared by rules that report at a declaration. An entry that cannot be named
/// is left out, as eslint-plugin-vue's `{ emitName: null }` entries are. The
/// flag is set when a type was left unresolved, so a caller can retry with a
/// cross-file world.
pub(crate) fn define_emits_entries<'a>(
    call: &CallExpression<'a>,
    semantic: &oxc_semantic::Semantic<'a>,
    program: &'a oxc_ast::ast::Program<'a>,
    sibling: Option<&oxc_ast::ast::Program<'_>>,
    source: &str,
    resolved_type_world: Option<&vize_croquis::types::ResolvedTypeWorld>,
) -> (Vec<MacroEntry>, bool) {
    let resolver = Resolver {
        semantic,
        program,
        sibling,
        source,
        resolved_type_world,
    };
    let mut collect = Collect::default();
    if let Some(argument) = call.arguments.first().and_then(Argument::as_expression) {
        resolver.object_or_array_into(argument, Block::Own, 0, &mut collect);
    } else if let Some(declared) = call
        .type_arguments
        .as_ref()
        .and_then(|arguments| arguments.params.first())
    {
        collect = resolver.type_collect(declared);
    }
    (collect.entries, collect.unknown)
}

/// The named events of a Vue object's literal `emits` option, with their nodes.
pub(crate) fn options_emits_entries(object: &ObjectExpression<'_>) -> Vec<MacroEntry> {
    let mut collect = Collect::default();
    for property in &object.properties {
        if let ObjectPropertyKind::ObjectProperty(property) = property
            && property_name(&property.key, property.computed).as_deref() == Some("emits")
        {
            define_value_into(&property.value, &mut collect);
        }
    }
    collect.entries
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
