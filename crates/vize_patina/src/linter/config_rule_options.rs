//! Shared application of the public `linter.ruleOptions` shape.

use super::config::Linter;
use crate::rules::{
    ComponentCasing, HtmlSelfClosingHtmlOptions, HtmlSelfClosingOptions, HtmlSelfClosingStyle,
    HyphenationStyle, NoMutatingPropsOptions, SfcElementOrderGroup, SfcElementOrderOptions,
    VOnEventHyphenationStyle,
};
use vize_l0::config::{
    ConfigLintRuleOptions, CustomEventNameCasing, HtmlSelfClosingStyle as ConfigSelfClosingStyle,
    HyphenationStyle as ConfigHyphenationStyle, TemplateComponentNameCasing,
    VBindStyle as ConfigVBindStyle, VOnStyle as ConfigVOnStyle,
};

impl Linter {
    /// Apply the same typed per-rule options used by the CLI and language server.
    pub fn with_config_rule_options(mut self, options: &ConfigLintRuleOptions) -> Self {
        self = self
            .with_restricted_globals(options.restricted_globals())
            .with_restricted_members(options.restricted_members())
            .with_musea_design_tokens(options.musea_design_tokens());
        if let Some(globals) = options.component_registration_globals() {
            self = self.with_component_registration_globals(globals.to_vec());
        }
        if let Some(names) = options.palpable_content_directives() {
            self = self.with_palpable_content_directives(names.to_vec());
        }
        if let Some(value) = options.strict_boolean_expressions() {
            self = self.with_strict_boolean_expressions_options(value);
        }
        if let Some(value) = options.define_props_destructuring() {
            self = self.with_define_props_destructuring(value);
        }
        if let Some(value) = options.component_name_in_template_casing_options() {
            self = self.with_component_name_in_template_casing_policy(
                component_casing(value.casing),
                value.registered_components_only,
                value.globals.clone(),
            );
        }
        if let Some(value) = options.custom_event_name_casing() {
            self = self.with_custom_event_name_casing(match value {
                CustomEventNameCasing::CamelCase => {
                    crate::rules::script::EventNameCasing::CamelCase
                }
                CustomEventNameCasing::KebabCase => {
                    crate::rules::script::EventNameCasing::KebabCase
                }
            });
        }
        if let Some(value) = options.no_mutating_props() {
            self = self.with_no_mutating_props_options(NoMutatingPropsOptions {
                shallow_only: value.shallow_only,
            });
        }
        if let Some(value) = options.sfc_element_order() {
            self = self.with_sfc_element_order_options(SfcElementOrderOptions {
                order: value
                    .order
                    .into_iter()
                    .map(|group| SfcElementOrderGroup::new(group.selectors()))
                    .collect(),
            });
        }
        if let Some(value) = options.html_self_closing() {
            self = self.with_html_self_closing_options(HtmlSelfClosingOptions {
                html: HtmlSelfClosingHtmlOptions {
                    void: self_closing_style(value.html.void_elements),
                    normal: self_closing_style(value.html.normal),
                    component: self_closing_style(value.html.component),
                },
                svg: self_closing_style(value.svg),
                math: self_closing_style(value.math),
            });
        }
        if let Some(value) = options.v_on_event_hyphenation() {
            self = self.with_v_on_event_hyphenation(match value {
                ConfigHyphenationStyle::Always => VOnEventHyphenationStyle::Always,
                ConfigHyphenationStyle::Never => VOnEventHyphenationStyle::Never,
            });
        }
        if let Some(value) = options.v_on_style() {
            self = self.with_v_on_style(match value {
                ConfigVOnStyle::Shorthand => crate::rules::vue::VOnStyleOption::Shorthand,
                ConfigVOnStyle::Longform => crate::rules::vue::VOnStyleOption::Longform,
            });
        }
        if let Some(value) = options.v_bind_style() {
            self = self.with_v_bind_style(match value {
                ConfigVBindStyle::Shorthand => crate::rules::vue::VBindStyleOption::Shorthand,
                ConfigVBindStyle::Longform => crate::rules::vue::VBindStyleOption::Longform,
            });
        }
        if let Some(value) = options.attribute_hyphenation() {
            self = self.with_attribute_hyphenation(match value {
                ConfigHyphenationStyle::Always => HyphenationStyle::Always,
                ConfigHyphenationStyle::Never => HyphenationStyle::Never,
            });
        }
        if let Some(value) = options.valid_v_on_modifiers() {
            self = self.with_valid_v_on_modifiers(value.to_vec());
        }
        if let Some(value) = options.valid_v_slot_allow_modifiers() {
            self = self.with_valid_v_slot_allow_modifiers(value);
        }
        self
    }
}

fn component_casing(value: TemplateComponentNameCasing) -> ComponentCasing {
    match value {
        TemplateComponentNameCasing::PascalCase => ComponentCasing::PascalCase,
        TemplateComponentNameCasing::KebabCase => ComponentCasing::KebabCase,
    }
}

fn self_closing_style(value: ConfigSelfClosingStyle) -> HtmlSelfClosingStyle {
    match value {
        ConfigSelfClosingStyle::Always => HtmlSelfClosingStyle::Always,
        ConfigSelfClosingStyle::Never => HtmlSelfClosingStyle::Never,
        ConfigSelfClosingStyle::Any => HtmlSelfClosingStyle::Any,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LintPreset;

    #[test]
    fn cli_component_globals_reach_the_registration_rule() {
        let source = "<template><MyButton /></template>";
        let enabled = Some(vec!["vue/require-component-registration".into()]);
        let baseline = Linter::with_preset(LintPreset::Opinionated)
            .with_enabled_rules(enabled.clone())
            .lint_sfc(source, "App.vue");
        assert!(
            baseline
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.rule_name == "vue/require-component-registration")
        );

        let options: ConfigLintRuleOptions = serde_json::from_value(serde_json::json!({
            "vue/require-component-registration": { "globals": ["MyButton"] }
        }))
        .unwrap();
        let configured = Linter::with_preset(LintPreset::Opinionated)
            .with_enabled_rules(enabled)
            .with_config_rule_options(&options)
            .lint_sfc(source, "App.vue");
        assert!(
            !configured
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.rule_name == "vue/require-component-registration")
        );
    }
}
