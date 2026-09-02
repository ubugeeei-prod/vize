//! Lowering JSX elements and fragments into [`ElementNode`]s.

use oxc_ast::ast::{JSXAttributeItem, JSXElement, JSXElementName, JSXFragment};
use oxc_span::{GetSpan, Span};
use vize_relief::{DirectiveNode, ElementNode, ElementType, PropNode};
use vize_s0::String;

use super::{Lowerer, name};

/// Vue's dynamic-component tag.
///
/// `<a.b.c/>` names a component **value**, not a component name, so it lowers
/// to `<component :is="a.b.c">`: the backends turn that into
/// `resolveDynamicComponent(a.b.c)`, which passes a non-string straight
/// through and so mounts exactly what `@vue/babel-plugin-jsx`'s
/// `createVNode(a.b.c, …)` mounts. Emitting the dotted path as a *name*
/// instead (`resolveComponent("a.b.c")`) looks up a component nobody
/// registered, which resolves to nothing at runtime (#3421).
const DYNAMIC_COMPONENT_TAG: &str = "component";

impl<'a, 'm, 's: 'a> Lowerer<'a, 'm, 's> {
    /// Lower a JSX element into an [`ElementNode`] (tag, kind, props, children).
    pub(crate) fn lower_element_node(&mut self, element: &JSXElement<'_>) -> ElementNode<'a> {
        let opening = &element.opening_element;
        self.reject_unsupported_namespace(&opening.name);
        let custom_element_tag =
            name::identifier_name(&opening.name).filter(|tag| self.is_babel_custom_element(tag));
        let bound_custom_element = custom_element_tag.is_some_and(|tag| {
            !vize_s0::is_html_tag(tag)
                && !vize_s0::is_svg_tag(tag)
                && self.is_bound_jsx_identifier(&opening.name)
        });
        // A member-expression or bound predicate match is a value, so it becomes
        // the `:is` binding of a dynamic component rather than a tag string.
        let expression_tag = name::expression_tag_span(&opening.name)
            .or_else(|| bound_custom_element.then(|| opening.name.span()));
        let tag = match expression_tag {
            Some(_) => String::from(DYNAMIC_COMPONENT_TAG),
            None => name::element_tag(&opening.name),
        };
        let loc = self.mapper().location(element.span);
        let mut node = ElementNode::new(self.bump(), self.bump().alloc_str(&tag), loc);
        let is_custom_element = custom_element_tag.is_some();
        if is_custom_element && !bound_custom_element {
            self.custom_element_spans
                .push((element.span.start, element.span.end));
        }
        node.tag_type = element_type(
            &opening.name,
            self.uses_babel_compat(),
            is_custom_element && !bound_custom_element,
        );
        node.is_self_closing = element.closing_element.is_none();
        // `v-models` and `v-slots` are component-only. Native mode classifies a
        // dashed lowercase tag as an intrinsic element here, but the DOM backend
        // still resolves it with `resolveComponent`; Babel compatibility
        // classifies it as a component during lowering, matching the plugin.
        let on_component = !is_custom_element
            && (node.tag_type == ElementType::Component || node.tag.contains('-'));
        let has_v_slots = opening.attributes.iter().any(|item| {
            matches!(item, JSXAttributeItem::Attribute(attr) if self.is_v_slots_attribute(attr))
        });
        node.props = self.lower_attributes(&opening.attributes, on_component);
        if let Some(span) = expression_tag {
            // First, so the emitted props object reads `<component :is="…" …>`
            // like the template spelling it stands for. Codegen filters `is`
            // out of the props object for a dynamic component.
            node.props.insert(0, self.is_binding(span));
        }
        // Components route through slot synthesis (object/render-prop children
        // become `<template v-slot>`s); intrinsic elements lower children
        // directly.
        if node.tag_type == ElementType::Component && !is_custom_element {
            self.lower_component_children_into(&mut node, &element.children, has_v_slots);
        } else {
            node.children = self.lower_element_children(&element.children);
        }
        // `v-slots` contributes slot templates, appended after the element's own
        // children so those still become the `default` slot when the slots object
        // does not name one (#3418).
        self.apply_v_slots(&mut node, &opening.attributes, on_component);
        node
    }

    /// Lower a JSX fragment (`<>...</>`) that has to become a **single** node:
    /// a `v-if` branch or a `v-for` body (`cond ? <><a/><b/></> : <c/>`).
    /// Fragments in a child list or a slot body are spliced into that list
    /// instead and never reach here.
    ///
    /// Lowered as an [`ElementType::Template`] node — Vize's IR for "these
    /// children, with no element of their own", i.e. the `<template v-if>` /
    /// `<template v-for>` shape every backend already handles, which the VDOM
    /// backend emits as `createElementBlock(Fragment, …, STABLE_FRAGMENT)`.
    /// Tagging it `Fragment` as a *component* instead produced
    /// `resolveComponent("Fragment")`, a component nobody registers, so the
    /// branch rendered nothing (#3421).
    pub(crate) fn lower_fragment_node(&mut self, fragment: &JSXFragment<'_>) -> ElementNode<'a> {
        let loc = self.mapper().location(fragment.span);
        let mut node = ElementNode::new(self.bump(), "template", loc);
        node.tag_type = ElementType::Template;
        node.children = self.lower_children(&fragment.children);
        node
    }

    /// `:is="<source text of span>"`, the dynamic-component binding standing in
    /// for a member-expression tag.
    fn is_binding(&self, span: Span) -> PropNode<'a> {
        let loc = self.mapper().location(span);
        let mut directive = DirectiveNode::new(self.bump(), "bind", loc);
        directive.arg = Some(self.static_expr("is", span));
        directive.exp = Some(self.dyn_expr(span));
        PropNode::Directive(self.boxed(directive))
    }

    /// Report a JSX tag carrying a namespace no backend can resolve.
    fn reject_unsupported_namespace(&mut self, name: &JSXElementName<'_>) {
        let Some((namespace, span)) = name::unsupported_namespace(name) else {
            return;
        };
        let tag = self.mapper().slice(span);
        self.reject_at(
            span,
            format_args!(
                "unsupported JSX tag namespace `{namespace}:`; only `svg:` and `math:` name a real \
                 element namespace, so `{tag}` would be emitted verbatim as a tag name nothing \
                 resolves (`@vue/babel-plugin-jsx` rejects every namespaced tag)"
            ),
        );
    }
}

fn element_type(
    name: &JSXElementName<'_>,
    babel_compat: bool,
    is_custom_element: bool,
) -> ElementType {
    if !is_custom_element && name::is_component(name, babel_compat) {
        ElementType::Component
    } else {
        ElementType::Element
    }
}
