//! HTML nesting the Vapor template string cannot carry.
//!
//! A `template()` string is parsed again by `innerHTML`, which re-parents
//! children the HTML parser does not accept where they were written (a `<div>`
//! in a `<p>`, an `<a>` in an `<a>`, a `<tr>` directly in a `<table>`).
//! `@vue/compiler-vapor` keeps the authored tree instead: such a child is
//! emitted as its own template and inserted at runtime. The rule is
//! `isValidHTMLNesting` from `@vue/compiler-dom`, which only looks at the
//! direct parent; this is the same table.

/// Whether `child` may be written directly inside `parent` in one template
/// string (`isValidHTMLNesting(parent, child)`).
#[inline]
pub(crate) fn is_valid_html_nesting(parent: &str, child: &str) -> bool {
    // Every rule names its parent or its child; most pairs name neither.
    !listed(parent, &PARENTS) && !listed(child, &CHILDREN) || rules(parent, child)
}

/// First letters of the tags a rule names as parent, by tag length.
const PARENTS: [u32; 9] = letters(&[
    "head", "optgroup", "select", "table", "tr", "colgroup", "tbody", "thead", "tfoot", "script",
    "iframe", "option", "textarea", "style", "title", "p", "svg",
]);

/// First letters of the tags a rule names as child, by tag length.
const CHILDREN: [u32; 11] = letters(&[
    "html",
    "body",
    "head",
    "td",
    "th",
    "colgroup",
    "caption",
    "tbody",
    "tfoot",
    "thead",
    "col",
    "tr",
    "dd",
    "dt",
    "figcaption",
    "summary",
    "area",
    "a",
    "button",
    "form",
    "li",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
]);

#[expect(
    clippy::indexing_slicing,
    reason = "constant evaluation rejects an index out of range at build time"
)]
const fn letters<const N: usize>(tags: &[&str]) -> [u32; N] {
    let mut masks = [0; N];
    let mut index = 0;
    while index < tags.len() {
        let tag = tags[index].as_bytes();
        masks[tag.len()] |= 1 << (tag[0] - b'a');
        index += 1;
    }
    masks
}

/// Whether some rule may name `tag`: false is exact, true asks [`rules`].
#[inline]
fn listed(tag: &str, masks: &[u32]) -> bool {
    let bytes = tag.as_bytes();
    match (masks.get(bytes.len()), bytes.first()) {
        (Some(mask), Some(&first)) => mask >> (first.wrapping_sub(b'a') & 31) & 1 != 0,
        _ => false,
    }
}

fn rules(parent: &str, child: &str) -> bool {
    if parent == "template" {
        return true;
    }
    if let Some(valid) = only_valid_children(parent, child) {
        return valid;
    }
    if let Some(valid) = only_valid_parents(parent, child) {
        return valid;
    }
    !known_invalid_child(parent, child) && !known_invalid_parent(parent, child)
}

/// Elements that accept only the listed children.
fn only_valid_children(parent: &str, child: &str) -> Option<bool> {
    Some(match parent {
        "head" => matches!(
            child,
            "base"
                | "basefront"
                | "bgsound"
                | "link"
                | "meta"
                | "title"
                | "noscript"
                | "noframes"
                | "style"
                | "script"
                | "template"
        ),
        "optgroup" => child == "option",
        "select" => matches!(child, "optgroup" | "option" | "hr"),
        "table" => matches!(child, "caption" | "colgroup" | "tbody" | "tfoot" | "thead"),
        "tr" => matches!(child, "td" | "th"),
        "colgroup" => child == "col",
        "tbody" | "thead" | "tfoot" => child == "tr",
        "script" | "iframe" | "option" | "textarea" | "style" | "title" => false,
        _ => return None,
    })
}

/// Elements that accept only the listed parents.
fn only_valid_parents(parent: &str, child: &str) -> Option<bool> {
    Some(match child {
        "html" => false,
        "body" | "head" => parent == "html",
        "td" | "th" => parent == "tr",
        "colgroup" | "caption" | "tbody" | "tfoot" | "thead" => parent == "table",
        "col" => parent == "colgroup",
        "tr" => matches!(parent, "tbody" | "thead" | "tfoot"),
        "dd" | "dt" => matches!(parent, "dl" | "div"),
        "figcaption" => parent == "figure",
        "summary" => parent == "details",
        "area" => parent == "map",
        _ => return None,
    })
}

/// Children the listed elements reject.
fn known_invalid_child(parent: &str, child: &str) -> bool {
    match parent {
        "p" => matches!(
            child,
            "address"
                | "article"
                | "aside"
                | "blockquote"
                | "center"
                | "details"
                | "dialog"
                | "dir"
                | "div"
                | "dl"
                | "fieldset"
                | "figure"
                | "footer"
                | "form"
                | "h1"
                | "h2"
                | "h3"
                | "h4"
                | "h5"
                | "h6"
                | "header"
                | "hgroup"
                | "hr"
                | "li"
                | "main"
                | "nav"
                | "menu"
                | "ol"
                | "p"
                | "pre"
                | "section"
                | "table"
                | "ul"
        ),
        "svg" => matches!(
            child,
            "b" | "blockquote"
                | "br"
                | "code"
                | "dd"
                | "div"
                | "dl"
                | "dt"
                | "em"
                | "embed"
                | "h1"
                | "h2"
                | "h3"
                | "h4"
                | "h5"
                | "h6"
                | "hr"
                | "i"
                | "img"
                | "li"
                | "menu"
                | "meta"
                | "ol"
                | "p"
                | "pre"
                | "ruby"
                | "s"
                | "small"
                | "span"
                | "strong"
                | "sub"
                | "sup"
                | "table"
                | "u"
                | "ul"
                | "var"
        ),
        _ => false,
    }
}

/// Parents the listed elements reject.
fn known_invalid_parent(parent: &str, child: &str) -> bool {
    match child {
        "a" | "button" | "form" | "li" => parent == child,
        "dd" | "dt" => matches!(parent, "dd" | "dt"),
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
            matches!(parent, "h1" | "h2" | "h3" | "h4" | "h5" | "h6")
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::is_valid_html_nesting;

    #[test]
    fn follows_the_compiler_dom_table() {
        for (parent, child, valid) in [
            ("template", "tr", true),
            ("p", "div", false),
            ("p", "span", true),
            ("p", "ul", false),
            ("p", "p", false),
            ("a", "a", false),
            ("a", "span", true),
            ("form", "form", false),
            ("li", "li", false),
            ("ul", "li", true),
            ("table", "tr", false),
            ("table", "tbody", true),
            ("tbody", "tr", true),
            ("tr", "td", true),
            ("div", "td", false),
            ("section", "dd", false),
            ("div", "dd", true),
            ("dd", "dt", false),
            ("h1", "h2", false),
            ("select", "div", false),
            ("select", "option", true),
            ("option", "span", false),
            ("svg", "p", false),
            ("svg", "circle", true),
            ("details", "summary", true),
            ("div", "summary", false),
            ("div", "div", true),
        ] {
            assert_eq!(
                is_valid_html_nesting(parent, child),
                valid,
                "<{child}> in <{parent}>"
            );
        }
    }
}
