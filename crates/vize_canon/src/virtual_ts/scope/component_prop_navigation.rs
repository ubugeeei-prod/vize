use std::ops::Range;

use vize_carton::{String, append, cstr};
use vize_croquis::croquis::{ComponentUsage, PassedProp};

use crate::virtual_ts::component_reference::component_binding_reference;
use crate::virtual_ts::helpers::{to_camel_case, to_safe_identifier_fragment};
use crate::virtual_ts::semantic_links::{VizeSemanticLink, VizeSemanticLinkKind};
use crate::virtual_ts::types::VizeMapping;

use super::component_navigation::{is_ts_identifier, push_ts_single_quoted_literal};
use super::context::ComponentPropsContext;

pub(super) fn emit_references(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    semantic_links: &mut Vec<VizeSemanticLink>,
    ctx: &ComponentPropsContext<'_>,
    checkable_usages: &[(usize, &ComponentUsage)],
) {
    ts.push_str("\n  // Component template navigation references\n");
    for &(idx, usage) in checkable_usages {
        let component_ref = component_binding_reference(
            ctx.summary,
            ctx.options,
            ctx.syntactic_type_only_imported_names,
            usage.name.as_str(),
        );
        let tag_src_start = (ctx.template_offset + usage.start + 1) as usize;
        let tag_src_end = tag_src_start + usage.name.len();

        ts.push_str("  void ");
        let tag_gen_start = ts.len();
        ts.push_str(&component_ref);
        let tag_gen_end = ts.len();
        ts.push_str(";\n");
        let tag_gen_range = tag_gen_start..tag_gen_end;
        mappings.push(VizeMapping {
            gen_range: tag_gen_range.clone(),
            src_range: tag_src_start..tag_src_end,
            sub_spans: Vec::new(),
        });

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
}

fn emit_prop_references(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    semantic_links: &mut Vec<VizeSemanticLink>,
    ctx: &ComponentPropsContext<'_>,
    idx: usize,
    usage: &ComponentUsage,
    component_gen_range: &Range<usize>,
) {
    let component_type_name = to_safe_identifier_fragment(usage.name.as_str());
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
            append!(
                *ts,
                "  const {props_ref} = undefined as unknown as __{component_type_name}_Props_{idx} & Record<string, unknown>;\n"
            );
            emitted_props_ref = true;
        }

        let camel_prop_name = to_camel_case(prop.name.as_str());
        append!(*ts, "  void {props_ref}");
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
        ts.push_str(";\n");
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
