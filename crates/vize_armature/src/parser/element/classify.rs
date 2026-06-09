//! Element-type classification, component detection, and namespace resolution.

use vize_relief::ast::*;

use super::super::Parser;

impl<'a> Parser<'a> {
    pub(super) fn is_invalid_html_self_closing(&self, element: &ElementNode<'a>) -> bool {
        element.ns == Namespace::Html
            && element.tag_type == ElementType::Element
            && (!self.options.custom_renderer || vize_carton::is_html_tag(element.tag.as_str()))
            && !(self.options.is_void_tag)(element.tag.as_str())
    }

    /// Determine element type (element, component, slot, template)
    pub(in crate::parser) fn determine_element_type(
        &self,
        element: &ElementNode<'a>,
    ) -> ElementType {
        let tag = element.tag.as_str();

        // Check for slot
        if tag == "slot" {
            return ElementType::Slot;
        }

        // Check for template
        if tag == "template" {
            // Template with v-if, v-for, or v-slot is a template element
            let has_structural_directive = element.props.iter().any(|p| {
                matches!(p, PropNode::Directive(d) if matches!(d.name.as_str(), "if" | "else-if" | "else" | "for" | "slot"))
            });
            if has_structural_directive {
                return ElementType::Template;
            }
        }

        // Check if it's a component
        if self.is_component(tag) {
            return ElementType::Component;
        }

        ElementType::Element
    }

    /// Check if tag is a component
    pub(in crate::parser) fn is_component(&self, tag: &str) -> bool {
        // Core built-in components
        if matches!(
            tag,
            "Teleport"
                | "Suspense"
                | "KeepAlive"
                | "BaseTransition"
                | "Transition"
                | "TransitionGroup"
        ) {
            return true;
        }

        // Custom element check
        if let Some(is_custom) = self.options.is_custom_element
            && is_custom(tag)
        {
            return false;
        }

        if self.options.custom_renderer {
            return tag.chars().next().is_some_and(|c| c.is_uppercase()) || tag.contains('-');
        }

        // Native tag check
        if let Some(is_native) = self.options.is_native_tag {
            if !is_native(tag) {
                return true;
            }
        } else {
            // Default: check if starts with uppercase
            if tag.chars().next().is_some_and(|c| c.is_uppercase()) {
                return true;
            }
        }

        false
    }

    /// Resolve the foreign (SVG/MathML) namespace for a start tag whose
    /// configured `get_namespace` callback returned HTML. An `<svg>`/`<math>`
    /// root (or any SVG/MathML tag) seeds the namespace; otherwise descendants
    /// inherit the nearest open ancestor's foreign namespace unless that
    /// ancestor is an HTML integration point (`<foreignObject>`/`<desc>`/
    /// `<title>` for SVG, `<annotation-xml>` and the MathML text containers for
    /// MathML), which switch their subtree back to HTML. Mirrors the boundary
    /// handling in the DOM compiler's `get_namespace` so namespace-unaware
    /// callbacks still classify foreign elements correctly.
    pub(super) fn foreign_namespace_for(&self, tag: &str) -> Option<Namespace> {
        if vize_carton::is_svg_tag(tag) {
            return Some(Namespace::Svg);
        }
        if vize_carton::is_math_ml_tag(tag) {
            return Some(Namespace::MathMl);
        }

        let parent = self.stack.last()?;
        let parent_tag = parent.element.tag.as_str();
        match parent.element.ns {
            Namespace::Svg => {
                let svg_to_html = matches!(parent_tag, "foreignObject" | "desc" | "title");
                (!svg_to_html).then_some(Namespace::Svg)
            }
            Namespace::MathMl => {
                let mathml_to_html = matches!(
                    parent_tag,
                    "annotation-xml" | "mi" | "mo" | "mn" | "ms" | "mtext"
                );
                (!mathml_to_html).then_some(Namespace::MathMl)
            }
            Namespace::Html => None,
        }
    }

    pub(super) fn should_force_html_namespace(&self, tag: &str) -> bool {
        if !self.options.custom_renderer {
            return false;
        }

        if matches!(tag, "svg" | "math") {
            return false;
        }

        if self
            .stack
            .last()
            .is_some_and(|entry| matches!(entry.element.ns, Namespace::Svg | Namespace::MathMl))
        {
            return false;
        }

        tag.chars().next().is_some_and(|c| c.is_lowercase())
            && !tag.contains('-')
            && !vize_carton::is_html_tag(tag)
    }
}
