use std::ops::Range;

use vize_carton::{String, append, cstr};
use vize_croquis::croquis::{ComponentUsage, PassedProp};

use crate::virtual_ts::helpers::{to_camel_case, to_safe_identifier, to_safe_identifier_fragment};
use crate::virtual_ts::types::VizeMapping;

use super::context::ComponentPropsContext;

pub(super) fn has_navigable_props(ctx: &ComponentPropsContext<'_>, usage: &ComponentUsage) -> bool {
    usage.props.iter().any(|prop| {
        prop.name.as_str() != "key"
            && prop.name.as_str() != "ref"
            && prop_navigation_source_range(ctx.template_source, prop).is_some()
    })
}

pub(super) fn emit_references(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    ctx: &ComponentPropsContext<'_>,
    checkable_usages: &[(usize, &ComponentUsage)],
) {
    ts.push_str("\n  // Component template navigation references\n");
    for &(idx, usage) in checkable_usages {
        let component_ref = to_safe_identifier(usage.name.as_str());
        let component_type_name = to_safe_identifier_fragment(usage.name.as_str());
        let tag_src_start = (ctx.template_offset + usage.start + 1) as usize;
        let tag_src_end = tag_src_start + usage.name.len();

        ts.push_str("  void ");
        let tag_gen_start = ts.len();
        ts.push_str(&component_ref);
        let tag_gen_end = ts.len();
        ts.push_str(";\n");
        mappings.push(VizeMapping {
            gen_range: tag_gen_start..tag_gen_end,
            src_range: tag_src_start..tag_src_end,
            sub_spans: Vec::new(),
        });

        emit_prop_references(ts, mappings, ctx, idx, usage, component_type_name.as_str());
    }
}

fn emit_prop_references(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    ctx: &ComponentPropsContext<'_>,
    idx: usize,
    usage: &ComponentUsage,
    component_type_name: &str,
) {
    let props_ref = cstr!("__vize_props_nav_{idx}");
    let mut emitted_props_ref = false;
    for prop in &usage.props {
        if prop.name.as_str() == "key" || prop.name.as_str() == "ref" {
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

fn push_ts_single_quoted_literal(ts: &mut String, value: &str) -> Range<usize> {
    ts.push('\'');
    let start = ts.len();
    for ch in value.chars() {
        match ch {
            '\\' => ts.push_str("\\\\"),
            '\'' => ts.push_str("\\'"),
            '\n' => ts.push_str("\\n"),
            '\r' => ts.push_str("\\r"),
            '\t' => ts.push_str("\\t"),
            _ => ts.push(ch),
        }
    }
    let end = ts.len();
    ts.push('\'');
    start..end
}

fn is_ts_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first == '_' || first == '$' || first.is_ascii_alphabetic()) {
        return false;
    }
    chars.all(|ch| ch == '_' || ch == '$' || ch.is_ascii_alphanumeric())
}
