mod report;
use report::property_has_markerless_list;

use crate::context::LintContext;
use crate::diagnostic::{Fix, LintDiagnostic, TextEdit};
use crate::markup::{MarkupBindingKind, MarkupContext, MarkupElement, MarkupRule};
use crate::rule::{Rule, RuleMeta};
use crate::rules::css_engine::parse_stylesheet;
use lightningcss::declaration::DeclarationBlock;
use lightningcss::properties::list::ListStyleType;
use lightningcss::properties::{Property, PropertyId};
use lightningcss::rules::CssRule as LCssRule;
use lightningcss::selector::{Component, Selector};
use vize_l0::FxHashSet;
use vize_relief::{ElementNode, ElementType, PropNode};

use super::helpers::{
    AuthoredAttr, get_implicit_role, get_implicit_role_by_attr,
    get_static_or_bound_literal_attribute_value,
};
use crate::rules::a11y::helpers::string_literal_value;

/// Disallow redundant ARIA roles
#[derive(Default)]
pub struct NoRedundantRoles;

impl NoRedundantRoles {
    fn first_static_attribute_value<'a>(
        element: &MarkupElement<'a>,
        name: &str,
        literal_bind: bool,
    ) -> Option<&'a str> {
        let mut found = false;
        let mut value = None;
        element.walk_bindings(&mut |binding| {
            if found {
                return;
            }
            match binding.kind() {
                MarkupBindingKind::Attribute if binding.is_static_unqualified_arg_exact(name) => {
                    found = true;
                    value = binding.static_value();
                }
                // Vue renders a string-literal bind as the static attribute value.
                // JSX `role={"button"}` stays outside that boundary.
                MarkupBindingKind::Bind
                    if literal_bind && binding.is_static_unqualified_arg_exact(name) =>
                {
                    let Some(expression) = binding.expression() else {
                        return;
                    };
                    let Some(literal) = string_literal_value(expression) else {
                        return;
                    };
                    found = true;
                    value = Some(literal);
                }
                _ => {}
            }
        });
        value
    }

    fn markup_implicit_role<'a>(element: &MarkupElement<'a>) -> Option<&'static str> {
        let tag = element.tag();
        if !element.is_unqualified_tag_exact(tag) {
            return None;
        }

        get_implicit_role_by_attr(tag, |name| markup_authored_attr(element, name))
    }

    /// Range of the first static `role` attribute, matching the value probe.
    fn redundant_role_attribute_range(
        element: &MarkupElement<'_>,
        literal_bind: bool,
    ) -> Option<crate::ir::ByteRange> {
        let mut seen = false;
        let mut range = None;
        element.walk_bindings(&mut |binding| {
            if seen {
                return;
            }
            match binding.kind() {
                MarkupBindingKind::Attribute if binding.is_static_unqualified_arg_exact("role") => {
                    seen = true;
                    if binding.static_value().is_some() {
                        range = Some(binding.range());
                    }
                }
                MarkupBindingKind::Bind
                    if literal_bind && binding.is_static_unqualified_arg_exact("role") =>
                {
                    let Some(expression) = binding.expression() else {
                        return;
                    };
                    if string_literal_value(expression).is_some() {
                        seen = true;
                        range = Some(binding.range());
                    }
                }
                _ => {}
            }
        });
        range
    }

    fn keeps_markerless_list_role(
        ctx: &LintContext<'_>,
        tag: &str,
        class: Option<&str>,
        role: &str,
    ) -> bool {
        if role != "list" || !matches!(tag, "ol" | "ul") {
            return false;
        }

        let Some(class) = class else {
            return false;
        };
        let classes = class
            .split_ascii_whitespace()
            .filter(|class| !class.is_empty())
            .collect::<FxHashSet<_>>();
        let Some(descriptor) = ctx.sfc_descriptor() else {
            return false;
        };

        descriptor.styles.iter().any(|style| {
            style.src.is_none()
                && style
                    .lang
                    .as_deref()
                    .is_none_or(|lang| lang.eq_ignore_ascii_case("css"))
                && style_has_markerless_list_class(style.content.as_ref(), &classes)
        })
    }
}

fn markup_authored_attr<'a>(element: &MarkupElement<'a>, name: &str) -> AuthoredAttr<'a> {
    let mut found = AuthoredAttr::Missing;
    element.walk_bindings(&mut |binding| {
        if found != AuthoredAttr::Missing {
            return;
        }
        match binding.kind() {
            MarkupBindingKind::Attribute if binding.is_unqualified_arg_exact(name) => {
                found = AuthoredAttr::Static(binding.static_value().unwrap_or(""));
            }
            MarkupBindingKind::Bind if binding.is_static_unqualified_arg_exact(name) => {
                found = match binding.expression() {
                    Some(expression) => {
                        let expression = expression.trim();
                        if expression.is_empty() || matches!(expression, "null" | "undefined") {
                            AuthoredAttr::Missing
                        } else {
                            match string_literal_value(expression) {
                                Some(literal) => AuthoredAttr::Static(literal),
                                None => AuthoredAttr::Dynamic,
                            }
                        }
                    }
                    None => AuthoredAttr::Dynamic,
                };
            }
            _ => {}
        }
    });
    found
}

fn style_has_markerless_list_class(source: &str, classes: &FxHashSet<&str>) -> bool {
    let Ok(sheet) = parse_stylesheet(source) else {
        return false;
    };
    sheet
        .rules
        .0
        .iter()
        .any(|rule| css_rule_has_markerless_list_class(rule, classes))
}

fn css_rule_has_markerless_list_class(rule: &LCssRule, classes: &FxHashSet<&str>) -> bool {
    match rule {
        LCssRule::Style(rule) => {
            rule.selectors
                .0
                .iter()
                .any(|selector| selector_matches_class(selector, classes))
                && declarations_have_markerless_list(&rule.declarations)
        }
        LCssRule::Media(rule) => rule
            .rules
            .0
            .iter()
            .any(|rule| css_rule_has_markerless_list_class(rule, classes)),
        LCssRule::Supports(rule) => rule
            .rules
            .0
            .iter()
            .any(|rule| css_rule_has_markerless_list_class(rule, classes)),
        LCssRule::LayerBlock(rule) => rule
            .rules
            .0
            .iter()
            .any(|rule| css_rule_has_markerless_list_class(rule, classes)),
        _ => false,
    }
}

fn selector_matches_class(selector: &Selector, classes: &FxHashSet<&str>) -> bool {
    selector.iter().any(|component| {
        matches!(component, Component::Class(class) if classes.contains(class.0.as_ref()))
    })
}

fn declarations_have_markerless_list(declarations: &DeclarationBlock) -> bool {
    declarations
        .declarations
        .iter()
        .chain(declarations.important_declarations.iter())
        .any(property_has_markerless_list)
}

#[cfg(test)]
mod tests {
    use super::NoRedundantRoles;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoRedundantRoles));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_no_role() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<nav>Navigation</nav>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_different_role() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div role="navigation">Navigation</div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_nav_navigation() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<nav aria-label="Primary" role="navigation">Navigation</nav>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 1);
        let unnamed =
            linter.lint_template(r#"<nav role="navigation">Navigation</nav>"#, "test.vue");
        assert_eq!(unnamed.warning_count, 0, "{:?}", unnamed.diagnostics);
    }

    #[test]
    fn test_invalid_button_button() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<button role="button">Click</button>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_main_main() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<main role="main">Content</main>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_valid_markerless_list_role_workaround() {
        let linter = create_linter();
        let result = linter.lint_sfc(
            r#"<template>
  <ul class="plain" role="list"><li>a</li></ul>
</template>
<style scoped>
.plain { list-style: none; }
</style>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_list_role_without_markerless_style() {
        let linter = create_linter();
        let result = linter.lint_sfc(
            r#"<template>
  <ul class="plain" role="list"><li>a</li></ul>
</template>
<style scoped>
.plain { list-style: disc; }
</style>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_valid_component_skipped() {
        let linter = create_linter();
        let result =
            linter.lint_template(r#"<MyNav role="navigation">Navigation</MyNav>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_fix_removes_redundant_role_attribute() {
        let linter = create_linter();
        let source = r#"<nav aria-label="Primary" role="navigation">Navigation</nav>"#;
        let result = linter.lint_template(source, "test.vue");
        assert_eq!(result.warning_count, 1);
        let fixed = result.diagnostics[0]
            .fix
            .as_ref()
            .expect("redundant role emits an edit")
            .apply(source);
        assert_eq!(fixed, r#"<nav aria-label="Primary">Navigation</nav>"#);
        assert_eq!(linter.lint_template(&fixed, "test.vue").warning_count, 0);

        let with_class = r#"<button class="ok" role="button">Click</button>"#;
        let result = linter.lint_template(with_class, "test.vue");
        let fixed = result.diagnostics[0]
            .fix
            .as_ref()
            .expect("edit")
            .apply(with_class);
        assert_eq!(fixed, r#"<button class="ok">Click</button>"#);

        let first_only = r#"<button role="button" role="switch">Click</button>"#;
        let result = linter.lint_template(first_only, "test.vue");
        let fixed = result.diagnostics[0]
            .fix
            .as_ref()
            .expect("edit")
            .apply(first_only);
        assert_eq!(fixed, r#"<button role="switch">Click</button>"#);
        assert_eq!(linter.lint_template(&fixed, "test.vue").warning_count, 0);

        let sfc = "<template>\n  <main role=\"main\">Content</main>\n</template>\n";
        let result = linter.lint_sfc(sfc, "Page.vue");
        assert_eq!(result.warning_count, 1);
        let fixed = result.diagnostics[0]
            .fix
            .as_ref()
            .expect("sfc edit")
            .apply(sfc);
        assert_eq!(fixed, "<template>\n  <main>Content</main>\n</template>\n");

        let jsx = r#"const A = () => <nav aria-label="Primary" role="navigation" />;"#;
        let result = linter.lint_jsx(jsx, "test.jsx", vize_atelier_jsx::JsxLang::Jsx);
        assert_eq!(result.warning_count, 1);
        let fixed = result.diagnostics[0]
            .fix
            .as_ref()
            .expect("jsx edit")
            .apply(jsx);
        assert_eq!(fixed, r#"const A = () => <nav aria-label="Primary" />;"#);
        assert_eq!(
            linter
                .lint_jsx(&fixed, "test.jsx", vize_atelier_jsx::JsxLang::Jsx)
                .warning_count,
            0
        );
    }

    #[test]
    fn third_party_implicit_roles_follow_html_aria() {
        let cases = [
            (r#"<nav role="navigation">x</nav>"#, 0),
            (r#"<section role="region">x</section>"#, 0),
            (
                r#"<select role="listbox" aria-label="s"><option>a</option></select>"#,
                0,
            ),
            (
                r#"<select role="combobox" aria-label="s"><option>a</option></select>"#,
                1,
            ),
            (
                r#"<select multiple role="listbox" aria-label="s"><option>a</option></select>"#,
                1,
            ),
            (
                r#"<select size="3" role="listbox" aria-label="s"><option>a</option></select>"#,
                1,
            ),
            // A bound href stays outside the implicit-role probe.
            (r#"<a :href="href" role="link">x</a>"#, 0),
            (r#"<img src="/a.png" alt="" role="presentation" />"#, 1),
            (r#"<img src="/a.png" alt="" role="img" />"#, 0),
            (
                r#"<input type="text" list="l" role="combobox" aria-label="i" />"#,
                1,
            ),
            (r#"<input :type="kind" role="textbox" aria-label="i" />"#, 0),
            (r#"<form role="form">x</form>"#, 0),
            (
                r#"<details><summary role="button">s</summary>x</details>"#,
                1,
            ),
            (r#"<div role="generic">x</div>"#, 1),
            (r#"<span role="generic">x</span>"#, 1),
            (r#"<p role="paragraph">x</p>"#, 1),
            // A string-literal bind renders as the static role.
            (r#"<button type="button" :role="'button'">x</button>"#, 1),
            (r#"<button type="button" :role="kind">x</button>"#, 0),
            (r#"<img src="/a.png" alt="" :role="'img'" />"#, 0),
        ];
        for (source, warnings) in cases {
            let result = create_linter().lint_template(source, "test.vue");
            assert_eq!(
                result.warning_count, warnings,
                "{source}: {:?}",
                result.diagnostics
            );
        }
    }
}
