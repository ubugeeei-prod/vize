//! Options for the configurable eslint-plugin-vue rules.
//!
//! Each rule checks nothing until its option is present. A missing limit, or a
//! limit of zero, means unlimited.

use serde::{Deserialize, Serialize};

use crate::String;

/// `vue/max-props`. Absent or zero means unlimited.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct MaxPropsOptions {
    /// Maximum number of props. `None` or `0` checks nothing.
    pub max_props: Option<u32>,
}

/// `vue/max-template-depth`. Absent or zero means unlimited.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct MaxTemplateDepthOptions {
    /// Maximum element nesting. `None` or `0` checks nothing.
    pub max_depth: Option<u32>,
}

/// A string or a list of strings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RestrictedNames {
    /// One name.
    One(String),
    /// Several names.
    Many(Vec<String>),
}

impl RestrictedNames {
    /// Names the rule compares.
    pub fn names(&self) -> Vec<String> {
        match self {
            Self::One(name) => vec![name.clone()],
            Self::Many(names) => names.clone(),
        }
    }
}

/// One restricted call: an import from `module` reached through `path`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CallAfterAwaitSpec {
    /// Module specifier, for example `vue`.
    pub module: String,
    /// Import path. A missing path means `default`.
    #[serde(default)]
    pub path: Option<RestrictedNames>,
    /// Message that replaces the default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl CallAfterAwaitSpec {
    /// Path segments. A missing path is `default`.
    pub fn segments(&self) -> Vec<String> {
        match &self.path {
            Some(names) => {
                let names = names.names();
                if names.is_empty() {
                    vec!["default".into()]
                } else {
                    names
                }
            }
            None => vec!["default".into()],
        }
    }
}

/// One spec or a list of specs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CallAfterAwaitOptions {
    /// A single restriction.
    One(CallAfterAwaitSpec),
    /// Several restrictions.
    Many(Vec<CallAfterAwaitSpec>),
}

impl CallAfterAwaitOptions {
    /// Every configured restriction.
    pub fn restrictions(&self) -> Vec<CallAfterAwaitSpec> {
        match self {
            Self::One(spec) => vec![spec.clone()],
            Self::Many(specs) => specs.clone(),
        }
    }
}

/// `vue/no-unsupported-features`. An empty version checks nothing.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct UnsupportedFeaturesOptions {
    /// Vue version range. Empty checks nothing.
    pub version: Option<String>,
    /// Feature ids to skip, such as `define-model`.
    pub ignores: Option<Vec<String>>,
}

/// Flattened options for the thirteen configurable Vue rules.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ConfiguredVueRuleOptions {
    /// `vue/max-props`.
    #[serde(rename = "vue/max-props", skip_serializing_if = "Option::is_none")]
    pub max_props: Option<MaxPropsOptions>,
    /// `vue/max-template-depth`.
    #[serde(
        rename = "vue/max-template-depth",
        skip_serializing_if = "Option::is_none"
    )]
    pub max_template_depth: Option<MaxTemplateDepthOptions>,
    /// `vue/no-restricted-block`.
    #[serde(
        rename = "vue/no-restricted-block",
        skip_serializing_if = "Option::is_none"
    )]
    pub no_restricted_block: Option<RestrictedNames>,
    /// `vue/no-restricted-call-after-await`.
    #[serde(
        rename = "vue/no-restricted-call-after-await",
        skip_serializing_if = "Option::is_none"
    )]
    pub no_restricted_call_after_await: Option<CallAfterAwaitOptions>,
    /// `vue/no-restricted-class`.
    #[serde(
        rename = "vue/no-restricted-class",
        skip_serializing_if = "Option::is_none"
    )]
    pub no_restricted_class: Option<RestrictedNames>,
    /// `vue/no-restricted-component-names`.
    #[serde(
        rename = "vue/no-restricted-component-names",
        skip_serializing_if = "Option::is_none"
    )]
    pub no_restricted_component_names: Option<RestrictedNames>,
    /// `vue/no-restricted-component-options`.
    #[serde(
        rename = "vue/no-restricted-component-options",
        skip_serializing_if = "Option::is_none"
    )]
    pub no_restricted_component_options: Option<RestrictedNames>,
    /// `vue/no-restricted-custom-event`.
    #[serde(
        rename = "vue/no-restricted-custom-event",
        skip_serializing_if = "Option::is_none"
    )]
    pub no_restricted_custom_event: Option<RestrictedNames>,
    /// `vue/no-restricted-html-elements`.
    #[serde(
        rename = "vue/no-restricted-html-elements",
        skip_serializing_if = "Option::is_none"
    )]
    pub no_restricted_html_elements: Option<RestrictedNames>,
    /// `vue/no-restricted-props`.
    #[serde(
        rename = "vue/no-restricted-props",
        skip_serializing_if = "Option::is_none"
    )]
    pub no_restricted_props: Option<RestrictedNames>,
    /// `vue/no-restricted-static-attribute`.
    #[serde(
        rename = "vue/no-restricted-static-attribute",
        skip_serializing_if = "Option::is_none"
    )]
    pub no_restricted_static_attribute: Option<RestrictedNames>,
    /// `vue/no-restricted-v-on`.
    #[serde(
        rename = "vue/no-restricted-v-on",
        skip_serializing_if = "Option::is_none"
    )]
    pub no_restricted_v_on: Option<RestrictedNames>,
    /// `vue/no-unsupported-features`.
    #[serde(
        rename = "vue/no-unsupported-features",
        skip_serializing_if = "Option::is_none"
    )]
    pub no_unsupported_features: Option<UnsupportedFeaturesOptions>,
}

impl ConfiguredVueRuleOptions {
    /// Whether no configurable Vue rule has options.
    pub fn is_empty(&self) -> bool {
        self.max_props.is_none()
            && self.max_template_depth.is_none()
            && self.no_restricted_block.is_none()
            && self.no_restricted_call_after_await.is_none()
            && self.no_restricted_class.is_none()
            && self.no_restricted_component_names.is_none()
            && self.no_restricted_component_options.is_none()
            && self.no_restricted_custom_event.is_none()
            && self.no_restricted_html_elements.is_none()
            && self.no_restricted_props.is_none()
            && self.no_restricted_static_attribute.is_none()
            && self.no_restricted_v_on.is_none()
            && self.no_unsupported_features.is_none()
    }

    /// Apply a later config layer.
    pub fn merge_from(&mut self, overlay: &Self) {
        if overlay.max_props.is_some() {
            self.max_props = overlay.max_props.clone();
        }
        if overlay.max_template_depth.is_some() {
            self.max_template_depth = overlay.max_template_depth.clone();
        }
        if overlay.no_restricted_block.is_some() {
            self.no_restricted_block = overlay.no_restricted_block.clone();
        }
        if overlay.no_restricted_call_after_await.is_some() {
            self.no_restricted_call_after_await = overlay.no_restricted_call_after_await.clone();
        }
        if overlay.no_restricted_class.is_some() {
            self.no_restricted_class = overlay.no_restricted_class.clone();
        }
        if overlay.no_restricted_component_names.is_some() {
            self.no_restricted_component_names = overlay.no_restricted_component_names.clone();
        }
        if overlay.no_restricted_component_options.is_some() {
            self.no_restricted_component_options = overlay.no_restricted_component_options.clone();
        }
        if overlay.no_restricted_custom_event.is_some() {
            self.no_restricted_custom_event = overlay.no_restricted_custom_event.clone();
        }
        if overlay.no_restricted_html_elements.is_some() {
            self.no_restricted_html_elements = overlay.no_restricted_html_elements.clone();
        }
        if overlay.no_restricted_props.is_some() {
            self.no_restricted_props = overlay.no_restricted_props.clone();
        }
        if overlay.no_restricted_static_attribute.is_some() {
            self.no_restricted_static_attribute = overlay.no_restricted_static_attribute.clone();
        }
        if overlay.no_restricted_v_on.is_some() {
            self.no_restricted_v_on = overlay.no_restricted_v_on.clone();
        }
        if overlay.no_unsupported_features.is_some() {
            self.no_unsupported_features = overlay.no_unsupported_features.clone();
        }
    }
}

impl super::ConfigLintRuleOptions {
    /// Options for the configurable Vue rules.
    #[inline]
    pub fn configured_vue(&self) -> &ConfiguredVueRuleOptions {
        &self.configured
    }
}
