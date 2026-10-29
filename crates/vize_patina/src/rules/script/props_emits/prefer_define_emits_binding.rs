//! script/prefer-define-emits-binding
//!
//! Prefer the typed function captured from `<script setup>`'s `defineEmits`
//! over the template-only `$emit` helper. The replacement is offered only when
//! the macro has a plain identifier capture and that identifier is not shadowed
//! by the surrounding template scope.

use oxc_ast::ast::Program;
use vize_l0::cstr;

use super::compiler_macro_bindings::{DefineEmitsCapture, define_emits_capture};
use crate::diagnostic::{Fix, LintDiagnostic, Severity, TextEdit};
use crate::rules::script::template_scan::{DOLLAR_EMIT, for_each_template_call_with_replacement};
use crate::rules::script::{ScriptLintResult, ScriptRule, ScriptRuleMeta, SfcScriptContext};

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/prefer-define-emits-binding",
    description: "Prefer the captured defineEmits function over template $emit",
    default_severity: Severity::Warning,
};

/// Prefer the typed function captured from `defineEmits` over template `$emit`.
pub struct PreferDefineEmitsBinding;

impl ScriptRule for PreferDefineEmitsBinding {
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
        if !sfc.is_script_setup {
            return;
        }
        let binding = match define_emits_capture(program) {
            DefineEmitsCapture::Missing => return,
            DefineEmitsCapture::Unbound => None,
            DefineEmitsCapture::Bound(DOLLAR_EMIT) => return,
            DefineEmitsCapture::Bound(binding) => Some(binding),
        };
        let Some((root, template_offset)) = sfc.template_ast() else {
            return;
        };

        for_each_template_call_with_replacement(root, binding, |call, replacement_shadowed| {
            if call.callee != DOLLAR_EMIT {
                return;
            }
            let replacement = binding.filter(|binding| binding.is_ascii() && !replacement_shadowed);
            report(
                template_offset + call.callee_start,
                template_offset + call.callee_end,
                binding,
                replacement,
                result,
            );
        });
    }
}

fn report(
    start: u32,
    end: u32,
    binding: Option<&str>,
    replacement: Option<&str>,
    result: &mut ScriptLintResult,
) {
    let diagnostic = LintDiagnostic::warn(
        META.name,
        "Use the function returned by defineEmits() instead of template `$emit`.",
        start,
        end,
    )
    .with_label(
        "template $emit bypasses the defineEmits binding",
        start,
        end,
    );

    let diagnostic = match (binding, replacement) {
        (Some(binding), Some(replacement)) => diagnostic
            .with_help(cstr!(
                "Call `{binding}(...)` so event names and payloads are checked against the defineEmits contract."
            ))
            .with_fix(Fix::new(
                cstr!("Replace `$emit` with `{replacement}`"),
                TextEdit::replace(start, end, replacement),
            )),
        (Some(binding), None) => diagnostic.with_help(cstr!(
            "The captured `{binding}` binding cannot be inserted safely in this template scope; rename it before replacing `$emit`."
        )),
        (None, _) => diagnostic.with_help(
            "Capture the only defineEmits() call in an immutable plain identifier before replacing `$emit`.",
        ),
    };
    result.add_diagnostic(diagnostic);
}

#[cfg(test)]
#[expect(clippy::string_slice, reason = "tests assert authored byte ranges")]
mod tests;
