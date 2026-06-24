//! Typed per-rule lint options.
//!
//! A handful of script rules accept project-local configuration so teams can
//! enforce their own architecture conventions (e.g. forbidding direct access to
//! `process` or `window.localStorage`) through `vize lint` instead of running a
//! sidecar ESLint. Options live under `linter.ruleOptions.<rule-name>` and are
//! parsed into typed structs (no untyped `serde_json::Value`) so the schema is
//! discoverable and validation stays strict.
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
    use super::{LintRuleOptions, RestrictedGlobal, RestrictedMember};

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
