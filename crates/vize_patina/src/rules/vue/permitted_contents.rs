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
use vize_relief::ast::{ElementNode, ElementType};

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

        // 3 & 4. Required children: check if parent constrains direct children
        if let Some(parent) = ctx.parent_element() {
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
mod tests {
    use super::{PermittedContents, required_children};
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(PermittedContents));
        Linter::with_registry(registry)
    }

    // ===== Valid cases =====

    #[test]
    fn test_valid_inline_in_inline() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<p><span>text</span></p>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_block_in_block() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div><p>text</p></div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_list_with_li() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<ul><li>item</li></ul>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_list_with_known_intrinsic_member_component_li() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<ul><motion.li>item</motion.li></ul>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_table_structure() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<table><thead><tr><th>Head</th></tr></thead><tbody><tr><td>Cell</td></tr></tbody></table>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_template_wrapper_in_list() {
        let linter = create_linter();
        // <template> is allowed as a transparent wrapper inside lists
        let result = linter.lint_template(
            r#"<ul><template v-for="item in items"><li>{{ item }}</li></template></ul>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_component_in_any_context() {
        let linter = create_linter();
        // Components are skipped — can render anything
        let result = linter.lint_template(r#"<p><MyComponent /></p>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_nested_non_interactive() {
        let linter = create_linter();
        let result = linter.lint_template(r##"<a href="#"><span>text</span></a>"##, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_flow_content_in_anchor_when_context_allows_flow() {
        let linter = create_linter();
        let result = linter.lint_template(
            r##"<main><a href="#"><h2>Documentation</h2><div>Read the guide</div></a></main>"##,
            "test.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_select_with_options() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<select><option>A</option><option>B</option></select>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_select_with_optgroup() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<select><optgroup label="Group"><option>A</option></optgroup></select>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    // ===== Invalid: Block in inline =====

    #[test]
    fn test_repaired_div_after_p() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<p><div>block</div></p>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_div_in_span() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<span><div>block</div></span>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_repaired_h1_after_p() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<p><h1>heading</h1></p>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_ul_in_span() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<span><ul><li>item</li></ul></span>"#, "test.vue");
        // ul in span: block_in_inline error
        // But li in ul is valid
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_repaired_flow_content_in_anchor_when_outer_context_is_phrasing() {
        let linter = create_linter();
        let result =
            linter.lint_template(r##"<p><a href="#"><div>block</div></a></p>"##, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    // ===== Invalid: Interactive nesting =====

    #[test]
    fn test_repaired_a_in_a() {
        let linter = create_linter();
        let result =
            linter.lint_template(r##"<a href="#"><a href="#">nested</a></a>"##, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_repaired_button_in_button() {
        let linter = create_linter();
        let result =
            linter.lint_template(r#"<button><button>nested</button></button>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_button_in_a() {
        let linter = create_linter();
        let result =
            linter.lint_template(r##"<a href="#"><button>click</button></a>"##, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    // ===== Invalid: List content model =====

    #[test]
    fn test_invalid_div_in_ul() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<ul><div>not li</div></ul>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_span_in_ol() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<ol><span>not li</span></ol>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_unknown_component_in_ul() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<ul><MyItem /></ul>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_unknown_member_component_in_ul() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<ul><foo.li>item</foo.li></ul>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_known_intrinsic_member_component_div_in_ul() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<ul><motion.div>item</motion.div></ul>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    // ===== Invalid: Table content model =====

    #[test]
    fn test_repaired_div_in_table() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<table><div>not valid</div></table>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_repaired_span_in_tr() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<table><tr><span>not td/th</span></tr></table>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    // ===== Invalid: Select content model =====

    #[test]
    fn test_invalid_div_in_select() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<select><div>not option</div></select>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    // ===== Helper function tests =====

    #[test]
    fn test_required_children_lookup() {
        assert_eq!(required_children("ul"), Some(["li"].as_slice()));
        assert_eq!(required_children("ol"), Some(["li"].as_slice()));
        assert_eq!(
            required_children("table"),
            Some(
                [
                    "thead", "tbody", "tfoot", "tr", "caption", "colgroup", "col"
                ]
                .as_slice()
            )
        );
        assert_eq!(required_children("tr"), Some(["td", "th"].as_slice()));
        assert!(required_children("div").is_none());
    }
}
