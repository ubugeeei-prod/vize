use super::Linter;

#[test]
fn test_lint_sfc_css_logical_properties_after_import_reports_once() {
    let linter =
        Linter::new().with_enabled_rules(Some(vec!["css/prefer-logical-properties".into()]));
    let sfc = r#"<template><div/></template>
<style scoped>
@import "~/design/styles/breakpoint.css";

.mp-snackbar {
  position: fixed;
  top: var(--space-4);
  left: var(--space-4);
  right: var(--space-4);
}
</style>
"#;
    let result = linter.lint_sfc(sfc, "Snackbar.vue");
    let logical_diags: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == "css/prefer-logical-properties")
        .collect();

    assert_eq!(
        logical_diags.len(),
        1,
        "logical property diagnostics should be deduplicated: {:?}",
        result.diagnostics
    );
}
