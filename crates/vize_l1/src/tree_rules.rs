//! Tree rules shared by the lexer scope recorder and surface builder.

use vize_l0::{is_html_tag, is_math_ml_tag, is_svg_tag};
use vize_relief::Namespace;

pub(crate) fn element_namespace(tag: &str, parent: Option<(Namespace, &str)>) -> Namespace {
    if is_svg_tag(tag) {
        Namespace::Svg
    } else if is_math_ml_tag(tag) {
        Namespace::MathMl
    } else {
        parent.map_or(Namespace::Html, |(ns, parent_tag)| {
            children_ns(ns, parent_tag)
        })
    }
}

pub(crate) fn is_interactive_html_tree_tag(tag: &str) -> bool {
    is_html_tag(tag) && (tag.eq_ignore_ascii_case("a") || tag.eq_ignore_ascii_case("button"))
}

fn children_ns(ns: Namespace, tag: &str) -> Namespace {
    match ns {
        Namespace::Svg if matches!(tag, "foreignObject" | "desc" | "title") => Namespace::Html,
        Namespace::MathMl if matches!(tag, "mi" | "mo" | "mn" | "ms" | "mtext") => Namespace::Html,
        other => other,
    }
}
