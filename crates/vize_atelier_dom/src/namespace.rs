//! HTML/SVG/MathML namespace resolution for the DOM platform.

use vize_atelier_core::Namespace;

/// Get the namespace for an element based on its parent.
///
/// Mirrors the HTML tree-construction namespace rules used by `@vue/compiler-dom`: a tag
/// that names a foreign root (`<svg>`/`<math>`) always (re)enters that namespace, otherwise
/// the element inherits its parent's namespace — except across the HTML integration points
/// where SVG/MathML hand their descendants back to the HTML namespace.
pub(crate) fn get_namespace(tag: &str, parent: Option<&str>) -> Namespace {
    if vize_carton::is_svg_tag(tag) {
        return Namespace::Svg;
    }
    if vize_carton::is_math_ml_tag(tag) {
        return Namespace::MathMl;
    }

    // Inherit namespace from the parent, honouring the integration-point boundaries.
    if let Some(parent_tag) = parent {
        // Inside SVG, <foreignObject>/<desc>/<title> switch their descendants back to HTML
        // (e.g. a <div> inside <foreignObject> must NOT be in the SVG namespace).
        let svg_to_html = matches!(parent_tag, "foreignObject" | "desc" | "title");
        if vize_carton::is_svg_tag(parent_tag) && !svg_to_html {
            return Namespace::Svg;
        }
        // Inside MathML, <annotation-xml> and the text containers (<mi>/<mo>/<mn>/<ms>/
        // <mtext>) are HTML integration points; their descendants are HTML.
        let mathml_to_html = matches!(
            parent_tag,
            "annotation-xml" | "mi" | "mo" | "mn" | "ms" | "mtext"
        );
        if vize_carton::is_math_ml_tag(parent_tag) && !mathml_to_html {
            return Namespace::MathMl;
        }
    }

    Namespace::Html
}
