//! script/no-unused-emit-declarations
//!
//! Flag declared events that are never emitted in `<script setup>`.
//!
//! `defineEmits` declares the events a component can emit, as an array
//! (`defineEmits(['change'])`), a runtime object
//! (`defineEmits({ change: null })`), or a type literal
//! (`defineEmits<{ change: [id: number] }>()` / the call-signature form
//! `defineEmits<{ (e: 'change', id: number): void }>()`). A declared event that
//! is never emitted is dead surface area: a parent listening for it will never
//! be called, which usually means the emit was renamed or removed but its
//! declaration left behind.
//!
//! This is intentionally pragmatic. It tracks the captured emit function
//! (`const emit = defineEmits(...)`) and the string-literal event names passed
//! as its first argument (`emit('change')`). An event declared but never emitted
//! through that function is reported. When the `defineEmits` return value is not
//! assigned to a binding (e.g. a bare `defineEmits([...])`), emits cannot be
//! tracked, so nothing is reported.
//!
//! Port of [`vue/no-unused-emit-declarations`](https://eslint.vuejs.org/rules/no-unused-emit-declarations.html).
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! const emit = defineEmits(['change', 'unused'])
//! emit('change')
//! // `unused` is never emitted
//! ```
//!
//! ### Valid
//! ```ts
//! const emit = defineEmits(['change'])
//! emit('change')
//! ```

use oxc_ast::ast::{
    Argument, ArrayExpressionElement, BindingPattern, CallExpression, Expression,
    ObjectPropertyKind, Program, PropertyKey, Statement, TSLiteral, TSSignature, TSType,
};
use oxc_ast_visit::{Visit, walk::walk_call_expression};
use oxc_span::{GetSpan, Span};

use vize_carton::{CompactString, FxHashSet};

use super::super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/no-unused-emit-declarations",
    description: "Flag declared events that are never emitted",
    default_severity: Severity::Warning,
};

/// Flag declared events that are never emitted.
pub struct NoUnusedEmitDeclarations;

impl ScriptRule for NoUnusedEmitDeclarations {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &META
    }

    #[inline]
    fn uses_ast(&self) -> bool {
        true
    }

    fn check_program<'a>(
        &self,
        program: &'a Program<'a>,
        _source: &str,
        offset: usize,
        result: &mut ScriptLintResult,
    ) {
        // Resolve the captured emit function and its declared events. Without an
        // identifier binding we cannot track usage, so we bail.
        let Some(declaration) = find_emit_declaration(program) else {
            return;
        };
        if declaration.events.is_empty() {
            return;
        }

        // Collect every event name emitted through the captured function.
        let mut used = FxHashSet::default();
        let mut collector = EmittedCollector {
            emit_name: declaration.binding,
            used: &mut used,
        };
        collector.visit_program(program);

        for event in &declaration.events {
            if !used.contains(event.name.as_str()) {
                report(&event.name, event.span, offset, result);
            }
        }
    }
}

/// A single declared event name with the span to report it at.
struct DeclaredEvent {
    name: CompactString,
    span: Span,
}

/// The resolved `const <binding> = defineEmits(...)` declaration.
struct EmitDeclaration<'a> {
    binding: &'a str,
    events: Vec<DeclaredEvent>,
}

/// Find the first top-level `const <id> = defineEmits(...)` and collect the
/// declared event names. Returns `None` when there is no such assigned
/// `defineEmits` call (an unassigned call cannot be tracked).
fn find_emit_declaration<'a>(program: &'a Program<'a>) -> Option<EmitDeclaration<'a>> {
    for statement in &program.body {
        let Statement::VariableDeclaration(declaration) = statement else {
            continue;
        };
        for declarator in &declaration.declarations {
            let BindingPattern::BindingIdentifier(id) = &declarator.id else {
                continue;
            };
            let Some(call) = declarator.init.as_ref().and_then(unwrap_call) else {
                continue;
            };
            if !is_define_emits(call) {
                continue;
            }
            let events = collect_declared_events(call);
            return Some(EmitDeclaration {
                binding: id.name.as_str(),
                events,
            });
        }
    }
    None
}

/// Collect declared event names from a `defineEmits(...)` call: its type
/// literal (property or call signatures) or its runtime array/object argument.
fn collect_declared_events(call: &CallExpression<'_>) -> Vec<DeclaredEvent> {
    let mut events = Vec::new();

    if let Some(type_arguments) = &call.type_arguments {
        for type_param in &type_arguments.params {
            if let TSType::TSTypeLiteral(literal) = type_param {
                for member in &literal.members {
                    collect_event_from_signature(member, &mut events);
                }
            }
        }
        return events;
    }

    match call.arguments.first() {
        Some(Argument::ArrayExpression(array)) => {
            for element in &array.elements {
                if let ArrayExpressionElement::StringLiteral(literal) = element {
                    events.push(DeclaredEvent {
                        name: CompactString::new(literal.value.as_str()),
                        span: literal.span,
                    });
                }
            }
        }
        Some(Argument::ObjectExpression(object)) => {
            for property in &object.properties {
                if let ObjectPropertyKind::ObjectProperty(property) = property
                    && !property.computed
                    && let Some(name) = property_key_name(&property.key)
                {
                    events.push(DeclaredEvent {
                        name: CompactString::new(name),
                        span: property.key.span(),
                    });
                }
            }
        }
        _ => {}
    }

    events
}

/// Extract an event name from a type-literal member: a property signature
/// (`change: [...]`) or a call signature (`(e: 'change', ...): void`).
fn collect_event_from_signature(member: &TSSignature<'_>, events: &mut Vec<DeclaredEvent>) {
    match member {
        TSSignature::TSPropertySignature(property) => {
            if let Some(name) = property_key_name(&property.key) {
                events.push(DeclaredEvent {
                    name: CompactString::new(name),
                    span: property.key.span(),
                });
            }
        }
        TSSignature::TSMethodSignature(method) => {
            if let Some(name) = property_key_name(&method.key) {
                events.push(DeclaredEvent {
                    name: CompactString::new(name),
                    span: method.key.span(),
                });
            }
        }
        TSSignature::TSCallSignatureDeclaration(call_signature) => {
            if let Some(first) = call_signature.params.items.first()
                && let Some(annotation) = &first.type_annotation
                && let TSType::TSLiteralType(literal_type) = &annotation.type_annotation
                && let TSLiteral::StringLiteral(string) = &literal_type.literal
            {
                events.push(DeclaredEvent {
                    name: CompactString::new(string.value.as_str()),
                    span: string.span,
                });
            }
        }
        _ => {}
    }
}

/// Walks the program collecting the first string-literal argument of every call
/// to the captured emit function (`emit('change')`).
struct EmittedCollector<'a, 'set> {
    emit_name: &'a str,
    used: &'set mut FxHashSet<CompactString>,
}

impl<'a> Visit<'a> for EmittedCollector<'_, '_> {
    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        if let Expression::Identifier(callee) = &it.callee
            && callee.name.as_str() == self.emit_name
            && let Some(Argument::StringLiteral(literal)) = it.arguments.first()
        {
            self.used.insert(CompactString::new(literal.value.as_str()));
        }
        walk_call_expression(self, it);
    }
}

/// Whether the callee is the bare `defineEmits` compiler macro.
fn is_define_emits(call: &CallExpression<'_>) -> bool {
    matches!(
        &call.callee,
        Expression::Identifier(identifier) if identifier.name.as_str() == "defineEmits"
    )
}

fn unwrap_call<'a, 'b>(expression: &'b Expression<'a>) -> Option<&'b CallExpression<'a>> {
    match expression {
        Expression::CallExpression(call) => Some(call),
        Expression::ParenthesizedExpression(paren) => unwrap_call(&paren.expression),
        Expression::TSAsExpression(ts) => unwrap_call(&ts.expression),
        Expression::TSSatisfiesExpression(ts) => unwrap_call(&ts.expression),
        Expression::TSNonNullExpression(ts) => unwrap_call(&ts.expression),
        _ => None,
    }
}

fn report(name: &str, span: Span, offset: usize, result: &mut ScriptLintResult) {
    let start = offset as u32 + span.start;
    let end = offset as u32 + span.end;

    let mut message = CompactString::with_capacity(name.len() + 40);
    message.push_str("The '");
    message.push_str(name);
    message.push_str("' event is declared but never emitted.");

    let diagnostic = LintDiagnostic::warn(META.name, message, start, end)
        .with_label("declared but never emitted", start, end)
        .with_help(
            "Emit this event via the captured emit function, or remove it from the defineEmits \
             declaration.",
        );
    result.add_diagnostic(diagnostic);
}

fn property_key_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.as_str()),
        PropertyKey::StringLiteral(string) => Some(string.value.as_str()),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
