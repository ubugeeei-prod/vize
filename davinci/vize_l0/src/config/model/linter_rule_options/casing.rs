use serde::{Deserialize, Serialize};

use super::ConfigLintRuleOptions;
use super::multi_word_component_names::deserialize_unique_ignores;

/// Options for `vue/component-name-in-template-casing`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct ComponentNameInTemplateCasingOptions {
    pub casing: TemplateComponentNameCasing,
    pub registered_components_only: bool,
    pub globals: Vec<crate::String>,
    /// Authored tag names the rule leaves alone: an exact name or a
    /// `/pattern/flags` regular expression, as in eslint-plugin-vue.
    #[serde(
        deserialize_with = "deserialize_unique_ignores",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub ignores: Vec<crate::String>,
}

impl Default for ComponentNameInTemplateCasingOptions {
    fn default() -> Self {
        Self {
            casing: TemplateComponentNameCasing::default(),
            registered_components_only: true,
            globals: Vec::new(),
            ignores: Vec::new(),
        }
    }
}

/// Component tag casing accepted by `vue/component-name-in-template-casing`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TemplateComponentNameCasing {
    #[default]
    #[serde(rename = "PascalCase")]
    PascalCase,
    #[serde(rename = "kebab-case")]
    KebabCase,
}

/// Options for `script/custom-event-name-casing`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct CustomEventNameCasingOptions {
    pub casing: CustomEventNameCasing,
}

/// Event name casing accepted by `script/custom-event-name-casing`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum CustomEventNameCasing {
    #[default]
    #[serde(rename = "camelCase")]
    CamelCase,
    #[serde(rename = "lowercase")]
    Lowercase,
    #[serde(rename = "kebab-case")]
    KebabCase,
}

/// Casing required of a component definition's `name` by
/// `vue/component-definition-name-casing`, written as eslint-plugin-vue's
/// direct option: `"PascalCase"` (the default) or `"kebab-case"`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComponentDefinitionNameCasingStyle {
    #[default]
    #[serde(rename = "PascalCase")]
    PascalCase,
    #[serde(rename = "kebab-case")]
    KebabCase,
}

impl ConfigLintRuleOptions {
    /// Configured casing for `vue/component-definition-name-casing`.
    pub fn component_definition_name_casing(&self) -> Option<ComponentDefinitionNameCasingStyle> {
        self.component_definition_name_casing
    }

    /// Configured casing for `vue/component-name-in-template-casing`.
    #[inline]
    pub fn component_name_in_template_casing(&self) -> Option<TemplateComponentNameCasing> {
        self.component_name_in_template_casing
            .as_ref()
            .map(|options| options.casing)
    }

    /// Authored registration scope for component tag casing.
    pub fn component_name_in_template_casing_options(
        &self,
    ) -> Option<&ComponentNameInTemplateCasingOptions> {
        self.component_name_in_template_casing.as_ref()
    }

    /// Configured casing for `script/custom-event-name-casing`.
    #[inline]
    pub fn custom_event_name_casing(&self) -> Option<CustomEventNameCasing> {
        self.custom_event_name_casing
            .as_ref()
            .map(|options| options.casing)
    }
}
