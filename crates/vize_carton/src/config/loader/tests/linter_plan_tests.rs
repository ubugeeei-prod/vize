//! Linter-plan configuration regressions.

use super::*;

#[test]
fn preserves_scopes_and_resolves_rules_in_declaration_order() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("vize.config.json");
    std::fs::write(
        &config_path,
        r#"{
  "basePath": "workspace",
  "ignores": ["generated/**"],
  "linter": { "rules": { "base": "error", "shared": "warn" } },
  "entries": [
    {
      "files": ["src/**/*.vue"],
      "linter": { "rules": { "first": "warn", "shared": "error" } }
    },
    {
      "basePath": "packages/admin",
      "files": ["src/**/*.vue"],
      "ignores": ["src/generated/**"],
      "linter": { "rules": { "last": "error", "shared": "off" } }
    }
  ]
}"#,
    )
    .unwrap();

    let (_, plan, _) =
        load_config_and_linter_plan_with_lint_features_and_source(Some(&config_path));
    let resolved = plan.resolve_matching_entries(&[1, 0, 1]);

    assert_eq!(
        resolved.rules,
        crate::FxHashMap::from_iter([
            ("base".into(), LintRuleSeverity::Error),
            ("first".into(), LintRuleSeverity::Warn),
            ("last".into(), LintRuleSeverity::Error),
            ("shared".into(), LintRuleSeverity::Off),
        ])
    );
    assert_eq!(plan.entries[0].files.as_deref().unwrap().len(), 1);
    assert_eq!(
        plan.entries[0].files.as_deref().unwrap()[0].as_str(),
        "src/**/*.vue"
    );
    assert_eq!(plan.entries[1].base_path.as_deref(), Some("packages/admin"));
    assert_eq!(plan.entries[1].ignores, ["src/generated/**"]);
    assert_eq!(plan.global_ignores.len(), 1);
    assert_eq!(plan.global_ignores[0].base_path, None);
    assert_eq!(plan.global_ignores[0].pattern.as_str(), "generated/**");
}
