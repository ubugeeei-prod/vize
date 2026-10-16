use super::{CONTRACT, classify};
use serde_json::{Value, json};

fn fixture(index: usize) -> (Value, Value, Value, &'static str) {
    let contract: Value = serde_json::from_str(CONTRACT).unwrap();
    let case = contract["cases"][index].clone();
    let rule = case["finding"]["ruleId"].as_str().unwrap();
    let severity = if case["finding"]["severity"] == "warning" {
        "warn"
    } else {
        "error"
    };
    let context = json!({
        "revision": case["revision"], "preset": case["preset"],
        "runtimeVersions": case["runtimeVersions"],
        "rules": { rule: severity },
    });
    let finding = case["finding"].clone();
    (
        case,
        context,
        finding,
        [
            include_str!(
                "../../../tests/_fixtures/differential/linter/reference-coverage-8849/AppShell.vue.txt"
            ),
            include_str!(
                "../../../tests/_fixtures/differential/linter/reference-coverage-8849/card.vue.txt"
            ),
        ][index],
    )
}

#[test]
fn exact_source_and_packet_are_required() {
    for index in 0..2 {
        let (case, context, finding, source) = fixture(index);
        let answer = classify(
            case["project"].as_str().unwrap(),
            &context,
            &finding,
            source,
            || false,
        )
        .unwrap();
        assert_eq!(answer["category"], "reference-coverage-gap");
        assert_eq!(answer["sourceSha256"], case["sourceSha256"]);
        assert_eq!(answer["options"], json!([]));
        assert!(
            classify("other-project", &context, &finding, source, || panic!(
                "must not scan reference packets for unrelated findings"
            ))
            .is_none()
        );
        assert!(
            classify(
                case["project"].as_str().unwrap(),
                &context,
                &finding,
                &format!("{source}\n"),
                || false
            )
            .is_none()
        );
        for field in [
            "file",
            "ruleId",
            "severity",
            "line",
            "column",
            "endLine",
            "endColumn",
            "message",
        ] {
            let mut changed = finding.clone();
            changed[field] = match changed[field].as_u64() {
                Some(number) => json!(number + 1),
                None => json!(format!("{}x", changed[field].as_str().unwrap())),
            };
            assert!(
                classify(
                    case["project"].as_str().unwrap(),
                    &context,
                    &changed,
                    source,
                    || false
                )
                .is_none(),
                "{field}"
            );
        }
    }
}

#[test]
fn changed_options_versions_preset_revision_and_new_reference_coverage_are_scored() {
    for index in 0..2 {
        let (case, context, finding, source) = fixture(index);
        let rule_pointer = format!(
            "/rules/{}",
            finding["ruleId"].as_str().unwrap().replace('/', "~1")
        );
        for (pointer, value) in [
            ("/revision", json!("new revision")),
            ("/preset", json!("opinionated")),
            ("/runtimeVersions/eslint-plugin-vue", json!("next")),
            ("/runtimeVersions/vue-eslint-parser", json!("next")),
            ("/runtimeVersions/eslint", json!("next")),
            ("/runtimeVersions/@typescript-eslint~1parser", json!("next")),
            (
                rule_pointer.as_str(),
                json!(["warn", {"atComponent":"shorthand"}]),
            ),
            (rule_pointer.as_str(), json!("off")),
        ] {
            let mut changed = context.clone();
            *changed.pointer_mut(pointer).unwrap() = value;
            assert!(
                classify(
                    case["project"].as_str().unwrap(),
                    &changed,
                    &finding,
                    source,
                    || false
                )
                .is_none(),
                "{pointer}"
            );
        }
        let mut reference = finding.clone();
        reference["column"] = json!(reference["column"].as_u64().unwrap() + 1);
        assert!(
            classify(
                case["project"].as_str().unwrap(),
                &context,
                &finding,
                source,
                || reference["ruleId"] == finding["ruleId"]
            )
            .is_none()
        );
    }
}
