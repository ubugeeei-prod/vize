//! vue/permitted-contents
//!
//! Detect HTML content model violations. Based on markuplint's `permitted-contents` rule.
//!
//! Checks for:
//! 1. **Block in inline**: Block elements inside phrasing-only parents (e.g., `<div>` in `<p>`)
//! 2. **Interactive nesting**: Interactive elements nested inside other interactive elements
//! 3. **List content model**: Direct children of `<ul>`/`<ol>` must be `<li>`
//! 4. **Table content model**: `<table>` children must be valid table elements
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <template>
//!   <p><div>block in inline</div></p>
//!   <a href="#"><a href="#">nested link</a></a>
//!   <ul><div>not a list item</div></ul>
//! </template>
//! ```
//!
//! ### Valid
//! ```vue
//! <template>
//!   <p><span>inline in inline</span></p>
//!   <ul><li>list item</li></ul>
//! </template>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_carton::is_html_tag;
use vize_relief::{ElementNode, ElementType};

static META: RuleMeta = RuleMeta {
    name: "vue/permitted-contents",
    description: "Enforce HTML content model rules",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Elements that only permit phrasing (inline) content
const PHRASING_ONLY_PARENTS: &[&str] = &[
    "p", "span", "em", "strong", "small", "s", "cite", "q", "dfn", "abbr", "ruby", "rt", "rp",
    "data", "time", "code", "var", "samp", "kbd", "sub", "sup", "i", "b", "u", "mark", "bdi",
    "bdo", "label",
];

/// Block-level / flow-only elements that cannot appear inside phrasing parents
const BLOCK_ELEMENTS: &[&str] = &[
    "div",
    "p",
    "section",
    "article",
    "aside",
    "header",
    "footer",
    "nav",
    "main",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "ul",
    "ol",
    "dl",
    "table",
    "form",
    "fieldset",
    "figure",
    "figcaption",
    "blockquote",
    "pre",
    "hr",
    "address",
    "details",
    "summary",
    "hgroup",
    "search",
];

/// Interactive elements that must not be nested
const INTERACTIVE_ELEMENTS: &[&str] = &["a", "button", "details", "label", "select", "textarea"];

/// Component namespaces that expose intrinsic HTML wrappers as `namespace.tag`.
const INTRINSIC_MEMBER_COMPONENT_NAMESPACES: &[&str] = &["motion"];

/// Check if an element is a phrasing-only parent
#[inline]
fn is_phrasing_only_parent(tag: &str) -> bool {
    PHRASING_ONLY_PARENTS.contains(&tag)
}

/// Check if an element is a block element
#[inline]
fn is_block_element(tag: &str) -> bool {
    BLOCK_ELEMENTS.contains(&tag)
}

/// Check if an element is interactive
#[inline]
fn is_interactive_element(tag: &str) -> bool {
    INTERACTIVE_ELEMENTS.contains(&tag)
}

/// Check if an element has a transparent content model.
#[inline]
fn is_transparent_parent(tag: &str) -> bool {
    tag == "a"
}

/// Get required direct children for a parent element (if constrained)
fn required_children(parent: &str) -> Option<&'static [&'static str]> {
    match parent {
        "ul" | "ol" | "menu" => Some(&["li"]),
        "dl" => Some(&["dt", "dd", "div"]),
        "table" => Some(&[
            "thead", "tbody", "tfoot", "tr", "caption", "colgroup", "col",
        ]),
        "thead" | "tbody" | "tfoot" => Some(&["tr"]),
        "tr" => Some(&["td", "th"]),
        "colgroup" => Some(&["col"]),
        "select" => Some(&["option", "optgroup"]),
        "optgroup" => Some(&["option"]),
        _ => None,
    }
}

fn intrinsic_member_component_tag(tag: &str) -> Option<&str> {
    let (namespace, member) = tag.split_once('.')?;
    if INTRINSIC_MEMBER_COMPONENT_NAMESPACES.contains(&namespace) && is_html_tag(member) {
        Some(member)
    } else {
        None
    }
}

fn content_model_tag(tag: &str) -> &str {
    intrinsic_member_component_tag(tag).unwrap_or(tag)
}

fn nearest_non_transparent_parent<'ctx, 'a>(
    ctx: &'ctx LintContext<'a>,
) -> Option<(&'ctx str, &'ctx str)> {
    ctx.element_stack.iter().rev().skip(1).find_map(|ancestor| {
        let raw_tag = ancestor.tag.as_str();
        let tag = content_model_tag(raw_tag);
        (!is_transparent_parent(tag)).then_some((raw_tag, tag))
    })
}

#[derive(Default)]
pub struct PermittedContents;

impl Rule for PermittedContents {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        // Allow <template> as a transparent wrapper (v-for, v-if, v-slot)
        if element.tag_type == ElementType::Template {
            return;
        }

        // Skip <slot> elements
        if element.tag_type == ElementType::Slot {
            return;
        }

        let raw_tag = element.tag.as_str();
        let tag = content_model_tag(raw_tag);
        let has_intrinsic_mapping = tag != raw_tag;
        let is_unknown_component =
            element.tag_type == ElementType::Component && !has_intrinsic_mapping;

        // 1. Block in inline: check if this block element has a phrasing-only ancestor
        if !is_unknown_component
            && is_block_element(tag)
            && let Some((parent_raw_tag, parent_tag)) = nearest_non_transparent_parent(ctx)
            && is_phrasing_only_parent(parent_tag)
        {
            let message = ctx.t_fmt(
                "vue/permitted-contents.block_in_inline",
                &[("child", raw_tag), ("parent", parent_raw_tag)],
            );
            ctx.error(message, &element.loc);
        }

        // 2. Interactive nesting: check if this interactive element is inside another
        if !is_unknown_component
            && is_interactive_element(tag)
            && ctx.has_ancestor(|ancestor| {
                is_interactive_element(content_model_tag(ancestor.tag.as_str()))
            })
        {
            let message = ctx.t_fmt(
                "vue/permitted-contents.interactive_nesting",
                &[("tag", raw_tag)],
            );
            ctx.error(message, &element.loc);
        }

        // 3 & 4. Required children: check if parent constrains direct children.
        // A custom component is exempt: its rendered root element is unknown, so
        // `<ul><MyItem /></ul>` (where `MyItem` renders an `<li>`) is valid.
        if !is_unknown_component && let Some(parent) = ctx.parent_element() {
            let parent_tag = content_model_tag(parent.tag.as_str());
            if let Some(allowed) = required_children(parent_tag)
                && !allowed.contains(&tag)
            {
                let message = ctx.t_fmt(
                    "vue/permitted-contents.invalid_child",
                    &[("child", raw_tag), ("parent", parent.tag.as_str())],
                );
                ctx.error(message, &element.loc);
            }
        }
    }
}

#[cfg(test)]
#[path = "permitted_contents_tests.rs"]
mod tests;
