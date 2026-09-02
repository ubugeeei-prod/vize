//! Linter-plan configuration regressions.

use crate::config::load_config_and_linter_plan_with_config_rule_options_and_lint_features_and_source;

use super::*;

#[test]
fn preserves_scopes_and_resolves_rules_in_declaration_order() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("vize.config.json");
    std::fs::write(
        &config_path,
        r##"{
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
}"##,
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

#[test]
fn preserves_scoped_rule_options_for_options_only_entries() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("vize.config.json");
    std::fs::write(
        &config_path,
        r##"{
  "linter": {
    "ruleOptions": {
      "musea/prefer-design-tokens": {
        "tokens": [
          { "path": "color.primary", "value": "#3b82f6" }
        ]
      },
      "script/no-restricted-members": {
        "members": [
          {
            "object": "window",
            "property": "localStorage",
            "message": "Use shared storage."
          }
        ]
      }
    }
  },
  "entries": [
    {
      "files": ["src/admin/**/*.vue"],
      "linter": {
        "ruleOptions": {
          "musea/prefer-design-tokens": {
            "tokens": [
              { "path": "color.admin", "value": "#2563eb", "tier": "semantic" }
            ]
          },
          "script/no-restricted-members": {
            "members": [
              {
                "object": "window",
                "property": "sessionStorage",
                "message": "Use admin storage."
              }
            ]
          }
        }
      }
    }
  ]
}"##,
    )
    .unwrap();

    let (_, plan, _) =
        load_config_and_linter_plan_with_config_rule_options_and_lint_features_and_source(Some(
            &config_path,
        ));
    let base = plan.resolve_matching_entries(&[]);
    let admin = plan.resolve_matching_entries(&[0]);

    assert_eq!(plan.plan.entries.len(), 1);
    assert_eq!(plan.entry_rule_options.len(), 1);
    assert_eq!(
        base.rule_options.restricted_members(),
        [(
            "window".into(),
            "localStorage".into(),
            Some("Use shared storage.".into())
        )]
    );
    assert_eq!(
        base.rule_options.musea_design_tokens(),
        [("#3b82f6".into(), "color.primary".into(), "primitive".into())]
    );
    assert_eq!(
        admin.rule_options.restricted_members(),
        [(
            "window".into(),
            "sessionStorage".into(),
            Some("Use admin storage.".into())
        )]
    );
    assert_eq!(
        admin.rule_options.musea_design_tokens(),
        [("#2563eb".into(), "color.admin".into(), "semantic".into())]
    );
}
