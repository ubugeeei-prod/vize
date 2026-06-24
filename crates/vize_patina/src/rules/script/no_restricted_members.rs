//! script/no-restricted-members
//!
//! Disallow `<object>.<property>` member accesses from a project-configured
//! list. The rule has **no built-in default list**: it only fires when a
//! project configures `linter.ruleOptions`, making it the project-local-rule
//! mechanism for migrating off sidecar ESLint architecture rules.
//!
//! Each configured entry is an `{ object, property }` pair (with an optional
//! advisory `message`). For example `{ "object": "window", "property":
//! "localStorage" }` flags `window.localStorage`. The object must be a bare
//! identifier reference -- `foo.window.localStorage` does not match because the
//! receiver of `.localStorage` there is not the identifier `window`.
//!
//! ## Configuration
//!
//! ```jsonc
//! {
//!   "linter": {
//!     "rules": { "script/no-restricted-members": "error" },
//!     "ruleOptions": {
//!       "script/no-restricted-members": {
//!         "members": [
//!           { "object": "window", "property": "localStorage", "message": "Use authStorage." },
//!           { "object": "globalThis", "property": "process" }
//!         ]
//!       }
//!     }
//!   }
//! }
//! ```
//!
//! ## Examples (with the config above)
//!
//! ### Invalid
//! ```ts
//! const token = window.localStorage.getItem('auth.token')
//! const env = globalThis.process.env
//! ```
//!
//! ### Valid
//! ```ts
//! const token = authStorage.read('auth.token')
//! ```
//!
//! Refs: #1891 (project-local custom rules during migration).

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{Expression, Program, StaticMemberExpression};
use oxc_ast_visit::{Visit, walk::walk_static_member_expression};
use vize_carton::String;

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/no-restricted-members",
    description: "Disallow project-configured object.property member accesses",
    default_severity: Severity::Error,
};

/// A configured restricted `<object>.<property>` access and its message.
struct RestrictedEntry {
    object: String,
    property: String,
    message: String,
}

/// Disallow `<object>.<property>` member accesses from a configured list.
///
/// Off unless configured: the static registry references a rule with no list,
/// so it never fires until a project supplies `members`.
pub struct NoRestrictedMembers {
    /// `None` (or empty) -> the rule never fires; `Some(_)` -> this list applies.
    members: Option<Box<[RestrictedEntry]>>,
}

impl NoRestrictedMembers {
    /// The built-in (empty) rule. Never fires until configured.
    pub const fn new() -> Self {
        Self { members: None }
    }

    /// Build a rule with a project-configured member list.
    ///
    /// `entries` is `(object, property, optional message)`; when a message is
    /// absent a generic advisory is generated.
    pub fn configured<I, M>(entries: I) -> Self
    where
        I: IntoIterator<Item = (String, String, Option<M>)>,
        M: Into<String>,
    {
        let members: Box<[RestrictedEntry]> = entries
            .into_iter()
            .map(|(object, property, message)| {
                let message = message.map(Into::into).unwrap_or_else(|| {
                    vize_carton::cstr!("Don't access `{object}.{property}` directly.")
                });
                RestrictedEntry {
                    object,
                    property,
                    message,
                }
            })
            .collect();
        Self {
            members: (!members.is_empty()).then_some(members),
        }
    }

    /// Whether this rule has any configured entries.
    #[inline]
    fn is_active(&self) -> bool {
        self.members.is_some()
    }

    /// Message for an `object.property` access, if restricted.
    fn advice_for(&self, object: &str, property: &str) -> Option<&str> {
        self.members
            .as_ref()?
            .iter()
            .find(|entry| entry.object == object && entry.property == property)
            .map(|entry| entry.message.as_str())
    }
}

impl Default for NoRestrictedMembers {
    fn default() -> Self {
        Self::new()
    }
}

impl ScriptRule for NoRestrictedMembers {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &META
    }

    #[inline]
    fn uses_ast(&self) -> bool {
        true
    }

    #[inline]
    fn check_program<'a>(
        &self,
        program: &'a Program<'a>,
        _source: &str,
        offset: usize,
        result: &mut ScriptLintResult,
    ) {
        if !self.is_active() {
            return;
        }
        let mut visitor = RestrictedMembersVisitor {
            rule: self,
            offset,
            result,
        };
        visitor.visit_program(program);
    }
}

struct RestrictedMembersVisitor<'r> {
    rule: &'r NoRestrictedMembers,
    offset: usize,
    result: &'r mut ScriptLintResult,
}

impl<'a> Visit<'a> for RestrictedMembersVisitor<'_> {
    fn visit_static_member_expression(&mut self, it: &StaticMemberExpression<'a>) {
        if let Expression::Identifier(object) = &it.object
            && let Some(advice) = self
                .rule
                .advice_for(object.name.as_str(), it.property.name.as_str())
        {
            self.result.add_diagnostic(LintDiagnostic::error(
                META.name,
                advice,
                self.offset as u32 + it.span.start,
                self.offset as u32 + it.span.end,
            ));
        }

        walk_static_member_expression(self, it);
    }
}

#[cfg(test)]
mod tests {
    use super::NoRestrictedMembers;
    use crate::diagnostic::Severity;
    use crate::rules::script::ScriptLinter;
    use vize_carton::String;

    fn configured_linter(entries: Vec<(&str, &str, Option<&str>)>) -> ScriptLinter {
        let mut linter = ScriptLinter::new();
        let entries = entries.into_iter().map(|(object, property, message)| {
            (
                String::from(object),
                String::from(property),
                message.map(String::from),
            )
        });
        linter.add_rule(Box::new(NoRestrictedMembers::configured(entries)));
        linter
    }

    #[test]
    fn unconfigured_rule_never_fires() {
        let mut linter = ScriptLinter::new();
        linter.add_rule(Box::new(NoRestrictedMembers::new()));
        let result = linter.lint("const x = window.localStorage.getItem('k');", 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn flags_configured_member_access() {
        let linter = configured_linter(vec![("window", "localStorage", Some("Use authStorage."))]);
        let result = linter.lint("const x = window.localStorage.getItem('k');", 0);
        assert_eq!(result.error_count, 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.rule_name, "script/no-restricted-members");
        assert_eq!(diag.severity, Severity::Error);
        assert_eq!(diag.message, "Use authStorage.");
        // Span covers `window.localStorage` (offsets 10..29 in the source).
        assert_eq!(diag.start, 10);
        assert_eq!(diag.end, 29);
    }

    #[test]
    fn missing_message_uses_generic_advice() {
        let linter = configured_linter(vec![("globalThis", "process", None)]);
        let result = linter.lint("const env = globalThis.process.env;", 0);
        assert_eq!(result.error_count, 1);
        assert_eq!(
            result.diagnostics[0].message,
            "Don't access `globalThis.process` directly."
        );
    }

    #[test]
    fn does_not_flag_unlisted_member() {
        let linter = configured_linter(vec![("window", "localStorage", None)]);
        let result = linter.lint("const x = window.sessionStorage.getItem('k');", 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn does_not_flag_when_object_is_not_a_bare_identifier() {
        // The receiver of `.localStorage` is `foo.window`, not the identifier
        // `window`, so it must not match.
        let linter = configured_linter(vec![("window", "localStorage", None)]);
        let result = linter.lint("const x = foo.window.localStorage;", 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn flags_multiple_distinct_members() {
        let linter = configured_linter(vec![
            ("window", "localStorage", Some("a")),
            ("window", "sessionStorage", Some("b")),
        ]);
        let source = "window.localStorage.clear(); window.sessionStorage.clear();";
        let result = linter.lint(source, 0);
        assert_eq!(result.error_count, 2);
        assert_eq!(result.diagnostics[0].message, "a");
        assert_eq!(result.diagnostics[1].message, "b");
    }
}
