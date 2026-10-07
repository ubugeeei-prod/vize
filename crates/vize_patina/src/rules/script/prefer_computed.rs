//! script/prefer-computed
//!
//! Prefer computed properties over manually syncing reactive state.
//!
//! When a value can be derived from other reactive state, use `computed()`
//! instead of manually updating a separate `ref` with a watcher.
//!
//! This follows the principle: "reactive state that can be computed from
//! other state should use computed properties, not be actively defined."
//!
//! ## Examples
//!
//! ### Not Recommended
//! ```ts
//! const count = ref(0)
//! const doubled = ref(0)
//!
//! watch(count, (val) => {
//!   doubled.value = val * 2
//! })
//! ```
//!
//! ### Recommended
//! ```ts
//! const count = ref(0)
//! const doubled = computed(() => count.value * 2)
//! ```

use oxc_ast::ast::{CallExpression, Expression, Program};
use oxc_ast_visit::{Visit, walk::walk_call_expression};
use oxc_span::Span;

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta, SfcScriptContext};
use crate::diagnostic::{LintDiagnostic, Severity};

mod analysis;
mod template;
use analysis::{Inventory, derived_assignment};
use template::template_writes;

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/prefer-computed",
    description: "Prefer computed() for derived reactive state",
    default_severity: Severity::Warning,
};

/// Prefer computed over watched refs.
pub struct PreferComputed;

impl ScriptRule for PreferComputed {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &META
    }
    fn uses_ast(&self) -> bool {
        true
    }
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
        offset: usize,
        sfc: SfcScriptContext<'_>,
        result: &mut ScriptLintResult,
    ) {
        // A sibling script or an unparsed template can contain unobservable writes.
        if sfc.is_sfc
            && (!sfc.sole_script_block
                || (sfc.template_source.is_some() && sfc.template_root.is_none()))
        {
            return;
        }
        let inventory = Inventory::collect(program);
        let template_writes = template_writes(sfc);
        let mut visitor = PreferComputedVisitor {
            offset,
            result,
            inventory: &inventory,
            template_writes: &template_writes,
        };
        visitor.visit_program(program);
    }
}

struct PreferComputedVisitor<'rule> {
    offset: usize,
    result: &'rule mut ScriptLintResult,
    inventory: &'rule Inventory,
    template_writes: &'rule vize_l0::FxHashSet<vize_l0::CompactString>,
}

impl<'a> Visit<'a> for PreferComputedVisitor<'_> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if let Some(callee_span) = self.watch_callee_span(call)
            && let Some((target, _)) = derived_assignment(call, self.inventory)
            && !self.template_writes.contains(target)
        {
            self.result.add_diagnostic(
                LintDiagnostic::warn(
                    META.name,
                    "Consider using computed() instead of watch() for derived state",
                    self.offset as u32 + callee_span.start,
                    self.offset as u32 + callee_span.end,
                )
                .with_help(
                    "If the watch callback only assigns to a ref based on the watched value, \
                     use computed() instead: `const derived = computed(() => source.value * 2)`",
                ),
            );
        }
        walk_call_expression(self, call);
    }
}

impl PreferComputedVisitor<'_> {
    fn watch_callee_span(&self, call: &CallExpression<'_>) -> Option<Span> {
        let Expression::Identifier(identifier) = unwrap_expression(&call.callee) else {
            return None;
        };
        self.inventory
            .is_vue_factory(identifier.name.as_str(), "watch")
            .then_some(identifier.span)
    }
}

/// Strip parentheses and TS-only wrappers.
fn unwrap_expression<'a, 'b>(expression: &'b Expression<'a>) -> &'b Expression<'a> {
    expression.get_inner_expression()
}

#[cfg(test)]
mod tests;
