//! `vue/prop-name-casing` behaviour, pinned finding by finding.
//!
//! The rule reads the component's own `defineProps` declaration, exactly as
//! `eslint-plugin-vue@10.9.2` does, so every expectation is a complete
//! diagnostic list over a whole SFC with the reported range resolved back to the
//! declaration text it must cover.

use crate::diagnostic::Severity;
use crate::linter::{LintResult, Linter};

fn lint_sfc(sfc: &str) -> LintResult {
    Linter::new()
        .with_enabled_rules(Some(vec!["vue/prop-name-casing".into()]))
        .lint_sfc(sfc, "Probe.vue")
}

fn findings(sfc: &str) -> Vec<(&'static str, Severity, u32, u32, String)> {
    lint_sfc(sfc)
        .diagnostics
        .iter()
        .map(|diagnostic| {
            (
                diagnostic.rule_name,
                diagnostic.severity,
                diagnostic.start,
                diagnostic.end,
                diagnostic.message.to_string(),
            )
        })
        .collect()
}

/// The finding `name` must produce, anchored at its written declaration.
fn reported(
    sfc: &str,
    name: &str,
    declaration: &str,
) -> (&'static str, Severity, u32, u32, String) {
    assert_eq!(
        sfc.matches(declaration).count(),
        1,
        "declaration {declaration:?} must occur exactly once"
    );
    let start = sfc.find(declaration).expect("prop declaration") as u32;
    (
        "vue/prop-name-casing",
        Severity::Warning,
        start,
        start + declaration.len() as u32,
        format!("Prop \"{name}\" is not in camelCase"),
    )
}

fn none() -> Vec<(&'static str, Severity, u32, u32, String)> {
    Vec::new()
}

#[test]
fn a_camel_case_runtime_object_declaration_is_clean() {
    let sfc = "<script setup>\ndefineProps({ myProp: String })\n</script>\n\n<template><p /></template>\n";
    assert_eq!(findings(sfc), none());
}

#[test]
fn a_kebab_case_runtime_object_declaration_is_reported() {
    let sfc = "<script setup>\ndefineProps({ 'my-prop': String })\n</script>\n\n<template><p /></template>\n";
    assert_eq!(
        findings(sfc),
        vec![reported(sfc, "my-prop", "'my-prop': String")]
    );
}

#[test]
fn a_snake_case_declaration_is_reported_under_the_camel_case_default() {
    let sfc = "<script setup>\ndefineProps({ my_prop: String })\n</script>\n\n<template><p /></template>\n";
    assert_eq!(
        findings(sfc),
        vec![reported(sfc, "my_prop", "my_prop: String")]
    );
}

#[test]
fn a_single_lowercase_word_is_camel_case() {
    let sfc =
        "<script setup>\ndefineProps({ count: Number })\n</script>\n\n<template><p /></template>\n";
    assert_eq!(findings(sfc), none());
}

#[test]
fn a_template_attribute_written_in_camel_case_is_not_this_rule() {
    // `vue/attribute-hyphenation` owns the template side; reporting it here
    // duplicated that rule under a name upstream uses for the declaration.
    let sfc = "<script setup>\ndefineProps({ myProp: String })\n</script>\n\n<template><Child myProp=\"x\" /></template>\n";
    assert_eq!(findings(sfc), none());
}

#[test]
fn every_offending_prop_of_a_declaration_is_reported() {
    let sfc = "<script setup>\ndefineProps({ my_prop: String, ok: Number, 'other-prop': Boolean })\n</script>\n\n<template><p /></template>\n";
    assert_eq!(
        findings(sfc),
        vec![
            reported(sfc, "my_prop", "my_prop: String"),
            reported(sfc, "other-prop", "'other-prop': Boolean"),
        ]
    );
}

#[test]
fn a_component_without_props_is_clean() {
    let sfc = "<script setup>\nconst x = 1\n</script>\n\n<template><p>{{ x }}</p></template>\n";
    assert_eq!(findings(sfc), none());
}

#[test]
fn a_type_reference_in_an_intersection_is_not_a_prop_name() {
    let sfc = "<script setup lang=\"ts\">\ntype Local = { goodName?: string }\ntype ExternalProps = {}\ndefineProps<Local & ExternalProps>()\n</script>\n";
    assert_eq!(findings(sfc), none());
}

#[test]
fn a_script_only_options_component_is_checked() {
    let sfc = "<script>export default { props: { _instance: null } }</script>";
    assert_eq!(
        findings(sfc),
        vec![reported(sfc, "_instance", "_instance: null")]
    );
}

#[test]
fn a_method_signature_prop_is_checked_with_the_property_form() {
    let sfc = r#"<script setup lang="ts">
defineProps<{
  on_open?: () => void;
  on_select?(): void;
}>();
</script>
"#;
    assert_eq!(
        findings(sfc),
        vec![
            reported(sfc, "on_open", "on_open?: () => void;"),
            reported(sfc, "on_select", "on_select?(): void;"),
        ]
    );
}

#[test]
fn template_less_setup_and_computed_options_keys_are_checked() {
    let setup = "<script setup lang=\"ts\">defineProps<{ greeting_text: number }>()</script>";
    assert_eq!(
        findings(setup),
        vec![reported(setup, "greeting_text", "greeting_text: number")]
    );

    let options = "<script>export default { props: { [\"greeting-text\"]: String } }</script><template><p /></template>";
    assert_eq!(
        findings(options),
        vec![reported(
            options,
            "greeting-text",
            "[\"greeting-text\"]: String"
        )]
    );
}

#[test]
fn template_literal_prop_names_are_checked_with_and_without_a_template() {
    let cases = [
        (
            "defineProps([`$el`, \"$data\", `snake_case`, \"other_name\"]);",
            "`snake_case`",
            "\"other_name\"",
        ),
        (
            "defineProps({\n  [`$el`]: String,\n  $data: String,\n  [`snake_case`]: String,\n  other_name: String,\n});",
            "[`snake_case`]: String",
            "other_name: String",
        ),
    ];
    for (body, snake, other) in cases {
        for sfc in [
            format!("<script setup lang=\"ts\">\n{body}\n</script>\n"),
            format!(
                "<script setup lang=\"ts\">\n{body}\n</script>\n\n<template><div /></template>\n"
            ),
        ] {
            assert_eq!(
                findings(&sfc),
                vec![
                    reported(&sfc, "snake_case", snake),
                    reported(&sfc, "other_name", other),
                ],
                "{sfc}"
            );
        }
    }

    let interpolated =
        "<script setup>\ndefineProps([`ok_name_${'x'}`])\n</script>\n<template><p /></template>\n";
    assert_eq!(findings(interpolated), none());
}

#[test]
fn an_imported_props_type_is_checked_without_a_template() {
    let dir = std::env::temp_dir().join(format!(
        "vize-prop-casing-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("props.ts"),
        "export interface Props {\n  snake_case: string;\n}\n",
    )
    .unwrap();
    let source = "<script setup lang=\"ts\">\nimport type { Props } from \"./props\";\n\ndefineProps<Props>();\n</script>\n";
    let with_template = format!("{source}\n<template><div /></template>\n");
    let filename = dir.join("Imported.vue");
    let filename = filename.to_str().expect("utf-8 temp path");
    for sfc in [source, with_template.as_str()] {
        let result = Linter::new()
            .with_enabled_rules(Some(vec!["vue/prop-name-casing".into()]))
            .lint_sfc(sfc, filename);
        let marker = "defineProps<Props>()";
        let start = sfc.find(marker).expect("call") as u32 + "defineProps<".len() as u32;
        let end = start + "Props".len() as u32;
        assert_eq!(
            result
                .diagnostics
                .iter()
                .map(|diagnostic| {
                    (
                        diagnostic.rule_name,
                        diagnostic.severity,
                        diagnostic.start,
                        diagnostic.end,
                        diagnostic.message.to_string(),
                    )
                })
                .collect::<Vec<_>>(),
            vec![(
                "vue/prop-name-casing",
                Severity::Warning,
                start,
                end,
                "Prop \"snake_case\" is not in camelCase".to_string(),
            )],
            "{sfc}\n{:?}",
            result.diagnostics
        );
    }
    let _ = std::fs::remove_dir_all(&dir);
}
