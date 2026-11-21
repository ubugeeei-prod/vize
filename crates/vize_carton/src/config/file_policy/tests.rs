use super::*;
use crate::config::{
    ConfigDocument, formatter_lint_plan::FormatterLinterPlan, matcher::CompilerWhitespacePlan,
};

fn compare(document: &ConfigDocument, dir: &Path, paths: &[PathBuf]) -> ProjectFilePlan {
    let lint = document.linter_plan_with_config_rule_options();
    let entries = document.compiler_entries();
    let mode = document.compiler_whitespace();
    let old_lint = FormatterLinterPlan::new(lint.clone(), dir);
    let old_whitespace = CompilerWhitespacePlan::new(mode, &entries, dir);
    let shared = ProjectFilePlan::new(lint, mode, &entries, dir);
    for path in paths {
        let policy = shared.resolve(path);
        assert_eq!(policy.lint, old_lint.resolve(path), "{path:?}");
        assert_eq!(
            policy.preserves_text,
            old_whitespace.preserves_text_for(path),
            "{path:?}"
        );
    }
    shared
}

#[test]
fn duplicate_nonadjacent_matches_preserve_independent_declaration_order() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let document = serde_json::from_str(r#"{
        "ignores":["src/private/**","!src/private/Keep.vue"],
        "compiler":{"whitespace":"condense"},
        "entries":[
          {"files":["src/**"],"compiler":{"whitespace":"preserve"},"linter":{"rules":{"vue/v-bind-style":"off"}}},
          {"basePath":"src","files":["**/*.vue"],"compiler":{"whitespace":"condense"},"linter":{"ruleOptions":{"vue/v-bind-style":"longform"}}},
          {"files":["src/**"],"compiler":{"whitespace":"preserve"},"linter":{"rules":{"vue/v-bind-style":"warn"}}},
          {"files":["src/**","!src/excluded/**"],"ignores":["**/No.vue"],"linter":{"ruleOptions":{"vue/v-bind-style":"shorthand"}}},
          {"files":["src/excluded/**"],"compiler":{"whitespace":"condense"}},
          {"files":["src/private/**"],"compiler":{"whitespace":"preserve"}}
        ]
    }"#).unwrap();
    let paths: Vec<_> = [
        "src/A.vue",
        "src/No.vue",
        "src/excluded/A.vue",
        "src/private/A.vue",
        "src/private/Keep.vue",
        "src/./dir/../A.vue",
        "src\\private\\Keep.vue",
        "other/A.vue",
    ]
    .map(|p| root.join(p))
    .into();
    let shared = compare(&document, root, &paths);
    assert_eq!(shared.scopes.len(), 5);
    assert!(shared.resolve(&paths[0]).preserves_text);
    let excluded = shared.resolve(&paths[2]);
    assert!(!excluded.preserves_text);
    let ignored = shared.resolve(&paths[3]);
    assert_eq!(ignored.lint, None);
    assert!(ignored.preserves_text);
    assert!(shared.resolve(&paths[4]).lint.is_some());
}

#[test]
fn no_entries_and_the_original_project_fixtures_match() {
    let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for (file, text) in [
        ("default", "{}"),
        (
            "crates/vize/tests/fixtures/lint-config-paths/vize.config.json",
            include_str!(
                "../../../../../crates/vize/tests/fixtures/lint-config-paths/vize.config.json"
            ),
        ),
        (
            "crates/vize/tests/fixtures/nuxt2_false_positive/vize.config.json",
            include_str!(
                "../../../../../crates/vize/tests/fixtures/nuxt2_false_positive/vize.config.json"
            ),
        ),
    ] {
        let config: ConfigDocument = serde_json::from_str(text).unwrap();
        let config_file = base.join(file);
        let dir = config_file.parent().unwrap();
        let mut paths = Vec::new();
        davinci_test_support::corpus::collect_vue_files(
            &base.join("tests/_fixtures/differential"),
            &mut paths,
        );
        paths.push(dir.join("src/Panel.vue"));
        paths.push(PathBuf::from("../app/Relative.vue"));
        compare(&config, dir, &paths);
    }
}

#[test]
fn original_scoped_ignore_and_lint_config_paths_keep_their_tool_specific_meaning() {
    use crate::config::LintRuleSeverity;
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let entry: ConfigDocument = serde_json::from_str(include_str!(
        "../../../../../tests/_fixtures/config-scoped-entry-ignores-11146/vue-options.json"
    ))
    .unwrap();
    let component = root.join("src/components/button.vue");
    let story = root.join("src/components/button.story.vue");
    let plan = compare(&entry, root, &[component.clone(), story.clone()]);
    let rule = "vue/require-scoped-style";
    assert_eq!(
        plan.resolve(&component)
            .lint
            .unwrap()
            .config
            .rules
            .get(rule),
        Some(&LintRuleSeverity::Off)
    );
    let on_story = plan.resolve(&story).lint.unwrap();
    assert_ne!(
        on_story.config.rules.get(rule),
        Some(&LintRuleSeverity::Off)
    );

    let paths: ConfigDocument = serde_json::from_str(include_str!(
        "../../../../../crates/vize/tests/fixtures/lint-config-paths/vize.config.json"
    ))
    .unwrap();
    let generated = root.join("gen/Image.vue");
    let legacy = root.join("src/legacy/Image.vue");
    let plan = compare(&paths, root, &[generated.clone(), legacy.clone()]);
    assert!(plan.resolve(&generated).lint.is_none());
    assert_eq!(
        plan.resolve(&legacy)
            .lint
            .unwrap()
            .config
            .rules
            .get("a11y/alt-text"),
        Some(&LintRuleSeverity::Off)
    );
}

mod corpus;
mod profile;
