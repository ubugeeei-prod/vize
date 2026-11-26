//! html/no-empty-palpable-content
//!
//! Detect elements that should have visible (palpable) content but are empty.
//! Based on markuplint's `no-empty-palpable-content` rule.
//!
//! Directives known to set the content count as content: `v-html`, `v-text`
//! and vue-i18n's `v-t`. Other custom directives do not, because many only
//! focus or style the element; list the ones that fill content in the
//! `contentDirectives` option.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <template>
//!   <p></p>
//!   <li></li>
//!   <td></td>
//! </template>
//! ```
//!
//! ### Valid
//! ```vue
//! <template>
//!   <p>text</p>
//!   <li>item</li>
//!   <td>cell</td>
//!   <p v-t="'title'"></p>
//! </template>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use crate::rules::html::helpers::{PALPABLE_CONTENT_ELEMENTS, has_palpable_content};
use vize_relief::{ElementNode, ElementType, PropNode};

static META: RuleMeta = RuleMeta {
    name: "html/no-empty-palpable-content",
    description: "Disallow empty elements that expect visible content",
    category: RuleCategory::HtmlConformance,
    fixable: false,
    default_severity: Severity::Warning,
};

#[derive(Default)]
/// Implements the `html/no-empty-palpable-content` lint rule.
pub struct NoEmptyPalpableContent;

impl NoEmptyPalpableContent {
    /// Configure exact bare directive names without changing the default rule.
    pub fn with_content_directives(names: Vec<vize_l0::String>) -> impl Rule {
        ContentDirectives { names }
    }
}

struct ContentDirectives {
    names: Vec<vize_l0::String>,
}

impl Rule for ContentDirectives {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn template_hooks(&self) -> crate::rule::RuleHooks {
        crate::rule::RuleHooks::ELEMENT
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        if element.props.iter().any(|prop| {
            matches!(prop, PropNode::Directive(dir) if self.names.iter().any(|name| name.as_str() == dir.name))
        }) {
            return;
        }
        NoEmptyPalpableContent.enter_element(ctx, element);
    }
}

impl Rule for NoEmptyPalpableContent {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn template_hooks(&self) -> crate::rule::RuleHooks {
        crate::rule::RuleHooks::ELEMENT
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        if element.tag_type == ElementType::Component {
            return;
        }

        let tag = element.tag;
        if !PALPABLE_CONTENT_ELEMENTS.contains(&tag) {
            return;
        }

        // Skip if element has aria-label, a directive known to set its
        // content, or slot. vue-i18n's `v-t` writes the translated message to
        // `textContent` as `v-text` does.
        let has_content_source = element.props.iter().any(|prop| match prop {
            PropNode::Attribute(attr) => {
                attr.name == "aria-label" || attr.name == "aria-labelledby"
            }
            PropNode::Directive(dir) => {
                dir.name == "html" || dir.name == "text" || dir.name == "t" || dir.name == "slot"
            }
        });

        if has_content_source {
            return;
        }

        if !has_palpable_content(element) {
            let message = ctx.t_fmt("html/no-empty-palpable-content.message", &[("tag", tag)]);
            let help = ctx.t("html/no-empty-palpable-content.help");
            ctx.warn_with_help(message, &element.loc, help);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NoEmptyPalpableContent;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoEmptyPalpableContent));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_with_text() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<p>text</p>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_with_interpolation() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<p>{{ text }}</p>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_with_child_element() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<p><span>text</span></p>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_with_template_child_content() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<p><template v-if="ready"><span>text</span></template></p>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_with_aria_label() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<p aria-label="description"></p>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_with_v_html() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<p v-html="content"></p>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_with_v_text() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<p v-text="content"></p>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_with_vue_i18n_v_t() {
        let linter = create_linter();
        for source in [
            r#"<p v-t="'title'"></p>"#,
            r#"<li v-t="{ path: 'menu.home', locale: 'ja' }"></li>"#,
            r#"<td v-t.preserve="'cell'"></td>"#,
        ] {
            let result = linter.lint_template(source, "test.vue");
            assert_eq!(result.warning_count, 0, "{source} must be valid");
        }
    }

    // A custom directive may only focus or style the element, so it does not
    // count as content unless the project configures it.
    #[test]
    fn test_invalid_with_unconfigured_custom_directive() {
        let linter = create_linter();
        for source in [
            r#"<p v-focus></p>"#,
            r#"<p v-tooltip="'hint'"></p>"#,
            r#"<p v-tt="'title'"></p>"#,
        ] {
            let result = linter.lint_template(source, "test.vue");
            assert_eq!(result.warning_count, 1, "{source} must stay reported");
        }
    }

    #[test]
    fn pinned_vue_i18n_v_t_sfc() {
        let linter = create_linter();
        let result = linter.lint_sfc(
            include_str!(
                "../../../../../tests/_fixtures/differential/linter/palpable-content-vue-i18n-v-t/Title.vue.txt"
            ),
            "Title.vue",
        );
        let messages: Vec<&str> = result
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.message.as_str())
            .collect();
        assert_eq!(
            messages,
            ["<p> element is empty but expects visible content"]
        );
    }

    #[test]
    fn test_valid_div_empty() {
        let linter = create_linter();
        // div is NOT in palpable content elements
        let result = linter.lint_template(r#"<div></div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_empty_p() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<p></p>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_whitespace_only_p() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<p>   </p>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_empty_li() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<ul><li></li></ul>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_empty_td() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<table><tr><td></td></tr></table>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_empty_option() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<select><option></option></select>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }
}
