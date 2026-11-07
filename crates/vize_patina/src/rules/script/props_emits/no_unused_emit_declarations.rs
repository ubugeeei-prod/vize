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

fn check_options_api<'a>(
    program: &'a Program<'a>,
    source: &str,
    offset: usize,
    sfc: SfcScriptContext<'_>,
    result: &mut ScriptLintResult,
) {
    let built = SemanticBuilder::new().with_build_nodes(true).build(program);
    let objects = VueObjects::collect(program, source, &built.semantic, sfc.is_sfc);
    let mut template_used = FxHashSet::default();
    if let Some(template) = sfc.template_source {
        collect_template_emitted_events(template, "", &mut template_used);
    }
    let mut visitor = OptionsUnusedCollector {
        semantic: &built.semantic,
        objects: &objects,
        frames: Vec::new(),
        template_used: &template_used,
        offset,
        result,
    };
    visitor.visit_program(program);
}

struct OptionsFrame {
    events: Vec<DeclaredEvent>,
    used: FxHashSet<CompactString>,
    emit: Option<SymbolId>,
    context: Option<SymbolId>,
    escaped: bool,
}

struct OptionsUnusedCollector<'o, 'r, 'a> {
    semantic: &'o Semantic<'a>,
    objects: &'o VueObjects,
    frames: Vec<OptionsFrame>,
    template_used: &'o FxHashSet<CompactString>,
    offset: usize,
    result: &'r mut ScriptLintResult,
}

impl<'a> Visit<'a> for OptionsUnusedCollector<'_, '_, 'a> {
    fn visit_object_expression(&mut self, object: &ObjectExpression<'a>) {
        let events = self
            .objects
            .kind(object)
            .and_then(|_| collect_options_declared_events(object));
        let pushed = events.is_some();
        if let Some(events) = events {
            let (emit, context) = setup_context(object);
            self.frames.push(OptionsFrame {
                events,
                used: self.template_used.clone(),
                emit,
                context,
                escaped: false,
            });
        }
        walk_object_expression(self, object);
        if pushed && let Some(frame) = self.frames.pop() {
            for event in frame.events {
                if !frame.escaped && !frame.used.contains(event.name.as_str()) {
                    report(&event.name, event.span, self.offset, self.result);
                }
            }
        }
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if self.dispatches(&call.callee)
            && let Some(Argument::StringLiteral(event)) = call.arguments.first()
            && let Some(frame) = self.frames.last_mut()
        {
            frame.used.insert(CompactString::new(event.value.as_str()));
        }
        let escaped = self.frames.last().is_some_and(|frame| {
            [frame.emit, frame.context]
                .into_iter()
                .flatten()
                .any(|symbol| {
                    call.arguments
                        .iter()
                        .any(|argument| self.argument_references_symbol(argument, symbol))
                })
        });
        if escaped && let Some(frame) = self.frames.last_mut() {
            frame.escaped = true;
        }
        walk_call_expression(self, call);
    }
}

impl<'a> OptionsUnusedCollector<'_, '_, 'a> {
    fn dispatches(&self, callee: &Expression<'a>) -> bool {
        let Some(frame) = self.frames.last() else {
            return false;
        };
        match callee.without_parentheses() {
            Expression::Identifier(identifier) => {
                frame.emit.is_some() && self.symbol(identifier) == frame.emit
            }
            expression => {
                let Some(member) = expression.as_member_expression() else {
                    return false;
                };
                match member.static_property_name() {
                    Some("emit") => {
                        frame.context.is_some()
                            && matches!(
                                member.object().without_parentheses(),
                                Expression::Identifier(identifier)
                                    if self.symbol(identifier) == frame.context
                            )
                    }
                    Some("$emit") => matches!(
                        member.object().without_parentheses(),
                        Expression::ThisExpression(_)
                    ),
                    _ => false,
                }
            }
        }
    }

    fn symbol(&self, identifier: &IdentifierReference<'a>) -> Option<SymbolId> {
        self.semantic
            .scoping()
            .get_reference(identifier.reference_id.get()?)
            .symbol_id()
    }

    fn argument_references_symbol(&self, argument: &Argument<'a>, symbol: SymbolId) -> bool {
        match argument {
            Argument::ObjectExpression(object) => object.properties.iter().any(|property| {
                matches!(property, ObjectPropertyKind::ObjectProperty(property)
                    if self.expression_is_symbol(&property.value, symbol))
            }),
            Argument::ArrayExpression(array) => array.elements.iter().any(|element| {
                element
                    .as_expression()
                    .is_some_and(|expression| self.expression_is_symbol(expression, symbol))
            }),
            _ => argument
                .as_expression()
                .is_some_and(|expression| self.expression_is_symbol(expression, symbol)),
        }
    }

    fn expression_is_symbol(&self, expression: &Expression<'a>, symbol: SymbolId) -> bool {
        match expression.without_parentheses() {
            Expression::Identifier(identifier) => self.symbol(identifier) == Some(symbol),
            expression => expression.as_member_expression().is_some_and(|member| {
                matches!(member.object().without_parentheses(), Expression::Identifier(identifier)
                    if self.symbol(identifier) == Some(symbol))
            }),
        }
    }
}

fn collect_options_declared_events(object: &ObjectExpression<'_>) -> Option<Vec<DeclaredEvent>> {
    let value = object.properties.iter().find_map(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return None;
        };
        (!property.computed && property_key_name(&property.key) == Some("emits"))
            .then_some(&property.value)
    })?;
    let mut events = Vec::new();
    match value.without_parentheses() {
        Expression::ArrayExpression(array) => {
            for element in &array.elements {
                let ArrayExpressionElement::StringLiteral(literal) = element else {
                    // Keep independently known literal declarations even when
                    // the array also contains a dynamic event name. The
                    // unknown entry cannot make a different literal event used.
                    continue;
                };
                events.push(DeclaredEvent {
                    name: CompactString::new(literal.value.as_str()),
                    span: literal.span,
                    origin_offset: None,
                });
            }
        }
        Expression::ObjectExpression(object) => {
            for property in &object.properties {
                let ObjectPropertyKind::ObjectProperty(property) = property else {
                    continue;
                };
                if property.computed {
                    continue;
                }
                let Some(name) = property_key_name(&property.key) else {
                    continue;
                };
                events.push(DeclaredEvent {
                    name: CompactString::new(name),
                    span: property.key.span(),
                    origin_offset: None,
                });
            }
        }
        _ => return None,
    }
    Some(events)
}

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

/// Resolve local type literals the same way vue-eslint-parser's merged SFC
/// program does. Imported types stay unknown, but a type/interface declared in
/// either script block retains the authored member span for the diagnostic.
struct TypeEventResolver<'a, 'b> {
    own: &'a Program<'a>,
    sibling: Option<&'b Program<'b>>,
    sibling_offset: Option<usize>,
}

impl TypeEventResolver<'_, '_> {
    fn collect(
        &self,
        ty: &TSType<'_>,
        block: TypeBlock,
        seen: &mut Vec<(TypeBlock, Span)>,
        events: &mut Vec<DeclaredEvent>,
    ) {
        match ty {
            TSType::TSUnionType(union) => {
                for ty in &union.types {
                    self.collect(ty, block, seen, events);
                }
            }
            TSType::TSIntersectionType(intersection) => {
                for ty in &intersection.types {
                    self.collect(ty, block, seen, events);
                }
            }
            TSType::TSParenthesizedType(parenthesized) => {
                self.collect(&parenthesized.type_annotation, block, seen, events);
            }
            TSType::TSTypeLiteral(literal) => {
                self.collect_members(&literal.members, block, events);
            }
            TSType::TSFunctionType(function) if function.this_param.is_none() => {
                collect_event_from_parameters(
                    &function.params,
                    function.span,
                    self.origin(block),
                    events,
                );
            }
            TSType::TSTypeReference(reference) => {
                let TSTypeName::IdentifierReference(identifier) = &reference.type_name else {
                    return;
                };
                let Some((declaration, declaration_block)) =
                    self.declaration(identifier.name.as_str(), block)
                else {
                    return;
                };
                let span = match declaration {
                    LocalType::Interface(interface) => interface.span,
                    LocalType::Alias(alias) => alias.span,
                };
                let key = (declaration_block, span);
                if seen.contains(&key) {
                    return;
                }
                seen.push(key);
                match declaration {
                    LocalType::Interface(interface) => {
                        self.collect_members(&interface.body.body, declaration_block, events);
                    }
                    LocalType::Alias(alias) => {
                        self.collect(&alias.type_annotation, declaration_block, seen, events);
                    }
                }
                seen.pop();
            }
            _ => {}
        }
    }

    fn collect_members(
        &self,
        members: &[TSSignature<'_>],
        block: TypeBlock,
        events: &mut Vec<DeclaredEvent>,
    ) {
        for member in members {
            collect_event_from_signature(member, self.origin(block), events);
        }
    }

    fn origin(&self, block: TypeBlock) -> Option<usize> {
        match block {
            TypeBlock::Own => None,
            TypeBlock::Sibling => self.sibling_offset,
        }
    }

    fn declaration(&self, name: &str, block: TypeBlock) -> Option<(LocalType<'_>, TypeBlock)> {
        match block {
            TypeBlock::Own => top_level_type(self.own, name)
                .map(|declaration| (declaration, TypeBlock::Own))
                .or_else(|| {
                    top_level_type(self.sibling?, name)
                        .map(|declaration| (declaration, TypeBlock::Sibling))
                }),
            TypeBlock::Sibling => self
                .sibling
                .and_then(|program| top_level_type(program, name))
                .map(|declaration| (declaration, TypeBlock::Sibling))
                .or_else(|| {
                    top_level_type(self.own, name).map(|declaration| (declaration, TypeBlock::Own))
                }),
        }
    }
}

fn top_level_type<'a>(program: &'a Program<'a>, name: &str) -> Option<LocalType<'a>> {
    let mut found = None;
    for statement in &program.body {
        let declaration = match statement {
            Statement::TSInterfaceDeclaration(interface) => LocalType::Interface(interface),
            Statement::TSTypeAliasDeclaration(alias) => LocalType::Alias(alias),
            Statement::ExportNamedDeclaration(export) => match &export.declaration {
                Some(Declaration::TSInterfaceDeclaration(interface)) => {
                    LocalType::Interface(interface)
                }
                Some(Declaration::TSTypeAliasDeclaration(alias)) => LocalType::Alias(alias),
                _ => continue,
            },
            _ => continue,
        };
        let declared_name = match declaration {
            LocalType::Interface(interface) => interface.id.name.as_str(),
            LocalType::Alias(alias) => alias.id.name.as_str(),
        };
        if declared_name == name {
            if found.is_some() {
                return None;
            }
            found = Some(declaration);
        }
    }
    found
}

fn collect_event_from_parameters(
    params: &oxc_ast::ast::FormalParameters<'_>,
    span: Span,
    origin_offset: Option<usize>,
    events: &mut Vec<DeclaredEvent>,
) {
    if let Some(first) = params.items.first()
        && let Some(annotation) = &first.type_annotation
    {
        collect_event_names_from_type(&annotation.type_annotation, span, origin_offset, events);
    }
}

fn collect_event_names_from_type(
    ty: &TSType<'_>,
    span: Span,
    origin_offset: Option<usize>,
    events: &mut Vec<DeclaredEvent>,
) {
    match ty {
        TSType::TSLiteralType(literal_type) => {
            if let TSLiteral::StringLiteral(string) = &literal_type.literal {
                events.push(DeclaredEvent {
                    name: CompactString::new(string.value.as_str()),
                    span,
                    origin_offset,
                });
            }
        }
        TSType::TSUnionType(union) => {
            for ty in &union.types {
                collect_event_names_from_type(ty, span, origin_offset, events);
            }
        }
        TSType::TSParenthesizedType(parenthesized) => collect_event_names_from_type(
            &parenthesized.type_annotation,
            span,
            origin_offset,
            events,
        ),
        _ => {}
    }
}

/// Extract an event name from a type-literal member: a property signature
/// (`change: [...]`) or a call signature (`(e: 'change', ...): void`).
fn collect_event_from_signature(
    member: &TSSignature<'_>,
    origin_offset: Option<usize>,
    events: &mut Vec<DeclaredEvent>,
) {
    match member {
        TSSignature::TSPropertySignature(property) => {
            if let Some(name) = property_key_name(&property.key) {
                events.push(DeclaredEvent {
                    name: CompactString::new(name),
                    span: property.span,
                    origin_offset,
                });
            }
        }
        TSSignature::TSMethodSignature(method) => {
            if let Some(name) = property_key_name(&method.key) {
                events.push(DeclaredEvent {
                    name: CompactString::new(name),
                    span: method.span,
                    origin_offset,
                });
            }
        }
        TSSignature::TSCallSignatureDeclaration(call_signature) => {
            collect_event_from_parameters(
                &call_signature.params,
                call_signature.span,
                origin_offset,
                events,
            );
        }
        _ => {}
    }
}

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
