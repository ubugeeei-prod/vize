//! script/no-use-computed-property-like-method
//!
//! Disallow calling an Options API `computed` property like a method. A computed
//! entry exposes a *value*, not a function, so `this.fullName()` (when
//! `fullName` is a computed) evaluates the value and immediately calls it —
//! throwing at runtime unless it happens to return a function. The call
//! parentheses are almost always a mistake for `this.fullName`.
//!
//! Port of [`vue/no-use-computed-property-like-method`](https://eslint.vuejs.org/rules/no-use-computed-property-like-method.html),
//! scoped to the Options API: the computed names come from the `computed`
//! option, and both places they can be called are checked.
//!
//! ## Script call sites
//!
//! `this.<computedName>(...)` inside a direct member of the options object,
//! where `this` is the component instance. A non-arrow function nested deeper
//! rebinds `this`, so calls there are skipped.
//!
//! ## Template call sites
//!
//! `{{ total() }}`, `:title="total()"`, `@click="total()"` — the template
//! reaches a computed by bare name, and this is exactly where the mistake is
//! made, since `{{ total() }}` throws at runtime unless the computed happens to
//! return a function. Recovering those calls *creates* findings from template
//! evidence, so they come from the template AST and an oxc parse of each
//! expression; see [`super::template_scan`] for the over-match analysis and the
//! shadowing rules.

mod computed_names;
#[cfg(test)]
mod tests;

use super::template_scan::{TemplateCall, for_each_template_call};
use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta, SfcScriptContext};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{CallExpression, Expression, Program};
use oxc_ast_visit::{Visit, walk::walk_call_expression};
use oxc_span::Span;
use vize_s0::{CompactString, FxHashSet};

use self::computed_names::{collect_computed_names, find_component_options};

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/no-use-computed-property-like-method",
    description: "Disallow calling an Options API computed property like a method",
    default_severity: Severity::Error,
};

/// Disallow calling an Options API `computed` property like a method.
pub struct NoUseComputedPropertyLikeMethod;

impl ScriptRule for NoUseComputedPropertyLikeMethod {
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
        let Some(options) = find_component_options(program) else {
            return;
        };
        let computed_names = collect_computed_names(options);
        if computed_names.is_empty() {
            return;
        }
        let mut visitor = ComputedCallVisitor {
            computed_names: &computed_names,
            offset,
            result,
            fn_depth: 0,
        };
        visitor.visit_object_expression(options);

        check_template(&computed_names, sfc, result);
    }
}

/// The template half: a bare `computedName(...)` in a template expression.
fn check_template(
    computed_names: &FxHashSet<CompactString>,
    sfc: SfcScriptContext<'_>,
    result: &mut ScriptLintResult,
) {
    let Some((root, template_offset)) = sfc.template_ast() else {
        return;
    };
    for_each_template_call(root, |call: TemplateCall<'_>| {
        if !computed_names.contains(call.callee) {
            return;
        }
        report(
            template_offset + call.start,
            template_offset + call.end,
            call.callee,
            TEMPLATE_HELP,
            result,
        );
    });
}

/// Walks the component and reports `this.<computedName>(...)` member calls.
///
/// A direct member function binds `this` to the component instance, so a call
/// there is reported (`fn_depth == 1`). A non-arrow function nested inside a
/// member rebinds `this`, so deeper calls are skipped to avoid false positives.
/// Arrow functions keep the lexical `this` and do not change the depth.
struct ComputedCallVisitor<'rule> {
    computed_names: &'rule FxHashSet<CompactString>,
    offset: usize,
    result: &'rule mut ScriptLintResult,
    /// Non-arrow function nesting depth from the options object; `1` is a direct
    /// member (its `this` is the instance), deeper layers have rebound `this`.
    fn_depth: u32,
}

impl<'a> Visit<'a> for ComputedCallVisitor<'_> {
    fn visit_function(
        &mut self,
        it: &oxc_ast::ast::Function<'a>,
        flags: oxc_syntax::scope::ScopeFlags,
    ) {
        self.fn_depth += 1;
        oxc_ast_visit::walk::walk_function(self, it, flags);
        self.fn_depth -= 1;
    }

    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        if self.fn_depth == 1
            && let Expression::StaticMemberExpression(member) = &it.callee
            && matches!(&member.object, Expression::ThisExpression(_))
            && self.computed_names.contains(member.property.name.as_str())
        {
            self.report(it.span, member.property.name.as_str());
        }
        walk_call_expression(self, it);
    }
}

impl ComputedCallVisitor<'_> {
    fn report(&mut self, span: Span, name: &str) {
        let start = self.offset as u32 + span.start;
        let end = self.offset as u32 + span.end;
        report(start, end, name, SCRIPT_HELP, self.result);
    }
}

const SCRIPT_HELP: &str = "A computed property exposes a value, not a function. Read it as \
     `this.<name>` (drop the call parentheses), or move the logic into a \
     `method` if you need to invoke it.";
const TEMPLATE_HELP: &str = "A computed property exposes a value, not a function. Drop the call \
     parentheses (`{{ name }}`), or move the logic into a `method` if you need to invoke it.";

fn report(start: u32, end: u32, name: &str, help: &'static str, result: &mut ScriptLintResult) {
    let mut message = CompactString::with_capacity(name.len() + 56);
    message.push_str("'");
    message.push_str(name);
    message.push_str("' is a computed property and must not be called like a method.");
    let diagnostic = LintDiagnostic::error(META.name, message, start, end)
        .with_label("computed value called as a function", start, end)
        .with_help(help);
    result.add_diagnostic(diagnostic);
}
