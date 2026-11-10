//! Methods on the stable public lint-rule option subset.

use super::LintRuleOptions;
use crate::String;

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
