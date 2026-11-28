//! script/no-template-ref-outside-instance
//!
//! Disallow `useTemplateRef()` where no component instance runs it.
//!
//! `useTemplateRef(key)` registers `key` on the component instance that is
//! current when it runs. Vue sets that instance while setup runs, and while a
//! lifecycle hook runs. Anywhere else there is none: Vue only warns in
//! development, and the returned ref is never set. The rule reports a call
//!
//! - at the top level of a module: a plain `<script>`, a `.js` or `.ts` file;
//! - in a callback an API always runs after setup: `setTimeout`,
//!   `setInterval`, `requestAnimationFrame` and the other timers, a `watch`
//!   callback (without `immediate`), `nextTick`, `.then` / `.catch` /
//!   `.finally`, and `addEventListener`;
//! - in a local function of setup that setup never calls, and whose every
//!   use is one of those callbacks, a call from one, or a template `v-on`
//!   handler.
//!
//! Setup is `<script setup>`, the `setup()` option of a component, and the
//! function given to `defineComponent()`. A callback setup passes straight to
//! any other call (`onMounted(() => ...)`, `computed(() => ...)`) and a local
//! function setup calls run in its instance, as the virtual TypeScript
//! generator's template ref owners model it. A function whose owner the
//! script cannot pin down, such as one passed to an unknown composable,
//! returned, exported or stored, is not reported. Calls after an `await` in
//! `setup()` are `script/no-lifecycle-after-await`'s.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <script setup lang="ts">
//! import { useTemplateRef } from 'vue'
//! function focus() {
//!   useTemplateRef<HTMLInputElement>('input').value?.focus()
//! }
//! setTimeout(() => useTemplateRef('input'))
//! </script>
//! <template>
//!   <input ref="input" />
//!   <button @click="focus">Focus</button>
//! </template>
//! ```
//!
//! ### Valid
//! ```vue
//! <script setup lang="ts">
//! import { onMounted, useTemplateRef } from 'vue'
//! const input = useTemplateRef<HTMLInputElement>('input')
//! function focus() {
//!   input.value?.focus()
//! }
//! onMounted(() => focus())
//! </script>
//! <template>
//!   <input ref="input" />
//!   <button @click="focus">Focus</button>
//! </template>
//! ```

mod owners;
mod reach;
mod sibling;
mod template_uses;
#[cfg(test)]
mod tests;

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta, SfcScriptContext};
use crate::diagnostic::{LintDiagnostic, Severity};
use crate::linter::sfc_scripts::BlockSemantic;
use oxc_ast::ast::Program;
use oxc_ast_visit::Visit;
use vize_l0::{CompactString, cstr};

use self::owners::{Owner, Walk};
use self::reach::Placement;

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/no-template-ref-outside-instance",
    description: "Disallow useTemplateRef() where no component instance runs it",
    default_severity: Severity::Error,
};

const MESSAGE: &str = "`useTemplateRef()` is called where no component instance runs it.";

/// Disallow `useTemplateRef()` where no component instance runs it.
pub struct NoTemplateRefOutsideInstance;

impl ScriptRule for NoTemplateRefOutsideInstance {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &META
    }

    #[inline]
    fn uses_ast(&self) -> bool {
        true
    }

    /// `v-on` handlers decide whether a `<script setup>` function runs after
    /// setup, so they come from the parsed template.
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
        if memchr::memmem::find(source.as_bytes(), b"useTemplateRef").is_none() {
            return;
        }
        let built = BlockSemantic::of(sfc.semantic_of(program), program, false);
        if !built.diagnostics.is_empty() {
            return;
        }
        let top = if sfc.is_script_setup {
            Owner::Setup
        } else {
            Owner::Module
        };
        let mut walk = Walk::new(built.semantic.scoping(), top);
        walk.collect_imports(program, sfc.sibling_block());
        if (sfc.is_sfc && !sfc.is_script_setup) || sfc.is_jsx_file {
            walk.plan_export_default(program);
        }
        walk.visit_program(program);
        if walk.sites.is_empty() {
            return;
        }
        let placements = reach::place(&walk, &sfc);
        for (site, placement) in walk.sites.iter().zip(placements) {
            let label = match placement {
                Placement::Instance => continue,
                Placement::Module => CompactString::new("called at the module level"),
                Placement::Deferred(api) => cstr!("called in a `{api}` callback"),
                Placement::Local(symbol) => {
                    let name = built.semantic.scoping().symbol_name(symbol);
                    cstr!("called in `{name}`, which setup never calls")
                }
            };
            let help = match placement {
                Placement::Module => {
                    "At the module level no component is being set up, so Vue never sets the \
                     returned ref. Call `useTemplateRef()` in `<script setup>` or `setup()`."
                }
                _ => {
                    "This code runs after setup has returned, when there is no current \
                     component instance, so Vue never sets the returned ref. Call \
                     `useTemplateRef()` in setup and read the ref here."
                }
            };
            let start = offset as u32 + site.span.start;
            let end = offset as u32 + site.span.end;
            result.add_diagnostic(
                LintDiagnostic::error(META.name, CompactString::new(MESSAGE), start, end)
                    .with_label(label, start, end)
                    .with_help(help),
            );
        }
    }
}
