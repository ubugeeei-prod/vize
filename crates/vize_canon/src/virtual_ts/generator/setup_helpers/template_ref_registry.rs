//! Per-SFC registry of static template `ref="name"` attributes (#3896).
//!
//! `useTemplateRef("box")` must type as the target the template registers
//! under that name — `Readonly<ShallowRef<HTMLDivElement | null>>` for a
//! `<div ref="box">`, or the child's public instance for
//! `<Child ref="box">`. Otherwise dereferencing before mount, or reading a
//! private child setup binding, is silently approved where vue-tsc reports.
//!
//! The registry follows the Vue toolchain: a ref inside `v-for` holds an array
//! of its targets, a name registered more than once holds the union of them,
//! and a literal name the template never registers is an error. Only a dynamic
//! `:ref` binding stays outside it.

use vize_carton::{FxHashSet, String, append, is_native_tag};
use vize_croquis::Croquis;
use vize_relief::{ElementNode, ElementType, Namespace, PropNode, RootNode, TemplateChildNode};

use crate::virtual_ts::{
    component_reference::{component_binding_reference, resolved_component_binding_reference},
    types::VirtualTsOptions,
};

use super::push_ts_string_literal;

/// One registry entry: the authored ref name and the element tag it names.
struct RegisteredRef {
    name: String,
    kind: RegisteredRefKind,
    /// Declared inside `v-for`: the ref holds one target per iteration.
    in_v_for: bool,
}

pub(super) struct TemplateRefRegistry {
    pub(super) body: String,
    pub(super) includes_dom_element: bool,
    pub(super) includes_component: bool,
}

enum RegisteredRefKind {
    Element {
        tag: String,
        /// SVG elements resolve through `SVGElementTagNameMap` first. The parser
        /// already propagates the namespace to descendants, so a nested
        /// `<svg><a ref="link" /></svg>` is distinguishable from a top-level
        /// `<a ref="link" />` even though both register the tag `a`.
        is_svg: bool,
    },
    Component {
        reference: String,
    },
}

/// The rendered `__VizeTemplateRefs` object-type body, or `None` when no
/// static plain-element ref exists (generation then keeps the untyped shim).
///
/// Retyping the shim is the registry's only route to a diagnostic, so a setup
/// scope that never names `useTemplateRef` cannot observe it: skip both the
/// collection walk and the extra type declarations there rather than make
/// every SFC with a `ref="name"` attribute pay for them.
pub(super) fn template_ref_registry(
    summary: &Croquis,
    options: &VirtualTsOptions,
    script_content: Option<&str>,
    template_ast: Option<&RootNode<'_>>,
    syntactic_type_only_imported_names: &FxHashSet<vize_carton::CompactString>,
) -> Option<TemplateRefRegistry> {
    if !script_content.is_some_and(|script| script.contains("useTemplateRef")) {
        return None;
    }
    let root = template_ast?;
    let mut refs: Vec<RegisteredRef> = Vec::new();
    for child in root.children.iter() {
        collect(
            child,
            false,
            summary,
            options,
            syntactic_type_only_imported_names,
            &mut refs,
        );
    }
    if refs.is_empty() {
        return None;
    }

    // A name registered more than once holds whichever target is mounted, so
    // its type is the union of them, in template order.
    let mut names: Vec<&str> = Vec::new();
    for entry in &refs {
        if !names.contains(&entry.name.as_str()) {
            names.push(entry.name.as_str());
        }
    }

    let mut body = String::default();
    let mut includes_dom_element = false;
    let mut includes_component = false;
    for name in names {
        // Both values are authored text, so they are escaped as TypeScript
        // string literals: a raw `\` in `ref="path\name"` would otherwise open
        // an escape sequence and silently key the registry under a different
        // name, and a trailing one would invalidate the whole virtual file.
        let mut name_literal = String::default();
        push_ts_string_literal(&mut name_literal, name);
        append!(body, " {name_literal}: ");
        for (index, entry) in refs.iter().filter(|entry| entry.name == name).enumerate() {
            if index > 0 {
                body.push_str(" | ");
            }
            match &entry.kind {
                RegisteredRefKind::Element { tag, is_svg } => {
                    includes_dom_element = true;
                    let mut tag_literal = String::default();
                    push_ts_string_literal(&mut tag_literal, tag.as_str());
                    let svg_argument = if *is_svg { ", true" } else { "" };
                    append!(body, "__VizeDomElement<{tag_literal}{svg_argument}>");
                    if entry.in_v_for {
                        body.push_str("[]");
                    }
                }
                RegisteredRefKind::Component { reference } => {
                    includes_component = true;
                    if entry.in_v_for {
                        append!(
                            body,
                            "(__VizeTemplateComponentRef<typeof {reference}> | null)[]"
                        );
                    } else {
                        append!(body, "__VizeTemplateComponentRef<typeof {reference}>");
                    }
                }
            }
        }
        body.push(';');
    }
    if body.is_empty() {
        return None;
    }
    body.push(' ');
    Some(TemplateRefRegistry {
        body,
        includes_dom_element,
        includes_component,
    })
}

fn collect(
    node: &TemplateChildNode<'_>,
    in_v_for: bool,
    summary: &Croquis,
    options: &VirtualTsOptions,
    syntactic_type_only_imported_names: &FxHashSet<vize_carton::CompactString>,
    refs: &mut Vec<RegisteredRef>,
) {
    match node {
        TemplateChildNode::Element(element) => collect_element(
            element,
            in_v_for,
            summary,
            options,
            syntactic_type_only_imported_names,
            refs,
        ),
        TemplateChildNode::If(if_node) => {
            for branch in if_node.branches.iter() {
                for child in branch.children.iter() {
                    collect(
                        child,
                        in_v_for,
                        summary,
                        options,
                        syntactic_type_only_imported_names,
                        refs,
                    );
                }
            }
        }
        TemplateChildNode::For(for_node) => {
            for child in for_node.children.iter() {
                collect(
                    child,
                    true,
                    summary,
                    options,
                    syntactic_type_only_imported_names,
                    refs,
                );
            }
        }
        _ => {}
    }
}

fn collect_element(
    element: &ElementNode<'_>,
    in_v_for: bool,
    summary: &Croquis,
    options: &VirtualTsOptions,
    syntactic_type_only_imported_names: &FxHashSet<vize_carton::CompactString>,
    refs: &mut Vec<RegisteredRef>,
) {
    // Before the structural transform, an inline `v-for` is still a directive
    // on the element itself, not a wrapping `For` node.
    let in_v_for = in_v_for
        || element
            .props
            .iter()
            .any(|prop| matches!(prop, PropNode::Directive(directive) if directive.name == "for"));
    {
        for prop in element.props.iter() {
            let PropNode::Attribute(attribute) = prop else {
                continue;
            };
            if attribute.name != "ref" {
                continue;
            }
            let Some(value) = attribute.value.as_ref() else {
                continue;
            };
            let name = value.content;
            if !name.is_empty() {
                let kind = match element.tag_type {
                    ElementType::Element => component_ref_kind_for_element_tag(
                        summary,
                        options,
                        syntactic_type_only_imported_names,
                        element.ns,
                        element.tag,
                    )
                    .unwrap_or_else(|| RegisteredRefKind::Element {
                        tag: String::from(element.tag),
                        is_svg: matches!(element.ns, Namespace::Svg),
                    }),
                    ElementType::Component => RegisteredRefKind::Component {
                        reference: component_binding_reference(
                            summary,
                            options,
                            syntactic_type_only_imported_names,
                            element.tag,
                        ),
                    },
                    _ => continue,
                };
                refs.push(RegisteredRef {
                    name: String::from(name),
                    kind,
                    in_v_for,
                });
            }
        }
    }
    for child in element.children.iter() {
        collect(
            child,
            in_v_for,
            summary,
            options,
            syntactic_type_only_imported_names,
            refs,
        );
    }
}

fn component_ref_kind_for_element_tag(
    summary: &Croquis,
    options: &VirtualTsOptions,
    syntactic_type_only_imported_names: &FxHashSet<vize_carton::CompactString>,
    namespace: Namespace,
    tag: &str,
) -> Option<RegisteredRefKind> {
    if namespace != Namespace::Html {
        return None;
    }
    if is_native_tag(tag) {
        return None;
    }
    resolved_component_binding_reference(summary, options, syntactic_type_only_imported_names, tag)
        .map(|reference| RegisteredRefKind::Component { reference })
}

#[cfg(test)]
mod tests;
