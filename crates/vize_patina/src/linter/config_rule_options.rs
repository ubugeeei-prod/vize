//! Shared application of the public `linter.ruleOptions` shape.

use super::config::Linter;
use crate::rules::{
    ComponentCasing, HtmlSelfClosingHtmlOptions, HtmlSelfClosingOptions, HtmlSelfClosingStyle,
    HyphenationStyle, NoMutatingPropsOptions, SfcElementOrderGroup, SfcElementOrderOptions,
    VOnEventHyphenationStyle,
};
use vize_l0::config::{
    ConfigLintRuleOptions, CustomEventNameCasing, HtmlQuoteStyle as ConfigHtmlQuoteStyle,
    HtmlSelfClosingStyle as ConfigSelfClosingStyle, HyphenationStyle as ConfigHyphenationStyle,
    TemplateComponentNameCasing, VBindStyle as ConfigVBindStyle, VOnStyle as ConfigVOnStyle,
    VSlotNamedStyle as ConfigVSlotNamedStyle, VSlotStyle as ConfigVSlotStyle,
    VSlotStyleValue as ConfigVSlotStyleValue, VSlotUniformStyle as ConfigVSlotUniformStyle,
};

impl Linter {
    /// Apply the same typed per-rule options used by the CLI and language server.
    pub fn with_config_rule_options(mut self, options: &ConfigLintRuleOptions) -> Self {
        self = self
            .with_restricted_globals(options.restricted_globals())
            .with_restricted_members(options.restricted_members())
            .with_musea_design_tokens(options.musea_design_tokens());
        if let Some(globals) = options.component_registration_globals() {
            self = self.with_component_registration_options(
                globals.to_vec(),
                options.component_registration_ignore_patterns(),
            );
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
        if let Some(value) = options.define_macros_order() {
            self = self.with_define_macros_order(value);
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
                CustomEventNameCasing::Lowercase => {
                    crate::rules::script::EventNameCasing::Lowercase
                }
                CustomEventNameCasing::KebabCase => {
                    crate::rules::script::EventNameCasing::KebabCase
                }
            });
        }
        if let Some(value) = options.prop_name_casing() {
            self = self.with_prop_name_casing(match value {
                vize_l0::config::PropNameCasingStyle::CamelCase => {
                    crate::rules::vue::PropNameCasingOption::CamelCase
                }
                vize_l0::config::PropNameCasingStyle::SnakeCase => {
                    crate::rules::vue::PropNameCasingOption::SnakeCase
                }
            });
        }
        if let Some(value) = options.slot_name_casing() {
            self = self.with_slot_name_casing(match value {
                vize_l0::config::SlotNameCasingStyle::CamelCase => {
                    crate::rules::vue::SlotNameCasingStyle::CamelCase
                }
                vize_l0::config::SlotNameCasingStyle::KebabCase => {
                    crate::rules::vue::SlotNameCasingStyle::KebabCase
                }
                vize_l0::config::SlotNameCasingStyle::Singleword => {
                    crate::rules::vue::SlotNameCasingStyle::Singleword
                }
            });
        }
        if let Some(value) = options.no_mutating_props() {
            self = self.with_no_mutating_props_options(NoMutatingPropsOptions {
                shallow_only: value.shallow_only,
            });
        }
        if let Some(value) = options.html_button_has_type() {
            self = self.with_html_button_has_type_options(value.button, value.submit, value.reset);
        }
        if let Some(value) = options.no_use_v_if_with_v_for() {
            self = self.with_no_use_v_if_with_v_for_options(value.allow_using_iteration_var);
        }
        if let Some(value) = options.warn_custom_directive() {
            self = self.with_warn_custom_directive_ignore(value.ignore.clone());
        }
        if let Some(value) = options.no_bare_strings_in_template() {
            self = self.with_no_bare_strings_allowlist(value.allowlist.clone());
        }
        if let Some(value) = options.anchor_has_content() {
            self = self.with_anchor_accessible_children(value.accessible_children.clone());
        }
        if let Some(value) = options.label_has_for() {
            self = self.with_label_control_components(value.control_components.clone());
        }
        if let Some(value) = options.enforce_style_attribute() {
            use crate::rules::vue::StyleAttribute;
            use vize_l0::config::StyleAttributeMode;

            let allowed = value
                .allow
                .iter()
                .map(|mode| match mode {
                    StyleAttributeMode::Plain => StyleAttribute::Plain,
                    StyleAttributeMode::Scoped => StyleAttribute::Scoped,
                    StyleAttributeMode::Module => StyleAttribute::Module,
                })
                .collect::<Vec<_>>();
            self.registry
                .replace(Box::new(crate::rules::vue::EnforceStyleAttribute::new(
                    &allowed,
                )));
        }
        if let Some(value) = options.no_restricted_syntax() {
            self.registry
                .replace(Box::new(crate::rules::vue::NoRestrictedSyntax::new(
                    value
                        .selectors
                        .iter()
                        .map(|entry| (entry.selector.clone(), entry.message.clone()))
                        .collect(),
                )));
        }
        if options.quote_props().is_some() {
            self.registry
                .replace(Box::new(crate::rules::vue::QuoteProps));
        }
        if let Some(value) = options.sort_keys() {
            self.registry
                .replace(Box::new(crate::rules::vue::SortKeys::new(
                    matches!(value.order, vize_l0::config::SortKeysOrder::Desc),
                    value.case_sensitive,
                    value.natural,
                )));
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
        if let Some(value) = options.html_quotes() {
            self = self.with_html_quotes(crate::rules::vue::HtmlQuotes {
                style: match value.style() {
                    ConfigHtmlQuoteStyle::Double => crate::rules::vue::HtmlQuotesOption::Double,
                    ConfigHtmlQuoteStyle::Single => crate::rules::vue::HtmlQuotesOption::Single,
                },
                avoid_escape: value.avoid_escape(),
            });
        }
        if let Some(value) = options.mustache_interpolation_spacing() {
            self = self.with_mustache_interpolation_spacing(match value {
                vize_l0::config::MustacheInterpolationSpacingStyle::Always => {
                    crate::rules::vue::MustacheInterpolationSpacingStyle::Always
                }
                vize_l0::config::MustacheInterpolationSpacingStyle::Never => {
                    crate::rules::vue::MustacheInterpolationSpacingStyle::Never
                }
            });
        }
        if let Some(value) = options.v_on_event_hyphenation_options() {
            let style = match value.style {
                ConfigHyphenationStyle::Always => VOnEventHyphenationStyle::Always,
                ConfigHyphenationStyle::Never => VOnEventHyphenationStyle::Never,
            };
            self = self.with_v_on_event_hyphenation_policy(style, value.ignore, value.autofix);
        }
        if let Some(value) = options.v_on_style() {
            self = self.with_v_on_style(match value {
                ConfigVOnStyle::Shorthand => crate::rules::vue::VOnStyleOption::Shorthand,
                ConfigVOnStyle::Longform => crate::rules::vue::VOnStyleOption::Longform,
            });
        }
        if let Some(value) = options.v_slot_style() {
            self = self.with_v_slot_style(v_slot_style(value));
        }
        if let Some(value) = options.v_bind_style() {
            self = self.with_v_bind_style(match value {
                ConfigVBindStyle::Shorthand => crate::rules::vue::VBindStyleOption::Shorthand,
                ConfigVBindStyle::Longform => crate::rules::vue::VBindStyleOption::Longform,
            });
        }
        if let Some(value) = options.attribute_hyphenation() {
            let style = match value {
                ConfigHyphenationStyle::Always => HyphenationStyle::Always,
                ConfigHyphenationStyle::Never => HyphenationStyle::Never,
            };
            self = match options.attribute_hyphenation_ignore() {
                Some(ignore) => self.with_attribute_hyphenation_policy(
                    style,
                    ignore.to_vec(),
                    options
                        .attribute_hyphenation_ignore_tags()
                        .unwrap_or_default()
                        .to_vec(),
                ),
                None => self.with_attribute_hyphenation(style),
            };
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

fn v_slot_style(value: ConfigVSlotStyle) -> crate::rules::vue::VSlotStyle {
    use crate::rules::vue::{VSlotStyle, VSlotStyleOption};

    match value {
        ConfigVSlotStyle::Uniform(style) => VSlotStyle::uniform(match style {
            ConfigVSlotUniformStyle::Shorthand => VSlotStyleOption::Shorthand,
            ConfigVSlotUniformStyle::Longform => VSlotStyleOption::Longform,
        }),
        ConfigVSlotStyle::ByPosition(options) => VSlotStyle {
            at_component: v_slot_style_value(options.at_component),
            default_slot: v_slot_style_value(options.default_slot),
            named: match options.named {
                ConfigVSlotNamedStyle::Shorthand => VSlotStyleOption::Shorthand,
                ConfigVSlotNamedStyle::Longform => VSlotStyleOption::Longform,
            },
        },
    }
}

fn v_slot_style_value(value: ConfigVSlotStyleValue) -> crate::rules::vue::VSlotStyleOption {
    match value {
        ConfigVSlotStyleValue::Shorthand => crate::rules::vue::VSlotStyleOption::Shorthand,
        ConfigVSlotStyleValue::Longform => crate::rules::vue::VSlotStyleOption::Longform,
        ConfigVSlotStyleValue::VSlot => crate::rules::vue::VSlotStyleOption::VSlot,
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
mod macro_order;

#[cfg(test)]
mod no_use_v_if_with_v_for;

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

    #[test]
    fn html_quote_config_reaches_style_and_avoid_escape() {
        let source = r#"<template><div class="card" title="don't"></div></template>"#;
        let options: ConfigLintRuleOptions = serde_json::from_value(serde_json::json!({
            "vue/html-quotes": { "style": "single", "avoidEscape": true }
        }))
        .unwrap();
        let result = Linter::with_preset(LintPreset::Incremental)
            .with_enabled_rules(Some(vec!["vue/html-quotes".into()]))
            .with_config_rule_options(&options)
            .lint_sfc(source, "App.vue");
        assert_eq!(result.warning_count, 1);
        assert_eq!(result.diagnostics[0].rule_name, "vue/html-quotes");
        assert_eq!(
            &source[result.diagnostics[0].start as usize..result.diagnostics[0].end as usize],
            "\"card\""
        );
    }

    #[test]
    fn mustache_spacing_config_reaches_the_selected_rule() {
        let source = "<template><div>{{ value }}</div></template>";
        let options: ConfigLintRuleOptions = serde_json::from_value(serde_json::json!({
            "vue/mustache-interpolation-spacing": "never"
        }))
        .unwrap();
        let result = Linter::with_preset(LintPreset::Incremental)
            .with_enabled_rules(Some(vec!["vue/mustache-interpolation-spacing".into()]))
            .with_config_rule_options(&options)
            .lint_sfc(source, "App.vue");
        assert_eq!(result.warning_count, 2);
        assert!(
            result
                .diagnostics
                .iter()
                .all(|diagnostic| { diagnostic.rule_name == "vue/mustache-interpolation-spacing" })
        );
    }

    #[test]
    fn html_button_type_config_reaches_the_selected_rule() {
        let source = r#"<template><button type="button">Cancel</button><button type="submit">Save</button><button type="reset">Reset</button></template>"#;
        let options: ConfigLintRuleOptions = serde_json::from_value(serde_json::json!({
            "vue/html-button-has-type": {
                "button": true,
                "submit": false,
                "reset": false
            }
        }))
        .unwrap();
        let result = Linter::with_preset(LintPreset::Opinionated)
            .with_enabled_rules(Some(vec!["vue/html-button-has-type".into()]))
            .with_config_rule_options(&options)
            .lint_sfc(source, "App.vue");
        assert_eq!(result.warning_count, 2, "{:#?}", result.diagnostics);
        assert_eq!(
            result
                .diagnostics
                .iter()
                .map(|diagnostic| (diagnostic.rule_name, diagnostic.message.as_str()))
                .collect::<Vec<_>>(),
            vec![
                (
                    "vue/html-button-has-type",
                    "'submit' is forbidden by the configured '<button>' type policy",
                ),
                (
                    "vue/html-button-has-type",
                    "'reset' is forbidden by the configured '<button>' type policy",
                ),
            ]
        );
    }

    #[test]
    fn reported_real_project_options_remove_only_the_configured_findings() {
        let source = r#"<script setup>
const tip = "Copied";
const version = "1.2.0";
const count = 3;
const alt = "Get the app";
const label = "Name";
const name = "";
</script>
<template>
  <button type="button" v-hover-tooltip="tip">{{ tip }}</button>
  <span>v{{ version }}</span>
  <span>x{{ count }}</span>
  <kbd>W</kbd>
  <a href="https://example.com/app">
    <RetryImage src="/badge.png" :alt="alt" />
  </a>
  <label>
    {{ label }}
    <CInputField v-model="name" />
  </label>
  <button v-unregistered>Other</button>
</template>"#;
        let enabled = Some(vec![
            "vue/warn-custom-directive".into(),
            "vue/no-bare-strings-in-template".into(),
            "a11y/anchor-has-content".into(),
            "a11y/label-has-for".into(),
        ]);
        let options: ConfigLintRuleOptions = serde_json::from_value(serde_json::json!({
            "vue/warn-custom-directive": { "ignore": ["hover-tooltip"] },
            "vue/no-bare-strings-in-template": {
                "allowlist": ["/^[DRWXYmvx]$/u"]
            },
            "a11y/anchor-has-content": { "accessibleChildren": ["RetryImage"] },
            "a11y/label-has-for": { "controlComponents": ["CInputField"] }
        }))
        .unwrap();
        let result = Linter::with_preset(LintPreset::Incremental)
            .with_enabled_rules(enabled)
            .with_config_rule_options(&options)
            .lint_sfc(source, "Repro.vue");
        let rules = result
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.rule_name)
            .collect::<Vec<_>>();
        assert_eq!(
            rules,
            [
                "vue/warn-custom-directive",
                "vue/no-bare-strings-in-template"
            ],
            "{:#?}",
            result.diagnostics
        );
    }

    #[test]
    fn eslint_vue_compatibility_options_reach_selected_rules() {
        let source = r#"<script>
export default { z: 1, a: 2 }
</script>
<template>{{ condition ? { "quoted": value }.quoted : fallback }}</template>
<style>.card { color: red }</style>"#;
        let options: ConfigLintRuleOptions = serde_json::from_value(serde_json::json!({
            "vue/enforce-style-attribute": { "allow": ["plain", "scoped"] },
            "vue/no-restricted-syntax": { "selectors": ["ConditionalExpression"] },
            "vue/quote-props": "as-needed",
            "vue/sort-keys": {
                "order": "desc",
                "caseSensitive": true,
                "natural": false
            }
        }))
        .unwrap();
        let result = Linter::with_preset(LintPreset::Incremental)
            .with_enabled_rules(Some(vec![
                "vue/enforce-style-attribute".into(),
                "vue/no-restricted-syntax".into(),
                "vue/quote-props".into(),
                "vue/sort-keys".into(),
            ]))
            .with_config_rule_options(&options)
            .lint_sfc(source, "App.vue");
        let rules = result
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.rule_name)
            .collect::<Vec<_>>();
        assert_eq!(
            rules,
            ["vue/no-restricted-syntax", "vue/quote-props"],
            "{:#?}",
            result.diagnostics
        );
    }
}
