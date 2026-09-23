//! Strict native attribute names are checked against the installed Vue types.

use crate::virtual_ts::VizeMapping;
use vize_carton::{String, append, is_native_tag};
use vize_relief::{ExpressionNode, PropNode, RootNode, TemplateChildNode};

pub(super) fn emit(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    root: &RootNode<'_>,
    offset: u32,
) {
    for child in &root.children {
        visit(ts, mappings, child, offset);
    }
}

fn visit(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    child: &TemplateChildNode<'_>,
    offset: u32,
) {
    let TemplateChildNode::Element(element) = child else {
        return;
    };
    let fragment = element.tag == "template" && element.props.iter().any(|prop| {
        matches!(prop,
            PropNode::Directive(d) if matches!(d.name, "for" | "if" | "else-if" | "else" | "slot")
        )
    });
    let slot_template = element
        .props
        .iter()
        .any(|prop| matches!(prop, PropNode::Directive(d) if d.name == "slot"));
    if fragment && !slot_template {
        emit_fragment_prop_names(ts, mappings, element, offset);
    }
    if is_native_tag(element.tag) && !fragment {
        for prop in &element.props {
            let (name, location) = match prop {
                PropNode::Attribute(attribute) => (attribute.name, &attribute.name_loc),
                PropNode::Directive(directive) if directive.name == "bind" => {
                    let Some(ExpressionNode::Simple(argument)) = &directive.arg else {
                        continue;
                    };
                    if !argument.is_static {
                        continue;
                    }
                    (argument.content, &argument.loc)
                }
                _ => continue,
            };
            // Vue accepts user data attributes independently of DOM typings.
            if name.starts_with("data-") {
                continue;
            }
            let (Ok(tag), Ok(key)) = (
                serde_json::to_string(element.tag),
                serde_json::to_string(name),
            ) else {
                continue;
            };
            let start = ts.len();
            append!(*ts, "  const __vize_native_key_{}", location.span.start);
            let name_end = ts.len();
            append!(
                *ts,
                ": unknown extends __VizeNativeElement<{tag}> ? unknown : {key} extends keyof __VizeNativeElement<{tag}> ? unknown : __VizeComponentAttrCamel<{key}> extends keyof __VizeNativeElement<{tag}> ? unknown : never = {key};\n"
            );
            mappings.push(VizeMapping {
                gen_range: start + 8..name_end,
                src_range: (offset + location.span.start) as usize
                    ..(offset + location.span.end) as usize,
                sub_spans: Vec::new(),
            });
            append!(*ts, "  void __vize_native_key_{};\n", location.span.start);
        }
    }
    for child in &element.children {
        visit(ts, mappings, child, offset);
    }
}

/// A `<template v-if>` / `<template v-for>` renders no element, so a prop
/// bound on it (other than the reserved `key`) is dropped at runtime.
/// `vue-tsc` checks that props object against the `<template>` element's own
/// attributes, so an unknown name is an excess property (TS2353) on the
/// authored attribute name; a real `<template>` attribute passes.
fn emit_fragment_prop_names(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    element: &vize_relief::ElementNode<'_>,
    offset: u32,
) {
    for prop in &element.props {
        let (name, location) = match prop {
            PropNode::Attribute(attribute) => (attribute.name, &attribute.name_loc),
            PropNode::Directive(directive) if directive.name == "bind" => {
                let Some(ExpressionNode::Simple(argument)) = &directive.arg else {
                    continue;
                };
                if !argument.is_static {
                    continue;
                }
                (argument.content, &argument.loc)
            }
            _ => continue,
        };
        if matches!(name, "key" | "ref") || name.starts_with("data-") {
            continue;
        }
        let Ok(key) = serde_json::to_string(name) else {
            continue;
        };
        append!(
            *ts,
            "  const __vize_fragment_prop_{}: Partial<__VizeNativeElement<\"template\">> = {{ ",
            location.span.start
        );
        let key_start = ts.len();
        ts.push_str(key.as_str());
        let key_end = ts.len();
        append!(
            *ts,
            ": undefined as never }};\n  void __vize_fragment_prop_{};\n",
            location.span.start
        );
        mappings.push(VizeMapping {
            gen_range: key_start..key_end,
            src_range: (offset + location.span.start) as usize
                ..(offset + location.span.end) as usize,
            sub_spans: Vec::new(),
        });
    }
}
