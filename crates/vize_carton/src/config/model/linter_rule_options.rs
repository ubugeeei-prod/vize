//! Typed per-rule lint options.
//!
//! A handful of script rules accept project-local configuration so teams can
//! enforce their own architecture conventions (e.g. forbidding direct access to
//! `process` or `window.localStorage`) through `vize lint` instead of running a
//! sidecar ESLint. Options live under `linter.ruleOptions.<rule-name>` and are
//! parsed into typed structs (no untyped `serde_json::Value`) so the schema is
//! discoverable and option payload validation stays strict.
//!
//! Refs: #1891 (project-local custom rules during migration).

use serde::{Deserialize, Serialize};

use crate::String;

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
    /// Options for `vue/component-name-in-template-casing`.
    #[serde(rename = "vue/component-name-in-template-casing")]
    component_name_in_template_casing: Option<ComponentNameInTemplateCasingOptions>,
    /// Options for `script/custom-event-name-casing`.
    #[serde(rename = "script/custom-event-name-casing")]
    custom_event_name_casing: Option<CustomEventNameCasingOptions>,
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

    /// Stable subset exposed by the original `load_linter_rule_options` API.
    #[inline]
    pub fn stable_options(&self) -> &LintRuleOptions {
        &self.stable
    }

    /// Whether no rule options are configured.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.stable.is_empty()
            && self.component_name_in_template_casing.is_none()
            && self.custom_event_name_casing.is_none()
    }

    /// Configured deny list for `script/no-restricted-globals`.
    #[inline]
    pub fn restricted_globals(&self) -> Vec<(String, Option<String>)> {
        self.stable.restricted_globals()
    }

    /// Configured deny list for `script/no-restricted-members`.
    #[inline]
    pub fn restricted_members(&self) -> Vec<(String, String, Option<String>)> {
        self.stable.restricted_members()
    }

    /// Configured casing for `vue/component-name-in-template-casing`.
    #[inline]
    pub fn component_name_in_template_casing(&self) -> Option<TemplateComponentNameCasing> {
        self.component_name_in_template_casing
            .as_ref()
            .map(|options| options.casing)
    }

    /// Configured casing for `script/custom-event-name-casing`.
    #[inline]
    pub fn custom_event_name_casing(&self) -> Option<CustomEventNameCasing> {
        self.custom_event_name_casing
            .as_ref()
            .map(|options| options.casing)
    }

    /// Apply a later config layer to this option set.
    pub fn merge_from(&mut self, overlay: &Self) {
        self.stable.merge_from(&overlay.stable);
        if let Some(options) = &overlay.component_name_in_template_casing {
            self.component_name_in_template_casing = Some(*options);
        }
        if let Some(options) = &overlay.custom_event_name_casing {
            self.custom_event_name_casing = Some(*options);
        }
    }
}

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

/// Options for `script/no-restricted-globals`.
///
/// When `globals` is non-empty it **replaces** the rule's built-in deny list;
/// otherwise the built-in defaults (`process`, `localStorage`, `sessionStorage`)
/// apply.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct NoRestrictedGlobalsOptions {
    /// Restricted global identifier references.
    pub globals: Vec<RestrictedGlobal>,
}

/// A single restricted global entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RestrictedGlobal {
    /// Identifier name to forbid (e.g. `process`).
    pub name: String,
    /// Optional advisory message shown in the diagnostic.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Options for `script/no-restricted-members`.
///
/// The rule is off unless `members` is configured; there is no built-in default
/// list. This is the project-local-rule mechanism: each entry flags an
/// `<object>.<property>` member access.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct NoRestrictedMembersOptions {
    /// Restricted `<object>.<property>` member accesses.
    pub members: Vec<RestrictedMember>,
}

/// A single restricted member-access entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RestrictedMember {
    /// Object identifier (e.g. `window`).
    pub object: String,
    /// Property name accessed on the object (e.g. `localStorage`).
    pub property: String,
    /// Optional advisory message shown in the diagnostic.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::{
        ComponentNameInTemplateCasingOptions, ConfigLintRuleOptions, CustomEventNameCasing,
        CustomEventNameCasingOptions, LintRuleOptions, RestrictedGlobal, RestrictedMember,
        TemplateComponentNameCasing,
    };

    #[test]
    fn empty_options_deserialize_to_default() {
        let options = serde_json::from_str::<LintRuleOptions>("{}").unwrap();
        assert_eq!(options, LintRuleOptions::default());
        assert!(options.is_empty());
    }

    #[test]
    fn deserializes_restricted_globals_with_and_without_message() {
        let json = r#"{
            "script/no-restricted-globals": {
                "globals": [
                    { "name": "process", "message": "Use a typed config helper." },
                    { "name": "localStorage" }
                ]
            }
        }"#;
        let options = serde_json::from_str::<LintRuleOptions>(json).unwrap();
        let globals = options.no_restricted_globals.unwrap().globals;
        assert_eq!(
            globals,
            vec![
                RestrictedGlobal {
                    name: "process".into(),
                    message: Some("Use a typed config helper.".into()),
                },
                RestrictedGlobal {
                    name: "localStorage".into(),
                    message: None,
                },
            ]
        );
        assert!(options.no_restricted_members.is_none());
    }

    #[test]
    fn deserializes_restricted_members() {
        let json = r#"{
            "script/no-restricted-members": {
                "members": [
                    { "object": "window", "property": "localStorage", "message": "Use authStorage." },
                    { "object": "globalThis", "property": "process" }
                ]
            }
        }"#;
        let options = serde_json::from_str::<LintRuleOptions>(json).unwrap();
        let members = options.no_restricted_members.unwrap().members;
        assert_eq!(
            members,
            vec![
                RestrictedMember {
                    object: "window".into(),
                    property: "localStorage".into(),
                    message: Some("Use authStorage.".into()),
                },
                RestrictedMember {
                    object: "globalThis".into(),
                    property: "process".into(),
                    message: None,
                },
            ]
        );
        assert!(options.no_restricted_globals.is_none());
    }

    #[test]
    fn deserializes_casing_options() {
        let json = r#"{
            "vue/component-name-in-template-casing": { "casing": "kebab-case" },
            "script/custom-event-name-casing": { "casing": "camelCase" }
        }"#;
        let options = serde_json::from_str::<ConfigLintRuleOptions>(json).unwrap();
        assert_eq!(
            options.component_name_in_template_casing,
            Some(ComponentNameInTemplateCasingOptions {
                casing: TemplateComponentNameCasing::KebabCase
            })
        );
        assert_eq!(
            options.custom_event_name_casing,
            Some(CustomEventNameCasingOptions {
                casing: CustomEventNameCasing::CamelCase
            })
        );
        assert_eq!(
            options.component_name_in_template_casing(),
            Some(TemplateComponentNameCasing::KebabCase)
        );
        assert_eq!(
            options.custom_event_name_casing(),
            Some(CustomEventNameCasing::CamelCase)
        );
    }

    #[test]
    fn stable_lint_rule_options_keep_legacy_shape() {
        let json = r#"{
            "script/no-restricted-globals": {
                "globals": [{ "name": "process" }]
            },
            "vue/component-name-in-template-casing": { "casing": "kebab-case" }
        }"#;
        let options = serde_json::from_str::<ConfigLintRuleOptions>(json).unwrap();
        assert_eq!(
            options.stable_options().restricted_globals(),
            [("process".into(), None)]
        );
        assert!(options.component_name_in_template_casing().is_some());
    }

    #[test]
    fn unknown_fields_are_rejected() {
        // Typed structs reject unknown keys inside an entry so config typos surface.
        let json = r#"{
            "script/no-restricted-globals": {
                "globals": [{ "name": "process", "bogus": true }]
            }
        }"#;
        assert!(serde_json::from_str::<LintRuleOptions>(json).is_err());
    }
}
