//! Builder methods that thread project-local options into the two configurable
//! script rules (`script/no-restricted-globals`, `script/no-restricted-members`).
//!
//! These rules are `&'static` singletons in the registry, so a configured rule
//! is built as a boxed instance and stored in [`Linter::script_rule_overrides`];
//! the script-rule dispatch prefers the override over the static singleton.
//!
//! Refs: #1891 (project-local custom rules during migration).

use super::config::Linter;
use crate::rules::script::{NoRestrictedMembers, RestrictedGlobals};
use vize_carton::String;

impl Linter {
    /// Configure the deny list for `script/no-restricted-globals`.
    ///
    /// Each entry is `(name, optional message)`. A non-empty list **replaces**
    /// the rule's built-in defaults; an empty list leaves the defaults in place.
    /// Enabling the rule itself is still governed by the usual rule-enable
    /// configuration; this only customizes its data.
    #[inline]
    pub fn with_restricted_globals(mut self, globals: Vec<(String, Option<String>)>) -> Self {
        if globals.is_empty() {
            return self;
        }
        self.script_rule_overrides.insert(
            "script/no-restricted-globals",
            Box::new(RestrictedGlobals::configured(globals)),
        );
        self
    }

    /// Configure the member-access deny list for `script/no-restricted-members`.
    ///
    /// Each entry is `(object, property, optional message)`. The rule is off
    /// unless this list is non-empty (there is no built-in default).
    #[inline]
    pub fn with_restricted_members(
        mut self,
        members: Vec<(String, String, Option<String>)>,
    ) -> Self {
        if members.is_empty() {
            return self;
        }
        self.script_rule_overrides.insert(
            "script/no-restricted-members",
            Box::new(NoRestrictedMembers::configured(members)),
        );
        self
    }

    /// Apply both project-local script-rule deny lists in one call.
    #[inline]
    pub fn with_restricted_rules(
        self,
        globals: Vec<(String, Option<String>)>,
        members: Vec<(String, String, Option<String>)>,
    ) -> Self {
        self.with_restricted_globals(globals)
            .with_restricted_members(members)
    }
}
