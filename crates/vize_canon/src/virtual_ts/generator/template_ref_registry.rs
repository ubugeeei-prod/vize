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
use vize_relief::{ElementNode, ElementType, PropNode, RootNode, TemplateChildNode};

/// One registry entry: the authored ref name and the element tag it names.
struct RegisteredRef {
    name: String,
    tag: String,
}

/// The rendered `__VizeTemplateRefs` object-type body, or `None` when no
/// static plain-element ref exists (generation then keeps the untyped shim).
pub(super) fn template_ref_registry(template_ast: Option<&RootNode<'_>>) -> Option<String> {
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
        append!(
            body,
            " \"{}\": __VizeDomElement<\"{}\">;",
            entry.name,
            entry.tag
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
            if !name.is_empty() && !name.contains('"') && !element.tag.contains('"') {
                refs.push(RegisteredRef {
                    name: String::from(name),
                    tag: element.tag.clone(),
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
        let allocator = vize_carton::Bump::new();
        let (root, _) = vize_armature::parse(&allocator, template);
        super::template_ref_registry(Some(&root))
    }

    #[test]
    fn static_plain_element_refs_register_their_tag() {
        assert_eq!(
            registry_of(r#"<div ref="box" /><svg ref="pic" />"#).as_deref(),
            Some(r#" "box": __VizeDomElement<"div">; "pic": __VizeDomElement<"svg">; "#)
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
