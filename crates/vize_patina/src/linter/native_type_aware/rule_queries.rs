use super::{
    markers::{push_emit_validator_markers, push_prop_type_markers, MacroQuery, QueryKind},
    parsing::is_runtime_array_macro,
    push_warning, LintResult, Linter, RULE_REQUIRE_TYPED_EMITS, RULE_REQUIRE_TYPED_PROPS,
};
use crate::diagnostic::LintDiagnostic;
use vize_croquis::{virtual_ts::VirtualTsOutput, Croquis};

pub(super) struct MacroWarning<'a> {
    pub kind: QueryKind,
    pub base_offset: u32,
    pub rule_name: &'static str,
    pub message: &'static str,
    pub help: &'a str,
    pub should_warn: bool,
}

pub(super) fn collect_prop_queries(
    linter: &Linter,
    analysis: &Croquis,
    result: &mut LintResult,
    script_block: &vize_atelier_sfc::SfcScriptBlock<'_>,
    virtual_ts: &mut VirtualTsOutput,
    macro_queries: &mut Vec<MacroQuery>,
) {
    if !(linter.registry.has_rule(RULE_REQUIRE_TYPED_PROPS)
        && linter.is_rule_enabled(RULE_REQUIRE_TYPED_PROPS))
    {
        return;
    }

    if let Some(call) = analysis.macros.define_props() {
        if call.type_args.is_none() {
            if is_runtime_array_macro(call.runtime_args.as_ref().map(|args| args.as_str())) {
                push_warning(
                    result,
                    LintDiagnostic::warn(
                        RULE_REQUIRE_TYPED_PROPS,
                        "Prop should have a type definition",
                        script_block.loc.start as u32 + call.start,
                        script_block.loc.start as u32 + call.end,
                    )
                    .with_help(
                        "Use `defineProps<Props>()` or a runtime prop object with concrete constructor types.",
                    ),
                );
            } else if analysis
                .macros
                .props()
                .iter()
                .any(|prop| prop.prop_type.is_none())
            {
                push_prop_type_markers(
                    virtual_ts,
                    call.runtime_args.as_ref().map(|args| args.as_str()),
                    call.start,
                    call.end,
                    macro_queries,
                );
            }
        }
    }
}

pub(super) fn collect_emit_queries(
    linter: &Linter,
    analysis: &Croquis,
    result: &mut LintResult,
    script_block: &vize_atelier_sfc::SfcScriptBlock<'_>,
    virtual_ts: &mut VirtualTsOutput,
    macro_queries: &mut Vec<MacroQuery>,
) {
    if !(linter.registry.has_rule(RULE_REQUIRE_TYPED_EMITS)
        && linter.is_rule_enabled(RULE_REQUIRE_TYPED_EMITS))
    {
        return;
    }

    if let Some(call) = analysis.macros.define_emits() {
        if call.type_args.is_none() {
            if is_runtime_array_macro(call.runtime_args.as_ref().map(|args| args.as_str())) {
                push_warning(
                    result,
                    LintDiagnostic::warn(
                        RULE_REQUIRE_TYPED_EMITS,
                        "Emit should have a type definition",
                        script_block.loc.start as u32 + call.start,
                        script_block.loc.start as u32 + call.end,
                    )
                    .with_help(
                        "Use `defineEmits<...>()` or a validator object with typed payload parameters.",
                    ),
                );
            } else if analysis
                .macros
                .emits()
                .iter()
                .any(|emit| emit.payload_type.is_none())
            {
                push_emit_validator_markers(
                    virtual_ts,
                    call.runtime_args.as_ref().map(|args| args.as_str()),
                    call.start,
                    call.end,
                    macro_queries,
                );
            }
        }
    }
}

pub(super) fn push_macro_warning(
    result: &mut LintResult,
    queries: &[MacroQuery],
    warning: MacroWarning<'_>,
) {
    if !warning.should_warn {
        return;
    }
    if let Some(query) = queries.iter().find(|query| query.kind == warning.kind) {
        push_warning(
            result,
            LintDiagnostic::warn(
                warning.rule_name,
                warning.message,
                warning.base_offset + query.source_start,
                warning.base_offset + query.source_end,
            )
            .with_help(warning.help),
        );
    }
}
