//! Native attribute names are excess-checked when `vueCompilerOptions.target`
//! is omitted. A resolved Vue 3 target leaves those names unchecked.

use crate::virtual_ts::VizeMapping;
use vize_carton::{String, append, is_native_tag};
use vize_relief::{ExpressionNode, PropNode, RootNode, TemplateChildNode};

pub(super) fn emit(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    root: &RootNode<'_>,
    offset: u32,
    vue3_compiler_target: bool,
) {
    for child in &root.children {
        visit(ts, mappings, child, offset, vue3_compiler_target);
    }
}

fn visit(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    child: &TemplateChildNode<'_>,
    offset: u32,
    vue3_compiler_target: bool,
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
    // Vue 3 accepts intrinsic names that `NativeElements` does not list.
    // An omitted target still excess-checks the name (TS2353 on the key).
    // Assigning the name to `never` was TS2322 and anchored the binding note
    // on a neighboring attribute.
    if is_native_tag(element.tag) && !fragment && !vue3_compiler_target {
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
            append!(
                *ts,
                "  const __vize_native_key_{}: Partial<__VizeNativeElement<{tag}>> = {{ ",
                location.span.start
            );
            let key_start = ts.len();
            ts.push_str(key.as_str());
            let key_end = ts.len();
            append!(
                *ts,
                ": undefined as never }};\n  void __vize_native_key_{};\n",
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
    for child in &element.children {
        visit(ts, mappings, child, offset, vue3_compiler_target);
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
