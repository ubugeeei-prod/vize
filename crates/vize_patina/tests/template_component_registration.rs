use serde_json::{Value, json};
use vize_patina::{HelpLevel, LintPreset, LintResult, Linter};

fn complete(result: &LintResult) -> Value {
    let diagnostics: Vec<_> =
        result
            .diagnostics
            .iter()
            .map(|d| {
                let labels: Vec<_> = d.labels.iter().map(|label| json!({
            "message": label.message.as_str(), "start": label.start, "end": label.end,
        })).collect();
                json!({
                    "rule_name": d.rule_name, "severity": d.severity, "message": d.message.as_str(),
                    "start": d.start, "end": d.end, "help": d.help.as_ref().map(|h| h.as_str()),
                    "labels": labels, "fix": d.fix,
                })
            })
            .collect();
    json!({
        "filename": result.filename.as_str(), "error_count": result.error_count,
        "warning_count": result.warning_count, "diagnostics": diagnostics,
    })
}

const RULE: &str = "vue/component-name-in-template-casing";
const FIXTURES: &str = "fixtures/issue-8142-template-casing";

fn fixture(name: &str) -> Value {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join(FIXTURES)
        .join(name);
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn apply_edits(source: &str, result: &LintResult) -> String {
    let mut edits: Vec<_> = result
        .diagnostics
        .iter()
        .filter_map(|d| d.fix.as_ref())
        .flat_map(|fix| &fix.edits)
        .collect();
    edits.sort_by_key(|edit| edit.start);
    let mut fixed = source.to_owned();
    for edit in edits.into_iter().rev() {
        fixed.replace_range(edit.start as usize..edit.end as usize, &edit.new_text);
    }
    fixed
}

#[test]
fn complete_registered_only_corpus_preserves_source_diagnostics_and_fix_bytes() {
    let cases = fixture("source-after.json");
    assert_eq!(cases.as_array().unwrap().len(), 32);
    let before = fixture("source-before.json");
    assert_eq!(
        before["sourceRevision"],
        "b8ea16711117c502b14724e1eb1e4a7022a7f1d7"
    );
    for case in cases.as_array().unwrap() {
        let source = case["source"].as_str().unwrap();
        let filename = case["result"]["filename"].as_str().unwrap();
        let mut rules = vec![RULE.into()];
        if case["registrationRule"] == true {
            rules.push("vue/require-component-registration".into());
        }
        let linter = Linter::with_preset(LintPreset::Incremental)
            .with_enabled_rules(Some(rules))
            .with_help_level(HelpLevel::Full);
        let result = linter.lint_sfc(source, filename);
        assert_eq!(complete(&result), case["result"], "{}", case["id"]);
        assert_eq!(
            apply_edits(source, &result),
            case["fixed"],
            "{}",
            case["id"]
        );
        let repeat = linter.lint_sfc(source, filename);
        assert_eq!(complete(&repeat), case["result"], "repeat {}", case["id"]);
        let fixed = case["fixed"].as_str().unwrap();
        assert_eq!(
            apply_edits(fixed, &linter.lint_sfc(fixed, filename)),
            fixed,
            "idempotent {}",
            case["id"]
        );
    }
}

#[test]
fn explicit_registration_policy_matches_independent_authored_option_laws() {
    let cases = fixture("options-after.json");
    assert_eq!(cases.as_array().unwrap().len(), 18);
    for case in cases.as_array().unwrap() {
        let options = &case["options"];
        let only = options["registeredComponentsOnly"]
            .as_bool()
            .unwrap_or(true);
        let globals = options["globals"]
            .as_array()
            .map(|names| {
                names
                    .iter()
                    .map(|name| name.as_str().unwrap().into())
                    .collect()
            })
            .unwrap_or_default();
        let linter = Linter::with_preset(LintPreset::Incremental)
            .with_enabled_rules(Some(vec![RULE.into()]))
            .with_help_level(HelpLevel::Full)
            .with_component_name_in_template_casing_policy(
                vize_patina::rules::ComponentCasing::PascalCase,
                only,
                globals,
            );
        let source = case["source"].as_str().unwrap();
        let result = linter.lint_sfc(source, "Probe.vue");
        assert_eq!(complete(&result), case["result"], "{}", case["id"]);
        assert_eq!(
            apply_edits(source, &result),
            case["fixed"],
            "{}",
            case["id"]
        );
    }
}

#[test]
fn registration_membership_and_final_bytes_match_every_frozen_independent_case() {
    let baseline = fixture("source-after.json");
    let options = fixture("options-after.json");
    let packets = [
        "oracle-paired.json",
        "oracle-registration.json",
        "oracle-registration-additional.json",
        "oracle-options.json",
        "oracle-boundaries.json",
    ];
    let mut index = 0;
    for (packet_index, file) in packets.iter().enumerate() {
        let packet = fixture(file);
        for (option_index, oracle) in packet["cases"].as_array().unwrap().iter().enumerate() {
            let case = if packet_index >= 3 {
                &options[option_index + if packet_index == 4 { 12 } else { 0 }]
            } else {
                &baseline[index]
            };
            let source = oracle["source"]
                .as_str()
                .or(oracle["input"].as_str())
                .unwrap();
            assert_eq!(case["source"], source);
            let messages = if packet_index == 0 {
                &oracle["result"]["messages"]
            } else {
                assert_eq!(
                    oracle["observations"][0], oracle["observations"][1],
                    "independent repeat {file}"
                );
                &oracle["observations"][0]
            };
            let messages = messages.as_array().unwrap();
            let diagnostics = case["result"]["diagnostics"].as_array().unwrap();
            for (independent_rule, source_rule) in [
                (RULE, RULE),
                (
                    "vue/no-undef-components",
                    "vue/require-component-registration",
                ),
            ] {
                assert_eq!(
                    messages
                        .iter()
                        .filter(|d| d["ruleId"] == independent_rule)
                        .count(),
                    diagnostics
                        .iter()
                        .filter(|d| d["rule_name"] == source_rule)
                        .count(),
                    "{}",
                    case["id"]
                );
            }
            let mut fixed = source.to_owned();
            let mut edits: Vec<_> = messages.iter().filter_map(|d| d.get("fix")).collect();
            edits.sort_by_key(|edit| edit["range"][0].as_u64().unwrap());
            for edit in edits.into_iter().rev() {
                fixed.replace_range(
                    edit["range"][0].as_u64().unwrap() as usize
                        ..edit["range"][1].as_u64().unwrap() as usize,
                    edit["text"].as_str().unwrap(),
                );
            }
            assert_eq!(case["fixed"], fixed, "{}", case["id"]);
            if packet_index < 3 {
                index += 1;
            }
        }
    }
    assert_eq!(index, 32);
}
