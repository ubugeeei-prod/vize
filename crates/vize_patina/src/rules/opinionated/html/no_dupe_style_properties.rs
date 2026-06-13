//! html/no-dupe-style-properties
//!
//! Disallow duplicate CSS properties inside an inline `style` attribute.
//! Cross-framework analogue of svelte's `no-dupe-style-properties` rule.
//!
//! Only the static `style` attribute is inspected. Dynamic `:style` bindings
//! are objects or expressions and are intentionally ignored.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <template>
//!   <div style="color: red; color: blue">text</div>
//!   <div style="margin: 0; MARGIN: 1px">text</div>
//! </template>
//! ```
//!
//! ### Valid
//! ```vue
//! <template>
//!   <div style="color: red; background: blue">text</div>
//!   <div :style="{ color: a, color: b }">text</div>
//! </template>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_carton::FxHashMap;
use vize_carton::String;
use vize_carton::ToCompactString;
use vize_relief::{ElementNode, ElementType, PropNode};

static META: RuleMeta = RuleMeta {
    name: "html/no-dupe-style-properties",
    description: "Disallow duplicate properties in inline style attributes",
    category: RuleCategory::HtmlConformance,
    fixable: false,
    default_severity: Severity::Warning,
};

#[derive(Default)]
pub struct NoDupeStyleProperties;

impl Rule for NoDupeStyleProperties {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        if element.tag_type == ElementType::Component {
            return;
        }

        for prop in &element.props {
            // Only inspect the static `style` attribute. Dynamic `:style`
            // bindings are directives and are skipped here.
            let PropNode::Attribute(attr) = prop else {
                continue;
            };
            if attr.name != "style" {
                continue;
            }
            let Some(value) = &attr.value else {
                continue;
            };

            let mut seen: FxHashMap<String, ()> = FxHashMap::default();
            for declaration in value.content.as_str().split(';') {
                // A declaration is `property: value`; the property name is the
                // text before the first colon.
                let property = match declaration.split_once(':') {
                    Some((name, _)) => name,
                    None => continue,
                };
                let normalized = property.trim().to_lowercase();
                if normalized.is_empty() {
                    continue;
                }
                let normalized = normalized.to_compact_string();

                if seen.insert(normalized.clone(), ()).is_some() {
                    let message = ctx.t_fmt(
                        "html/no-dupe-style-properties.message",
                        &[("property", normalized.as_str())],
                    );
                    let help = ctx.t("html/no-dupe-style-properties.help");
                    ctx.warn_with_help(message, &attr.loc, help);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NoDupeStyleProperties;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoDupeStyleProperties));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_unique_properties() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div style="color: red; background: blue">x</div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_single_property() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div style="color: red">x</div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_no_style() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div class="foo">x</div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_dynamic_style_ignored() {
        let linter = create_linter();
        // Dynamic :style bindings are objects/expressions and must be ignored.
        let result = linter.lint_template(
            r#"<div :style="{ color: a, color: b }">x</div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_duplicate_property() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div style="color: red; color: blue">x</div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_duplicate_case_insensitive() {
        let linter = create_linter();
        let result =
            linter.lint_template(r#"<div style="margin: 0; MARGIN: 1px">x</div>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_duplicate_with_whitespace() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div style="  color :red ;  color : blue ">x</div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_triple_duplicate() {
        let linter = create_linter();
        // Two warnings: second and third occurrences of `color`.
        let result = linter.lint_template(
            r#"<div style="color: red; color: blue; color: green">x</div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 2);
    }
}
