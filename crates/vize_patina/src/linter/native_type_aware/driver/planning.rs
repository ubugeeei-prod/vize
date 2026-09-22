//! Decide which macro warnings require a type probe.

use super::super::parsing::is_runtime_array_macro;
use super::super::{
    LintResult, Linter, RULE_REQUIRE_TYPED_EMITS, RULE_REQUIRE_TYPED_PROPS, push_script_warning,
};
use crate::diagnostic::LintDiagnostic;
use vize_croquis::Croquis;

#[inline]
pub(super) fn is_type_rule_active(linter: &Linter, rule_name: &str) -> bool {
    linter.registry.has_rule(rule_name) && linter.is_rule_enabled(rule_name)
}

pub(super) fn collect_prop_static_warning_or_probe_need(
    linter: &Linter,
    analysis: &Croquis,
    result: &mut LintResult,
    descriptor: &vize_atelier_sfc::SfcDescriptor<'_>,
) -> bool {
    if !is_type_rule_active(linter, RULE_REQUIRE_TYPED_PROPS) {
        return false;
    }

    let Some(call) = analysis.macros.define_props() else {
        return false;
    };
    if call.type_args.is_some() {
        return false;
    }

    if is_runtime_array_macro(call.runtime_args.as_ref().map(|args| args.as_str())) {
        push_script_warning(
            result,
            descriptor,
            LintDiagnostic::warn(RULE_REQUIRE_TYPED_PROPS, "Prop should have a type definition", call.start, call.end)
            .with_help(
                "Use `defineProps<Props>()` or a runtime prop object with concrete constructor types.",
            ),
        );
        return false;
    }

    analysis
        .macros
        .props()
        .iter()
        .any(|prop| prop.prop_type.is_none())
}

pub(super) fn collect_emit_static_warning_or_probe_need(
    linter: &Linter,
    analysis: &Croquis,
    result: &mut LintResult,
    descriptor: &vize_atelier_sfc::SfcDescriptor<'_>,
) -> bool {
    if !is_type_rule_active(linter, RULE_REQUIRE_TYPED_EMITS) {
        return false;
    }

    let Some(call) = analysis.macros.define_emits() else {
        return false;
    };
    if call.type_args.is_some() {
        return false;
    }

    if is_runtime_array_macro(call.runtime_args.as_ref().map(|args| args.as_str())) {
        push_script_warning(
            result,
            descriptor,
            LintDiagnostic::warn(
                RULE_REQUIRE_TYPED_EMITS,
                "Emit should have a type definition",
                call.start,
                call.end,
            )
            .with_help(
                "Use `defineEmits<...>()` or a validator object with typed payload parameters.",
            ),
        );
        return false;
    }

    analysis
        .macros
        .emits()
        .iter()
        .any(|emit| emit.payload_type.is_none())
}
