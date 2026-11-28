//! Options for eslint-plugin-vue template rules.
//!
//! Each field uses the upstream option name and default. An omitted object
//! keeps the rule's built-in behavior. An authored object replaces the earlier
//! layer's object as a whole.

use serde::{Deserialize, Serialize};

use crate::String;

/// `vue/no-duplicate-attributes`. Both fields default to `true`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct NoDuplicateAttributesOptions {
    /// Whether `class` and `:class` may appear on one element.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_coexist_class: Option<bool>,
    /// Whether `style` and `:style` may appear on one element.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_coexist_style: Option<bool>,
}

/// `vue/no-multiple-template-root`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct NoMultipleTemplateRootOptions {
    /// Report comments placed directly in the template root.
    pub disallow_comments: bool,
}

/// `vue/no-lone-template`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct NoLoneTemplateOptions {
    /// Accept a `<template>` that has an `id` or a `ref`.
    pub ignore_accessible: bool,
}

/// `vue/no-static-inline-styles`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct NoStaticInlineStylesOptions {
    /// Accept static values inside a `:style` binding.
    pub allow_binding: bool,
}

/// `vue/no-unused-components`. `ignoreWhenBindingPresent` defaults to `true`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct NoUnusedComponentsOptions {
    /// Report nothing in a file whose template binds `is` dynamically.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore_when_binding_present: Option<bool>,
}

/// `vue/no-v-html`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct NoVHtmlOptions {
    /// Accept a `v-html` whose expression matches this regular expression.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore_pattern: Option<String>,
}

/// Flattened options for the configurable eslint-plugin-vue template rules.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct TemplateRuleOptions {
    /// `vue/no-duplicate-attributes`.
    #[serde(
        rename = "vue/no-duplicate-attributes",
        skip_serializing_if = "Option::is_none"
    )]
    pub no_duplicate_attributes: Option<NoDuplicateAttributesOptions>,
    /// `vue/no-multiple-template-root`.
    #[serde(
        rename = "vue/no-multiple-template-root",
        skip_serializing_if = "Option::is_none"
    )]
    pub no_multiple_template_root: Option<NoMultipleTemplateRootOptions>,
    /// `vue/no-lone-template`.
    #[serde(
        rename = "vue/no-lone-template",
        skip_serializing_if = "Option::is_none"
    )]
    pub no_lone_template: Option<NoLoneTemplateOptions>,
    /// `vue/no-static-inline-styles`.
    #[serde(
        rename = "vue/no-static-inline-styles",
        skip_serializing_if = "Option::is_none"
    )]
    pub no_static_inline_styles: Option<NoStaticInlineStylesOptions>,
    /// `vue/no-unused-components`.
    #[serde(
        rename = "vue/no-unused-components",
        skip_serializing_if = "Option::is_none"
    )]
    pub no_unused_components: Option<NoUnusedComponentsOptions>,
    /// `vue/no-v-html`.
    #[serde(rename = "vue/no-v-html", skip_serializing_if = "Option::is_none")]
    pub no_v_html: Option<NoVHtmlOptions>,
}

impl TemplateRuleOptions {
    /// Whether no template rule has options.
    pub fn is_empty(&self) -> bool {
        self.no_duplicate_attributes.is_none()
            && self.no_multiple_template_root.is_none()
            && self.no_lone_template.is_none()
            && self.no_static_inline_styles.is_none()
            && self.no_unused_components.is_none()
            && self.no_v_html.is_none()
    }

    /// Apply a later config layer. Each authored object replaces the earlier one.
    pub fn merge_from(&mut self, overlay: &Self) {
        if overlay.no_duplicate_attributes.is_some() {
            self.no_duplicate_attributes = overlay.no_duplicate_attributes;
        }
        if overlay.no_multiple_template_root.is_some() {
            self.no_multiple_template_root = overlay.no_multiple_template_root;
        }
        if overlay.no_lone_template.is_some() {
            self.no_lone_template = overlay.no_lone_template;
        }
        if overlay.no_static_inline_styles.is_some() {
            self.no_static_inline_styles = overlay.no_static_inline_styles;
        }
        if overlay.no_unused_components.is_some() {
            self.no_unused_components = overlay.no_unused_components;
        }
        if overlay.no_v_html.is_some() {
            self.no_v_html = overlay.no_v_html.clone();
        }
    }
}

impl super::ConfigLintRuleOptions {
    /// Options for the configurable template rules.
    #[inline]
    pub fn template_rules(&self) -> &TemplateRuleOptions {
        &self.template_rules
    }
}

#[cfg(test)]
mod tests;
