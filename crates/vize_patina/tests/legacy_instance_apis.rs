//! Exact #9217 property spans, with explicit bounded provider blind spots.
use serde::Deserialize;
use vize_patina::{HelpLevel, LintPreset, Linter};

#[derive(Deserialize)]
struct Corpus {
    records: Vec<Case>,
}
#[derive(Deserialize)]
struct Gaps {
    gaps: Vec<Gap>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Gap {
    case_id: String,
    native_messages: Vec<NativeMessage>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct NativeMessage {
    rule_id: String,
    start: u32,
    end: u32,
}
#[derive(Deserialize)]
struct Case {
    id: String,
    source: String,
    messages: Vec<Message>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Message {
    rule_id: String,
    line: usize,
    column: usize,
    end_line: usize,
    end_column: usize,
}

fn byte_offset(source: &str, line: usize, column: usize) -> u32 {
    let start = source
        .split_inclusive('\n')
        .take(line - 1)
        .map(str::len)
        .sum::<usize>();
    let mut units = 1;
    for (index, character) in source.get(start..).unwrap().char_indices() {
        if units == column {
            return (start + index) as u32;
        }
        units += character.len_utf16();
    }
    assert_eq!(units, column);
    source.len() as u32
}

#[test]
fn complete_authored_provider_packets_have_exact_native_rule_and_property_spans() {
    let corpus: Corpus = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/lint/legacy-instance-apis-9217/independent.json"
    ))
    .unwrap();
    assert_eq!(corpus.records.len(), 55);
    let gaps: Gaps = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/lint/legacy-instance-apis-9217/provider-gaps.json"
    ))
    .unwrap();
    assert_eq!(gaps.gaps.len(), 5);
    for case in corpus.records {
        let mut expected: Vec<_> = case
            .messages
            .into_iter()
            .map(|message| {
                (
                    message.rule_id.replacen("vue/", "script/", 1),
                    byte_offset(&case.source, message.line, message.column),
                    byte_offset(&case.source, message.end_line, message.end_column),
                )
            })
            .collect();
        if let Some(gap) = gaps.gaps.iter().find(|gap| gap.case_id == case.id) {
            expected = gap
                .native_messages
                .iter()
                .map(|message| (message.rule_id.clone(), message.start, message.end))
                .collect();
        }
        expected.sort();
        for preset in [LintPreset::Essential, LintPreset::HappyPath] {
            let linter = Linter::with_preset(preset)
                .with_enabled_rules(Some(vec![
                    "script/no-deprecated-dollar-listeners-api".into(),
                    "script/no-deprecated-events-api".into(),
                ]))
                .with_help_level(HelpLevel::Full);
            let result = linter.lint_sfc(&case.source, "Component.vue");
            let mut actual: Vec<_> = result
                .diagnostics
                .into_iter()
                .map(|diagnostic| {
                    (
                        diagnostic.rule_name.to_string(),
                        diagnostic.start,
                        diagnostic.end,
                    )
                })
                .collect();
            actual.sort();
            assert_eq!(actual, expected, "{} ({preset:?})", case.id);
            let disabled = Linter::with_preset(preset)
                .with_disabled_rules(vec![
                    "script/no-deprecated-dollar-listeners-api".into(),
                    "script/no-deprecated-events-api".into(),
                ])
                .lint_sfc(&case.source, "Component.vue");
            assert!(
                !disabled
                    .diagnostics
                    .iter()
                    .any(|diagnostic| diagnostic.rule_name
                        == "script/no-deprecated-dollar-listeners-api"
                        || diagnostic.rule_name == "script/no-deprecated-events-api"),
                "{}",
                case.id
            );
        }
    }
}
