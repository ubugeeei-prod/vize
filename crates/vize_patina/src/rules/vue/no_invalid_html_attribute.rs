//! vue/no-invalid-html-attribute
//!
//! Disallow invalid static HTML attribute values. The first supported attribute
//! is `rel`, matching the default scope of `react/no-invalid-html-attribute`.

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::markup::{MarkupContext, MarkupElement, MarkupRule};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ElementNode;

static META: RuleMeta = RuleMeta {
    name: "vue/no-invalid-html-attribute",
    description: "Disallow invalid static values for HTML attributes",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

const REL_TAGS: &[&str] = &["a", "area", "form", "link"];
const REL_VALUES: &[(&str, &[&str])] = &[
    ("alternate", &["a", "area", "link"]),
    ("apple-touch-icon", &["link"]),
    ("apple-touch-startup-image", &["link"]),
    ("author", &["a", "area", "link"]),
    ("bookmark", &["a", "area"]),
    ("canonical", &["link"]),
    ("dns-prefetch", &["link"]),
    ("external", &["a", "area", "form"]),
    ("help", &["a", "area", "form", "link"]),
    ("icon", &["link"]),
    ("license", &["a", "area", "form", "link"]),
    ("manifest", &["link"]),
    ("mask-icon", &["link"]),
    ("modulepreload", &["link"]),
    ("next", &["a", "area", "form", "link"]),
    ("nofollow", &["a", "area", "form"]),
    ("noopener", &["a", "area", "form"]),
    ("noreferrer", &["a", "area", "form"]),
    ("opener", &["a", "area", "form"]),
    ("pingback", &["link"]),
    ("preconnect", &["link"]),
    ("prefetch", &["link"]),
    ("preload", &["link"]),
    ("prerender", &["link"]),
    ("prev", &["a", "area", "form", "link"]),
    ("search", &["a", "area", "form", "link"]),
    ("shortcut", &["link"]),
    ("stylesheet", &["link"]),
    ("tag", &["a", "area"]),
];

pub struct NoInvalidHtmlAttribute;

impl MarkupRule for NoInvalidHtmlAttribute {
    fn name(&self) -> &'static str {
        META.name
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        check_element(ctx.lint(), element);
    }
}

impl Rule for NoInvalidHtmlAttribute {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        Some(self)
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        check_element(ctx, &MarkupElement::new(element));
    }
}

fn check_element(ctx: &mut LintContext<'_>, element: &MarkupElement<'_>) {
    let tag = element.tag();
    if tag.contains('-') || element.is_component() {
        return;
    }
    let Some(attr) = element.static_attribute("rel") else {
        return;
    };
    let Some(value) = attr.value() else {
        report_empty(ctx, element);
        return;
    };
    let value = value.trim();
    if value.is_empty() {
        report_empty(ctx, element);
        return;
    }
    if !REL_TAGS.contains(&tag) {
        ctx.warn_at_with_help(
            ctx.t_fmt("vue/no-invalid-html-attribute.wrong_tag", &[("tag", tag)]),
            attr.range(),
            ctx.t("vue/no-invalid-html-attribute.help"),
        );
        return;
    }
    let tokens: Vec<&str> = value.split_whitespace().collect();
    for (index, token) in tokens.iter().copied().enumerate() {
        let Some(tags) = rel_allowed_tags(token) else {
            report_invalid(ctx, token, element);
            continue;
        };
        if !tags.contains(&tag) {
            report_invalid_for_tag(ctx, token, tag, element);
        }
        if token == "shortcut" && tokens.get(index + 1).copied() != Some("icon") {
            ctx.warn_at_with_help(
                ctx.t("vue/no-invalid-html-attribute.shortcut"),
                element.range(),
                ctx.t("vue/no-invalid-html-attribute.help"),
            );
        }
    }
}

fn rel_allowed_tags(value: &str) -> Option<&'static [&'static str]> {
    REL_VALUES
        .iter()
        .find_map(|(candidate, tags)| (*candidate == value).then_some(*tags))
}

fn report_empty(ctx: &mut LintContext<'_>, element: &MarkupElement<'_>) {
    ctx.warn_at_with_help(
        ctx.t("vue/no-invalid-html-attribute.empty"),
        element.range(),
        ctx.t("vue/no-invalid-html-attribute.help"),
    );
}

fn report_invalid(ctx: &mut LintContext<'_>, value: &str, element: &MarkupElement<'_>) {
    ctx.warn_at_with_help(
        ctx.t_fmt("vue/no-invalid-html-attribute.invalid", &[("value", value)]),
        element.range(),
        ctx.t("vue/no-invalid-html-attribute.help"),
    );
}

fn report_invalid_for_tag(
    ctx: &mut LintContext<'_>,
    value: &str,
    tag: &str,
    element: &MarkupElement<'_>,
) {
    ctx.warn_at_with_help(
        ctx.t_fmt(
            "vue/no-invalid-html-attribute.invalid_for_tag",
            &[("value", value), ("tag", tag)],
        ),
        element.range(),
        ctx.t("vue/no-invalid-html-attribute.help"),
    );
}

#[cfg(test)]
mod tests {
    use super::NoInvalidHtmlAttribute;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;
    use vize_atelier_jsx::JsxLang;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoInvalidHtmlAttribute));
        Linter::with_registry(registry)
    }

    #[test]
    fn valid_rel_on_anchor() {
        let result = create_linter().lint_template(
            r#"<a href="/" rel="noopener noreferrer">Home</a>"#,
            "App.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn reports_unknown_rel_value() {
        let result = create_linter().lint_template(r#"<a href="/" rel="friend">x</a>"#, "App.vue");
        assert_eq!(result.warning_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn reports_rel_value_on_wrong_tag() {
        let result = create_linter().lint_template(r#"<a rel="stylesheet">x</a>"#, "App.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn reports_rel_on_wrong_element() {
        let result = create_linter().lint_template(r#"<div rel="noopener">x</div>"#, "App.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn allows_shortcut_icon_pair_on_link() {
        let result = create_linter().lint_template(
            r#"<link rel="shortcut icon" href="/favicon.ico">"#,
            "App.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn reports_unpaired_shortcut() {
        let result = create_linter()
            .lint_template(r#"<link rel="shortcut" href="/favicon.ico">"#, "App.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn skips_dynamic_rel() {
        let result = create_linter().lint_template(r#"<a :rel="rel">x</a>"#, "App.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn reports_jsx_unknown_rel_value() {
        let result = create_linter().lint_jsx(
            r#"const App = () => <a href="/" rel="friend">x</a>;"#,
            "App.jsx",
            JsxLang::Jsx,
        );
        assert_eq!(result.warning_count, 1);
    }
}
