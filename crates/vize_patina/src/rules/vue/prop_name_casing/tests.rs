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

#[test]
fn an_aliased_macro_type_is_checked_like_a_relative_import() {
    let dir = std::env::temp_dir().join(format!(
        "vize-paths-alias-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::create_dir_all(dir.join("lib")).unwrap();
    std::fs::create_dir_all(dir.join("node_modules/some-ui")).unwrap();
    std::fs::write(
        dir.join("tsconfig.json"),
        r#"{"compilerOptions":{"moduleResolution":"bundler","paths":{"lib/*":["./lib/*"]}}}"#,
    )
    .unwrap();
    std::fs::write(
        dir.join("lib/types.ts"),
        "export interface PickEmits { pick: [id: string] }\nexport interface PickProps { row_id: string; unused?: string; $el?: string }\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("src/local.ts"),
        "import type { PickEmits as AliasedEmits, PickProps as AliasedProps } from \"lib/types\";\nexport type Emits = AliasedEmits;\nexport type Props = Omit<AliasedProps, \"x\"> & { tag: string };\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("node_modules/some-ui/package.json"),
        r#"{"name":"some-ui","types":"./index.d.ts"}"#,
    )
    .unwrap();
    std::fs::write(
        dir.join("node_modules/some-ui/index.d.ts"),
        "export interface PickProps { row_id: string }\n",
    )
    .unwrap();
    let component = "\
<script setup lang=\"ts\">
import type { PickEmits, PickProps } from \"SPECIFIER\";

const { row_id } = defineProps<PickProps>();
const emit = defineEmits<PickEmits>();
</script>

<template>
  <button type=\"button\" @click=\"emit('pik', row_id)\" />
</template>
";
    let alias = dir.join("src/Alias.vue");
    let relative = dir.join("src/Relative.vue");
    let wrapped = dir.join("src/Wrapped.vue");
    let package = dir.join("src/Package.vue");
    std::fs::write(&alias, component.replace("SPECIFIER", "lib/types")).unwrap();
    std::fs::write(&relative, component.replace("SPECIFIER", "../lib/types")).unwrap();
    std::fs::write(
        &wrapped,
        component
            .replace("PickEmits, PickProps", "Emits, Props")
            .replace("SPECIFIER", "./local")
            .replace("PickProps", "Props")
            .replace("PickEmits", "Emits"),
    )
    .unwrap();
    std::fs::write(
        &package,
        "<script setup lang=\"ts\">\nimport type { PickProps } from \"some-ui\";\ndefineProps<PickProps>();\n</script>\n",
    )
    .unwrap();

    let rules = [
        "vue/prop-name-casing",
        "vue/no-unused-properties",
        "script/require-explicit-emits",
        "script/no-reserved-keys",
        "script/require-default-prop",
    ];
    for path in [&alias, &relative, &wrapped] {
        let source = std::fs::read_to_string(path).unwrap();
        let result = Linter::new()
            .with_enabled_rules(Some(rules.iter().copied().map(Into::into).collect()))
            .lint_sfc(&source, path.to_str().expect("utf-8"));
        for (rule, message) in [
            ("vue/prop-name-casing", "row_id"),
            ("vue/no-unused-properties", "unused"),
            ("script/require-explicit-emits", "pik"),
            ("script/no-reserved-keys", "$el"),
        ] {
            assert!(
                result.diagnostics.iter().any(|diagnostic| {
                    diagnostic.rule_name == rule && diagnostic.message.contains(message)
                }),
                "{path:?} missing {rule} {message}: {:?}",
                result
                    .diagnostics
                    .iter()
                    .map(|diagnostic| format!("{}: {}", diagnostic.rule_name, diagnostic.message))
                    .collect::<Vec<_>>()
            );
        }
    }
    let defaults = dir.join("src/Defaults.vue");
    std::fs::write(
        &defaults,
        "<script setup lang=\"ts\">\nimport type { PickProps } from \"lib/types\";\nwithDefaults(defineProps<PickProps>(), {});\n</script>\n",
    )
    .unwrap();
    let defaults_source = std::fs::read_to_string(&defaults).unwrap();
    let defaults_result = Linter::new()
        .with_enabled_rules(Some(vec!["script/require-default-prop".into()]))
        .lint_sfc(&defaults_source, defaults.to_str().expect("utf-8"));
    assert!(
        defaults_result.diagnostics.iter().any(|diagnostic| {
            diagnostic.rule_name == "script/require-default-prop"
                && diagnostic.message.contains("unused")
        }),
        "{:?}",
        defaults_result.diagnostics
    );
    let package_source = std::fs::read_to_string(&package).unwrap();
    let package_result = Linter::new()
        .with_enabled_rules(Some(vec!["vue/prop-name-casing".into()]))
        .lint_sfc(&package_source, package.to_str().expect("utf-8"));
    assert!(
        package_result.diagnostics.iter().any(|diagnostic| {
            diagnostic.rule_name == "vue/prop-name-casing" && diagnostic.message.contains("row_id")
        }),
        "{:?}",
        package_result.diagnostics
    );
    let _ = std::fs::remove_dir_all(&dir);
}
