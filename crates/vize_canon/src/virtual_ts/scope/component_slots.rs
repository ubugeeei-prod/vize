//! Required-slot checks for component usages.

use vize_carton::{FxHashSet, String, append, cstr};
use vize_croquis::analysis::ComponentUsage;

use crate::virtual_ts::expressions::ComponentPropCheckContext;
use crate::virtual_ts::types::VizeMapping;

use super::vif_guard::append_ignored_vif_guard_open;

pub(super) fn append_component_slot_check_helpers(ts: &mut String) {
    // `$slots` declarations in third-party component libraries frequently
    // describe callable slot names, not whether the parent must provide them.
    // Required-slot checks therefore trust only Vize's own explicit marker.
    ts.push_str(
        "  type __VizeStructuralSlots<C> = C extends { readonly __vizeSlots?: infer __S } ? NonNullable<__S> : any;\n",
    );
    ts.push_str(
        "  type __VizeRequiredSlotKeys<__S> = { [__K in keyof __S]-?: undefined extends __S[__K] ? never : __K }[keyof __S];\n",
    );
    ts.push_str(
        "  type __VizeMissingRequiredSlots<__S, __P> = Exclude<__VizeRequiredSlotKeys<__S>, keyof __P>;\n",
    );
    ts.push_str(
        "  type __VizeRequiredSlots<__S, __P> = __VizeIsAny<__S> extends true ? {} : string extends keyof __S ? {} : [keyof __S] extends [never] ? {} : [__VizeMissingRequiredSlots<__S, __P>] extends [never] ? {} : { readonly __vizeMissingSlots: __VizeMissingRequiredSlots<__S, __P> };\n",
    );
}

pub(super) fn generate_component_slot_checks(
    ctx: &mut ComponentPropCheckContext<'_, '_>,
    usage: &ComponentUsage,
    idx: usize,
    component_ref: &str,
) {
    let ts = &mut *ctx.ts;
    let mappings = &mut *ctx.mappings;
    let indent = ctx.indent;
    let expr_indent = if usage.vif_guard.is_some() {
        cstr!("{indent}  ")
    } else {
        String::from(indent)
    };

    if let Some(ref guard) = usage.vif_guard {
        append_ignored_vif_guard_open(ts, indent, guard, "Inference-only guard");
    }

    let contract_name = cstr!("__VizeSlotContract_{idx}");
    append!(
        *ts,
        "{expr_indent}type {contract_name} = __VizeStructuralSlots<typeof {component_ref}>;\n"
    );

    let provided_slots = provided_slots_type(usage);
    let check_name = cstr!("__vize_required_slots_{idx}");
    let gen_start = ts.len();
    append!(
        *ts,
        "{expr_indent}const {check_name}: __VizeRequiredSlots<{contract_name}, {provided_slots}> = {{}};\n"
    );
    append!(*ts, "{expr_indent}void {check_name};\n");
    let gen_end = ts.len();
    let tag_src_start = (ctx.source_context.offset + usage.start + 1) as usize;
    mappings.push(VizeMapping {
        gen_range: gen_start..gen_end,
        src_range: tag_src_start..tag_src_start + usage.name.len(),
        sub_spans: Vec::new(),
    });

    if usage.vif_guard.is_some() {
        append!(*ts, "{indent}}}\n");
    }
}

fn provided_slots_type(usage: &ComponentUsage) -> String {
    if usage.slots.iter().any(|slot| slot.name_is_dynamic) {
        return String::from("Record<string, true>");
    }
    if usage.slots.is_empty() {
        return String::from("{}");
    }

    let mut seen = FxHashSet::default();
    let mut output = String::from("{");
    for slot in &usage.slots {
        if !seen.insert(slot.name.clone()) {
            continue;
        }
        output.push_str(" readonly ");
        push_ts_string_literal(&mut output, slot.name.as_str());
        output.push_str(": true;");
    }
    output.push_str(" }");
    output
}

fn push_ts_string_literal(output: &mut String, value: &str) {
    output.push('"');
    for character in value.chars() {
        match character {
            '\\' => output.push_str("\\\\"),
            '"' => output.push_str("\\\""),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            _ => output.push(character),
        }
    }
    output.push('"');
}
