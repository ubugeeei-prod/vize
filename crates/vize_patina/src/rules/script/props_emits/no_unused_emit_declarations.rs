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
//! through that function is reported. Passing the function to another call
//! escapes it to code this rule cannot inspect, so all declarations are then
//! treated as potentially used, matching eslint-plugin-vue. Calls are counted from the macro
//! declaration onward, matching eslint-plugin-vue's source-order visitor. When
//! the `defineEmits` return value is not assigned to a binding (e.g. a bare
//! `defineEmits([...])`), only template `$emit(...)` calls can use declarations.
//!
//! Top-level `<script setup>` bindings are template-visible, so when the block
//! is linted as part of an SFC the same call forms are also resolved from the
//! raw `<template>` source (see [`super::template_emits`]): the captured
//! binding invoked from an event handler (`@click='emit("save")'`, including
//! handlers nested in `v-for` / `v-if`) and the built-in `$emit` helper
//! (`@click="$emit('save')"`) both count as usage (#3209). Dynamic event names
//! stay untracked in the template exactly as in the script.
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
//!
//! ```vue
//! <script setup lang="ts">
//! const emit = defineEmits<{ save: [] }>();
//! </script>
//! <template>
//!   <button @click='emit("save")'>Save</button>
//! </template>
//! ```

use oxc_ast::ast::{
    Argument, ArrayExpressionElement, BindingPattern, CallExpression, Declaration, Expression,
    IdentifierReference, ObjectExpression, ObjectPropertyKind, Program, Statement,
    TSInterfaceDeclaration, TSLiteral, TSSignature, TSType, TSTypeAliasDeclaration, TSTypeName,
};
use oxc_ast_visit::{
    Visit,
    walk::{walk_call_expression, walk_object_expression},
};
use oxc_semantic::{Semantic, SemanticBuilder, SymbolId};
use oxc_span::{GetSpan, Span};

use vize_l0::{CompactString, FxHashSet};

use super::super::vue_object::VueObjects;
use super::super::{ScriptLintResult, ScriptRule, ScriptRuleMeta, SfcScriptContext};
use super::require_explicit_emits::setup_context;
use super::template_emits::collect_template_emitted_events;
use crate::diagnostic::{LintDiagnostic, Severity};

mod keys;

use keys::property_key_name;

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
        source: &str,
        offset: usize,
        result: &mut ScriptLintResult,
    ) {
        // Keep the parse-owning `check` path functional: without SFC context
        // only script-side emit calls are observable.
        self.check_program_with_sfc(program, source, offset, SfcScriptContext::default(), result);
    }

    fn check_program_with_sfc<'a>(
        &self,
        program: &'a Program<'a>,
        source: &str,
        offset: usize,
        sfc: SfcScriptContext<'_>,
        result: &mut ScriptLintResult,
    ) {
        if let Some(declaration) = find_emit_declaration(program, sfc)
            && !declaration.events.is_empty()
        {
            // Collect every event name emitted through the captured function.
            let mut used = FxHashSet::default();
            let mut escaped = false;
            if let Some(binding) = declaration.binding {
                let mut collector = EmittedCollector {
                    emit_name: binding,
                    used: &mut used,
                    after: declaration.start,
                    escaped: false,
                };
                collector.visit_program(program);
                escaped = collector.escaped;
            }
            collect_define_model_events(program, &mut used);

            // Top-level `<script setup>` bindings are template-visible, so emit
            // calls in template expressions count as usage too (#3209).
            if let Some(template) = sfc.template_source {
                collect_template_emitted_events(
                    template,
                    declaration.binding.unwrap_or(""),
                    &mut used,
                );
            }

            for event in &declaration.events {
                if !escaped && !used.contains(event.name.as_str()) {
                    report(
                        &event.name,
                        event.span,
                        event.origin_offset.unwrap_or(offset),
                        result,
                    );
                }
            }
        }

        if !sfc.is_script_setup {
            check_options_api(program, source, offset, sfc, result);
        }
    }
}

include!("no_unused_emit_declarations/options_api.rs");
fn collect_define_model_events(program: &Program<'_>, used: &mut FxHashSet<CompactString>) {
    if super::compiler_macro_is_shadowed(program, "defineModel") {
        return;
    }
    for statement in &program.body {
        let Statement::VariableDeclaration(declaration) = statement else {
            continue;
        };
        for declarator in &declaration.declarations {
            let Some(call) = declarator.init.as_ref().and_then(unwrap_call) else {
                continue;
            };
            if !matches!(&call.callee, Expression::Identifier(identifier) if identifier.name == "defineModel")
            {
                continue;
            }
            let model = match call.arguments.first() {
                Some(Argument::StringLiteral(literal)) => literal.value.as_str(),
                None => "modelValue",
                _ => continue,
            };
            let mut event = CompactString::new("update:");
            event.push_str(model);
            used.insert(event);
        }
    }
}

/// A single declared event name with the span to report it at.
struct DeclaredEvent {
    name: CompactString,
    span: Span,
    /// Sibling `<script>` declarations have a different file origin. `None`
    /// means the block currently being linted.
    origin_offset: Option<usize>,
}

/// The resolved `const <binding> = defineEmits(...)` declaration.
struct EmitDeclaration<'a> {
    binding: Option<&'a str>,
    start: u32,
    events: Vec<DeclaredEvent>,
}

/// Find the first top-level `defineEmits(...)` and collect the declared event
/// names. The captured binding is optional: an unassigned macro cannot be
/// called from script, but its declarations can still be used through the
/// template-only `$emit` helper.
fn find_emit_declaration<'a>(
    program: &'a Program<'a>,
    sfc: SfcScriptContext<'_>,
) -> Option<EmitDeclaration<'a>> {
    for statement in &program.body {
        if let Statement::VariableDeclaration(declaration) = statement {
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
                return Some(EmitDeclaration {
                    binding: Some(id.name.as_str()),
                    start: call.span.start,
                    events: collect_declared_events(call, program, sfc),
                });
            }
        } else if let Statement::ExpressionStatement(expression) = statement
            && let Some(call) = unwrap_call(&expression.expression)
            && is_define_emits(call)
        {
            let events = collect_declared_events(call, program, sfc);
            return Some(EmitDeclaration {
                binding: None,
                start: call.span.start,
                events,
            });
        }
    }
    None
}

/// Collect declared event names from a `defineEmits(...)` call: its type
/// literal (property or call signatures) or its runtime array/object argument.
fn collect_declared_events(
    call: &CallExpression<'_>,
    program: &Program<'_>,
    sfc: SfcScriptContext<'_>,
) -> Vec<DeclaredEvent> {
    let mut events = Vec::new();

    if let Some(type_arguments) = &call.type_arguments {
        let resolver = TypeEventResolver {
            own: program,
            sibling: sfc.sibling_program,
            sibling_offset: sfc.sibling_offset,
        };
        for type_param in &type_arguments.params {
            resolver.collect(type_param, TypeBlock::Own, &mut Vec::new(), &mut events);
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
                        origin_offset: None,
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
                        origin_offset: None,
                    });
                }
            }
        }
        _ => {}
    }

    events
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TypeBlock {
    Own,
    Sibling,
}

#[derive(Clone, Copy)]
enum LocalType<'a> {
    Interface(&'a TSInterfaceDeclaration<'a>),
    Alias(&'a TSTypeAliasDeclaration<'a>),
}

include!("no_unused_emit_declarations/type_events.rs");
/// Walks the program collecting the first string-literal argument of every call
/// to the captured emit function (`emit('change')`) after its declaration.
struct EmittedCollector<'a, 'set> {
    emit_name: &'a str,
    used: &'set mut FxHashSet<CompactString>,
    after: u32,
    escaped: bool,
}

impl<'a> Visit<'a> for EmittedCollector<'_, '_> {
    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        if let Expression::Identifier(callee) = &it.callee
            && callee.name.as_str() == self.emit_name
            && callee.span.start > self.after
            && let Some(Argument::StringLiteral(literal)) = it.arguments.first()
        {
            self.used.insert(CompactString::new(literal.value.as_str()));
        }
        if it.span.start > self.after {
            self.escaped |= it
                .arguments
                .iter()
                .any(|argument| argument_references_emit(argument, self.emit_name));
        }
        walk_call_expression(self, it);
    }
}

fn argument_references_emit(argument: &Argument<'_>, emit_name: &str) -> bool {
    match argument {
        Argument::ObjectExpression(object) => object.properties.iter().any(|property| {
            matches!(property, ObjectPropertyKind::ObjectProperty(property)
                if direct_emit_reference(&property.value, emit_name))
        }),
        Argument::ArrayExpression(array) => array.elements.iter().any(|element| {
            element
                .as_expression()
                .is_some_and(|expression| direct_emit_reference(expression, emit_name))
        }),
        _ => argument
            .as_expression()
            .is_some_and(|expression| direct_emit_reference(expression, emit_name)),
    }
}

fn direct_emit_reference(expression: &Expression<'_>, emit_name: &str) -> bool {
    let expression = expression.without_parentheses();
    if matches!(expression, Expression::Identifier(identifier) if identifier.name == emit_name) {
        return true;
    }
    expression.as_member_expression().is_some_and(|member| {
        matches!(member.object().without_parentheses(), Expression::Identifier(identifier)
            if identifier.name == emit_name)
    })
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

#[cfg(test)]
#[expect(clippy::string_slice, reason = "tests assert by panicking")]
mod tests;
