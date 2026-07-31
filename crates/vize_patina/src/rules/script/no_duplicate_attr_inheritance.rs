//! script/no-duplicate-attr-inheritance
//!
//! Flag a component that applies its fallthrough attributes twice.
//!
//! Ports [`vue/no-duplicate-attr-inheritance`](https://eslint.vuejs.org/rules/no-duplicate-attr-inheritance.html),
//! which warns about *double attribute application*: a component that keeps the
//! default attribute inheritance (`inheritAttrs: true`) **and** also forwards
//! `$attrs` manually with `v-bind="$attrs"` on its root element ends up applying
//! the fallthrough attributes twice.
//!
//! Two spellings of the same defect are reported:
//!
//! * **An explicit `inheritAttrs: true`.** `true` is the framework default, so
//!   stating it changes nothing; it is the location upstream also reports when
//!   both signals are present, and because `true` is unconditionally the default
//!   this half fires with zero false positives even without the template.
//! * **A root `v-bind="$attrs"` with the default left implicit.** This is the
//!   template half, and it is the one upstream leads with. It needs the
//!   `<template>` AST — see [`template`] for the over-match analysis.
//!
//! When both are present only the first fires: the explicit `true` is the
//! smaller, more actionable edit, and two diagnostics for one defect is noise.
//!
//! ## What stays silent
//!
//! * `inheritAttrs: false` — the intended opt-out, in either spelling.
//! * An `inheritAttrs` whose value is not a boolean literal: the effective value
//!   is unknown, so neither direction may be assumed.
//! * `v-bind="$attrs"` on a **nested** element. That is the documented way to
//!   forward attributes to an inner node, is idiomatic, and is normally paired
//!   with `inheritAttrs: false` — reporting it would be a false positive.
//! * A multi-root (fragment) template: there is no single element for the
//!   fallthrough attributes to land on.
//! * An SFC with **both** `<script>` and `<script setup>`. The rule is invoked
//!   once per block and each sees only half the script surface, so an
//!   `inheritAttrs: false` in the sibling block would be invisible. See
//!   [`SfcScriptContext::sole_script_block`].
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! defineOptions({ inheritAttrs: true })
//! export default { inheritAttrs: true }
//! ```
//!
//! ```vue
//! <script setup lang="ts"></script>
//!
//! <template>
//!   <div v-bind="$attrs"></div>
//! </template>
//! ```
//!
//! ### Valid
//! ```ts
//! defineOptions({ inheritAttrs: false }) // intentional opt-out
//! export default {}                      // default inheritance, unstated
//! ```

mod options;
mod template;
#[cfg(test)]
mod tests;

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta, SfcScriptContext};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{BooleanLiteral, Program};

use self::options::{InheritAttrs, declared_inherit_attrs};
use self::template::{AttrsSpread, root_attrs_spread};

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

const SPREAD_MESSAGE: &str = "`v-bind=\"$attrs\"` on the root element applies the fallthrough \
     attributes twice, because `inheritAttrs` defaults to true.";
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
        let mut stated = false;
        for declared in declared_inherit_attrs(program) {
            stated = true;
            if let InheritAttrs::Literal(literal) = declared
                && literal.value
            {
                report_redundant_true(literal, offset, result);
            }
        }
        // The template half only speaks about the *implicit* default. An
        // explicit `true` was just reported at the more actionable location, an
        // explicit `false` is the intended opt-out, and an opaque value is
        // unknowable — all three are done here.
        if stated {
            return;
        }
        check_template(sfc, result);
    }
}

/// The template half: a root `v-bind="$attrs"` with `inheritAttrs` left default.
fn check_template(sfc: SfcScriptContext<'_>, result: &mut ScriptLintResult) {
    if !sfc.sole_script_block {
        return;
    }
    let Some((root, template_offset)) = sfc.template_ast() else {
        return;
    };
    if let Some(spread) = root_attrs_spread(root) {
        report_attrs_spread(&spread, template_offset, result);
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

/// Report the root `v-bind="$attrs"`, at its location in the template.
fn report_attrs_spread(spread: &AttrsSpread, template_offset: u32, result: &mut ScriptLintResult) {
    let start = template_offset + spread.start;
    let end = template_offset + spread.end;
    result.add_diagnostic(
        LintDiagnostic::warn(META.name, SPREAD_MESSAGE, start, end)
            .with_label(SPREAD_LABEL, start, end)
            .with_help(SPREAD_HELP),
    );
}
