//! DOM tag configuration shared between compiler-dom and runtime-dom.

use phf::phf_set;

/// HTML tags
/// https://developer.mozilla.org/en-US/docs/Web/HTML/Element
pub static HTML_TAGS: phf::Set<&'static str> = phf_set! {
    "html", "body", "base", "head", "link", "meta", "style", "title",
    "address", "article", "aside", "footer", "header", "hgroup",
    "h1", "h2", "h3", "h4", "h5", "h6", "nav", "section",
    "div", "dd", "dl", "dt", "figcaption", "figure", "picture", "hr",
    "img", "li", "main", "ol", "p", "pre", "ul",
    "a", "b", "abbr", "bdi", "bdo", "br", "cite", "code", "data", "dfn",
    "em", "i", "kbd", "mark", "q", "rp", "rt", "ruby", "s", "samp",
    "small", "span", "strong", "sub", "sup", "time", "u", "var", "wbr",
    "area", "audio", "map", "track", "video", "embed", "object", "param",
    "source", "canvas", "script", "noscript", "del", "ins",
    "caption", "col", "colgroup", "table", "thead", "tbody", "td", "th", "tr",
    "button", "datalist", "fieldset", "form", "input", "label", "legend",
    "meter", "optgroup", "option", "output", "progress", "select", "textarea",
    "details", "dialog", "menu", "search", "summary", "template", "blockquote",
    "iframe", "tfoot"
};

/// SVG tags
/// https://developer.mozilla.org/en-US/docs/Web/SVG/Element
pub static SVG_TAGS: phf::Set<&'static str> = phf_set! {
    "svg", "animate", "animateMotion", "animateTransform", "circle",
    "clipPath", "color-profile", "defs", "desc", "discard", "ellipse",
    "feBlend", "feColorMatrix", "feComponentTransfer", "feComposite",
    "feConvolveMatrix", "feDiffuseLighting", "feDisplacementMap",
    "feDistantLight", "feDropShadow", "feFlood", "feFuncA", "feFuncB",
    "feFuncG", "feFuncR", "feGaussianBlur", "feImage", "feMerge",
    "feMergeNode", "feMorphology", "feOffset", "fePointLight",
    "feSpecularLighting", "feSpotLight", "feTile", "feTurbulence",
    "filter", "foreignObject", "g", "hatch", "hatchpath", "image",
    "line", "linearGradient", "marker", "mask", "mesh", "meshgradient",
    "meshpatch", "meshrow", "metadata", "mpath", "path", "pattern",
    "polygon", "polyline", "radialGradient", "rect", "set", "solidcolor",
    "stop", "switch", "symbol", "text", "textPath", "title", "tspan",
    "unknown", "use", "view"
};

/// MathML tags
/// https://www.w3.org/TR/mathml4/ (content elements excluded)
pub static MATH_TAGS: phf::Set<&'static str> = phf_set! {
    "annotation", "annotation-xml", "maction", "maligngroup", "malignmark",
    "math", "menclose", "merror", "mfenced", "mfrac", "mfraction", "mglyph",
    "mi", "mlabeledtr", "mlongdiv", "mmultiscripts", "mn", "mo", "mover",
    "mpadded", "mphantom", "mprescripts", "mroot", "mrow", "ms", "mscarries",
    "mscarry", "msgroup", "msline", "mspace", "msqrt", "msrow", "mstack",
    "mstyle", "msub", "msubsup", "msup", "mtable", "mtd", "mtext", "mtr",
    "munder", "munderover", "none", "semantics"
};

/// Void (self-closing) tags
pub static VOID_TAGS: phf::Set<&'static str> = phf_set! {
    "area", "base", "br", "col", "embed", "hr", "img", "input",
    "link", "meta", "param", "source", "track", "wbr"
};

/// Check if tag is a valid HTML tag
#[inline]
pub fn is_html_tag(tag: &str) -> bool {
    HTML_TAGS.contains(tag)
}

/// Check if tag is a valid SVG tag
#[inline]
pub fn is_svg_tag(tag: &str) -> bool {
    SVG_TAGS.contains(tag)
}

/// Check if tag is a valid MathML tag
#[inline]
pub fn is_math_ml_tag(tag: &str) -> bool {
    MATH_TAGS.contains(tag)
}

/// Check if tag is a void (self-closing) tag
#[inline]
pub fn is_void_tag(tag: &str) -> bool {
    VOID_TAGS.contains(tag)
}

/// Check if tag is a native tag (HTML, SVG, or MathML)
#[inline]
pub fn is_native_tag(tag: &str) -> bool {
    is_html_tag(tag) || is_svg_tag(tag) || is_math_ml_tag(tag)
}

/// Special tags that contain raw text
pub static RAW_TEXT_TAGS: phf::Set<&'static str> = phf_set! {
    "style", "script", "textarea", "title"
};

/// Check if tag contains raw text
#[inline]
pub fn is_raw_text_tag(tag: &str) -> bool {
    RAW_TEXT_TAGS.contains(tag)
}

/// RCDATA tags (parsed character data)
pub static RCDATA_TAGS: phf::Set<&'static str> = phf_set! {
    "textarea", "title"
};

/// Check if tag is an RCDATA tag
#[inline]
pub fn is_rcdata_tag(tag: &str) -> bool {
    RCDATA_TAGS.contains(tag)
}

#[cfg(test)]
mod tests {
    use super::{is_html_tag, is_raw_text_tag, is_svg_tag, is_void_tag};

    #[test]
    fn test_html_tags() {
        assert!(is_html_tag("div"));
        assert!(is_html_tag("search"));
        assert!(is_html_tag("span"));
        assert!(is_html_tag("template"));
        assert!(!is_html_tag("custom-element"));
    }

    #[test]
    fn test_svg_tags() {
        assert!(is_svg_tag("svg"));
        assert!(is_svg_tag("path"));
        assert!(is_svg_tag("circle"));
        assert!(!is_svg_tag("div"));
    }

    #[test]
    fn test_void_tags() {
        assert!(is_void_tag("br"));
        assert!(is_void_tag("img"));
        assert!(is_void_tag("input"));
        assert!(!is_void_tag("div"));
    }

    #[test]
    fn test_raw_text_tags() {
        assert!(is_raw_text_tag("script"));
        assert!(is_raw_text_tag("style"));
        assert!(!is_raw_text_tag("div"));
    }
}
