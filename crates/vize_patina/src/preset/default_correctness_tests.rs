//! Authored complete default-preset results; no runtime snapshot recapture.

use crate::{HelpLevel, LintPreset, LintResult, Linter};
use serde_json::{Value, json};

fn corpus() -> Value {
    serde_json::from_str(include_str!(
        "../../../../tests/_fixtures/lint-default-correctness/current-range-cases.json"
    ))
    .unwrap()
}

fn whole(result: &LintResult) -> Value {
    json!({
        "filename": result.filename,
        "error_count": result.error_count,
        "warning_count": result.warning_count,
        "diagnostics": result.diagnostics.iter().map(|d| json!({
            "rule_name": d.rule_name, "severity": d.severity, "message": d.message,
            "start": d.start, "end": d.end, "help": d.help, "fix": d.fix,
            "labels": d.labels.iter().map(|l| json!({
                "message": l.message, "start": l.start, "end": l.end
            })).collect::<Vec<_>>()
        })).collect::<Vec<_>>()
    })
}

#[test]
fn default_and_ecosystem_preserve_complete_authored_correctness_results() {
    let data = corpus();
    for case in data["cases"].as_array().unwrap() {
        if case["kind"] == "html" {
            continue;
        }
        let source = case["source"].as_str().unwrap();
        let filename = case["filename"].as_str().unwrap();
        let expected = json!({
            "filename": filename,
            "error_count": case["diagnostics"].as_array().unwrap().iter().filter(|d| d["severity"] == "error").count(),
            "warning_count": case["diagnostics"].as_array().unwrap().iter().filter(|d| d["severity"] == "warning").count(),
            "diagnostics": case["diagnostics"]
        });
        for preset in [LintPreset::HappyPath, LintPreset::Ecosystem] {
            let result = Linter::with_preset(preset)
                .with_help_level(HelpLevel::None)
                .lint_sfc(source, filename);
            assert_eq!(
                whole(&result),
                expected,
                "{} / {}",
                case["id"],
                preset.as_str()
            );
        }
    }
}

#[test]
fn promoted_rules_can_be_disabled_while_retaining_baseline_and_empty_incremental() {
    let data = corpus();
    let promoted: Vec<vize_l0::String> = data["promotedRules"]
        .as_array()
        .unwrap()
        .iter()
        .map(|name| name.as_str().unwrap().into())
        .collect();
    for case in &data["cases"].as_array().unwrap()[..6] {
        let source = case["source"].as_str().unwrap();
        let filename = case["filename"].as_str().unwrap();
        for (linter, expected_diagnostics) in [
            (
                Linter::new().with_disabled_rules(promoted.clone()),
                case["baselineDiagnostics"].clone(),
            ),
            (Linter::with_preset(LintPreset::Incremental), json!([])),
        ] {
            assert_eq!(
                whole(&linter.lint_sfc(source, filename)),
                json!({
                    "filename": filename, "error_count": 0,
                    "warning_count": expected_diagnostics.as_array().unwrap().len(),
                    "diagnostics": expected_diagnostics
                }),
                "{}",
                case["id"]
            );
        }
    }
    let petite = data["cases"].as_array().unwrap().last().unwrap();
    let result = Linter::new()
        .with_enabled_rules(Some(promoted))
        .lint_standalone_html(
            petite["source"].as_str().unwrap(),
            petite["filename"].as_str().unwrap(),
        );
    assert_eq!(
        whole(&result),
        json!({
            "filename": petite["filename"], "error_count": 0, "warning_count": 0, "diagnostics": []
        })
    );
}

#[test]
fn ecosystem_inherits_every_default_script_rule_and_presets_register_once() {
    let happy = super::builtin_script_rule_names(LintPreset::HappyPath);
    let eco = super::builtin_script_rule_names(LintPreset::Ecosystem);
    for name in happy {
        assert!(eco.contains(name), "ecosystem lost {name}");
    }
    let data = corpus();
    for preset in LintPreset::ALL {
        let mut registry = crate::rule::RuleRegistry::with_preset(preset);
        for name in data["promotedRules"].as_array().unwrap() {
            assert_eq!(
                registry.has_rule(name.as_str().unwrap()),
                preset != LintPreset::Incremental
            );
        }
        registry.register_opt_in_rules();
        for name in data["promotedRules"].as_array().unwrap() {
            assert_eq!(
                registry
                    .rule_names()
                    .iter()
                    .filter(|n| **n == name.as_str().unwrap())
                    .count(),
                1
            );
        }
    }
}

#[test]
fn configured_vue_versions_preserve_complete_supported_syntax_baselines() {
    use vize_l0::config::VueVersion;
    let data = corpus();
    for version in [VueVersion::V2, VueVersion::V2_7, VueVersion::V3] {
        for case in data["cases"].as_array().unwrap() {
            if case["kind"] == "html" {
                continue;
            }
            let findings = if version.is_legacy() {
                &case["baselineDiagnostics"]
            } else {
                &case["diagnostics"]
            };
            let expected = json!({
                "filename": case["filename"],
                "error_count": findings.as_array().unwrap().iter().filter(|d| d["severity"] == "error").count(),
                "warning_count": findings.as_array().unwrap().iter().filter(|d| d["severity"] == "warning").count(),
                "diagnostics": findings
            });
            for preset in [LintPreset::HappyPath, LintPreset::Ecosystem] {
                let result = Linter::with_preset(preset)
                    .with_vue_version(Some(version))
                    .with_help_level(HelpLevel::None)
                    .lint_sfc(
                        case["source"].as_str().unwrap(),
                        case["filename"].as_str().unwrap(),
                    );
                assert_eq!(
                    whole(&result),
                    expected,
                    "{} / {} / {}",
                    version.as_str(),
                    case["id"],
                    preset.as_str()
                );
            }
        }
    }
}

const DUPLICATE_CLASS_RULE: &str = "html/no-duplicate-class";

fn duplicate_class_findings(
    source: &str,
    filename: &str,
    attribute: &str,
    names: &[&str],
) -> Value {
    let start = source.find(attribute).unwrap() as u32;
    let end = start + attribute.len() as u32;
    json!({
        "filename": filename, "error_count": 0, "warning_count": names.len(),
        "diagnostics": names.iter().map(|name| json!({
            "rule_name": DUPLICATE_CLASS_RULE, "severity": "warning",
            "message": format!("Duplicate class name '{name}'"),
            "start": start, "end": end, "help": null, "fix": null, "labels": []
        })).collect::<Vec<_>>()
    })
}

fn duplicate_class_default(preset: LintPreset) -> bool {
    matches!(
        preset,
        LintPreset::HappyPath | LintPreset::Ecosystem | LintPreset::Opinionated | LintPreset::Nuxt
    )
}

#[test]
fn duplicate_class_defaults_preserve_whole_findings_in_every_markup_entrypoint() {
    use vize_atelier_jsx::JsxLang;
    let attribute = r#"class="btn btn""#;
    let default_source = r#"<template><div class="btn btn">Text</div></template>"#;
    for linter in [
        Linter::new(),
        Linter::with_registry(crate::RuleRegistry::with_happy_path()),
        Linter::with_registry(crate::RuleRegistry::with_recommended()),
    ] {
        let result = linter
            .with_help_level(HelpLevel::None)
            .lint_sfc(default_source, "PresetClasses.vue");
        assert_eq!(
            whole(&result),
            duplicate_class_findings(default_source, "PresetClasses.vue", attribute, &["btn"])
        );
    }
    for preset in LintPreset::ALL {
        let linter = Linter::with_preset(preset).with_help_level(HelpLevel::None);
        let names: &[&str] = if duplicate_class_default(preset) {
            &["btn"]
        } else {
            &[]
        };
        for (source, filename, mode) in [
            (
                r#"<div class="btn btn">Text</div>"#,
                "PresetClasses.vue",
                "template",
            ),
            (
                r#"<template><div class="btn btn">Text</div></template>"#,
                "PresetClasses.vue",
                "sfc",
            ),
            (
                r#"const PresetClasses = () => <div class="btn btn">Text</div>;"#,
                "PresetClasses.jsx",
                "jsx",
            ),
            (
                r#"const PresetClasses = (): JSX.Element => <div class="btn btn">Text</div>;"#,
                "PresetClasses.tsx",
                "tsx",
            ),
            (r#"<div class="btn btn">Text</div>"#, "index.html", "html"),
        ] {
            let result = match mode {
                "template" => linter.lint_template(source, filename),
                "sfc" => linter.lint_sfc(source, filename),
                "jsx" => linter.lint_jsx(source, filename, JsxLang::Jsx),
                "tsx" => linter.lint_jsx(source, filename, JsxLang::Tsx),
                "html" => linter.lint_standalone_html(source, filename),
                _ => unreachable!(),
            };
            assert_eq!(
                whole(&result),
                duplicate_class_findings(source, filename, attribute, names),
                "{} / {mode}",
                preset.as_str()
            );
        }
    }
}

#[test]
fn duplicate_class_defaults_report_exact_static_tokens_and_skip_dynamic_values() {
    let cases: &[(&str, &str, &[&str])] = &[
        (
            r#"<div class="x x x">Text</div>"#,
            r#"class="x x x""#,
            &["x"],
        ),
        (
            r#"<div class="a a b b">Text</div>"#,
            r#"class="a a b b""#,
            &["a", "b"],
        ),
        (
            r#"<div class="foo foo Foo Foo">Text</div>"#,
            r#"class="foo foo Foo Foo""#,
            &["foo", "Foo"],
        ),
        (
            "<!-- 前置 -->\n<div class=\"東京 東京\">Text</div>",
            "class=\"東京 東京\"",
            &["東京"],
        ),
        (
            "<div class=\"a\ta\r\na\">Text</div>",
            "class=\"a\ta\r\na\"",
            &["a"],
        ),
        (
            "<div class=\"a\u{c}a\">Text</div>",
            "class=\"a\u{c}a\"",
            &["a"],
        ),
        (
            "<div class=\"a\u{b}a\">Text</div>",
            "class=\"a\u{b}a\"",
            &[],
        ),
        (
            r#"<div class="foo Foo">Text</div>"#,
            r#"class="foo Foo""#,
            &[],
        ),
        (
            "<div class=\"a\u{a0}a\">Text</div>",
            "class=\"a\u{a0}a\"",
            &[],
        ),
        (
            "<div class=\"é e\u{301}\">Text</div>",
            "class=\"é e\u{301}\"",
            &[],
        ),
        (
            r#"<div :class="classes">Text</div>"#,
            r#":class="classes""#,
            &[],
        ),
        (
            r#"<div v-bind:class="classes">Text</div>"#,
            r#"v-bind:class="classes""#,
            &[],
        ),
        (
            r#"<div v-bind="attrs">Text</div>"#,
            r#"v-bind="attrs""#,
            &[],
        ),
        (r#"<div CLASS="a a">Text</div>"#, r#"CLASS="a a""#, &[]),
    ];
    for preset in LintPreset::ALL {
        // Every preset genuinely enables this selected rule here, including the
        // empty incremental preset. Negative cases cannot pass by omission.
        let linter = Linter::with_preset(preset)
            .with_additional_rules(vec![DUPLICATE_CLASS_RULE.into()])
            .with_enabled_rules(Some(vec![DUPLICATE_CLASS_RULE.into()]))
            .with_help_level(HelpLevel::None);
        for (source, attribute, names) in cases {
            assert_eq!(
                whole(&linter.lint_template(source, "PresetClasses.vue")),
                duplicate_class_findings(source, "PresetClasses.vue", attribute, names),
                "{} / {source}",
                preset.as_str()
            );
        }
    }
}

#[test]
fn duplicate_class_preset_and_explicit_overrides_keep_one_rule_and_exact_severity() {
    let source = r#"<div class="btn btn">Text</div>"#;
    let attribute = r#"class="btn btn""#;
    for preset in LintPreset::ALL {
        let mut registry = crate::RuleRegistry::with_preset(preset);
        assert_eq!(
            registry
                .rule_names()
                .iter()
                .filter(|name| **name == DUPLICATE_CLASS_RULE)
                .count(),
            usize::from(duplicate_class_default(preset))
        );
        registry.register_opt_in_rules();
        assert_eq!(
            registry
                .rule_names()
                .iter()
                .filter(|name| **name == DUPLICATE_CLASS_RULE)
                .count(),
            usize::from(duplicate_class_default(preset))
        );
        let linter = Linter::with_preset(preset)
            .with_additional_rules(vec![
                DUPLICATE_CLASS_RULE.into(),
                DUPLICATE_CLASS_RULE.into(),
            ])
            .with_help_level(HelpLevel::None);
        assert_eq!(
            whole(&linter.lint_template(source, "PresetClasses.vue")),
            duplicate_class_findings(source, "PresetClasses.vue", attribute, &["btn"]),
            "explicit / {}",
            preset.as_str()
        );
        let disabled = linter.with_disabled_rules(vec![DUPLICATE_CLASS_RULE.into()]);
        assert_eq!(
            whole(&disabled.lint_template(source, "PresetClasses.vue")),
            duplicate_class_findings(source, "PresetClasses.vue", attribute, &[]),
            "off / {}",
            preset.as_str()
        );
        let mut expected =
            duplicate_class_findings(source, "PresetClasses.vue", attribute, &["btn"]);
        expected["error_count"] = json!(1);
        expected["warning_count"] = json!(0);
        expected["diagnostics"][0]["severity"] = json!("error");
        let strict = Linter::with_preset(preset)
            .with_additional_rules(vec![DUPLICATE_CLASS_RULE.into()])
            .with_rule_severity_overrides(vec![(
                DUPLICATE_CLASS_RULE.into(),
                crate::Severity::Error,
            )])
            .with_help_level(HelpLevel::None);
        assert_eq!(
            whole(&strict.lint_template(source, "PresetClasses.vue")),
            expected,
            "error / {}",
            preset.as_str()
        );
    }
}

#[test]
fn duplicate_class_default_membership_is_independent_of_supported_vue_version() {
    use vize_l0::config::VueVersion;
    let source = r#"<template><div class="btn btn">Text</div></template>"#;
    for version in [VueVersion::V2, VueVersion::V2_7, VueVersion::V3] {
        for preset in LintPreset::ALL {
            let names: &[&str] = if duplicate_class_default(preset) {
                &["btn"]
            } else {
                &[]
            };
            let result = Linter::with_preset(preset)
                .with_vue_version(Some(version))
                .with_help_level(HelpLevel::None)
                .lint_sfc(source, "PresetClasses.vue");
            assert_eq!(
                whole(&result),
                duplicate_class_findings(source, "PresetClasses.vue", r#"class="btn btn""#, names),
                "{version:?} / {}",
                preset.as_str()
            );
        }
    }
}
