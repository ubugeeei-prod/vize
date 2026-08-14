//! Per-SFC registry of static template `ref="name"` attributes (#3896).
//!
//! `useTemplateRef("box")` must type as the element the template registers
//! under that name — `Readonly<ShallowRef<HTMLDivElement | null>>` for a
//! `<div ref="box">` — or dereferencing before mount is silently approved
//! where vue-tsc reports `TS18047`. The registry maps each statically named
//! ref on a plain element to `__VizeNativeElement<tag>`; everything the
//! registry cannot pin keeps today's `any`, so nothing new is reported for
//! dynamic `:ref` bindings, component refs, or refs declared inside `v-for`
//! (where the runtime value is an array) — under-claiming can only remove a
//! false negative, never invent a diagnostic.

use vize_carton::{FxHashSet, String, append};
use vize_relief::{ElementNode, ElementType, Namespace, PropNode, RootNode, TemplateChildNode};

use super::push_ts_string_literal;

/// One registry entry: the authored ref name and the element tag it names.
struct RegisteredRef {
    name: String,
    tag: String,
    /// SVG elements resolve through `SVGElementTagNameMap` first. The parser
    /// already propagates the namespace to descendants, so a nested
    /// `<svg><a ref="link" /></svg>` is distinguishable from a top-level
    /// `<a ref="link" />` even though both register the tag `a`.
    is_svg: bool,
}

/// The rendered `__VizeTemplateRefs` object-type body, or `None` when no
/// static plain-element ref exists (generation then keeps the untyped shim).
///
/// Retyping the shim is the registry's only route to a diagnostic, so a setup
/// scope that never names `useTemplateRef` cannot observe it: skip both the
/// collection walk and the extra type declarations there rather than make
/// every SFC with a `ref="name"` attribute pay for them.
pub(super) fn template_ref_registry(
    script_content: Option<&str>,
    template_ast: Option<&RootNode<'_>>,
) -> Option<String> {
    if !script_content.is_some_and(|script| script.contains("useTemplateRef")) {
        return None;
    }
    let root = template_ast?;
    let mut refs: Vec<RegisteredRef> = Vec::new();
    for child in root.children.iter() {
        collect(child, false, &mut refs);
    }
    if refs.is_empty() {
        return None;
    }

    // A name registered twice stays out: Vue keeps the last mounted element,
    // conditional branches make that order type-invisible, and a wrong pin is
    // worse than the `any` it replaces.
    let mut seen = FxHashSet::default();
    let mut duplicated = FxHashSet::default();
    for entry in &refs {
        if !seen.insert(entry.name.clone()) {
            duplicated.insert(entry.name.clone());
        }
    }

    let mut body = String::default();
    for entry in refs
        .iter()
        .filter(|entry| !duplicated.contains(&entry.name))
    {
        // Both values are authored text, so they are escaped as TypeScript
        // string literals: a raw `\` in `ref="path\name"` would otherwise open
        // an escape sequence and silently key the registry under a different
        // name, and a trailing one would invalidate the whole virtual file.
        let mut name_literal = String::default();
        push_ts_string_literal(&mut name_literal, entry.name.as_str());
        let mut tag_literal = String::default();
        push_ts_string_literal(&mut tag_literal, entry.tag.as_str());
        let svg_argument = if entry.is_svg { ", true" } else { "" };
        append!(
            body,
            " {name_literal}: __VizeDomElement<{tag_literal}{svg_argument}>;"
        );
    }
    if body.is_empty() {
        return None;
    }
    body.push(' ');
    Some(body)
}

fn collect(node: &TemplateChildNode<'_>, in_v_for: bool, refs: &mut Vec<RegisteredRef>) {
    match node {
        TemplateChildNode::Element(element) => collect_element(element, in_v_for, refs),
        TemplateChildNode::If(if_node) => {
            for branch in if_node.branches.iter() {
                for child in branch.children.iter() {
                    collect(child, in_v_for, refs);
                }
            }
        }
        TemplateChildNode::For(for_node) => {
            for child in for_node.children.iter() {
                collect(child, true, refs);
            }
        }
        _ => {}
    }
}

fn collect_element(element: &ElementNode<'_>, in_v_for: bool, refs: &mut Vec<RegisteredRef>) {
    // Before the structural transform, an inline `v-for` is still a directive
    // on the element itself, not a wrapping `For` node.
    let in_v_for = in_v_for
        || element
            .props
            .iter()
            .any(|prop| matches!(prop, PropNode::Directive(directive) if directive.name == "for"));
    let is_plain_element = matches!(element.tag_type, ElementType::Element);
    if is_plain_element && !in_v_for {
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
            let name = value.content.as_str();
            if !name.is_empty() {
                refs.push(RegisteredRef {
                    name: String::from(name),
                    tag: element.tag.clone(),
                    is_svg: matches!(element.ns, Namespace::Svg),
                });
            }
        }
    }
    for child in element.children.iter() {
        collect(child, in_v_for, refs);
    }
}

#[cfg(test)]
mod tests {
    fn registry_of(template: &str) -> Option<vize_carton::String> {
        registry_for("const box = useTemplateRef('box')", template)
    }

    fn registry_for(script: &str, template: &str) -> Option<vize_carton::String> {
        let allocator = vize_carton::Allocator::new();
        let (root, _) = vize_armature::parse(&allocator, template);
        super::template_ref_registry(Some(script), Some(&root))
    }

    #[test]
    fn a_script_that_never_names_use_template_ref_registers_nothing() {
        assert_eq!(
            registry_for("const label = 'hi'", r#"<div ref="box" />"#),
            None
        );
    }

    #[test]
    fn static_plain_element_refs_register_their_tag() {
        assert_eq!(
            registry_of(r#"<div ref="box" /><svg ref="pic" />"#).as_deref(),
            Some(r#" "box": __VizeDomElement<"div">; "pic": __VizeDomElement<"svg", true>; "#)
        );
    }

    #[test]
    fn svg_descendants_resolve_through_the_svg_tag_map() {
        // `a`, `script`, `style` and `title` live in both DOM tag-name maps, so
        // the namespace is what separates `SVGAElement` from
        // `HTMLAnchorElement` here.
        assert_eq!(
            registry_of(r#"<a ref="html" /><svg><a ref="vector" /></svg>"#).as_deref(),
            Some(r#" "html": __VizeDomElement<"a">; "vector": __VizeDomElement<"a", true>; "#)
        );
        // `<foreignObject>` is an HTML integration point: its subtree is HTML
        // again, so the inner `<a>` must not claim `SVGAElement`.
        assert_eq!(
            registry_of(r#"<svg><foreignObject><a ref="escaped" /></foreignObject></svg>"#)
                .as_deref(),
            Some(r#" "escaped": __VizeDomElement<"a">; "#)
        );
        // An SVG-only tag seeds the namespace by itself here (the parser's
        // `foreign_namespace_for`), so it is the SVG branch that resolves an
        // unwrapped `<circle>`, never an SVG fallback inside the HTML branch:
        // a tag the parser did put in the HTML namespace stops at `Element`.
        assert_eq!(
            registry_of(r#"<circle ref="shape" />"#).as_deref(),
            Some(r#" "shape": __VizeDomElement<"circle", true>; "#)
        );
    }

    #[test]
    fn ref_names_are_escaped_as_typescript_string_literals() {
        // A raw backslash would open an escape sequence in the generated key
        // (`\n` becoming a newline), and a trailing one would invalidate the
        // virtual file and take every diagnostic for the SFC with it.
        assert_eq!(
            registry_of(r#"<div ref="path\name" />"#).as_deref(),
            Some(r#" "path\\name": __VizeDomElement<"div">; "#)
        );
    }

    #[test]
    fn unpinnable_refs_stay_out_of_the_registry() {
        // Dynamic name.
        assert_eq!(registry_of(r#"<div :ref="target" />"#), None);
        // Component ref: the value is an instance, not a DOM node.
        assert_eq!(registry_of(r#"<Child ref="child" />"#), None);
        // Inside v-for the runtime value is an array.
        assert_eq!(
            registry_of(r#"<li v-for="it in xs" :key="it" ref="rows" />"#),
            None
        );
        // A name registered twice is order-dependent at runtime; leave it any.
        assert_eq!(registry_of(r#"<div ref="dup" /><span ref="dup" />"#), None);
    }

    #[test]
    fn conditional_branches_still_register() {
        assert_eq!(
            registry_of(r#"<div v-if="a" ref="only" /><section v-else><p ref="deep" /></section>"#)
                .as_deref(),
            Some(r#" "only": __VizeDomElement<"div">; "deep": __VizeDomElement<"p">; "#)
        );
    }
}
