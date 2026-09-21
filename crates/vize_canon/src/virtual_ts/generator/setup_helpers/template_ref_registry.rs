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
    types::{VirtualTsCheckOptions, VirtualTsOptions},
};

use super::super::template_record::TemplateRecord;
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
    /// The same registry as `$refs` holds it: a component ref is `null` until
    /// its component is mounted, where `useTemplateRef` adds that itself.
    pub(super) dollar_body: String,
    pub(super) includes_dom_element: bool,
    pub(super) includes_component: bool,
    /// Whether a component ref holds the instance the template instantiates.
    pub(super) includes_instantiated: bool,
}

/// Whether anything reads the registry: `useTemplateRef` in the script, or a
/// `$refs` the project asked to be typed by it.
pub(in crate::virtual_ts::generator) fn registers_template_refs(
    script: Option<&str>,
    checks: VirtualTsCheckOptions,
) -> bool {
    checks.infer_template_dollar_refs
        || checks.infer_component_dollar_refs
        || script.is_some_and(|script| script.contains("useTemplateRef"))
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
        /// Template-relative start of the element, the identity of its usage.
        start: u32,
    },
}

/// The rendered `__VizeTemplateRefs` object-type body, or `None` when no
/// static plain-element ref exists (generation then keeps the untyped shim).
///
/// Retyping the shim and typing `$refs` are the registry's only routes to a
/// diagnostic, so a component that does neither cannot observe it: skip both
/// the collection walk and the extra type declarations there rather than make
/// every SFC with a `ref="name"` attribute pay for them.
pub(super) fn template_ref_registry(
    summary: &Croquis,
    options: &VirtualTsOptions,
    script_content: Option<&str>,
    template_ast: Option<&RootNode<'_>>,
    syntactic_type_only_imported_names: &FxHashSet<vize_carton::CompactString>,
    (checks, record): (VirtualTsCheckOptions, &TemplateRecord),
) -> Option<TemplateRefRegistry> {
    if !registers_template_refs(script_content, checks) {
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
    let mut dollar_body = String::default();
    let mut includes_dom_element = false;
    let mut includes_component = false;
    let mut includes_instantiated = false;
    for name in names {
        // Both values are authored text, so they are escaped as TypeScript
        // string literals: a raw `\` in `ref="path\name"` would otherwise open
        // an escape sequence and silently key the registry under a different
        // name, and a trailing one would invalidate the whole virtual file.
        let mut name_literal = String::default();
        push_ts_string_literal(&mut name_literal, name);
        append!(body, " {name_literal}: ");
        append!(dollar_body, " {name_literal}: ");
        for (index, entry) in refs.iter().filter(|entry| entry.name == name).enumerate() {
            if index > 0 {
                body.push_str(" | ");
                dollar_body.push_str(" | ");
            }
            let mut target = String::default();
            let nullable = match &entry.kind {
                RegisteredRefKind::Element { tag, is_svg } => {
                    includes_dom_element = true;
                    let mut tag_literal = String::default();
                    push_ts_string_literal(&mut tag_literal, tag.as_str());
                    let svg_argument = if *is_svg { ", true" } else { "" };
                    append!(target, "__VizeDomElement<{tag_literal}{svg_argument}>");
                    false
                }
                RegisteredRefKind::Component { reference, start } => {
                    includes_component = true;
                    let declared =
                        vize_carton::cstr!("__VizeTemplateComponentRef<typeof {reference}>");
                    target = record.ref_instance(*start, declared.as_str(), reference.as_str());
                    includes_instantiated |= target != declared;
                    true
                }
            };
            match (entry.in_v_for, nullable) {
                (true, true) => {
                    append!(body, "({target} | null)[]");
                    append!(dollar_body, "({target} | null)[]");
                }
                (true, false) => {
                    append!(body, "{target}[]");
                    append!(dollar_body, "{target}[]");
                }
                (false, nullable) => {
                    body.push_str(target.as_str());
                    dollar_body.push_str(target.as_str());
                    if nullable {
                        dollar_body.push_str(" | null");
                    }
                }
            }
        }
        body.push(';');
        dollar_body.push(';');
    }
    if body.is_empty() {
        return None;
    }
    body.push(' ');
    dollar_body.push(' ');
    Some(TemplateRefRegistry {
        body,
        dollar_body,
        includes_dom_element,
        includes_component,
        includes_instantiated,
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
                    .map(|reference| RegisteredRefKind::Component {
                        reference,
                        start: element.loc.span.start,
                    })
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
                        start: element.loc.span.start,
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
) -> Option<String> {
    if namespace != Namespace::Html {
        return None;
    }
    if is_native_tag(tag) {
        return None;
    }
    resolved_component_binding_reference(summary, options, syntactic_type_only_imported_names, tag)
}

#[cfg(test)]
mod tests;
