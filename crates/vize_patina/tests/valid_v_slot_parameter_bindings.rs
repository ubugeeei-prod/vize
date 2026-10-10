//! Independently authored whole parameter packets; original campaigns stay intact.
use serde_json::{Value, json};
use vize_patina::{HelpLevel, LintPreset, LintResult, Linter};

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

#[test]
fn whole_parameter_rule_packets_repeat_for_both_modifier_options() {
    let corpus: Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/lint/slot-parameter-bindings-8142/controls.json"
    ))
    .unwrap();
    let cases = corpus["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 24);
    assert_eq!(corpus["referenceDifferences"].as_array().unwrap().len(), 2);
    let mut captures = Vec::new();
    let mut mismatches = Vec::new();
    for (allow, key) in [(false, "false"), (true, "true")] {
        let linter = Linter::with_preset(LintPreset::Incremental)
            .with_enabled_rules(Some(vec!["vue/valid-v-slot".into()]))
            .with_valid_v_slot_allow_modifiers(allow)
            .with_help_level(HelpLevel::Full);
        for repeat in [1, 2] {
            for case in cases {
                let source = case["source"].as_str().unwrap();
                let authored = &case["authored"];
                let start = authored["byteSpan"]["start"].as_u64().unwrap() as usize;
                let end = authored["byteSpan"]["end"].as_u64().unwrap() as usize;
                assert_eq!(source.get(start..end), authored["directive"].as_str());
                let actual = complete(&linter.lint_sfc(source, "ParameterSlot.vue"));
                let capture = json!({"id": case["id"], "allowModifiers": allow, "repeat": repeat,
                    "actual": actual, "expected": case["expectedByOption"][key]});
                if actual != case["expectedByOption"][key] {
                    mismatches.push(capture.clone());
                }
                captures.push(capture);
            }
        }
    }
    let artifact = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/differential/slot-parameter-source-packets.json");
    std::fs::create_dir_all(artifact.parent().unwrap()).unwrap();
    std::fs::write(artifact, serde_json::to_vec_pretty(&captures).unwrap()).unwrap();
    assert!(
        mismatches.is_empty(),
        "complete packet mismatches: {mismatches:#?}"
    );
}

#[test]
fn configured_but_unselected_and_disabled_parameter_rule_remains_silent() {
    let source = "<template><FancyPanel><template #[slot].modifier=\"{ slot = fallback }\">ready</template></FancyPanel></template>";
    for allow in [false, true] {
        let unselected = Linter::with_preset(LintPreset::Incremental)
            .with_valid_v_slot_allow_modifiers(allow)
            .lint_sfc(source, "ParameterSlot.vue");
        let disabled = Linter::with_preset(LintPreset::Incremental)
            .with_enabled_rules(Some(vec!["vue/valid-v-slot".into()]))
            .with_disabled_rules(vec!["vue/valid-v-slot".into()])
            .with_valid_v_slot_allow_modifiers(allow)
            .lint_sfc(source, "ParameterSlot.vue");
        for result in [unselected, disabled] {
            assert_eq!(
                complete(&result),
                json!({
                    "filename": "ParameterSlot.vue", "diagnostics": [], "error_count": 0, "warning_count": 0,
                })
            );
        }
    }
}
