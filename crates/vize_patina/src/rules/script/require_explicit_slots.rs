//! script/require-explicit-slots
//!
//! Require the slots a component uses to be explicitly typed with
//! `defineSlots<...>()`.
//!
//! When a component reads its slots programmatically — via `useSlots()` — but
//! never declares them with `defineSlots`, the slot names and their prop types
//! are invisible to the type-checker and to consumers of the component. The
//! recommended pattern is to declare slots up front with the type-only
//! `defineSlots<{ ... }>()` macro so the slot contract is explicit and typed.
//!
//! Mirrors [`vue/require-explicit-slots`](https://eslint.vuejs.org/rules/require-explicit-slots.html),
//! which applies to TypeScript only. It has two halves, and both are ported.
//!
//! ## Template slots must have declarations
//!
//! When the block *does* declare slots and the declared set is fully
//! enumerable, a `<slot name="footer" />` rendered by the template but absent
//! from that set is reported at the `<slot>` element.
//!
//! A dynamic `:name` is skipped because it cannot be compared statically, but
//! does not hide other static outlets. A type form that cannot be enumerated
//! contributes no known declarations, matching eslint-plugin-vue.
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! const props = defineProps<{ id: number }>()
//! const slots = useSlots()
//! ```
//!
//! ```vue
//! <script setup lang="ts">
//! defineSlots<{ default(): unknown }>()
//! </script>
//! <template>
//!   <div><slot /><slot name="footer" /></div>
//! </template>
//! ```
//!
//! ### Valid
//! ```ts
//! defineSlots<{ default(props: { msg: string }): unknown }>()
//! const slots = useSlots()
//! ```

mod declared;
mod template;
#[cfg(test)]
mod tests;

use oxc_ast::ast::Program;
use vize_l0::CompactString;

use crate::diagnostic::{LintDiagnostic, Severity};

use self::declared::DeclaredSlots;
use self::template::{RenderedSlot, RenderedSlots};
use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta, SfcScriptContext};

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/require-explicit-slots",
    description: "Require slots consumed via useSlots() to be explicitly typed with defineSlots<...>()",
    default_severity: Severity::Warning,
};

const UNDECLARED_HELP: &str = "Add this slot to the defineSlots type, e.g. \
     `defineSlots<{ footer(): unknown }>()`, or remove the <slot> outlet.";

/// Require `defineSlots<...>()` to cover every slot the component uses.
pub struct RequireExplicitSlots;

impl ScriptRule for RequireExplicitSlots {
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
        _source: &str,
        _offset: usize,
        sfc: SfcScriptContext<'_>,
        result: &mut ScriptLintResult,
    ) {
        // eslint-plugin-vue activates this rule only for an exact `lang="ts"`
        // script block. TSX is deliberately outside its document-fragment
        // gate even though it is otherwise TypeScript.
        if !sfc.is_sfc || !sfc.typescript || sfc.typescript_jsx {
            return;
        }
        // With both script kinds, run once from script setup. A normal-script
        // The regular-script `SlotsType` declaration is handled when it is the
        // sole script block. With both script kinds, run once from setup.
        if !sfc.sole_script_block && !sfc.is_script_setup {
            return;
        }
        let slots = declared::collect(program);
        match &slots.declared {
            DeclaredSlots::Absent => {
                check_template(&vize_l0::FxHashSet::default(), sfc, result);
            }
            // eslint-plugin-vue cannot resolve imported/referenced types here;
            // such a declaration contributes no statically known slot names.
            DeclaredSlots::Unknown => {
                check_template(&vize_l0::FxHashSet::default(), sfc, result);
            }
            DeclaredSlots::Known(names) => {
                check_template(names, sfc, result);
            }
        }
    }
}

/// The template half: every rendered `<slot>` must be in the declared set.
fn check_template(
    declared: &vize_l0::FxHashSet<CompactString>,
    sfc: SfcScriptContext<'_>,
    result: &mut ScriptLintResult,
) {
    let (Some((root, template_offset)), Some(source)) = (sfc.template_ast(), sfc.template_source)
    else {
        return;
    };
    let RenderedSlots::Known(rendered) = template::collect_rendered_slots(root, source);
    for slot in rendered {
        if !declared.contains(&slot.name) {
            report_undeclared_slot(&slot, template_offset, result);
        }
    }
}

fn report_undeclared_slot(
    slot: &RenderedSlot,
    template_offset: u32,
    result: &mut ScriptLintResult,
) {
    let start = template_offset + slot.start;
    let end = template_offset + slot.end;
    let mut message = CompactString::with_capacity(slot.name.len() + 64);
    message.push_str("Slot '");
    message.push_str(&slot.name);
    message.push_str("' is rendered in the template but not declared in defineSlots<...>().");
    result.add_diagnostic(
        LintDiagnostic::warn(META.name, message, start, end)
            .with_label("undeclared slot", start, end)
            .with_help(UNDECLARED_HELP),
    );
}
