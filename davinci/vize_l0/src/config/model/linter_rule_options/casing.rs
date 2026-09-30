use serde::{Deserialize, Serialize};

/// Options for `vue/component-name-in-template-casing`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct ComponentNameInTemplateCasingOptions {
    pub casing: TemplateComponentNameCasing,
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
    #[serde(rename = "kebab-case")]
    KebabCase,
}
