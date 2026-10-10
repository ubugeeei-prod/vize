#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_methods, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]

use super::{CONTRACTS, FILES, METHODS, application};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

macro_rules! files {
    ($prefix:literal) => {
        [
            include_str!(concat!(
                "../../../../tests/_fixtures/differential/lsp/event-rename/4075/named-model/",
                $prefix,
                "-Child.vue.txt"
            )),
            include_str!(concat!(
                "../../../../tests/_fixtures/differential/lsp/event-rename/4075/named-model/",
                $prefix,
                "-Parent.vue.txt"
            )),
            include_str!(concat!(
                "../../../../tests/_fixtures/differential/lsp/event-rename/4075/named-model/",
                $prefix,
                "-Other.vue.txt"
            )),
        ]
    };
}

const INPUTS: [&str; 3] = files!("input");
const GOLDENS: [&str; 3] = files!("updated");

pub fn bind(value: &Value, uris: &Value) -> Value {
    match value {
        Value::String(text) if text.starts_with('$') => {
            let name = text.strip_prefix('$').unwrap();
            assert_eq!(
                FILES.into_iter().find(|candidate| *candidate == name),
                Some(name),
                "unknown expected URI marker"
            );
            uris[name].clone()
        }
        Value::Array(values) => json!(values.iter().map(|v| bind(v, uris)).collect::<Vec<_>>()),
        Value::Object(values) => json!(
            values
                .iter()
                .map(|(key, v)| {
                    let key = if let Some(name) = key.strip_prefix('$') {
                        assert_eq!(
                            FILES.into_iter().find(|candidate| *candidate == name),
                            Some(name),
                            "unknown expected URI key"
                        );
                        uris[name].as_str().unwrap().to_string()
                    } else {
                        key.clone()
                    };
                    (key, bind(v, uris))
                })
                .collect::<serde_json::Map<_, _>>()
        ),
        _ => value.clone(),
    }
}

pub fn cases() -> Vec<Value> {
    assert_eq!(
        Sha256::digest(CONTRACTS.as_bytes())
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>(),
        "dce60b59a09de699b8ba9aa928463790ff14627c8d5f6d46c8a50e23886c70b8",
        "immutable pre-query authored corpus"
    );
    let authored: Value = serde_json::from_str(CONTRACTS).unwrap();
    assert_eq!(authored["schema"], "vize-named-model-editor-contracts-v1");
    let cases = authored["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 12);
    assert_eq!(
        cases
            .iter()
            .map(|case| case["queries"].as_array().unwrap().len())
            .sum::<usize>(),
        224
    );
    for (index, case) in cases.iter().enumerate() {
        let origin = ["child", "v-model", "update"][index / 4];
        let newline = ["\n", "\r\n"][(index % 4) / 2];
        let new_name = ["nextValue", "next-value"][index % 2];
        assert_eq!(case["origin"], origin);
        assert_eq!(case["newline"], newline);
        assert_eq!(case["newName"], new_name);
        assert_eq!(case["selectedApplicationQueryIndex"], 0);
        assert_eq!(case["sources"].as_object().unwrap().len(), 3);
        assert_eq!(case["completeGoldenFiles"].as_object().unwrap().len(), 3);
        for (offset, name) in FILES.iter().enumerate() {
            assert_eq!(case["sources"][name], INPUTS[offset].replace('\n', newline));
            assert_eq!(
                case["completeGoldenFiles"][name],
                GOLDENS[offset].replace('\n', newline)
            );
            assert_eq!(case["expectedVersion2Diagnostics"][name], json!([]));
            assert_eq!(
                case["expectedIndependentVersion3Diagnostics"][name],
                json!([])
            );
        }
        assert_eq!(
            case["sources"]["Other.vue"],
            case["completeGoldenFiles"]["Other.vue"]
        );
        let expected_sites = [
            ("Child.vue", 3, 21, 29),
            ("Parent.vue", 10, 28, 36),
            ("Parent.vue", 11, 17, 26),
            ("Parent.vue", 12, 10, 25),
            ("Parent.vue", 13, 10, 26),
        ];
        let refs: Vec<_> = expected_sites.iter().map(|(file, line, start, end)| {
            json!({"uri":format!("${file}"),"range":{"start":{"line":line,"character":start},"end":{"line":line,"character":end}}})
        }).collect();
        assert_eq!(case["fullReferences"], json!(refs));
        let selection = match origin {
            "child" => &expected_sites[..1],
            "v-model" => &expected_sites[1..3],
            "update" => &expected_sites[3..],
            _ => unreachable!("fixed origin sequence"),
        };
        let expected_positions: Vec<_> = selection.iter().flat_map(|(file, line, start, end)| {
            (*start..*end).map(move |character| json!({"file":file,"position":{"line":line,"character":character}}))
        }).collect();
        let queries = case["queries"].as_array().unwrap();
        assert_eq!(queries.len(), [8, 17, 31][index / 4]);
        for (offset, query) in queries.iter().enumerate() {
            assert_eq!(query["file"], expected_positions[offset]["file"]);
            assert_eq!(query["position"], expected_positions[offset]["position"]);
            assert_eq!(query["requests"].as_object().unwrap().len(), 4);
            assert_eq!(query["allowedWholeReplies"].as_object().unwrap().len(), 4);
            for (method_index, method) in METHODS.iter().enumerate() {
                let request = &query["requests"][method];
                let id = 2 + offset * 4 + method_index;
                assert_eq!(request["id"], id);
                assert_eq!(request["jsonrpc"], "2.0");
                assert_eq!(request["method"], format!("textDocument/{method}"));
                assert_eq!(request["params"]["position"], query["position"]);
                assert_eq!(
                    request["params"]["textDocument"]["uri"],
                    format!("${}", query["file"].as_str().unwrap())
                );
                for reply in query["allowedWholeReplies"][method].as_array().unwrap() {
                    assert_eq!(reply.as_object().unwrap().len(), 3);
                    assert_eq!(reply["jsonrpc"], "2.0");
                    assert_eq!(reply["id"], id);
                }
            }
            assert_eq!(query["requests"]["rename"]["params"]["newName"], new_name);
            assert_eq!(
                query["allowedWholeReplies"]["references"][0]["result"],
                case["fullReferences"]
            );
        }
        let uris = synthetic_uris();
        let bound = bind(case, &uris);
        for query in bound["queries"].as_array().unwrap() {
            let reply = &query["allowedWholeReplies"]["rename"][0];
            assert_eq!(
                application::apply(&case["sources"], &uris, reply).unwrap(),
                case["completeGoldenFiles"]
            );
        }
    }
    cases.clone()
}

fn synthetic_uris() -> Value {
    json!({"Child.vue":"file:///authored/src/Child.vue","Parent.vue":"file:///authored/src/Parent.vue","Other.vue":"file:///authored/src/Other.vue"})
}

#[test]
fn authored_named_model_oracles_preserve_twelve_whole_contexts() {
    assert_eq!(cases().len(), 12);
}

#[test]
#[should_panic(expected = "unknown expected URI marker")]
fn unknown_expected_uri_markers_are_refused() {
    bind(&json!("$Foreign.vue"), &synthetic_uris());
}

#[test]
#[should_panic(expected = "unknown expected URI key")]
fn unknown_expected_uri_keys_are_refused() {
    bind(&json!({"$Foreign.vue": []}), &synthetic_uris());
}

#[test]
fn incomplete_contaminated_and_unsupported_transactions_cannot_match_the_goldens() {
    let case = &cases()[0];
    let uris = synthetic_uris();
    let bound = bind(case, &uris);
    let valid = &bound["queries"][0]["allowedWholeReplies"]["rename"][0];
    let child = uris["Child.vue"].as_str().unwrap();
    let parent = uris["Parent.vue"].as_str().unwrap();
    let other = uris["Other.vue"].as_str().unwrap();
    let mut missing = valid.clone();
    missing["result"]["changes"]
        .as_object_mut()
        .unwrap()
        .remove(parent);
    assert_ne!(
        application::apply(&case["sources"], &uris, &missing).unwrap(),
        case["completeGoldenFiles"]
    );
    let mut contaminated = valid.clone();
    contaminated["result"]["changes"][other] = valid["result"]["changes"][child].clone();
    assert_ne!(
        application::apply(&case["sources"], &uris, &contaminated).unwrap(),
        case["completeGoldenFiles"]
    );
    let mut foreign = valid.clone();
    foreign["result"]["changes"]["file:///authored/node_modules/foreign.vue"] = json!([]);
    assert!(application::apply(&case["sources"], &uris, &foreign).is_err());
    let mut overlap = valid.clone();
    overlap["result"]["changes"][child]
        .as_array_mut()
        .unwrap()
        .push(valid["result"]["changes"][child][0].clone());
    assert!(application::apply(&case["sources"], &uris, &overlap).is_err());
    let unsupported = json!({"jsonrpc":"2.0","id":5,"result":{"documentChanges":[{"kind":"delete","uri":child}]}});
    assert!(application::apply(&case["sources"], &uris, &unsupported).is_err());
    let null = json!({"jsonrpc":"2.0","id":5,"result":null});
    assert_ne!(
        application::apply(&case["sources"], &uris, &null).unwrap(),
        case["completeGoldenFiles"]
    );
}
