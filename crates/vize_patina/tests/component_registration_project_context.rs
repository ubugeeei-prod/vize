//! Complete authored registration controls with genuine explicit project policy.
use serde_json::{Value, json};
use vize_patina::{HelpLevel, LintPreset, LintResult, Linter};
use vize_relief::options::CustomElementMatcher;

const RULE: &str = "vue/require-component-registration";
const CASES: &str = include_str!(
    "../../../tests/_fixtures/differential/lint/component-registration-context-8142/cases.json"
);

fn linter() -> Linter {
    Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(vec![RULE.into()]))
        .with_help_level(HelpLevel::Full)
}

fn complete(result: &LintResult) -> Value {
    json!({
        "filename": result.filename.as_str(),
        "diagnostics": result.diagnostics.iter().map(|d| json!({
            "rule_name": d.rule_name, "severity": d.severity,
            "message": d.message.as_str(), "start": d.start, "end": d.end,
            "help": d.help.as_ref().map(|help| help.as_str()),
            "labels": d.labels.iter().map(|l| json!({"message": l.message.as_str(), "start": l.start, "end": l.end})).collect::<Vec<_>>(),
            "fix": d.fix,
        })).collect::<Vec<_>>(),
        "error_count": result.error_count, "warning_count": result.warning_count,
    })
}

fn names(value: &Value) -> Vec<vize_l0::String> {
    value
        .as_array()
        .map(|names| {
            names
                .iter()
                .map(|name| name.as_str().unwrap().into())
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn whole_project_context_packets_repeat_in_both_configuration_orders() {
    let fixture: Value = serde_json::from_str(CASES).unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 30);
    let mut observations = Vec::new();
    for order in [false, true] {
        for repeat in [1, 2] {
            for case in cases {
                let source = case["source"].as_str().unwrap();
                let filename = case["filename"].as_str().unwrap();
                let globals = names(&case["nativeOptions"]["globals"]);
                let matcher =
                    CustomElementMatcher::from_patterns(names(&case["compiler"]["customElements"]));
                let configured = if order {
                    linter()
                        .with_custom_elements(matcher)
                        .with_component_registration_globals(globals)
                } else {
                    linter()
                        .with_component_registration_globals(globals)
                        .with_custom_elements(matcher)
                };
                observations.push(json!({
                    "caseId": case["id"], "source": source, "filename": filename,
                    "globalsFirst": !order, "repeat": repeat,
                    "compiler": case["compiler"], "nativeOptions": case["nativeOptions"],
                    "completePacket": complete(&configured.lint_sfc(source, filename)),
                }));
            }
        }
    }
    let target = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/differential/component-registration-context-8142");
    std::fs::create_dir_all(&target).unwrap();
    std::fs::write(
        target.join("whole-packets.json"),
        serde_json::to_vec_pretty(&json!({
            "schema": "vize.component-registration.actual-whole-source-context-packets",
            "observations": observations,
        }))
        .unwrap(),
    )
    .unwrap();
    // Retain every complete packet before judging any authored law.
    for observation in &observations {
        let case = cases
            .iter()
            .find(|case| case["id"] == observation["caseId"])
            .unwrap();
        assert_eq!(
            observation["completePacket"], case["authoredExpectedPacket"],
            "{}",
            case["id"]
        );
        let source = case["source"].as_str().unwrap();
        let start = case["tagSpan"]["start"].as_u64().unwrap() as usize;
        let end = case["tagSpan"]["end"].as_u64().unwrap() as usize;
        assert_eq!(source.get(start..end), case["authoredTag"].as_str());
    }
}

#[test]
fn native_builtin_and_explicit_project_names_keep_standalone_template_spans() {
    let source = "<div>界😀<x-widget/><x-neighbor/><RouterLink/><NuxtPage/><Transition/><svg><path/></svg><math><mi/></math></div>";
    let configured = linter()
        .with_custom_elements(CustomElementMatcher::from_patterns(vec!["x-widget".into()]))
        .with_component_registration_globals(vec!["RouterLink".into(), "NuxtPage".into()]);
    let result = configured.lint_template(source, "Standalone.vue");
    assert_eq!(result.warning_count, 1);
    assert_eq!(result.error_count, 0);
    assert_eq!(result.diagnostics.len(), 1);
    let diagnostic = &result.diagnostics[0];
    let start = source.find("x-neighbor").unwrap();
    assert_eq!(
        (diagnostic.start, diagnostic.end),
        (start as u32, (start + 10) as u32)
    );
    assert_eq!(diagnostic.rule_name, RULE);
}

#[test]
fn static_compiler_predicate_is_used_as_given_without_prefix_inference() {
    fn project_element(tag: &str) -> bool {
        tag == "ion-owned"
    }
    let result = linter()
        .with_custom_elements(CustomElementMatcher::from_static_predicate(project_element))
        .lint_sfc(
            "<template><ion-owned/><ion-missing/></template>",
            "StaticPolicy.vue",
        );
    assert_eq!(result.warning_count, 1);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].start, 23);
}

#[test]
fn explicit_nuxt_mode_preserves_auto_import_policy() {
    let source = "<template><NuxtPage/><OtherAutoImport/></template>";
    assert_eq!(linter().lint_sfc(source, "NuxtPolicy.vue").warning_count, 2);
    let result = Linter::with_preset(LintPreset::Nuxt)
        .with_enabled_rules(Some(vec![RULE.into()]))
        .lint_sfc(source, "NuxtPolicy.vue");
    assert!(result.diagnostics.is_empty());
}

#[test]
fn custom_element_policy_does_not_enable_unselected_or_disabled_rules() {
    let source = "<template><MissingWidget/><x-neighbor/></template>";
    for disabled in [false, true] {
        let configured = if disabled {
            linter().with_disabled_rules(vec![RULE.into()])
        } else {
            Linter::with_preset(LintPreset::Incremental)
        };
        let result = configured
            .with_custom_elements(CustomElementMatcher::from_patterns(vec!["x-owned".into()]))
            .lint_sfc(source, "Unselected.vue");
        assert!(
            result
                .diagnostics
                .iter()
                .all(|diagnostic| diagnostic.rule_name != RULE)
        );
    }
}

#[test]
fn jsx_and_tsx_fallback_rules_use_the_same_explicit_custom_element_policy() {
    use vize_atelier_jsx::JsxLang;
    let source = "const View = () => <section><x-owned /><x-missing /></section>;";
    let mut observations = Vec::new();
    for (lang, filename) in [
        (JsxLang::Jsx, "ProjectContext.jsx"),
        (JsxLang::Tsx, "ProjectContext.tsx"),
    ] {
        let configured = linter()
            .with_custom_elements(CustomElementMatcher::from_patterns(vec!["x-owned".into()]));
        for repeat in [0, 1] {
            let before = linter().lint_jsx(source, filename, lang);
            let actual = configured.lint_jsx(source, filename, lang);
            observations.push(
                json!({"source": source, "filename": filename, "repeat": repeat,
                "unconfigured": complete(&before), "configured": complete(&actual)}),
            );
        }
    }
    let target = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/differential/component-registration-context-8142");
    std::fs::create_dir_all(&target).unwrap();
    std::fs::write(
        target.join("jsx-whole-packets.json"),
        serde_json::to_vec_pretty(&observations).unwrap(),
    )
    .unwrap();
    for observation in observations {
        for (key, tags) in [
            ("unconfigured", &["x-owned", "x-missing"][..]),
            ("configured", &["x-missing"][..]),
        ] {
            let diagnostics = tags.iter().map(|tag| {
                let start = source.find(tag).unwrap();
                json!({"rule_name": RULE, "severity": "warning", "message": "Component is used but not explicitly imported",
                    "start": start, "end": start + tag.len(),
                    "help": "Import the component in <script setup> or register it in components option", "labels": [], "fix": null})
            }).collect::<Vec<_>>();
            assert_eq!(
                observation[key],
                json!({"filename": observation["filename"],
                "diagnostics": diagnostics, "error_count": 0, "warning_count": tags.len()})
            );
        }
    }
}
