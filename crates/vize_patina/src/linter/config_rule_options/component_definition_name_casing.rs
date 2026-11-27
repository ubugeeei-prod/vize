//! Apply `vue/component-definition-name-casing` options by replacing the
//! registered rule; an unselected rule stays unselected.

use super::super::config::Linter;
use crate::rules::vue::{ComponentDefinitionNameCasing, ComponentDefinitionNameCasingOption};
use vize_l0::config::ComponentDefinitionNameCasingStyle;

pub(super) fn apply(mut linter: Linter, style: ComponentDefinitionNameCasingStyle) -> Linter {
    let casing = match style {
        ComponentDefinitionNameCasingStyle::PascalCase => {
            ComponentDefinitionNameCasingOption::PascalCase
        }
        ComponentDefinitionNameCasingStyle::KebabCase => {
            ComponentDefinitionNameCasingOption::KebabCase
        }
    };
    linter
        .registry
        .replace(Box::new(ComponentDefinitionNameCasing { casing }));
    linter
}

#[cfg(test)]
mod tests {
    use crate::{LintPreset, Linter};
    use vize_l0::String;
    use vize_l0::config::ConfigLintRuleOptions;

    const RULE: &str = "vue/component-definition-name-casing";
    const SOURCE: &str = "<script>\nexport default { name: 'MyComponent' }\n</script>\n<script setup>\ndefineOptions({ name: 'my-component' })\n</script>\n";

    fn messages(linter: Linter) -> Vec<String> {
        linter
            .lint_sfc(SOURCE, "Component.vue")
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.rule_name == RULE)
            .map(|diagnostic| diagnostic.message.clone())
            .collect()
    }

    fn configured(options: &str) -> Linter {
        let options: ConfigLintRuleOptions = serde_json::from_str(options).unwrap();
        Linter::with_preset(LintPreset::Incremental)
            .with_enabled_rules(Some(vec![String::from(RULE)]))
            .with_config_rule_options(&options)
    }

    #[test]
    fn the_configured_style_replaces_the_default() {
        assert_eq!(
            messages(configured("{}")),
            ["Component name 'my-component' is not PascalCase"]
        );
        assert_eq!(
            messages(configured(
                r#"{"vue/component-definition-name-casing":"PascalCase"}"#
            )),
            ["Component name 'my-component' is not PascalCase"]
        );
        assert_eq!(
            messages(configured(
                r#"{"vue/component-definition-name-casing":"kebab-case"}"#
            )),
            ["Component name 'MyComponent' is not kebab-case"]
        );
    }

    #[test]
    fn options_alone_do_not_enable_the_rule() {
        let options: ConfigLintRuleOptions =
            serde_json::from_str(r#"{"vue/component-definition-name-casing":"kebab-case"}"#)
                .unwrap();
        let linter = Linter::with_preset(LintPreset::Incremental)
            .with_enabled_rules(Some(vec![String::from("vue/html-quotes")]))
            .with_config_rule_options(&options);
        assert!(messages(linter).is_empty());
    }
}
