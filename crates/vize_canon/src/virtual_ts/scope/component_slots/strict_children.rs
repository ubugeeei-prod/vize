use std::ops::Range;

use vize_carton::{String, append, cstr};
use vize_croquis::analysis::ComponentUsage;
use vize_relief::{ElementNode, RootNode, TemplateChildNode};

use crate::virtual_ts::expressions::ComponentPropCheckContext;
use crate::virtual_ts::helpers::to_safe_identifier_fragment;
use crate::virtual_ts::types::VizeMapping;

use super::child_types::collect_child_types;
use super::slot_syntax::{is_slot_directive, static_slot_directive};
use super::{ComponentSlotCheckMeta, push_ts_string_literal};

pub(super) fn generate_strict_slot_child_checks(
    ctx: &mut ComponentPropCheckContext<'_, '_>,
    usage: &ComponentUsage,
    idx: usize,
    contract_name: &str,
    meta: ComponentSlotCheckMeta<'_, '_>,
) {
    let Some(root) = meta.template_ast else {
        return;
    };
    let Some(element) = find_usage_element(root, usage) else {
        return;
    };
    let checks = collect_strict_slot_children(element, root.source, meta);
    if checks.is_empty() {
        return;
    }

    let ts = &mut *ctx.ts;
    let mappings = &mut *ctx.mappings;
    let base_indent = ctx.indent;
    let indent = if usage.vif_guard.is_some() {
        cstr!("{base_indent}  ")
    } else {
        String::from(ctx.indent)
    };

    for check in checks {
        let safe_slot_name = to_safe_identifier_fragment(check.name.as_str());
        let check_name = cstr!("__vize_slot_children_{idx}_{safe_slot_name}");
        let gen_start = ts.len();
        append!(
            *ts,
            "{indent}const {check_name}: __VizeSlotChildren<{contract_name}, "
        );
        push_ts_string_literal(ts, check.name.as_str());
        append!(
            *ts,
            "> = null as unknown as __VizeProvidedSlotChildren<{}>;\n",
            check.children_type.as_str()
        );
        append!(*ts, "{indent}void {check_name};\n");
        let gen_end = ts.len();
        let src_offset = ctx.source_context.offset as usize;
        mappings.push(VizeMapping {
            gen_range: gen_start..gen_end,
            src_range: (src_offset + check.src_range.start)..(src_offset + check.src_range.end),
            sub_spans: Vec::new(),
        });
    }
}

struct StrictSlotChildren {
    name: String,
    src_range: Range<usize>,
    children_type: String,
}

fn collect_strict_slot_children<'a, 'template>(
    element: &'a ElementNode<'template>,
    source: &str,
    meta: ComponentSlotCheckMeta<'a, 'template>,
) -> std::vec::Vec<StrictSlotChildren>
where
    'template: 'a,
{
    let mut checks = std::vec::Vec::new();
    let has_component_slot_directive = element.props.iter().any(is_slot_directive);
    let component_slot = element
        .props
        .iter()
        .find_map(|prop| static_slot_directive(prop, source));

    if let Some(slot) = component_slot.as_ref() {
        push_strict_slot_children(
            &mut checks,
            slot.name.as_str(),
            slot.src_range.clone(),
            collect_child_types(element.children.iter(), source, meta, false),
        );
    }

    for child in &element.children {
        let TemplateChildNode::Element(child_element) = child else {
            continue;
        };
        if child_element.tag != "template" {
            continue;
        }
        let Some(slot) = child_element
            .props
            .iter()
            .find_map(|prop| static_slot_directive(prop, source))
        else {
            continue;
        };
        push_strict_slot_children(
            &mut checks,
            slot.name.as_str(),
            slot.src_range,
            collect_child_types(child_element.children.iter(), source, meta, false),
        );
    }

    if !has_component_slot_directive {
        let direct_default_children =
            collect_child_types(element.children.iter(), source, meta, true);
        if let Some(src_range) = first_child_source_range(element.children.iter(), true) {
            push_strict_slot_children(&mut checks, "default", src_range, direct_default_children);
        }
    }

    checks
}

fn push_strict_slot_children(
    checks: &mut std::vec::Vec<StrictSlotChildren>,
    name: &str,
    src_range: Range<usize>,
    children: std::vec::Vec<String>,
) {
    if children.is_empty() {
        return;
    }
    let children_type = tuple_type(children);
    if let Some(existing) = checks.iter_mut().find(|check| check.name == name) {
        existing.children_type =
            merge_tuple_types(existing.children_type.as_str(), children_type.as_str());
        existing.src_range.start = existing.src_range.start.min(src_range.start);
        existing.src_range.end = existing.src_range.end.max(src_range.end);
        return;
    }
    checks.push(StrictSlotChildren {
        name: String::from(name),
        src_range,
        children_type,
    });
}

fn tuple_type(children: std::vec::Vec<String>) -> String {
    let mut output = String::from("[");
    for (index, child) in children.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        output.push_str(child.as_str());
    }
    output.push(']');
    output
}

fn merge_tuple_types(left: &str, right: &str) -> String {
    let mut output = String::from(left.trim_end_matches(']'));
    let right_inner = right.trim_start_matches('[').trim_end_matches(']');
    if !right_inner.is_empty() {
        if !output.ends_with('[') {
            output.push_str(", ");
        }
        output.push_str(right_inner);
    }
    output.push(']');
    output
}

fn first_child_source_range<'a, 'template, I>(
    children: I,
    skip_named_slot_templates: bool,
) -> Option<Range<usize>>
where
    'template: 'a,
    I: IntoIterator<Item = &'a TemplateChildNode<'template>>,
{
    for child in children {
        if let TemplateChildNode::Element(element) = child
            && skip_named_slot_templates
            && element.tag == "template"
            && element.props.iter().any(is_slot_directive)
        {
            continue;
        }
        if let TemplateChildNode::Text(text) = child
            && text.content.trim().is_empty()
        {
            continue;
        }
        let loc = child.loc();
        if loc.span.start == loc.span.end {
            continue;
        }
        return Some(loc.span.start as usize..loc.span.end as usize);
    }
    None
}

fn find_usage_element<'a, 'template>(
    root: &'a RootNode<'template>,
    usage: &ComponentUsage,
) -> Option<&'a ElementNode<'template>> {
    find_usage_element_in_children(root.children.iter(), usage)
}

fn find_usage_element_in_children<'a, 'template, I>(
    children: I,
    usage: &ComponentUsage,
) -> Option<&'a ElementNode<'template>>
where
    'template: 'a,
    I: IntoIterator<Item = &'a TemplateChildNode<'template>>,
{
    for child in children {
        match child {
            TemplateChildNode::Element(element) => {
                if element.loc.span.start == usage.start && element.loc.span.end == usage.end {
                    return Some(element);
                }
                if let Some(found) = find_usage_element_in_children(element.children.iter(), usage)
                {
                    return Some(found);
                }
            }
            TemplateChildNode::If(node) => {
                for branch in &node.branches {
                    if let Some(found) =
                        find_usage_element_in_children(branch.children.iter(), usage)
                    {
                        return Some(found);
                    }
                }
            }
            TemplateChildNode::IfBranch(branch) => {
                if let Some(found) = find_usage_element_in_children(branch.children.iter(), usage) {
                    return Some(found);
                }
            }
            TemplateChildNode::For(node) => {
                if let Some(found) = find_usage_element_in_children(node.children.iter(), usage) {
                    return Some(found);
                }
            }
            TemplateChildNode::Text(_)
            | TemplateChildNode::Comment(_)
            | TemplateChildNode::Interpolation(_)
            | TemplateChildNode::TextCall(_)
            | TemplateChildNode::CompoundExpression(_)
            | TemplateChildNode::Hoisted(_) => {}
        }
    }
    None
}
