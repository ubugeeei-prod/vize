use std::ops::Range;

use vize_carton::cstr;
use vize_croquis::croquis::{ComponentUsage, PassedProp, SlotUsage};

use crate::virtual_ts::component_reference::component_binding_reference;
use crate::virtual_ts::helpers::to_camel_case;
use crate::virtual_ts::semantic_links::{VizeSemanticLink, VizeSemanticLinkKind};
use crate::virtual_ts::types::{VizeMapping, VizeSubSpan};

use super::component_navigation::{is_ts_identifier, push_ts_single_quoted_literal};
use super::component_props::named_dynamic_navigation_reference;
use super::context::ComponentPropsContext;

pub(super) fn emit_references(
    ts: &mut crate::virtual_ts::TextBuf,
    mappings: &mut Vec<VizeMapping>,
    semantic_links: &mut Vec<VizeSemanticLink>,
    ctx: &ComponentPropsContext<'_, '_>,
    checkable_usages: &[(usize, &ComponentUsage)],
) {
    if !ctx.omit_navigation_references {
        ts.push_str("\n  // Component template navigation references\n");
    }
    for &(idx, usage) in checkable_usages {
        emit_usage_references(ts, mappings, semantic_links, ctx, idx, usage);
    }
}

pub(super) fn emit_usage_references(
    ts: &mut crate::virtual_ts::TextBuf,
    mappings: &mut Vec<VizeMapping>,
    semantic_links: &mut Vec<VizeSemanticLink>,
    ctx: &ComponentPropsContext<'_, '_>,
    idx: usize,
    usage: &ComponentUsage,
) {
    let named_dynamic_ref = named_dynamic_navigation_reference(ctx, usage);
    let inference_only = named_dynamic_ref.is_some();
    let component_ref = named_dynamic_ref.unwrap_or_else(|| {
        component_binding_reference(
            ctx.summary,
            ctx.options,
            ctx.syntactic_type_only_imported_names,
            usage.name.as_str(),
        )
    });

    if inference_only {
        ts.push_str("  // @ts-ignore Navigation-only reference; the authored v-bind statement owns diagnostics.\n");
    }
    ts.push_str("  void ");
    let tag_gen_start = ts.len();
    ts.push_str(&component_ref);
    let tag_gen_end = ts.len();
    ts.push_str(";\n");
    let tag_gen_range = tag_gen_start..tag_gen_end;
    if let Some(src_range) = crate::virtual_ts::component_tag::source_range(
        ctx.template_source,
        ctx.template_offset,
        usage,
    ) {
        mappings.push(VizeMapping {
            gen_range: tag_gen_range.clone(),
            src_range,
            sub_spans: Vec::new(),
        });
    }

    if !ctx.omit_navigation_references {
        emit_prop_references(
            ts,
            mappings,
            semantic_links,
            ctx,
            idx,
            usage,
            &tag_gen_range,
        );
    }
    // A slot reference also rejects a slot name the component lacks, so a
    // diagnostics-only projection keeps it.
    emit_slot_references(ts, mappings, ctx, idx, usage);
}

fn emit_prop_references(
    ts: &mut crate::virtual_ts::TextBuf,
    mappings: &mut Vec<VizeMapping>,
    semantic_links: &mut Vec<VizeSemanticLink>,
    ctx: &ComponentPropsContext<'_, '_>,
    idx: usize,
    usage: &ComponentUsage,
    component_gen_range: &Range<usize>,
) {
    let contract_prefix = crate::virtual_ts::generated_names::template_prefix();
    // A complete TypeScript binding pattern provides a stable completion point
    // even while the Vue attribute name is empty or only `:` has been typed.
    ts.push_str("  const { ");
    let completion_offset = ts.len();
    let component_navigation_props =
        crate::virtual_ts::helpers::shared_types::COMPONENT_NAVIGATION_PROPS.select(false);
    vize_l0::emit_ts!(
        ts,
        "}} = undefined as unknown as {component_navigation_props}<{contract_prefix}Props{idx}>;\n"
    );
    semantic_links.push(VizeSemanticLink {
        source_range: component_gen_range.clone(),
        target_range: completion_offset..completion_offset,
        kind: VizeSemanticLinkKind::VueComponentPropCompletion,
    });
    let props_ref = cstr!("__vize_props_nav_{idx}");
    let mut emitted_props_ref = false;
    for prop in &usage.props {
        if prop.name_is_dynamic || prop.name.as_str() == "key" || prop.name.as_str() == "ref" {
            continue;
        }
        let Some(source_range) = prop_navigation_source_range(ctx.template_source, prop) else {
            continue;
        };

        if !emitted_props_ref {
            // `globalThis.Record`: an authored `Record` cannot capture it, and an
            // undeclared key still has no definition. The references only navigate,
            // so TypeScript's own `@ts-ignore` keeps their checks
            // (`noPropertyAccessFromIndexSignature`, #12531) off every path; it
            // covers one line, which carries every reference statement.
            vize_l0::emit_ts!(
                ts,
                "  const {props_ref} = undefined as unknown as {contract_prefix}Props{idx} & globalThis.Record<string, unknown>;\n  // @ts-ignore Navigation-only references; no diagnostic owner.\n  void "
            );
            emitted_props_ref = true;
        } else {
            ts.push_str(" void ");
        }

        let camel_prop_name = to_camel_case(prop.name.as_str());
        ts.push_str(&props_ref);
        let prop_gen_range = if is_ts_identifier(camel_prop_name.as_str()) {
            ts.push('.');
            let prop_gen_start = ts.len();
            ts.push_str(camel_prop_name.as_str());
            prop_gen_start..ts.len()
        } else {
            ts.push('[');
            let range = push_ts_single_quoted_literal(ts, camel_prop_name.as_str());
            ts.push(']');
            range
        };
        ts.push(';');
        semantic_links.push(VizeSemanticLink {
            source_range: component_gen_range.clone(),
            target_range: prop_gen_range.clone(),
            kind: VizeSemanticLinkKind::VueComponentPropNavigation,
        });
        mappings.push(VizeMapping {
            gen_range: prop_gen_range,
            src_range: (ctx.template_offset as usize + source_range.start)
                ..(ctx.template_offset as usize + source_range.end),
            sub_spans: Vec::new(),
        });
    }
    if emitted_props_ref {
        ts.push('\n');
    }
}

fn prop_navigation_source_range(
    template_source: Option<&str>,
    prop: &PassedProp,
) -> Option<Range<usize>> {
    if prop.name_is_dynamic {
        return None;
    }
    let name = prop.name.as_str();
    if name.is_empty() {
        return None;
    }

    let start = prop.start as usize;
    let end = prop.end as usize;
    let source = template_source?;
    let raw = source.get(start..end)?;
    if let Some(relative_start) = raw.find(name) {
        return Some(start + relative_start..start + relative_start + name.len());
    }

    if name == "modelValue"
        && let Some(relative_start) = raw.find("v-model")
    {
        return Some(start + relative_start..start + relative_start + "v-model".len());
    }

    None
}

fn emit_slot_references(
    ts: &mut crate::virtual_ts::TextBuf,
    mappings: &mut Vec<VizeMapping>,
    ctx: &ComponentPropsContext<'_, '_>,
    idx: usize,
    usage: &ComponentUsage,
) {
    let slots_ref = cstr!("__vize_slots_nav_{idx}");
    let mut emitted_slots_ref = false;
    for (slot_index, slot) in usage.slots.iter().enumerate() {
        if slot.name_is_dynamic {
            continue;
        }
        let Some(source_range) = slot_navigation_source_range(ctx.template_source, slot) else {
            continue;
        };

        if !emitted_slots_ref {
            let component_ref = component_binding_reference(
                ctx.summary,
                ctx.options,
                ctx.syntactic_type_only_imported_names,
                usage.name.as_str(),
            );
            if ctx.hoist_shared_preamble {
                let navigation_slots =
                    crate::virtual_ts::helpers::shared_types::NAVIGATION_SLOTS.select(true);
                vize_l0::emit_ts!(
                    ts,
                    "  const {slots_ref} = undefined as unknown as {navigation_slots}<typeof {component_ref}>;\n"
                );
            } else {
                vize_l0::emit_ts!(
                    ts,
                    "  const {slots_ref} = undefined as unknown as $NavigationSlots<typeof {component_ref}>;\n"
                );
            }
            emitted_slots_ref = true;
        }

        let identifier = is_ts_identifier(slot.name.as_str());
        if identifier {
            ts.push_str("  const { ");
        } else {
            // Native quoted binding keys use string-literal search rather than
            // the owner's property identity. Indexed typeof also rejects missing keys.
            vize_l0::emit_ts!(
                ts,
                "  const __vize_slot_nav_{idx}_{slot_index} = undefined as unknown as typeof {slots_ref}["
            );
        }
        let access_start = ts.len();
        let slot_gen_range = if identifier {
            let slot_gen_start = ts.len();
            ts.push_str(slot.name.as_str());
            slot_gen_start..ts.len()
        } else {
            push_ts_single_quoted_literal(ts, slot.name.as_str())
        };
        let access_end = ts.len();
        if identifier {
            vize_l0::emit_ts!(
                ts,
                ": __vize_slot_nav_{idx}_{slot_index} }} = {slots_ref};\n  void __vize_slot_nav_{idx}_{slot_index};\n"
            );
        } else {
            vize_l0::emit_ts!(ts, "];\n  void __vize_slot_nav_{idx}_{slot_index};\n");
        }
        let src_range = (ctx.template_offset as usize + source_range.start)
            ..(ctx.template_offset as usize + source_range.end);
        mappings.push(VizeMapping {
            gen_range: access_start..access_end,
            src_range: src_range.clone(),
            sub_spans: vec![VizeSubSpan {
                gen_range: slot_gen_range,
                src_range,
            }],
        });
    }
}

pub(super) fn slot_navigation_source_range(
    template_source: Option<&str>,
    slot: &SlotUsage,
) -> Option<Range<usize>> {
    if slot.name_is_dynamic {
        return None;
    }
    let name = slot.name.as_str();
    if name.is_empty() {
        return None;
    }

    let start = slot.start as usize;
    let end = slot.end as usize;
    let source = template_source?;
    let raw = source.get(start..end)?;
    if !(raw.contains('#') || raw.contains("v-slot")) {
        return None;
    }
    raw.find(name)
        .map(|relative_start| start + relative_start..start + relative_start + name.len())
}
