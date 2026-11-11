//! script/require-explicit-emits
//!
//! Require every emitted event to be declared in `defineEmits` / the Options API
//! `emits` option. An undeclared emit is invisible to tooling and (since
//! Vue 3.3) falls through as a native DOM listener on the root element, which is
//! rarely intended. Declaring emits documents the component's event surface.
//!
//! Port of
//! [`vue/require-explicit-emits`](https://eslint.vuejs.org/rules/require-explicit-emits.html),
//! with its scope:
//!
//! * A Vue object (see [`crate::rules::script::vue_object`]) declares what its
//!   `emits` option lists, and nothing when the option is absent. Its
//!   `this.$emit('x')` calls (also through `const vm = this`) and the `emit` of
//!   its `setup(props, { emit })` / `setup(props, ctx)` are checked against it.
//! * `<script setup>` declares what its `defineEmits` macro lists. Script calls
//!   of the macro's binding after the macro are checked; without a macro the
//!   script is not.
//! * The template's `$emit('x')` (and a call of the `defineEmits` binding) is
//!   checked against `<script setup>` when the SFC has one, and otherwise
//!   against the exported (or last defined) component object.
//! * A declaration that cannot be fully enumerated — a spread, a computed key,
//!   an unresolved imported type, an untyped `defineEmits()` — declares every
//!   event. Only literal event names are checked.
//!
//! ```ts
//! const emit = defineEmits(['change'])
//! emit('change') // ok
//! emit('input')  // reported: 'input' is not declared
//! ```

mod calls;
mod declared;
mod types;

use oxc_ast::ast::Program;
use oxc_ast_visit::Visit;
use oxc_semantic::SemanticBuilder;

use vize_l0::{CompactString, FxHashSet};

use self::calls::{EmitCallVisitor, SetupOwner};
pub(crate) use self::calls::{dispatched_events, setup_context};
use self::declared::{Emits, define_emits};
pub(crate) use self::declared::{MacroEntry, define_emits_entries, options_emits_entries};
use self::types::Resolver;
use super::super::setup_macros::{DefineModel, setup_macros};
use super::super::template_scan::{DOLLAR_EMIT, for_each_template_call};
use super::super::vue_object::VueObjects;
use super::super::{ScriptLintResult, ScriptRule, ScriptRuleMeta, SfcScriptContext};
use crate::diagnostic::{LintDiagnostic, Severity};

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/require-explicit-emits",
    description: "Require emitted events to be declared in defineEmits or the emits option",
    default_severity: Severity::Warning,
};

/// Require every emitted event to be declared.
pub struct RequireExplicitEmits;

impl ScriptRule for RequireExplicitEmits {
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

    fn check_program<'a>(
        &self,
        program: &'a Program<'a>,
        source: &str,
        offset: usize,
        result: &mut ScriptLintResult,
    ) {
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
        // `emit` covers `$emit` and `ctx.emit`; `Emits` a renamed macro binding.
        if !source.contains("emit")
            && !source.contains("Emits")
            && !sfc
                .template_source
                .is_some_and(|template| template.contains("emit"))
        {
            return;
        }
        let built = SemanticBuilder::new().with_build_nodes(true).build(program);
        let semantic = &built.semantic;
        let resolver = Resolver {
            semantic,
            program,
            sibling: sfc.sibling_program,
            source,
            resolved_type_world: sfc.resolved_type_world,
        };
        let setup = sfc
            .is_script_setup
            .then(|| define_emits(&resolver))
            .flatten();
        // A `defineModel()` declares its `update:<name>` event (upstream 10.12.1).
        let models = if sfc.is_script_setup {
            model_events(program)
        } else {
            Vec::new()
        };
        let objects = VueObjects::collect(
            program,
            source,
            semantic,
            sfc.is_sfc && !sfc.is_script_setup,
        );
        let mut visitor = EmitCallVisitor {
            semantic,
            objects: &objects,
            models: &models,
            setup: setup.as_ref().map(|setup| SetupOwner {
                start: setup.start,
                emits: &setup.emits,
                emit: setup.binding.and_then(|(_, symbol)| symbol),
            }),
            stack: Vec::new(),
            component: None,
            offset: offset as u32,
            result,
            dispatched: None,
        };
        visitor.visit_program(program);
        let component = visitor.component.take();

        // The template sees `<script setup>` when there is one, and otherwise
        // the component of the only script block.
        if sfc.is_script_setup {
            let empty = Emits::Known(FxHashSet::default());
            let emits = setup.as_ref().map_or(&empty, |setup| &setup.emits);
            let binding = setup
                .as_ref()
                .and_then(|setup| setup.binding)
                .map(|(name, _)| name);
            check_template(emits, &models, binding, sfc, result);
        } else if sfc.sole_script_block
            && let Some((_, emits)) = component.as_ref()
        {
            check_template(emits, &[], None, sfc, result);
        }
    }
}

/// The template half: `$emit('x')` — or a call of the `defineEmits` binding —
/// naming an event the declaration does not contain.
fn check_template(
    declared: &Emits,
    models: &[(CompactString, u32)],
    binding: Option<&str>,
    sfc: SfcScriptContext<'_>,
    result: &mut ScriptLintResult,
) {
    let Some((root, template_offset)) = sfc.template_ast() else {
        return;
    };
    for_each_template_call(root, |call| {
        if call.callee != DOLLAR_EMIT && Some(call.callee) != binding {
            return;
        }
        let Some(event) = call.first_string_argument else {
            return;
        };
        if !declared.declares(event.value) && !models.iter().any(|(name, _)| name == event.value) {
            report(
                event.value,
                template_offset + event.start,
                template_offset + event.end,
                result,
            );
        }
    });
}

/// The `update:<name>` event of each `defineModel()` macro and its start.
fn model_events(program: &Program<'_>) -> Vec<(CompactString, u32)> {
    setup_macros(program)
        .into_iter()
        .filter(|found| found.name == "defineModel")
        .map(|found| {
            let mut event = CompactString::new("update:");
            event.push_str(&DefineModel::of(found.call).name);
            (event, found.call.span.start)
        })
        .collect()
}

fn report(name: &str, start: u32, end: u32, result: &mut ScriptLintResult) {
    let mut message = CompactString::with_capacity(name.len() + 48);
    message.push_str("The emitted event '");
    message.push_str(name);
    message.push_str("' is not declared in defineEmits or the emits option.");

    let diagnostic = LintDiagnostic::warn(META.name, message, start, end)
        .with_label("undeclared emitted event", start, end)
        .with_help(
            "Add this event to the defineEmits declaration (or the Options API `emits` \
             option) so the component's event surface is explicit.",
        );
    result.add_diagnostic(diagnostic);
}

#[cfg(test)]
#[expect(clippy::string_slice, reason = "tests assert by panicking")]
mod tests;
