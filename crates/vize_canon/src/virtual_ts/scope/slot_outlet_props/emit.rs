use std::ops::Range;

use vize_carton::{FxHashMap, FxHashSet, String, append};

use crate::virtual_ts::{expressions::ComponentPropSource, types::VizeMapping};

use super::super::context::ScopeGenContext;
use super::super::vif_guard::append_ignored_vif_guard_open;
use super::SlotOutlet;
use super::literal::append_slot_outlet_literal;

struct SlotOutletCheckContext<'a> {
    slot_outlets_by_scope: &'a FxHashMap<u32, Vec<SlotOutlet>>,
    template_prop_names: &'a FxHashSet<String>,
    source_context: ComponentPropSource<'a>,
    slots_type_ref: &'a str,
    indent: &'a str,
    infer: bool,
}

struct PayloadType {
    text: String,
    name_gen_range: Option<Range<usize>>,
}

pub(super) fn emit_slot_outlet_helpers(
    ts: &mut String,
    slot_outlets_by_scope: &FxHashMap<u32, Vec<SlotOutlet>>,
) {
    let mut needs_static = false;
    let mut needs_dynamic = false;
    let mut needs_spread = false;
    for outlet in slot_outlets_by_scope
        .values()
        .flat_map(|outlets| outlets.iter())
    {
        if !outlet.spread_props.is_empty() {
            needs_spread = true;
        }
        if outlet.name_is_dynamic {
            needs_dynamic = true;
        } else {
            needs_static = true;
        }
    }
    if !needs_static && !needs_dynamic && !needs_spread {
        return;
    }

    // Indexed access keeps generic slot payloads concrete with permissive fallbacks.
    ts.push_str("  type __VizeSlotOutletFn = (...args: any[]) => any;\n");
    if needs_static {
        ts.push_str(
            "  type __VizeSlotOutletTarget<__S, __K extends PropertyKey> = Extract<NonNullable<__S[__K & keyof __S]>, __VizeSlotOutletFn>;\n",
        );
        ts.push_str(
            "  type __VizeSlotOutletArgs<__S, __K extends PropertyKey> = [__VizeSlotOutletTarget<__S, __K>] extends [never] ? [] : Parameters<__VizeSlotOutletTarget<__S, __K>>;\n",
        );
        ts.push_str(
            "  type __VizeSlotOutletPayload<__S, __K extends PropertyKey> = [__VizeSlotOutletArgs<__S, __K>] extends [[]] ? unknown : __VizeSlotOutletArgs<__S, __K>[0];\n",
        );
    }
    if needs_dynamic {
        ts.push_str(
            "  type __VizeAnySlotOutletTarget<__S> = Extract<NonNullable<__S[keyof __S]>, __VizeSlotOutletFn>;\n",
        );
        ts.push_str(
            "  type __VizeAnySlotOutletArgs<__S> = [__VizeAnySlotOutletTarget<__S>] extends [never] ? [] : Parameters<__VizeAnySlotOutletTarget<__S>>;\n",
        );
        ts.push_str(
            "  type __VizeAnySlotOutletPayload<__S> = [__VizeAnySlotOutletArgs<__S>] extends [[]] ? unknown : __VizeAnySlotOutletArgs<__S>[0];\n",
        );
    }
    if needs_spread {
        ts.push_str(
            "  type __VizeSlotOutletSpreadPayload<__Expected, __T> = __Expected & (__T extends object ? __T : Record<string, unknown>);\n",
        );
        ts.push_str(
            "  function __vizeSlotOutletSpread<__Expected>() { return function <__T>(value: __T): __VizeSlotOutletSpreadPayload<__Expected, __T> { return value as any; }; }\n",
        );
    }
}

pub(in crate::virtual_ts::scope) fn generate_scope_slot_outlet_checks(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    scope_id: u32,
    ctx: &ScopeGenContext<'_, '_>,
    indent: &str,
) {
    if !ctx.check_options.check_props {
        return;
    }
    generate_slot_outlet_checks(
        ts,
        mappings,
        scope_id,
        SlotOutletCheckContext {
            slot_outlets_by_scope: &ctx.slot_outlets.by_scope,
            template_prop_names: ctx.template_prop_names,
            source_context: ComponentPropSource::new(
                ctx.template_source,
                ctx.template_offset,
                &ctx.summary.scopes,
            ),
            slots_type_ref: ctx.slot_outlets.slots_type.as_str(),
            indent,
            infer: ctx.slot_outlets.infer,
        },
    );
}

fn generate_slot_outlet_checks(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    scope_id: u32,
    ctx: SlotOutletCheckContext<'_>,
) {
    let SlotOutletCheckContext {
        slot_outlets_by_scope,
        template_prop_names,
        source_context,
        slots_type_ref,
        indent,
        infer,
    } = ctx;
    let Some(outlets) = slot_outlets_by_scope.get(&scope_id) else {
        return;
    };

    for outlet in outlets {
        if let Some(ref guard) = outlet.vif_guard {
            append_ignored_vif_guard_open(ts, indent, guard, "Inference-only guard");
        }
        let expr_indent = if outlet.vif_guard.is_some() {
            let mut nested = String::from(indent);
            nested.push_str("  ");
            nested
        } else {
            String::from(indent)
        };
        if infer {
            append!(
                *ts,
                "{expr_indent}var __vize_slot_payload_{} = ",
                outlet.start
            );
            append_slot_outlet_literal(
                ts,
                mappings,
                outlet,
                "unknown",
                template_prop_names,
                source_context,
                expr_indent.as_str(),
            );
            ts.push_str(";\n");
            if outlet.name_is_dynamic {
                append!(
                    *ts,
                    "{expr_indent}var __vize_slot_name_{} = ({});\n",
                    outlet.start,
                    outlet.name
                );
            }
            if outlet.vif_guard.is_some() {
                append!(*ts, "{indent}}}\n");
            }
            continue;
        }
        append!(*ts, "{expr_indent}((__vize_slot_props: ",);
        let payload_type = outlet_payload_type(outlet, slots_type_ref);
        let payload_type_gen_start = ts.len();
        ts.push_str(payload_type.text.as_str());
        ts.push_str(") => { void __vize_slot_props; })(");
        if let (Some(gen_range), Some(src_range)) = (
            payload_type.name_gen_range,
            outlet.name_source_range.clone(),
        ) {
            mappings.push(VizeMapping {
                gen_range: payload_type_gen_start + gen_range.start
                    ..payload_type_gen_start + gen_range.end,
                src_range: (source_context.offset + src_range.start) as usize
                    ..(source_context.offset + src_range.end) as usize,
                sub_spans: Vec::new(),
            });
        }
        let literal_range = append_slot_outlet_literal(
            ts,
            mappings,
            outlet,
            payload_type.text.as_str(),
            template_prop_names,
            source_context,
            expr_indent.as_str(),
        );
        ts.push_str(");\n");

        let tag_src_start = (source_context.offset + outlet.start + 1) as usize;
        mappings.push(VizeMapping {
            gen_range: literal_range,
            src_range: tag_src_start..tag_src_start + "slot".len(),
            sub_spans: Vec::new(),
        });
        if outlet.vif_guard.is_some() {
            append!(*ts, "{indent}}}\n");
        }
    }
}

fn outlet_payload_type(outlet: &SlotOutlet, slots_type_ref: &str) -> PayloadType {
    if outlet.name_is_dynamic {
        return PayloadType {
            text: vize_carton::cstr!("__VizeAnySlotOutletPayload<{slots_type_ref}>"),
            name_gen_range: None,
        };
    }

    let mut text = String::from("__VizeSlotOutletPayload<");
    text.push_str(slots_type_ref);
    text.push_str(", ");
    let name_gen_range = append_ts_string_literal(&mut text, outlet.name.as_str());
    text.push('>');
    PayloadType {
        text,
        name_gen_range: Some(name_gen_range),
    }
}

fn append_ts_string_literal(out: &mut String, value: &str) -> Range<usize> {
    out.push('"');
    let start = out.len();
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(ch),
        }
    }
    let end = out.len();
    out.push('"');
    start..end
}
