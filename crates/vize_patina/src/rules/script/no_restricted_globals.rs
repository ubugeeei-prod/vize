//! script/no-restricted-globals
//!
//! Disallow references to a set of restricted globals. The built-in default
//! list is `process` / `localStorage` / `sessionStorage`; a project can replace
//! it via `linter.ruleOptions` to enforce its own architecture boundaries when
//! migrating off sidecar ESLint rules like `no-access-process`.
//!
//! This rule is an opt-in transitional bridge for projects moving runtime-global
//! access behind typed wrappers. References are flagged whenever a listed name
//! appears as a bare identifier *reference* (a use), so it catches
//! `process.env.X`, `if (localStorage) ...`, etc. Local variables that shadow a
//! listed name are still flagged in this iteration — shadowing is rare in
//! scripts that aim to forbid the global, and full scope analysis is deferred.
//!
//! ## Configuration
//!
//! ```jsonc
//! {
//!   "linter": {
//!     "rules": { "script/no-restricted-globals": "error" },
//!     "ruleOptions": {
//!       "script/no-restricted-globals": {
//!         "globals": [ { "name": "process", "message": "Use a typed config helper." } ]
//!       }
//!     }
//!   }
//! }
//! ```
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! const flag = process.env.FEATURE_FLAG
//! const token = localStorage.getItem('auth.token')
//! sessionStorage.setItem('view.scroll', String(window.scrollY))
//! ```
//!
//! ### Valid
//! ```ts
//! // Use a typed config helper that distinguishes server vs. client.
//! const flag = useFeatureFlag('FEATURE_FLAG')
//!
//! // Use a typed wrapper that scopes keys and handles SSR / disabled storage.
//! const token = authStorage.read('auth.token')
//! viewStorage.write('view.scroll', String(window.scrollY))
//! ```
//!
//! Refs: #1891 (project-local custom rules during migration).

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{IdentifierReference, Program};
use oxc_ast_visit::Visit;
use vize_carton::String;

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/no-restricted-globals",
    description: "Disallow references to runtime-environment globals that must go through a typed wrapper",
    default_severity: Severity::Error,
};

/// Default `(identifier, advisory message)` deny list.
const DEFAULT_RESTRICTED: &[(&str, &str)] = &[
    (
        "process",
        "Don't reference `process` directly. Read environment values through a typed config helper so server and client paths stay distinct.",
    ),
    (
        "localStorage",
        "Don't reference `localStorage` directly. Use a typed wrapper that scopes keys, handles SSR, and tolerates storage being unavailable.",
    ),
    (
        "sessionStorage",
        "Don't reference `sessionStorage` directly. Use a typed wrapper that scopes keys, handles SSR, and tolerates storage being unavailable.",
    ),
];

/// A configured restricted global: identifier name plus its diagnostic message.
struct RestrictedEntry {
    name: String,
    message: String,
}

/// Which deny list a check should consult.
enum Lookup<'a> {
    /// The built-in [`DEFAULT_RESTRICTED`] list.
    Default,
    /// A project-configured, non-empty list.
    Configured(&'a [RestrictedEntry]),
}

impl Lookup<'_> {
    /// The advisory message for `name`, if it is restricted.
    fn advice_for(&self, name: &str) -> Option<&str> {
        match self {
            Lookup::Default => DEFAULT_RESTRICTED
                .iter()
                .find(|(restricted, _)| *restricted == name)
                .map(|(_, advice)| *advice),
            Lookup::Configured(entries) => entries
                .iter()
                .find(|entry| entry.name == name)
                .map(|entry| entry.message.as_str()),
        }
    }
}

/// Disallow references to the built-in default set of restricted globals.
///
/// This unit rule is what the static registry references; it always uses
/// [`DEFAULT_RESTRICTED`]. A project that configures a custom list gets a
/// [`RestrictedGlobals`] instance instead (built by the linter and preferred
/// over this singleton by the dispatch).
pub struct NoRestrictedGlobals;

impl NoRestrictedGlobals {
    /// The built-in rule with the default deny list.
    pub const fn new() -> Self {
        Self
    }
}

impl Default for NoRestrictedGlobals {
    fn default() -> Self {
        Self::new()
    }
}

impl ScriptRule for NoRestrictedGlobals {
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
        check_restricted(program, offset, result, &Lookup::Default);
    }
}

/// Disallow references to a project-configured set of restricted globals.
///
/// Built by the linter from `linter.ruleOptions` and stored as a boxed override
/// (the registry keeps the unit [`NoRestrictedGlobals`] default). An empty
/// configured list falls back to the built-in defaults, so an empty config never
/// silently disables the rule.
pub(crate) struct RestrictedGlobals {
    entries: Box<[RestrictedEntry]>,
}

impl RestrictedGlobals {
    /// Build a rule from a project-configured deny list.
    ///
    /// `entries` is `(name, optional message)`; when a message is absent a
    /// generic advisory is generated.
    pub(crate) fn configured<I, M>(entries: I) -> Self
    where
        I: IntoIterator<Item = (String, Option<M>)>,
        M: Into<String>,
    {
        let entries = entries
            .into_iter()
            .map(|(name, message)| {
                let message = message
                    .map(Into::into)
                    .unwrap_or_else(|| vize_carton::cstr!("Don't reference `{name}` directly."));
                RestrictedEntry { name, message }
            })
            .collect();
        Self { entries }
    }

    fn lookup(&self) -> Lookup<'_> {
        if self.entries.is_empty() {
            Lookup::Default
        } else {
            Lookup::Configured(&self.entries)
        }
    }
}

impl ScriptRule for RestrictedGlobals {
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
        check_restricted(program, offset, result, &self.lookup());
    }
}

/// Run the restricted-identifier check over `program` using `lookup`.
fn check_restricted(
    program: &Program<'_>,
    offset: usize,
    result: &mut ScriptLintResult,
    lookup: &Lookup<'_>,
) {
    let mut visitor = RestrictedVisitor {
        lookup,
        offset,
        result,
    };
    visitor.visit_program(program);
}

struct RestrictedVisitor<'r> {
    lookup: &'r Lookup<'r>,
    offset: usize,
    result: &'r mut ScriptLintResult,
}

impl<'a> Visit<'a> for RestrictedVisitor<'_> {
    fn visit_identifier_reference(&mut self, it: &IdentifierReference<'a>) {
        if let Some(advice) = self.lookup.advice_for(it.name.as_str()) {
            self.result.add_diagnostic(LintDiagnostic::error(
                META.name,
                advice,
                self.offset as u32 + it.span.start,
                self.offset as u32 + it.span.end,
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{NoRestrictedGlobals, RestrictedGlobals};
    use crate::diagnostic::Severity;
    use crate::rules::script::ScriptLinter;
    use vize_carton::String;

    fn create_linter() -> ScriptLinter {
        let mut linter = ScriptLinter::new();
        linter.add_rule(Box::new(NoRestrictedGlobals::new()));
        linter
    }

    fn configured_linter(entries: Vec<(&str, Option<&str>)>) -> ScriptLinter {
        let mut linter = ScriptLinter::new();
        let entries = entries
            .into_iter()
            .map(|(name, message)| (String::from(name), message.map(String::from)));
        linter.add_rule(Box::new(RestrictedGlobals::configured(entries)));
        linter
    }

    #[test]
    fn flags_process_env_access() {
        let result = create_linter().lint("const flag = process.env.FOO;", 0);
        assert_eq!(result.error_count, 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.rule_name, "script/no-restricted-globals");
        assert_eq!(diag.severity, Severity::Error);
        assert_eq!(
            diag.message,
            "Don't reference `process` directly. Read environment values through a typed config helper so server and client paths stay distinct."
        );
        assert_eq!(diag.start, 13);
        assert_eq!(diag.end, 20);
    }

    #[test]
    fn flags_local_storage_call() {
        let result = create_linter().lint("localStorage.getItem('k')", 0);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn flags_session_storage_call() {
        let result = create_linter().lint("sessionStorage.setItem('k', 'v')", 0);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn allows_property_named_process() {
        // `process` here is a property name (IdentifierName), not a reference.
        let result = create_linter().lint("const settings = { process: 1 };", 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn does_not_flag_other_identifiers() {
        let result = create_linter().lint("const ok = window.location.href;", 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn flags_multiple_references() {
        let source = "const a = process.env.A; const b = localStorage.getItem('b');";
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 2);
    }

    #[test]
    fn custom_list_replaces_defaults() {
        // `alert` is restricted; the default `process` is no longer flagged.
        let linter = configured_linter(vec![("alert", Some("No alert()."))]);
        let result = linter.lint("alert('hi'); const f = process.env.X;", 0);
        assert_eq!(result.error_count, 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.rule_name, "script/no-restricted-globals");
        assert_eq!(diag.message, "No alert().");
        assert_eq!(diag.start, 0);
        assert_eq!(diag.end, 5);
    }

    #[test]
    fn custom_list_without_message_uses_generic_advice() {
        let linter = configured_linter(vec![("alert", None)]);
        let result = linter.lint("alert('hi')", 0);
        assert_eq!(result.error_count, 1);
        assert_eq!(
            result.diagnostics[0].message,
            "Don't reference `alert` directly."
        );
    }

    #[test]
    fn empty_custom_list_falls_back_to_defaults() {
        let linter = configured_linter(vec![]);
        let result = linter.lint("const f = process.env.X;", 0);
        assert_eq!(result.error_count, 1);
    }
}
