//! Typed per-rule lint options.
//!
//! A handful of lint rules accept project-local configuration so teams can
//! enforce their own architecture and design-system conventions through
//! `vize lint` instead of running a sidecar tool. Options live under
//! `linter.ruleOptions.<rule-name>` and are parsed into typed structs (no
//! untyped `serde_json::Value`) so the schema is discoverable and validation
//! stays strict.

use serde::{Deserialize, Serialize};

use crate::String;

mod casing;
mod component_registration;
mod content_directives;
mod html_self_closing;
mod hyphenation;
mod merge;
mod no_mutating_props;
mod props_destructuring;
mod restrictions;
mod sfc_element_order;
mod strict_boolean;
mod valid_v_slot;

pub use casing::{
    ComponentNameInTemplateCasingOptions, CustomEventNameCasing, CustomEventNameCasingOptions,
    TemplateComponentNameCasing,
};
pub use html_self_closing::{
    HtmlSelfClosingHtmlOptions, HtmlSelfClosingOptions, HtmlSelfClosingStyle,
};
pub use hyphenation::HyphenationStyle;
pub use no_mutating_props::NoMutatingPropsOptions;
pub use props_destructuring::{DefinePropsDestructuringOptions, PropsDestructureMode};
pub use restrictions::{
    MuseaDesignToken, MuseaPreferDesignTokensOptions, NoRestrictedGlobalsOptions,
    NoRestrictedMembersOptions, RestrictedGlobal, RestrictedMember,
};
pub use sfc_element_order::{SfcElementOrderGroup, SfcElementOrderOptions};
pub use strict_boolean::StrictBooleanExpressionsOptions;

/// Per-rule configuration keyed by rule name.
///
/// Only the rules that actually accept options have typed fields; everything
/// else is ignored. The map is intentionally typed (rather than a free-form
/// `Value` bag) so unknown keys are rejected and the JSON schema is precise.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct LintRuleOptions {
    /// Options for `script/no-restricted-globals`.
    #[serde(rename = "script/no-restricted-globals")]
    pub no_restricted_globals: Option<NoRestrictedGlobalsOptions>,
    /// Options for `script/no-restricted-members`.
    #[serde(rename = "script/no-restricted-members")]
    pub no_restricted_members: Option<NoRestrictedMembersOptions>,
}

impl LintRuleOptions {
    /// Whether no rule options are configured.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.no_restricted_globals.is_none() && self.no_restricted_members.is_none()
    }

    /// Configured deny list for `script/no-restricted-globals` as
    /// `(name, optional message)` pairs. Empty when unconfigured.
    pub fn restricted_globals(&self) -> Vec<(String, Option<String>)> {
        self.no_restricted_globals
            .as_ref()
            .map(|options| {
                options
                    .globals
                    .iter()
                    .map(|global| (global.name.clone(), global.message.clone()))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Configured deny list for `script/no-restricted-members` as
    /// `(object, property, optional message)` tuples. Empty when unconfigured.
    pub fn restricted_members(&self) -> Vec<(String, String, Option<String>)> {
        self.no_restricted_members
            .as_ref()
            .map(|options| {
                options
                    .members
                    .iter()
                    .map(|member| {
                        (
                            member.object.clone(),
                            member.property.clone(),
                            member.message.clone(),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Apply a later config layer to this option set.
    pub fn merge_from(&mut self, overlay: &Self) {
        if let Some(options) = &overlay.no_restricted_globals {
            self.no_restricted_globals = Some(options.clone());
        }
        if let Some(options) = &overlay.no_restricted_members {
            self.no_restricted_members = Some(options.clone());
        }
    }
}

/// Full typed lint rule options parsed from config.
///
/// This wraps the stable public [`LintRuleOptions`] shape with new config-only
/// options, so existing Rust consumers can still construct `LintRuleOptions`
/// using the previous fields while the CLI and LSP can read newer rule options.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct ConfigLintRuleOptions {
    #[serde(flatten)]
    stable: LintRuleOptions,
    /// Options for `vue/require-component-registration`.
    #[serde(
        rename = "vue/require-component-registration",
        skip_serializing_if = "Option::is_none"
    )]
    require_component_registration: Option<component_registration::ComponentRegistrationOptions>,
    #[serde(
        rename = "html/no-empty-palpable-content",
        skip_serializing_if = "Option::is_none"
    )]
    no_empty_palpable_content: Option<content_directives::ContentDirectivesOptions>,
    #[serde(rename = "type/strict-boolean-expressions")]
    strict_boolean_expressions: Option<StrictBooleanExpressionsOptions>,
    /// Options for `script/define-props-destructuring`.
    #[serde(rename = "script/define-props-destructuring")]
    define_props_destructuring: Option<DefinePropsDestructuringOptions>,
    /// Options for `vue/component-name-in-template-casing`.
    #[serde(rename = "vue/component-name-in-template-casing")]
    component_name_in_template_casing: Option<ComponentNameInTemplateCasingOptions>,
    /// Options for `script/custom-event-name-casing`.
    #[serde(rename = "script/custom-event-name-casing")]
    custom_event_name_casing: Option<CustomEventNameCasingOptions>,
    /// Options for `vue/no-mutating-props`.
    #[serde(rename = "vue/no-mutating-props")]
    no_mutating_props: Option<NoMutatingPropsOptions>,
    /// Options for `vue/sfc-element-order`.
    #[serde(rename = "vue/sfc-element-order")]
    sfc_element_order: Option<SfcElementOrderOptions>,
    /// Options for `vue/html-self-closing`.
    #[serde(rename = "vue/html-self-closing")]
    html_self_closing: Option<HtmlSelfClosingOptions>,
    /// Options for `vue/v-on-event-hyphenation`.
    #[serde(rename = "vue/v-on-event-hyphenation")]
    v_on_event_hyphenation: Option<HyphenationStyle>,
    /// Options for `vue/attribute-hyphenation`.
    #[serde(rename = "vue/attribute-hyphenation")]
    attribute_hyphenation: Option<HyphenationStyle>,
    #[serde(rename = "vue/valid-v-slot", skip_serializing_if = "Option::is_none")]
    valid_v_slot: Option<valid_v_slot::ValidVSlotOptions>,
    /// Options for `musea/prefer-design-tokens`.
    #[serde(rename = "musea/prefer-design-tokens")]
    musea_prefer_design_tokens: Option<MuseaPreferDesignTokensOptions>,
}

impl ConfigLintRuleOptions {
    /// Build full config options from the stable public subset.
    #[inline]
    pub fn from_stable_options(stable: LintRuleOptions) -> Self {
        Self {
            stable,
            ..Self::default()
        }
    }

    /// Configured allowances; options alone do not enable the rule.
    pub fn strict_boolean_expressions(&self) -> Option<StrictBooleanExpressionsOptions> {
        self.strict_boolean_expressions
    }

    /// Stable subset exposed by the original `load_linter_rule_options` API.
    #[inline]
    pub fn stable_options(&self) -> &LintRuleOptions {
        &self.stable
    }

    /// Whether no rule options are configured.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.strict_boolean_expressions.is_none()
            && self.no_empty_palpable_content.is_none()
            && self.stable.is_empty()
            && self.define_props_destructuring.is_none()
            && self.component_name_in_template_casing.is_none()
            && self.custom_event_name_casing.is_none()
            && self.no_mutating_props.is_none()
            && self.sfc_element_order.is_none()
            && self.html_self_closing.is_none()
            && self.v_on_event_hyphenation.is_none()
            && self.attribute_hyphenation.is_none()
            && self.valid_v_slot.is_none()
            && self.musea_prefer_design_tokens.is_none()
            && self.require_component_registration.is_none()
    }

    /// Globally registered components permitted by the registration rule.
    pub fn component_registration_globals(&self) -> Option<&[String]> {
        self.require_component_registration
            .as_ref()
            .map(|options| options.globals.as_slice())
    }

    /// Configured deny list for `script/no-restricted-globals`.
    #[inline]
    pub fn restricted_globals(&self) -> Vec<(String, Option<String>)> {
        self.stable.restricted_globals()
    }

    /// Exact bare directive names that supply an element's visible content.
    pub fn palpable_content_directives(&self) -> Option<&[String]> {
        self.no_empty_palpable_content
            .as_ref()
            .map(|options| options.content_directives.as_slice())
    }

    /// Configured deny list for `script/no-restricted-members`.
    #[inline]
    pub fn restricted_members(&self) -> Vec<(String, String, Option<String>)> {
        self.stable.restricted_members()
    }

    /// Configured style for `script/define-props-destructuring`.
    #[inline]
    pub fn define_props_destructuring(&self) -> Option<PropsDestructureMode> {
        self.define_props_destructuring
            .map(|options| options.destructure)
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

    /// Configured options for `vue/no-mutating-props`.
    #[inline]
    pub fn no_mutating_props(&self) -> Option<NoMutatingPropsOptions> {
        self.no_mutating_props
    }

    /// Configured block order for `vue/sfc-element-order`.
    #[inline]
    pub fn sfc_element_order(&self) -> Option<SfcElementOrderOptions> {
        self.sfc_element_order.clone()
    }

    /// Configured self-closing style for `vue/html-self-closing`.
    #[inline]
    pub fn html_self_closing(&self) -> Option<HtmlSelfClosingOptions> {
        self.html_self_closing
    }

    /// Configured style for `vue/v-on-event-hyphenation`.
    #[inline]
    pub fn v_on_event_hyphenation(&self) -> Option<HyphenationStyle> {
        self.v_on_event_hyphenation
    }

    /// Configured style for `vue/attribute-hyphenation`.
    #[inline]
    pub fn attribute_hyphenation(&self) -> Option<HyphenationStyle> {
        self.attribute_hyphenation
    }

    /// Configured design-token primitive values for
    /// `musea/prefer-design-tokens` as `(value, path, tier)` tuples. Empty
    /// when unconfigured.
    pub fn musea_design_tokens(&self) -> Vec<(String, String, String)> {
        self.musea_prefer_design_tokens
            .as_ref()
            .map(|options| {
                options
                    .tokens
                    .iter()
                    .map(|token| (token.value.clone(), token.path.clone(), token.tier.clone()))
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests;
