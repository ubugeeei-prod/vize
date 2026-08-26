//! The legacy half of the surface projection ([`super::surface`]): the
//! transformed legacy tree walked into the shared owner shape.
//!
//! The legacy transform expands `v-model` in place (component models
//! become their `modelValue`/`onUpdate:` product props at the
//! directive's own source span; native models keep the directive and
//! gain the update handler at the same span), so the collector
//! **reconstructs** the authored contract from the products: authored
//! props never share a source span, product groups always do, and the
//! modifiers product rides at the stub span. Wrapper elements the S2
//! lowering unwraps (`<template v-if>` branch carriers, `<template
//! v-for>` wrappers) are skipped as owners with their leftover props
//! counted (`wrapper_attrs`).

use vize_atelier_core::{
    DirectiveNode, ElementNode, ElementType, ExpressionNode, PropNode, TemplateChildNode,
};
use vize_carton::String;

use super::surface::{PBind, PDirective, PModel, PName, PSurface, SurfaceCounters};
use super::surface_old_help::{
    arg_name, classifier_ambiguous, exp_text, exp_text_of, is_builtin_name, is_excluded_builtin,
    product_modifiers, slot_scope,
};

/// The surface of one kept owner. `outlet` excludes the consumed name
/// position by the S2 lowering's own selection rule, mirrored;
/// `excluded_key_span` is the branch key `extract_key_prop` removed
/// (already absent from the props — passed for documentation value
/// only).
fn owner_surface(el: &ElementNode<'_>, counters: &mut SurfaceCounters) -> PSurface {
    let outlet = el.tag == "slot";
    let component = el.tag_type == ElementType::Component;
    let mut surface = PSurface::default();

    // Product detection: spans of every directive prop; a `bind`/`on`
    // sharing its span with another directive prop is a model product.
    let spans: Vec<(u32, u32)> = el
        .props
        .iter()
        .filter_map(|prop| match prop {
            PropNode::Directive(dir) => Some((dir.loc.span.start, dir.loc.span.end)),
            PropNode::Attribute(_) => None,
        })
        .collect();
    let span_shared = |dir: &DirectiveNode<'_>| {
        let span = (dir.loc.span.start, dir.loc.span.end);
        spans.iter().filter(|other| **other == span).count() >= 2
    };
    let stub_span = |dir: &DirectiveNode<'_>| dir.loc.span.start == 0 && dir.loc.span.end == 0;

    let mut name_consumed = false;
    let mut skip: Vec<usize> = Vec::new();
    for (index, prop) in el.props.iter().enumerate() {
        if skip.contains(&index) {
            continue;
        }
        match prop {
            PropNode::Attribute(attr) => {
                if outlet && attr.name == "name" && !name_consumed {
                    name_consumed = true;
                    continue;
                }
                surface.attrs.push((
                    String::from(attr.name),
                    attr.value.as_ref().map(|value| String::from(value.content)),
                ));
            }
            PropNode::Directive(dir) => match dir.name {
                "slot" => {}
                "model" => {
                    let prop = match (component, dir.arg.as_ref()) {
                        (true, None) => Some(PName::Static(String::from("modelValue"))),
                        (_, Some(_)) => Some(arg_name(dir)),
                        (false, None) => None,
                    };
                    surface.models.push(PModel {
                        value: dir.exp.as_ref().and_then(exp_text),
                        prop,
                        mods: dir
                            .modifiers
                            .iter()
                            .map(|m| String::from(m.content))
                            .collect(),
                        component,
                    });
                }
                "bind" if span_shared(dir) => {
                    // A component model's product pair: the bind carries
                    // the contract, the on at the same span is consumed
                    // with it, and a stub-span modifiers bind may follow.
                    let mut mods = Vec::new();
                    for (later_index, later) in el.props.iter().enumerate().skip(index + 1) {
                        let PropNode::Directive(later_dir) = later else {
                            continue;
                        };
                        if later_dir.name == "on"
                            && span_shared(later_dir)
                            && later_dir.loc.span == dir.loc.span
                        {
                            skip.push(later_index);
                            // The optional modifiers product follows the
                            // pair directly.
                            if let Some(PropNode::Directive(mods_dir)) =
                                el.props.get(later_index + 1)
                                && mods_dir.name == "bind"
                                && stub_span(mods_dir)
                            {
                                skip.push(later_index + 1);
                                if let Some(exp) = mods_dir.exp.as_ref() {
                                    mods = product_modifiers(exp);
                                }
                            }
                            break;
                        }
                    }
                    let prop = dir.arg.as_ref().map(|_| arg_name(dir));
                    surface.models.push(PModel {
                        value: dir.exp.as_ref().and_then(exp_text),
                        prop,
                        mods,
                        component,
                    });
                }
                "on" if span_shared(dir) => {
                    // A native model's inserted update handler (its span
                    // is the kept model directive's) — realization, not
                    // surface.
                }
                "bind" => {
                    if outlet
                        && !name_consumed
                        && matches!(
                            dir.arg.as_ref(),
                            Some(ExpressionNode::Simple(exp)) if exp.is_static && exp.content == "name"
                        )
                    {
                        // The S2 selection rule mirrored: an authored
                        // non-blank value consumes the name; a valueless
                        // or blank spelling is the `drop.slot-name-hole`
                        // agreement (excluded, name still open).
                        let authored_nonblank = !dir.shorthand
                            && dir
                                .exp
                                .as_ref()
                                .and_then(exp_text_of)
                                .is_some_and(|text| !text.is_empty());
                        if authored_nonblank {
                            name_consumed = true;
                        }
                        continue;
                    }
                    surface.binds.push(PBind {
                        name: arg_name(dir),
                        mods: dir
                            .modifiers
                            .iter()
                            .map(|m| String::from(m.content))
                            .collect(),
                        value: dir.exp.as_ref().map(|exp| exp_text_of(exp)),
                    });
                }
                "on" => surface.ons.push(PBind {
                    name: match dir.arg.as_ref() {
                        None => PName::Spread,
                        Some(arg) => match arg_name(dir) {
                            PName::Spread => PName::Dynamic(exp_text_of(arg)),
                            name => name,
                        },
                    },
                    mods: dir
                        .modifiers
                        .iter()
                        .map(|m| String::from(m.content))
                        .collect(),
                    value: dir.exp.as_ref().map(|exp| exp_text_of(exp)),
                }),
                name if is_excluded_builtin(name) => counters.builtins_excluded += 1,
                name if !is_builtin_name(name) => surface.directives.push(PDirective {
                    name: String::from(name),
                    arg: dir.arg.as_ref().map(|_| arg_name(dir)),
                    mods: dir
                        .modifiers
                        .iter()
                        .map(|m| String::from(m.content))
                        .collect(),
                    value: dir.exp.as_ref().map(|exp| exp_text_of(exp)),
                }),
                _ => {}
            },
        }
    }
    surface
}

/// Collect every kept owner under `children`, outer before nested,
/// document order — the same order the S2 collector walks.
pub fn collect_surfaces(
    children: &[TemplateChildNode<'_>],
    pattern_scoped: bool,
    out: &mut Vec<PSurface>,
    counters: &mut SurfaceCounters,
) {
    for child in children {
        match child {
            TemplateChildNode::Element(el) => {
                let mut surface = owner_surface(el, counters);
                surface.pattern_scoped =
                    pattern_scoped || (classifier_ambiguous(el) && !surface.models.is_empty());
                out.push(surface);
                let child_pattern = match slot_scope(el) {
                    Some(pattern) => pattern_scoped || pattern,
                    None => pattern_scoped,
                };
                collect_surfaces(&el.children, child_pattern, out, counters);
            }
            TemplateChildNode::If(node) => {
                for branch in node.branches.iter() {
                    match &branch.children[..] {
                        [TemplateChildNode::Element(el)] if branch.is_template_if => {
                            counters.wrapper_attrs += el.props.len() as u64;
                            collect_surfaces(&el.children, pattern_scoped, out, counters);
                        }
                        _ => collect_surfaces(&branch.children, pattern_scoped, out, counters),
                    }
                }
            }
            TemplateChildNode::IfBranch(branch) => {
                collect_surfaces(&branch.children, pattern_scoped, out, counters);
            }
            TemplateChildNode::For(node) => match &node.children[..] {
                [TemplateChildNode::Element(el)] if el.tag == "template" => {
                    counters.wrapper_attrs += el.props.len() as u64;
                    collect_surfaces(&el.children, pattern_scoped, out, counters);
                }
                _ => collect_surfaces(&node.children, pattern_scoped, out, counters),
            },
            _ => {}
        }
    }
}
