//! script/no-restricted-globals
//!
//! Disallow references to a curated set of restricted globals
//! (`process`, `localStorage`, `sessionStorage`).
//!
//! This rule is an opt-in transitional bridge for projects migrating off
//! sidecar ESLint rule sets like `no-access-process` or `no-access-local-storage`
//! that enforce going through a typed wrapper instead of touching the raw
//! runtime global. Today the deny list is fixed; making it project-configurable
//! is tracked separately so this rule can ship as the smallest useful step.
//!
//! References to the listed names are flagged whenever they appear as bare
//! identifier *references* (uses), so the rule catches `process.env.X`,
//! `if (localStorage) ...`, `globalThis.sessionStorage`-free reads, etc. Local
//! variables that shadow these names will still be flagged in this iteration —
//! shadowing is rare in scripts that aim to forbid the global, and full scope
//! analysis is deferred to the configurable follow-up.
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
//!
//! Note: the deny list is intentionally byte-stable; the AST walk only
//! materializes when the source actually contains one of the names.

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{IdentifierReference, Program};
use oxc_ast_visit::Visit;

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/no-restricted-globals",
    description: "Disallow references to runtime-environment globals that must go through a typed wrapper",
    default_severity: Severity::Error,
};

/// (identifier, advisory message)
const RESTRICTED: &[(&str, &str)] = &[
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

/// Disallow references to a curated set of restricted globals.
pub struct NoRestrictedGlobals;

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
        let mut visitor = RestrictedVisitor { offset, result };
        visitor.visit_program(program);
    }
}

struct RestrictedVisitor<'r> {
    offset: usize,
    result: &'r mut ScriptLintResult,
}

impl<'a> Visit<'a> for RestrictedVisitor<'_> {
    fn visit_identifier_reference(&mut self, it: &IdentifierReference<'a>) {
        let name = it.name.as_str();
        for (restricted, advice) in RESTRICTED {
            if name == *restricted {
                self.result.add_diagnostic(LintDiagnostic::error(
                    META.name,
                    *advice,
                    self.offset as u32 + it.span.start,
                    self.offset as u32 + it.span.end,
                ));
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::diagnostic::Severity;
    use crate::rules::script::{NoRestrictedGlobals, ScriptLinter};

    fn create_linter() -> ScriptLinter {
        let mut linter = ScriptLinter::new();
        linter.add_rule(Box::new(NoRestrictedGlobals));
        linter
    }

    #[test]
    fn flags_process_env_access() {
        let result = create_linter().lint("const flag = process.env.FOO;", 0);
        assert_eq!(
            result.error_count, 1,
            "expected one error, got diagnostics: {:?}",
            result.diagnostics
        );
        let diag = &result.diagnostics[0];
        assert_eq!(diag.rule_name, "script/no-restricted-globals");
        assert_eq!(diag.severity, Severity::Error);
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
}
