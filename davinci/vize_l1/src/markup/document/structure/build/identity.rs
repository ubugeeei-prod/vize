//! Namespace-aware element identity and foreign-content integration points.

use vize_l0::{MATH_TAGS, Namespace, SVG_TAGS};

use super::Builder;
use crate::markup::Profile;

impl<P: Profile> Builder<'_, '_, P> {
    pub(super) fn start_identity(&self, raw: &str) -> Option<(&'static str, Namespace)> {
        let Some(parent) = self.current_element() else {
            return html_name::<P>(raw).map(|name| (name, Namespace::Html));
        };
        match parent.namespace {
            Namespace::Html => foreign_root::<P>(raw)
                .or_else(|| html_name::<P>(raw).map(|name| (name, Namespace::Html))),
            Namespace::Svg if svg_html_integration(parent.name) => foreign_root::<P>(raw)
                .or_else(|| html_name::<P>(raw).map(|name| (name, Namespace::Html))),
            Namespace::Svg => svg_name::<P>(raw).map(|name| (name, Namespace::Svg)),
            Namespace::MathMl if math_html_integration(parent.name, raw) => foreign_root::<P>(raw)
                .or_else(|| html_name::<P>(raw).map(|name| (name, Namespace::Html))),
            Namespace::MathMl => math_name::<P>(raw).map(|name| (name, Namespace::MathMl)),
        }
    }

    pub(super) fn end_identity(&self, raw: &str) -> Option<(&'static str, Namespace)> {
        match self.current_namespace().unwrap_or(Namespace::Html) {
            Namespace::Html => html_name::<P>(raw).map(|name| (name, Namespace::Html)),
            Namespace::Svg => svg_name::<P>(raw).map(|name| (name, Namespace::Svg)),
            Namespace::MathMl => math_name::<P>(raw).map(|name| (name, Namespace::MathMl)),
        }
    }
}

const HTML_NAMES: [&str; 88] = [
    "html",
    "head",
    "body",
    "base",
    "link",
    "meta",
    "title",
    "style",
    "script",
    "textarea",
    "a",
    "abbr",
    "b",
    "bdi",
    "bdo",
    "big",
    "cite",
    "code",
    "data",
    "dfn",
    "em",
    "font",
    "i",
    "kbd",
    "mark",
    "nobr",
    "q",
    "s",
    "samp",
    "small",
    "strike",
    "strong",
    "sub",
    "sup",
    "time",
    "tt",
    "u",
    "var",
    "div",
    "span",
    "br",
    "hr",
    "img",
    "input",
    "area",
    "embed",
    "source",
    "track",
    "wbr",
    "article",
    "aside",
    "footer",
    "header",
    "main",
    "nav",
    "section",
    "address",
    "blockquote",
    "figure",
    "figcaption",
    "hgroup",
    "search",
    "p",
    "ul",
    "ol",
    "menu",
    "li",
    "dl",
    "dt",
    "dd",
    "ruby",
    "rb",
    "rt",
    "rtc",
    "rp",
    "option",
    "optgroup",
    "select",
    "table",
    "caption",
    "colgroup",
    "col",
    "tbody",
    "thead",
    "tfoot",
    "tr",
    "td",
    "th",
];

fn html_name<P: Profile>(raw: &str) -> Option<&'static str> {
    HTML_NAMES
        .into_iter()
        .find(|name| profile_name_eq::<P>(raw, name))
}

fn svg_name<P: Profile>(raw: &str) -> Option<&'static str> {
    SVG_TAGS
        .iter()
        .copied()
        .find(|name| profile_name_eq::<P>(raw, name))
}

fn math_name<P: Profile>(raw: &str) -> Option<&'static str> {
    MATH_TAGS
        .iter()
        .copied()
        .find(|name| profile_name_eq::<P>(raw, name))
}

fn foreign_root<P: Profile>(raw: &str) -> Option<(&'static str, Namespace)> {
    if profile_name_eq::<P>(raw, "svg") {
        Some(("svg", Namespace::Svg))
    } else if profile_name_eq::<P>(raw, "math") {
        Some(("math", Namespace::MathMl))
    } else {
        None
    }
}

fn profile_name_eq<P: Profile>(raw: &str, expected: &str) -> bool {
    if P::FOLD_NAME_CASE {
        raw.eq_ignore_ascii_case(expected)
    } else {
        raw == expected
    }
}

pub(super) fn svg_html_integration(name: &str) -> bool {
    matches!(name, "foreignObject" | "desc" | "title")
}

pub(super) fn math_html_integration(parent: &str, raw: &str) -> bool {
    matches!(parent, "mi" | "mo" | "mn" | "ms" | "mtext")
        && !raw.eq_ignore_ascii_case("mglyph")
        && !raw.eq_ignore_ascii_case("malignmark")
}
