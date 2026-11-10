//! script/custom-event-name-casing
//!
//! Enforce configured casing for emitted custom event names.
//!
//! eslint-plugin-vue recommends a consistent casing for the events a component
//! emits. For Vue 3 the default is **camelCase**, so an emit whose event name is
//! kebab-case (`'my-event'`) or PascalCase (`'MyEvent'`) is inconsistent with the
//! recommended casing and is reported.
//!
//! This walks the emit call sites and checks the string-literal event name (the
//! first argument). Only string-literal event names are checked; a dynamic name
//! (`emit(eventName)`) carries no literal to inspect and is skipped. The
//! `update:` prefix used by `v-model` is reserved: its suffix mirrors an
//! authored prop name and is therefore not checked against event-name casing.
//! Explicit configuration can instead require unseparated lowercase names such
//! as `transitionstart`, or kebab-case names such as `transition-start`.
//!
//! ## Script call sites
//!
//! A call to the captured `defineEmits` binding (`const emit = defineEmits(...)`
//! then `emit('my-event')`), or a member call whose property is `emit`/`$emit`
//! (`ctx.emit('my-event')`, `this.$emit('my-event')`).
//!
//! ## Template call sites
//!
//! A template dispatches the same events, through the built-in `$emit` helper
//! (`@click="$emit('foo-bar')"`) or through the captured binding, which is
//! template-visible as a top-level `<script setup>` binding. Recovering them
//! *creates* findings from template evidence, so they come from the template
//! AST and an oxc parse of each expression; see [`super::template_scan`] for the
//! over-match analysis and the shadowing rules.
//!
//! The template half additionally requires the SFC to have a single script
//! block. The rule is invoked once per block, and unlike the script half — whose
//! call sites live in the block being linted — a template `$emit` is visible to
//! every block, so two blocks would report it twice.
//!
//! Mirrors [`vue/custom-event-name-casing`](https://eslint.vuejs.org/rules/custom-event-name-casing.html)
//! with the Vue 3 default (`camelCase`).
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! const emit = defineEmits(['my-event'])
//! emit('my-event')         // kebab-case → report
//! ```
//!
//! ### Valid
//! ```ts
//! const emit = defineEmits(['myEvent'])
//! emit('myEvent')
//! ```

use super::template_scan::{DOLLAR_EMIT, for_each_template_call};
use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta, SfcScriptContext};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{
    Argument, BindingPattern, CallExpression, Expression, IdentifierReference, ObjectExpression,
    Program, Statement,
};
use oxc_ast_visit::{
    Visit,
    walk::{walk_call_expression, walk_object_expression},
};
use oxc_semantic::{Semantic, SemanticBuilder};
use oxc_span::Span;
use oxc_syntax::symbol::SymbolId;
use vize_l0::CompactString;

use super::props_emits::require_explicit_emits::setup_context;
use super::vue_object::VueObjects;

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/custom-event-name-casing",
    description: "Enforce configured casing for emitted custom event names",
    default_severity: Severity::Error,
};

/// Enforce configured casing for emitted custom event names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EventNameCasing {
    #[default]
    /// Require `camelCase` event names.
    CamelCase,
    /// Require entirely lowercase event names.
    Lowercase,
    /// Require `kebab-case` event names.
    KebabCase,
}

/// Enforce configured casing for emitted custom event names.
pub struct CustomEventNameCasing {
    casing: EventNameCasing,
}

impl CustomEventNameCasing {
    /// Create the rule with the required event-name casing.
    pub const fn new(casing: EventNameCasing) -> Self {
        Self { casing }
    }
}

impl Default for CustomEventNameCasing {
    fn default() -> Self {
        Self::new(EventNameCasing::CamelCase)
    }
}

impl ScriptRule for CustomEventNameCasing {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &META
    }

    #[inline]
    fn uses_ast(&self) -> bool {
        true
    }

    #[inline]
    fn uses_template_ast(&self) -> bool {
        true
    }

    #[inline]
    fn check_program<'a>(
        &self,
        program: &'a Program<'a>,
        source: &str,
        offset: usize,
        result: &mut ScriptLintResult,
    ) {
        // Keep the parse-owning `check` path functional: without SFC context
        // only the script call sites are observable.
        self.check_program_with_sfc(program, source, offset, SfcScriptContext::default(), result);
    }

    fn check_program_with_sfc<'a>(
        &self,
        program: &'a Program<'a>,
        _source: &str,
        offset: usize,
        sfc: SfcScriptContext<'_>,
        result: &mut ScriptLintResult,
    ) {
        // The `<script setup>` emit function captured from `defineEmits(...)`, if
        // any. A bare `defineEmits([...])` without a binding cannot be tracked.
        let built = SemanticBuilder::new().with_build_nodes(true).build(program);
        let semantic = &built.semantic;
        let emit_binding = (!sfc.is_sfc || sfc.is_script_setup)
            .then(|| find_emit_binding(program))
            .flatten();
        let objects = VueObjects::collect(
            program,
            _source,
            semantic,
            sfc.is_sfc && !sfc.is_script_setup,
        );

        let mut visitor = CustomEventNameCasingVisitor {
            offset,
            result,
            semantic,
            objects: &objects,
            emit_binding: emit_binding.map(|(_, symbol)| symbol),
            setup_stack: Vec::new(),
            casing: self.casing,
            is_sfc: sfc.is_sfc,
        };
        visitor.visit_program(program);

        check_template(emit_binding.map(|(name, _)| name), self.casing, sfc, result);
    }

    fn check_scriptless_sfc(&self, sfc: SfcScriptContext<'_>, result: &mut ScriptLintResult) {
        check_template(None, self.casing, sfc, result);
    }

    #[inline]
    fn runs_on_scriptless_sfc(&self) -> bool {
        true
    }
}

/// The template half: `$emit('foo-bar')`, or a call of the captured binding.
fn check_template(
    emit_binding: Option<&str>,
    casing: EventNameCasing,
    sfc: SfcScriptContext<'_>,
    result: &mut ScriptLintResult,
) {
    if !sfc.sole_script_block {
        return;
    }
    let Some((root, template_offset)) = sfc.template_ast() else {
        return;
    };
    for_each_template_call(root, |call| {
        if call.callee != DOLLAR_EMIT && Some(call.callee) != emit_binding {
            return;
        }
        let Some(event) = call.first_string_argument else {
            return;
        };
        if is_valid_event_name(event.value, casing) {
            return;
        }
        report(
            event.value,
            casing,
            template_offset + event.start,
            template_offset + event.end,
            result,
        );
    });
}

struct CustomEventNameCasingVisitor<'a, 'result> {
    offset: usize,
    result: &'result mut ScriptLintResult,
    semantic: &'result Semantic<'a>,
    objects: &'result VueObjects,
    emit_binding: Option<SymbolId>,
    setup_stack: Vec<(Option<SymbolId>, Option<SymbolId>)>,
    casing: EventNameCasing,
    is_sfc: bool,
}

impl<'a> Visit<'a> for CustomEventNameCasingVisitor<'a, '_> {
    fn visit_object_expression(&mut self, it: &ObjectExpression<'a>) {
        if self.objects.kind(it).is_none() {
            walk_object_expression(self, it);
            return;
        }
        self.setup_stack.push(setup_context(it));
        walk_object_expression(self, it);
        self.setup_stack.pop();
    }

    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        if self.is_emit_call(it)
            && let Some((value, span)) = it.arguments.first().and_then(static_event_name)
        {
            self.check_event_name(value, span);
        }
        walk_call_expression(self, it);
    }
}

impl<'a> CustomEventNameCasingVisitor<'a, '_> {
    /// Whether the call is an emit of a custom event: a call to the captured
    /// `defineEmits` binding (`emit(...)`) or a member call whose property is
    /// `emit`/`$emit` (`ctx.emit(...)`, `this.$emit(...)`).
    fn is_emit_call(&self, call: &CallExpression<'a>) -> bool {
        match call.callee.without_parentheses() {
            Expression::Identifier(identifier) => {
                let symbol = self.symbol(identifier);
                symbol.is_some()
                    && (self.emit_binding == symbol
                        || self
                            .setup_stack
                            .last()
                            .is_some_and(|(emit, _)| *emit == symbol))
            }
            expression => self.is_emit_member(expression),
        }
    }

    fn is_emit_member(&self, expression: &Expression<'a>) -> bool {
        let member = match expression {
            Expression::ChainExpression(chain) => chain.expression.as_member_expression(),
            expression => expression.as_member_expression(),
        };
        let Some(member) = member else {
            return false;
        };
        match member.static_property_name() {
            // Matches eslint-plugin-vue's script-wide `$emit` visitor.
            Some("$emit") => self.is_sfc,
            Some("emit") => {
                let context = self.setup_stack.last().and_then(|(_, context)| *context);
                context.is_some()
                    && matches!(
                        member.object().without_parentheses(),
                        Expression::Identifier(identifier) if self.symbol(identifier) == context
                    )
            }
            _ => false,
        }
    }

    fn symbol(&self, identifier: &IdentifierReference<'a>) -> Option<SymbolId> {
        self.semantic
            .scoping()
            .get_reference(identifier.reference_id.get()?)
            .symbol_id()
    }

    fn check_event_name(&mut self, value: &str, span: Span) {
        if is_valid_event_name(value, self.casing) {
            return;
        }
        report(
            value,
            self.casing,
            self.offset as u32 + span.start,
            self.offset as u32 + span.end,
            self.result,
        );
    }
}

fn static_event_name<'a>(argument: &'a Argument<'a>) -> Option<(&'a str, Span)> {
    match argument {
        Argument::StringLiteral(literal) => Some((literal.value.as_str(), literal.span)),
        Argument::TemplateLiteral(literal) if literal.expressions.is_empty() => {
            Some((literal.single_quasi()?.as_str(), literal.span))
        }
        _ => None,
    }
}

fn report(
    value: &str,
    casing: EventNameCasing,
    start: u32,
    end: u32,
    result: &mut ScriptLintResult,
) {
    let casing_name = match casing {
        EventNameCasing::CamelCase => "camelCase",
        EventNameCasing::Lowercase => "lowercase",
        EventNameCasing::KebabCase => "kebab-case",
    };
    let mut message = CompactString::with_capacity(value.len() + 40);
    message.push_str("Custom event name '");
    message.push_str(value);
    message.push_str("' is not ");
    message.push_str(casing_name);
    message.push('.');

    let help = match casing {
        EventNameCasing::CamelCase => {
            "Vue 3 recommends camelCase for emitted event names; rename this event \
             to camelCase (e.g. `myEvent`)."
        }
        EventNameCasing::Lowercase => {
            "Rename this emitted event to lowercase without separators (e.g. `transitionstart`)."
        }
        EventNameCasing::KebabCase => "Rename this emitted event to kebab-case (e.g. `my-event`).",
    };
    let diagnostic = LintDiagnostic::error(META.name, message, start, end)
        .with_label(vize_l0::cstr!("expected {casing_name}"), start, end)
        .with_help(help);
    result.add_diagnostic(diagnostic);
}

/// Validate ordinary events while preserving the authored prop name after `update:`.
fn is_valid_event_name(value: &str, casing: EventNameCasing) -> bool {
    if value
        .strip_prefix("update:")
        .is_some_and(|prop| !prop.is_empty())
    {
        return true;
    }
    match casing {
        EventNameCasing::CamelCase => is_camel_case_event(value),
        EventNameCasing::Lowercase => is_lowercase_event(value),
        EventNameCasing::KebabCase => is_kebab_case_event(value),
    }
}

/// Whether `value` is an acceptable camelCase event name. Each `:`-separated
/// segment must match
/// `^[a-z][a-zA-Z0-9]*$`: a lowercase first character followed by alphanumerics.
fn is_camel_case_event(value: &str) -> bool {
    !value.is_empty() && value.split(':').all(is_camel_case_segment)
}

/// Whether a single segment matches `^[a-z][a-zA-Z0-9]*$`.
fn is_camel_case_segment(segment: &str) -> bool {
    // eslint-plugin-vue splits on `:` and its casing helper accepts empty
    // segments. Event-bus namespaces such as `bv::show::modal` therefore stay
    // outside this rule even though an empty whole event name is rejected by
    // the guard in `is_camel_case_event`.
    if segment.is_empty() {
        return true;
    }
    let mut chars = segment.chars();
    match chars.next() {
        Some(first) if first.is_ascii_lowercase() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric())
}

fn is_lowercase_event(value: &str) -> bool {
    !value.is_empty() && value.split(':').all(is_lowercase_segment)
}

fn is_lowercase_segment(segment: &str) -> bool {
    let mut chars = segment.chars();
    match chars.next() {
        Some(first) if first.is_ascii_lowercase() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
}

fn is_kebab_case_event(value: &str) -> bool {
    !value.is_empty() && value.split(':').all(is_kebab_case_segment)
}

fn is_kebab_case_segment(segment: &str) -> bool {
    let mut bytes = segment.bytes();
    let Some(first) = bytes.next() else {
        return false;
    };
    if !first.is_ascii_lowercase() {
        return false;
    }
    let mut previous_hyphen = false;
    for byte in bytes {
        match byte {
            b'a'..=b'z' | b'0'..=b'9' => previous_hyphen = false,
            b'-' if !previous_hyphen => previous_hyphen = true,
            _ => return false,
        }
    }
    !previous_hyphen
}

/// The identifier a top-level `const <id> = defineEmits(...)` binds the emit
/// function to, if present. An unassigned `defineEmits(...)` returns `None`.
fn find_emit_binding<'a>(program: &'a Program<'a>) -> Option<(&'a str, SymbolId)> {
    for statement in &program.body {
        let Statement::VariableDeclaration(declaration) = statement else {
            continue;
        };
        for declarator in &declaration.declarations {
            let BindingPattern::BindingIdentifier(id) = &declarator.id else {
                continue;
            };
            if declarator
                .init
                .as_ref()
                .is_some_and(|init| is_define_emits_call(init))
            {
                return Some((id.name.as_str(), id.symbol_id.get()?));
            }
        }
    }
    None
}

/// Whether the expression is a `defineEmits(...)` call, unwrapping the TS
/// `as`/`satisfies`/non-null and parenthesized wrappers.
fn is_define_emits_call(expression: &Expression<'_>) -> bool {
    match expression {
        Expression::CallExpression(call) => matches!(
            &call.callee,
            Expression::Identifier(identifier) if identifier.name.as_str() == "defineEmits"
        ),
        Expression::ParenthesizedExpression(paren) => is_define_emits_call(&paren.expression),
        Expression::TSAsExpression(ts) => is_define_emits_call(&ts.expression),
        Expression::TSSatisfiesExpression(ts) => is_define_emits_call(&ts.expression),
        Expression::TSNonNullExpression(ts) => is_define_emits_call(&ts.expression),
        _ => false,
    }
}

#[cfg(test)]
#[expect(clippy::string_slice, reason = "tests assert by panicking")]
mod tests;
