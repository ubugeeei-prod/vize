use std::path::Path;

use serde_json::{Value, json};

use super::super::{generate_vue_content_mapper_transform, protocol_spans};
use crate::virtual_ts::{VizeMapping, VizeSubSpan};

const ORIGINAL: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/",
    "content-mapper-quoted-event-definition/input.vue.txt"
));
const CASES: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/",
    "content-mapper-quoted-event-definition/cases.json"
));

#[test]
#[expect(clippy::string_slice, reason = "tests assert by panicking")]
fn original_call_signature_definition_keeps_the_complete_quoted_span() {
    let original_fixture = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../vize/tests/fixtures/content_mapper_project/src/CallSignatureChild.vue"
    ));
    assert_eq!(
        ORIGINAL, original_fixture,
        "original input bytes stay intact"
    );
    let corpus: Value = serde_json::from_str(CASES).unwrap();
    let expected = &corpus["original"];
    let result =
        generate_vue_content_mapper_transform(Path::new("CallSignatureChild.vue"), ORIGINAL)
            .expect("original transform");
    assert!(result.diagnostics.is_empty(), "{:#?}", result.diagnostics);
    let authored_map = result.text.find("type __VizeAuthoredEventMap").unwrap();
    let literal = expected["literal"].as_str().unwrap();
    let generated = authored_map + result.text[authored_map..].find(literal).unwrap();
    let original_start = expected["sourceRange"][0].as_u64().unwrap() as usize;
    let original_end = expected["sourceRange"][1].as_u64().unwrap() as usize;
    assert_eq!(&ORIGINAL[original_start..original_end], literal);
    let endpoint: Vec<_> = result
        .mappings
        .iter()
        .filter(|span| span.0[0] == generated)
        .collect();
    assert_eq!(
        serde_json::to_value(endpoint).unwrap(),
        json!([[
            generated,
            expected["spanLength"],
            original_start,
            original_end - original_start,
            expected["kind"],
            expected["features"]
        ]]),
        "complete original quoted definition endpoint"
    );
    assert!(
        result
            .mappings
            .windows(2)
            .all(|spans| { spans[0].0[0] + spans[0].0[1] <= spans[1].0[0] }),
        "protocol generated spans must be disjoint"
    );
}

#[test]
fn quoted_pairs_and_refusals_preserve_all_authored_protocol_results() {
    let corpus: Value = serde_json::from_str(CASES).unwrap();
    let cases = corpus["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 16);
    let mut actual = Vec::new();
    let mut expected = Vec::new();
    for case in cases {
        let mappings: Vec<_> = case["mappings"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| VizeMapping {
                gen_range: range(row, "generated"),
                src_range: range(row, "original"),
                sub_spans: row
                    .get("subSpans")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .map(|span| VizeSubSpan {
                        gen_range: range(span, "generated"),
                        src_range: range(span, "original"),
                    })
                    .collect(),
            })
            .collect();
        let spans = protocol_spans(
            case["source"].as_str().unwrap(),
            case["generated"].as_str().unwrap(),
            &mappings,
        );
        actual.push(json!({"name":case["name"],"spans":spans}));
        expected.push(json!({"name":case["name"],"spans":case["expected"]}));
    }
    assert_eq!(actual, expected, "all complete quoted-span corpus packets");
}

fn range(value: &Value, key: &str) -> std::ops::Range<usize> {
    value[key][0].as_u64().unwrap() as usize..value[key][1].as_u64().unwrap() as usize
}
