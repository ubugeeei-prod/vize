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
//! ## Script half — `useSlots()` without `defineSlots`
//!
//! Fires only when **all** of the following hold for a single `<script setup>`
//! block:
//!
//! * the block contains TypeScript syntax (a type annotation, interface, type
//!   alias, `defineProps<T>()`, etc.). Because a script rule does not receive
//!   the SFC `lang` attribute, the presence of TS syntax is used as a sound
//!   proxy for `lang="ts"`, matching `vue/require-explicit-slots`, which only
//!   runs for TypeScript SFCs. A block with no TS syntax at all is treated as
//!   JavaScript and never flagged.
//! * slots are clearly consumed: there is at least one `useSlots()` call.
//! * there is **no** `defineSlots(...)` / `defineSlots<...>()` declaration in
//!   the same block.
//!
//! The report is anchored at the first `useSlots()` call so the fix (adding a
//! `defineSlots` declaration) is obvious.
//!
//! ## Template half — a `<slot>` the declaration does not cover
//!
//! When the block *does* declare slots and the declared set is fully
//! enumerable, a `<slot name="footer" />` rendered by the template but absent
//! from that set is reported at the `<slot>` element.
//!
//! Both sides are deliberately conservative, in opposite ways:
//!
//! * The declaration must be a single `defineSlots<{ ... }>()` type literal.
//!   Anything the rule cannot enumerate — `defineSlots<Slots>()`, an index
//!   signature, a computed key — reports nothing, because a name missing from a
//!   half-read set is not evidence that it is undeclared. See
//!   [`declared::DeclaredSlots`].
//! * The rendered set must be fully static. One `<slot :name="x" />` anywhere
//!   makes the whole template unreadable for this purpose and the rule reports
//!   nothing for the component. See [`template::RenderedSlots`].
//!
//! Both leave real problems unreported, which is the tolerable direction: this
//! half *creates* findings from template evidence, so an over-match would be a
//! diagnostic on correct code. [`template`] documents the over-match probes.
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
use oxc_span::Span;
use vize_s0::CompactString;

use crate::diagnostic::{LintDiagnostic, Severity};

use self::declared::{DeclaredSlots, ScriptSlots};
use self::template::{RenderedSlot, RenderedSlots};
use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta, SfcScriptContext};

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/require-explicit-slots",
    description: "Require slots consumed via useSlots() to be explicitly typed with defineSlots<...>()",
    default_severity: Severity::Warning,
};

const MESSAGE: &str =
    "Slots consumed via useSlots() must be explicitly typed with defineSlots<...>().";
const HELP: &str = "Declare the slots with the type-only macro, e.g. \
     `defineSlots<{ default(props: {}): unknown }>()`.";
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
        // Keep the parse-owning `check` path functional: without SFC context
        // only the script half is observable.
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
        let slots = declared::collect(program);
        if !slots.has_ts_syntax {
            return;
        }
        match &slots.declared {
            DeclaredSlots::Absent => report_missing_declaration(&slots, offset, result),
            DeclaredSlots::Unknown => {}
            DeclaredSlots::Known(names) => {
                check_template(names, sfc, result);
            }
        }
    }
}

/// The script half: `useSlots()` consumed with no `defineSlots` at all.
fn report_missing_declaration(slots: &ScriptSlots, offset: usize, result: &mut ScriptLintResult) {
    let Some(span) = slots.first_use_slots else {
        return;
    };
    let (start, end) = script_span(span, offset);
    result.add_diagnostic(
        LintDiagnostic::warn(META.name, MESSAGE, start, end)
            .with_label("slots consumed here without defineSlots", start, end)
            .with_help(HELP),
    );
}

/// The template half: every rendered `<slot>` must be in the declared set.
fn check_template(
    declared: &vize_s0::FxHashSet<CompactString>,
    sfc: SfcScriptContext<'_>,
    result: &mut ScriptLintResult,
) {
    let (Some((root, template_offset)), Some(source)) = (sfc.template_ast(), sfc.template_source)
    else {
        return;
    };
    let RenderedSlots::Known(rendered) = template::collect_rendered_slots(root, source) else {
        return;
    };
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

#[inline]
fn script_span(span: Span, offset: usize) -> (u32, u32) {
    (offset as u32 + span.start, offset as u32 + span.end)
}
