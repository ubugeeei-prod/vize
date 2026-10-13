//! script/no-duplicate-attr-inheritance
//!
//! Match the pinned upstream SFC rule at genuine unbound $attrs references in
//! argumentless v-bind expressions. Disabled inheritance and foreign template
//! bindings stay silent. Standalone scripts retain the existing redundant-true
//! diagnostic, where no SFC template is available to correlate.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <script setup></script>
//! <template><div><button v-bind="$attrs">Save</button></div></template>
//! ```
//!
//! ### Valid
//! ```vue
//! <script setup>defineOptions({ inheritAttrs: false })</script>
//! <template><div><button v-bind="$attrs">Save</button></div></template>
//! ```

mod options;
mod template;
#[cfg(test)]
mod tests;

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta, SfcScriptContext};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{BooleanLiteral, Program};

use self::options::{InheritAttrs, declared_inherit_attrs};
use self::template::{AttrsSpread, attrs_spreads};
use vize_relief::RootNode;

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/no-duplicate-attr-inheritance",
    description: "Flag a component that applies its fallthrough attributes twice",
    default_severity: Severity::Warning,
};

const MESSAGE: &str = "`inheritAttrs: true` is redundant because it is the default.";
const LABEL: &str = "redundant default";
const HELP: &str = "Remove `inheritAttrs: true`: attribute inheritance is already on by default. \
     Keep this option only to opt out with `inheritAttrs: false` (for example when forwarding \
     `$attrs` manually with `v-bind=\"$attrs\"`).";

const SPREAD_MESSAGE: &str = "Set \"inheritAttrs\" to false.";
const SPREAD_LABEL: &str = "attributes applied twice";
const SPREAD_HELP: &str = "Opt out of the automatic inheritance so only this `v-bind` applies \
     them, e.g. `defineOptions({ inheritAttrs: false })` or `inheritAttrs: false` in the \
     component options.";

/// Flag a component that applies its fallthrough attributes twice.
pub struct NoDuplicateAttrInheritance;

impl ScriptRule for NoDuplicateAttrInheritance {
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
        // only the explicit-`true` half is observable.
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
        if !sfc.is_sfc {
            for declared in declared_inherit_attrs(program) {
                if let InheritAttrs::Literal(literal) = declared
                    && literal.value
                {
                    report_redundant_true(literal, offset, result);
                }
            }
            return;
        }
        if !sfc.sole_script_block {
            return;
        }
        for declared in declared_inherit_attrs(program) {
            match declared {
                InheritAttrs::Literal(literal) if literal.value => {}
                InheritAttrs::Literal(_) | InheritAttrs::Opaque => return,
            }
        }
        if let Some((root, template_offset)) = sfc.template_ast() {
            Self::check_scriptless_template(root, template_offset, result);
        }
    }
}

impl NoDuplicateAttrInheritance {
    /// Existing-stage entry for a scriptless SFC, using its cached validated AST.
    pub(crate) fn check_scriptless_template(
        root: &RootNode<'_>,
        template_offset: u32,
        result: &mut ScriptLintResult,
    ) {
        for spread in attrs_spreads(root) {
            report_attrs_spread(&spread, template_offset, result);
        }
    }
}

/// Report the redundant `inheritAttrs: true` boolean literal.
fn report_redundant_true(literal: &BooleanLiteral, offset: usize, result: &mut ScriptLintResult) {
    let start = offset as u32 + literal.span.start;
    let end = offset as u32 + literal.span.end;
    result.add_diagnostic(
        LintDiagnostic::warn(META.name, MESSAGE, start, end)
            .with_label(LABEL, start, end)
            .with_help(HELP),
    );
}

/// Report the first unbound $attrs identifier in each forwarding expression.
fn report_attrs_spread(spread: &AttrsSpread, template_offset: u32, result: &mut ScriptLintResult) {
    let start = template_offset + spread.start;
    let end = template_offset + spread.end;
    result.add_diagnostic(
        LintDiagnostic::warn(META.name, SPREAD_MESSAGE, start, end)
            .with_label(SPREAD_LABEL, start, end)
            .with_help(SPREAD_HELP),
    );
}
