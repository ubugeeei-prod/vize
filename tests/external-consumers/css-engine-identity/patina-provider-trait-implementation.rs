use vize_patina::css_ast::stylesheet::StyleSheet;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use vize_patina::rules::css::{CssLintResult, CssLinter, CssRule, CssRuleMeta};
use vize_patina::{LintDiagnostic, Severity};

static META: CssRuleMeta = CssRuleMeta {
    name: "css/external-identity",
    description: "External typed rule contract",
    default_severity: Severity::Warning,
};

struct ExternalRule(Arc<AtomicUsize>);

impl CssRule for ExternalRule {
    fn meta(&self) -> &'static CssRuleMeta {
        &META
    }

    fn check<'i>(
        &self,
        source: &'i str,
        stylesheet: &StyleSheet<'i>,
        offset: usize,
        result: &mut CssLintResult,
    ) {
        assert_eq!(source, "a{color:red}");
        assert_eq!(stylesheet.rules.0.len(), 1);
        assert_eq!(offset, 7);
        self.0.fetch_add(1, Ordering::SeqCst);
        result.add_diagnostic(LintDiagnostic::warn(META.name, "external called", 7, 8));
    }
}

fn main() {
    let calls = Arc::new(AtomicUsize::new(0));
    let mut linter = CssLinter::new();
    linter.add_rule(Box::new(ExternalRule(calls.clone())));
    let result = linter.lint("a{color:red}", 7);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(
        [
            result.error_count,
            result.warning_count,
            result.diagnostics.len()
        ],
        [0, 1, 1]
    );
    assert_eq!(result.diagnostics[0].rule_name, META.name);
    assert_eq!(
        [result.diagnostics[0].start, result.diagnostics[0].end],
        [7, 8]
    );
}
