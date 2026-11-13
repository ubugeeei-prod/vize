//! `components`, `accessibleChildren` and `accessibleDirectives` for
//! `a11y/anchor-has-content` and `a11y/heading-has-content`, as
//! eslint-plugin-vuejs-accessibility reads them.

use crate::markup::{MarkupBindingKind, MarkupElement};
use crate::rules::a11y::helpers::{canonical_component_name, matches_configured_component};
use vize_l0::String;
use vize_l0::config::AnchorHasContentOptions;
use vize_relief::{ElementNode, PropNode};

/// Configured names, normalized once.
#[derive(Debug, Clone, Default)]
pub(crate) struct ContentOptions {
    /// Components checked as if they were the native element.
    components: Vec<String>,
    /// Components whose rendered subtree is accessible content.
    accessible_children: Vec<String>,
    /// Directives, kebab-case and without `v-`, that fill their element.
    accessible_directives: Vec<String>,
}

impl ContentOptions {
    /// No configured names.
    pub(crate) const EMPTY: Self = Self {
        components: Vec::new(),
        accessible_children: Vec::new(),
        accessible_directives: Vec::new(),
    };

    pub(crate) fn from_config(options: &AnchorHasContentOptions) -> Self {
        Self {
            components: components(&options.components),
            accessible_children: components(&options.accessible_children),
            accessible_directives: options
                .accessible_directives
                .iter()
                .map(|name| directive_name(name))
                .filter(|name| !name.is_empty())
                .collect(),
        }
    }

    pub(crate) fn with_accessible_children(children: &[String]) -> Self {
        Self {
            accessible_children: components(children),
            ..Self::EMPTY
        }
    }

    /// Whether `tag` is a configured component checked like the native element.
    pub(crate) fn is_checked_component(&self, tag: &str) -> bool {
        matches_configured_component(tag, &self.components)
    }

    /// Whether `tag` is a configured component that renders accessible content.
    pub(crate) fn is_accessible_child(&self, tag: &str) -> bool {
        matches_configured_component(tag, &self.accessible_children)
    }

    /// Whether the element itself carries a configured directive.
    pub(crate) fn has_directive(&self, element: &ElementNode<'_>) -> bool {
        !self.accessible_directives.is_empty()
            && element.props.iter().any(|prop| {
                matches!(prop, PropNode::Directive(directive)
                    if self.names_directive(directive.name))
            })
    }

    /// Whether the markup element itself carries a configured directive.
    pub(crate) fn has_markup_directive(&self, element: &MarkupElement<'_>) -> bool {
        if self.accessible_directives.is_empty() {
            return false;
        }
        let mut found = false;
        element.walk_bindings(&mut |binding| {
            found |= binding.kind() == MarkupBindingKind::Custom
                && binding
                    .arg_name()
                    .is_some_and(|name| self.names_directive(name));
        });
        found
    }

    fn names_directive(&self, name: &str) -> bool {
        self.accessible_directives
            .iter()
            .any(|configured| configured.eq_ignore_ascii_case(name))
    }
}

fn components(names: &[String]) -> Vec<String> {
    names
        .iter()
        .map(|name| canonical_component_name(name))
        .collect()
}

/// `markupInline`, `markup-inline` and `v-markup-inline` all name the
/// directive written `v-markup-inline`.
fn directive_name(name: &str) -> String {
    let name = name.trim();
    let name = name.strip_prefix("v-").unwrap_or(name);
    canonical_component_name(name)
}

#[cfg(test)]
mod tests {
    use super::directive_name;
    use crate::LintPreset;
    use crate::linter::Linter;
    use vize_l0::config::ConfigLintRuleOptions;

    /// The start line of each report, for both content rules.
    fn report_lines(options: serde_json::Value, template: &str) -> Vec<(&'static str, usize)> {
        let options: ConfigLintRuleOptions = serde_json::from_value(options).unwrap();
        let source = vize_l0::cstr!("<template>\n{template}\n</template>");
        let result = Linter::with_preset(LintPreset::Incremental)
            .with_enabled_rules(Some(vec![
                "a11y/anchor-has-content".into(),
                "a11y/heading-has-content".into(),
            ]))
            .with_config_rule_options(&options)
            .lint_sfc(&source, "Headings.vue");
        let mut lines = result
            .diagnostics
            .iter()
            .map(|diagnostic| {
                let line = source
                    .get(..diagnostic.start as usize)
                    .unwrap_or_default()
                    .lines()
                    .count();
                (diagnostic.rule_name, line)
            })
            .collect::<Vec<_>>();
        lines.sort_by_key(|(_, line)| *line);
        lines
    }

    const ISSUE_TEMPLATE: &str = r#"<section>
  <h2 v-markup-inline="text" />
  <h3 v-text="text" />
  <h4 />
  <a href="/docs" v-markup-inline="text" />
  <a href="/faq" />
</section>"#;

    #[test]
    fn accessible_directives_fill_headings_and_anchors() {
        assert_eq!(
            report_lines(serde_json::json!({}), ISSUE_TEMPLATE),
            [
                ("a11y/heading-has-content", 3),
                ("a11y/heading-has-content", 5),
                ("a11y/anchor-has-content", 6),
                ("a11y/anchor-has-content", 7),
            ]
        );
        let options = serde_json::json!({
            "a11y/heading-has-content": { "accessibleDirectives": ["markup-inline"] },
            "a11y/anchor-has-content": { "accessibleDirectives": ["markupInline"] },
        });
        assert_eq!(
            report_lines(options, ISSUE_TEMPLATE),
            [
                ("a11y/heading-has-content", 5),
                ("a11y/anchor-has-content", 7)
            ]
        );
    }

    #[test]
    fn a_descendant_directive_counts_unless_the_anchor_child_is_hidden() {
        let options = serde_json::json!({
            "a11y/heading-has-content": { "accessibleDirectives": ["v-markup-inline"] },
            "a11y/anchor-has-content": { "accessibleDirectives": ["markup-inline"] },
        });
        let template = r#"<div>
  <h2><span v-markup-inline="text" /></h2>
  <a href="/a"><span v-markup-inline /></a>
  <a href="/b"><span aria-hidden="true" v-markup-inline="text" /></a>
  <h2 v-other="text" />
</div>"#;
        assert_eq!(
            report_lines(options, template),
            [
                ("a11y/anchor-has-content", 5),
                ("a11y/heading-has-content", 6)
            ]
        );
    }

    #[test]
    fn components_and_accessible_children_extend_both_rules() {
        let options = serde_json::json!({
            "a11y/heading-has-content": {
                "components": ["SectionTitle"],
                "accessibleChildren": ["TitleIcon"],
            },
            "a11y/anchor-has-content": { "components": ["NuxtLink"] },
        });
        let template = r#"<div>
  <SectionTitle />
  <section-title>Docs</section-title>
  <h2><TitleIcon /></h2>
  <h2><title-icon aria-hidden="true" /></h2>
  <NuxtLink to="/" />
  <nuxt-link to="/">Home</nuxt-link>
</div>"#;
        assert_eq!(
            report_lines(options, template),
            [
                ("a11y/heading-has-content", 3),
                ("a11y/heading-has-content", 6),
                ("a11y/anchor-has-content", 7),
            ]
        );
    }

    #[test]
    fn directive_names_normalize_to_the_template_spelling() {
        for name in [
            "markupInline",
            "markup-inline",
            "v-markup-inline",
            " markup-inline ",
        ] {
            assert_eq!(directive_name(name), "markup-inline");
        }
        assert_eq!(directive_name("v-"), "");
    }
}
