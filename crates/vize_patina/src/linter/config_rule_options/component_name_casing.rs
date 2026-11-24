use crate::{LintPreset, Linter};
use vize_l0::config::ConfigLintRuleOptions;

const RULE: &str = "vue/component-name-in-template-casing";
const PASCAL: &str = "Component should use PascalCase";
const KEBAB: &str = "Component should use kebab-case";

/// The reproduction of #13537.
const REPORTED: &str = "<template>\n  <primitive />\n</template>\n";
/// Its isolation row with a second, unlisted tag on line 3.
const WITH_FOO_BAR: &str = "<template>\n  <primitive />\n  <foo-bar />\n</template>\n";

fn linter(options: serde_json::Value) -> Linter {
    let options: ConfigLintRuleOptions =
        serde_json::from_value(serde_json::json!({ RULE: options })).unwrap();
    Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(vec![RULE.into()]))
        .with_config_rule_options(&options)
}

/// `(tag, message, fixed source)` for each report.
type Reports = Vec<(
    std::string::String,
    &'static str,
    Option<std::string::String>,
)>;

fn reports(options: serde_json::Value, source: &str) -> Reports {
    linter(options)
        .lint_sfc(source, "Foo.vue")
        .diagnostics
        .iter()
        .map(|diagnostic| {
            assert_eq!(diagnostic.rule_name, RULE);
            let message = if diagnostic.message == PASCAL {
                PASCAL
            } else {
                assert_eq!(diagnostic.message, KEBAB);
                KEBAB
            };
            (
                source[diagnostic.start as usize + 1..diagnostic.end as usize].to_owned(),
                message,
                diagnostic
                    .fix
                    .as_ref()
                    .map(|fix| fix.apply(source).to_string()),
            )
        })
        .collect()
}

fn reported_tags(options: serde_json::Value, source: &str) -> Vec<std::string::String> {
    reports(options, source)
        .into_iter()
        .map(|(tag, ..)| tag)
        .collect()
}

#[test]
fn the_reported_tag_is_exempt_once_it_is_listed() {
    let unlisted = serde_json::json!({ "registeredComponentsOnly": false });
    assert_eq!(
        reports(unlisted.clone(), REPORTED),
        [(
            "primitive".to_owned(),
            PASCAL,
            Some("<template>\n  <Primitive />\n</template>\n".to_owned()),
        )]
    );
    assert_eq!(
        reported_tags(unlisted, WITH_FOO_BAR),
        ["primitive", "foo-bar"]
    );

    let listed = serde_json::json!({ "ignores": ["primitive"], "registeredComponentsOnly": false });
    assert_eq!(reports(listed.clone(), REPORTED), Reports::new());
    assert_eq!(
        reports(listed, WITH_FOO_BAR),
        [(
            "foo-bar".to_owned(),
            PASCAL,
            Some("<template>\n  <primitive />\n  <FooBar />\n</template>\n".to_owned()),
        )]
    );
}

#[test]
fn globals_register_a_name_and_ignores_exempt_it_in_either_scope() {
    // `globals` means "also registered", so the tag is checked.
    for registered_components_only in [false, true] {
        assert_eq!(
            reported_tags(
                serde_json::json!({
                    "globals": ["primitive"],
                    "registeredComponentsOnly": registered_components_only,
                }),
                REPORTED,
            ),
            ["primitive"],
            "{registered_components_only}"
        );
        assert_eq!(
            reported_tags(
                serde_json::json!({
                    "globals": ["primitive"],
                    "ignores": ["primitive"],
                    "registeredComponentsOnly": registered_components_only,
                }),
                REPORTED,
            ),
            Vec::<std::string::String>::new(),
            "{registered_components_only}"
        );
    }
    // An unregistered tag was never checked in the registered-only scope.
    assert_eq!(
        reported_tags(
            serde_json::json!({ "registeredComponentsOnly": true }),
            REPORTED
        ),
        Vec::<std::string::String>::new()
    );
    assert_eq!(
        reported_tags(serde_json::json!({ "ignores": ["primitive"] }), REPORTED),
        Vec::<std::string::String>::new()
    );
}

#[test]
fn an_imported_component_is_exempt_by_its_authored_name_only() {
    let source = "<script setup lang=\"ts\">\nimport TresMesh from \"./TresMesh.vue\";\nimport FooBar from \"./FooBar.vue\";\n</script>\n\n<template>\n  <tres-mesh />\n  <foo-bar />\n</template>\n";
    assert_eq!(
        reported_tags(serde_json::json!({}), source),
        ["tres-mesh", "foo-bar"]
    );
    for (ignores, expected) in [
        (serde_json::json!(["tres-mesh"]), vec!["foo-bar"]),
        // A literal is the whole authored name, in the authored casing.
        (
            serde_json::json!(["TresMesh", "tres", "foo"]),
            vec!["tres-mesh", "foo-bar"],
        ),
        (serde_json::json!(["/^tres-/"]), vec!["foo-bar"]),
        (serde_json::json!(["/^TRES-/i", "/bar$/"]), vec![]),
        (serde_json::json!(["/-/"]), vec![]),
    ] {
        assert_eq!(
            reported_tags(serde_json::json!({ "ignores": ignores }), source),
            expected,
            "{ignores}"
        );
    }
}

#[test]
fn ignores_exempt_a_tag_from_the_kebab_case_policy_too() {
    let source = "<template>\n  <MyWidget />\n  <OtherWidget />\n</template>\n";
    let policy = |ignores: serde_json::Value| {
        serde_json::json!({
            "casing": "kebab-case",
            "ignores": ignores,
            "registeredComponentsOnly": false,
        })
    };
    assert_eq!(
        reports(policy(serde_json::json!([])), source),
        [
            (
                "MyWidget".to_owned(),
                KEBAB,
                Some("<template>\n  <my-widget />\n  <OtherWidget />\n</template>\n".to_owned()),
            ),
            (
                "OtherWidget".to_owned(),
                KEBAB,
                Some("<template>\n  <MyWidget />\n  <other-widget />\n</template>\n".to_owned()),
            ),
        ]
    );
    assert_eq!(
        reported_tags(policy(serde_json::json!(["MyWidget"])), source),
        ["OtherWidget"]
    );
    assert_eq!(
        reported_tags(policy(serde_json::json!(["/Widget$/"])), source),
        Vec::<std::string::String>::new()
    );
}
