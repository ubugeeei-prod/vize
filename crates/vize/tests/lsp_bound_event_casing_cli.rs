#![cfg(test)]
#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_methods, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]

use serde_json::{Value, json};

#[path = "lsp_bound_event_casing_cli/application.rs"]
mod application;
#[path = "support/lsp_process.rs"]
mod lsp_process;
#[path = "lsp_bound_event_casing_cli/session.rs"]
mod session;
use session::Session;

const CONTRACTS: &str = include_str!(
    "../../../tests/_fixtures/differential/lsp/event-rename/4075/bound-emitter/cases.json.txt"
);
const FILES: [&str; 3] = ["Child.vue", "Parent.vue", "Other.vue"];

macro_rules! fixture_files {
    ($form:literal, $prefix:literal) => {
        [
            include_str!(concat!(
                "../../../tests/_fixtures/differential/lsp/event-rename/4075/bound-emitter/",
                $form,
                "/",
                $prefix,
                "-Child.vue.txt"
            )),
            include_str!(concat!(
                "../../../tests/_fixtures/differential/lsp/event-rename/4075/bound-emitter/",
                $form,
                "/",
                $prefix,
                "-Parent.vue.txt"
            )),
            include_str!(concat!(
                "../../../tests/_fixtures/differential/lsp/event-rename/4075/bound-emitter/",
                $form,
                "/",
                $prefix,
                "-Other.vue.txt"
            )),
        ]
    };
}

#[test]
fn bound_record_event_casing_is_complete_at_every_authored_cursor() {
    assert_form("record");
}

#[test]
fn bound_call_signature_event_casing_is_complete_at_every_authored_cursor() {
    assert_form("call-signature");
}

#[test]
fn bound_runtime_array_event_casing_is_complete_at_every_authored_cursor() {
    assert_form("runtime-array");
}

#[test]
fn bound_runtime_object_event_casing_is_complete_at_every_authored_cursor() {
    assert_form("runtime-object");
}

#[test]
fn bound_generic_event_casing_is_complete_at_every_authored_cursor() {
    assert_form("generic");
}

fn assert_form(form: &str) {
    let (inputs, goldens) = match form {
        "record" => (
            fixture_files!("record", "input"),
            fixture_files!("record", "updated"),
        ),
        "call-signature" => (
            fixture_files!("call-signature", "input"),
            fixture_files!("call-signature", "updated"),
        ),
        "runtime-array" => (
            fixture_files!("runtime-array", "input"),
            fixture_files!("runtime-array", "updated"),
        ),
        "runtime-object" => (
            fixture_files!("runtime-object", "input"),
            fixture_files!("runtime-object", "updated"),
        ),
        "generic" => (
            fixture_files!("generic", "input"),
            fixture_files!("generic", "updated"),
        ),
        _ => unreachable!("fixed authored form"),
    };
    let authored: Value = serde_json::from_str(CONTRACTS).unwrap();
    let all_cases = authored["cases"].as_array().unwrap();
    assert_eq!(all_cases.len(), 20);
    assert_eq!(
        all_cases
            .iter()
            .map(|case| case["queries"].as_array().unwrap().len())
            .sum::<usize>(),
        660
    );
    let actual_forms: Vec<_> = all_cases
        .iter()
        .map(|case| case["form"].as_str().unwrap())
        .collect();
    let expected_forms: Vec<_> = [
        "record",
        "call-signature",
        "runtime-array",
        "runtime-object",
        "generic",
    ]
    .into_iter()
    .flat_map(|form| [form; 4])
    .collect();
    assert_eq!(
        actual_forms, expected_forms,
        "complete authored form sequence"
    );
    let cases: Vec<_> = authored["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|case| case["form"] == form)
        .collect();
    assert_eq!(cases.len(), 4);
    for case in &cases {
        let newline = case["newline"].as_str().unwrap();
        for (index, name) in FILES.iter().enumerate() {
            assert_eq!(
                case["sources"][name],
                json!(inputs[index].replace('\n', newline))
            );
            assert_eq!(
                case["completeGoldenFiles"][name],
                json!(goldens[index].replace('\n', newline))
            );
        }
    }
    let failures: Vec<_> = cases.into_iter().filter_map(assert_case).collect();
    assert_eq!(failures, Vec::<Value>::new(), "whole bound event contracts");
}

fn expected_response(id: i64, result: Value) -> Value {
    json!({"jsonrpc":"2.0","id":id,"result":result})
}

fn check(failures: &mut Vec<Value>, context: &str, actual: &Value, expected: &Value) {
    if actual != expected {
        failures.push(json!({"context":context,"actual":actual,"expected":expected}));
    }
}

fn assert_case(case: &Value) -> Option<Value> {
    let mut fixture = Session::new(case);
    let context = format!(
        "bound event {}, newline={:?}, newName={}",
        case["form"].as_str().unwrap(),
        case["newline"].as_str().unwrap(),
        case["newName"].as_str().unwrap(),
    );
    // Every complete vector and golden is immutable before any editor query.
    let references = case["fullReferences"]
        .as_array()
        .unwrap()
        .iter()
        .map(
            |site| json!({"uri":fixture.uri(site["file"].as_str().unwrap()),"range":site["range"]}),
        )
        .collect::<Vec<_>>();
    let mut changes = serde_json::Map::new();
    for edit in case["fullEdits"].as_array().unwrap() {
        let uri = fixture.uri(edit["file"].as_str().unwrap());
        changes
            .entry(uri)
            .or_insert_with(|| json!([]))
            .as_array_mut()
            .unwrap()
            .push(json!({"range":edit["range"],"newText":edit["newText"]}));
    }
    let target = &case["definitionFromAllParentAndCallPositions"][0];
    let definition =
        json!({"uri":fixture.uri(target["file"].as_str().unwrap()),"range":target["range"]});
    let rename = json!({"changes":changes});
    let allowed_publications = fixture.expected_publications(case);
    let mut failures = Vec::new();
    let guard = fixture.prove_native_runtime(&mut failures);
    let initial: Vec<_> = FILES
        .iter()
        .map(|name| {
            let actual = fixture.open(name, case["sources"][name].as_str().unwrap());
            check(
                &mut failures,
                "initial whole diagnostics",
                &actual,
                &fixture.publication(name, 1, json!([])),
            );
            actual
        })
        .collect();
    let mut observations = Vec::new();
    let mut wanted = Vec::new();
    for (index, query) in case["queries"].as_array().unwrap().iter().enumerate() {
        let params = json!({"textDocument":{"uri":fixture.uri(query["file"].as_str().unwrap())},"position":query["position"]});
        let (refs_id, refs) = fixture.request(
            "textDocument/references",
            params.clone(),
            json!({"context":{"includeDeclaration":true}}),
        );
        let (def_id, def) = fixture.request("textDocument/definition", params.clone(), json!({}));
        let (rename_id, actual_rename) = fixture.request(
            "textDocument/rename",
            params,
            json!({"newName":case["newName"]}),
        );
        let expected_refs = expected_response(refs_id, json!(references));
        let expected_rename = expected_response(rename_id, rename.clone());
        let allowed_definitions = json!([
            expected_response(def_id, definition.clone()),
            expected_response(def_id, json!([definition])),
        ]);
        check(
            &mut failures,
            "whole references response",
            &refs,
            &expected_refs,
        );
        check(
            &mut failures,
            "whole rename response",
            &actual_rename,
            &expected_rename,
        );
        // No independently authored declaration-origin definition vector exists.
        // All160 responses remain in the packet, with an explicit absent contract.
        let contracted = index >= 8;
        if contracted && !allowed_definitions.as_array().unwrap().contains(&def) {
            failures.push(json!({"context":"whole contracted definition response","query":query,"actual":def,"allowedWholeResponses":allowed_definitions}));
        }
        observations.push(json!({"query":query,"referencesReply":refs,"definitionReply":def,"renameReply":actual_rename,"definitionContracted":contracted}));
        wanted.push(json!({"query":query,"referencesReply":expected_refs,"renameReply":expected_rename,"definitionContracted":contracted,"allowedWholeDefinitionResponses":if contracted {allowed_definitions} else {Value::Null}}));
    }
    assert_eq!(observations.len(), 33);
    let original_disk: Vec<_> = FILES
        .iter()
        .map(|name| json!({"file":name,"text":fixture.read(name)}))
        .collect();
    for name in FILES {
        check(
            &mut failures,
            "original disk bytes after all queries",
            &json!(fixture.read(name)),
            &case["sources"][name],
        );
    }
    // Query0 is fixed before execution, never selected for a successful reply.
    let applied = application::apply(
        &case["sources"],
        &fixture.uris(),
        &observations[0]["renameReply"],
    );
    let application_error = applied.as_ref().err().cloned();
    let actual_files = applied.unwrap_or_else(|_| case["sources"].clone());
    let version2 = fixture.install(&actual_files, 2);
    let expected_version2 = fixture.expected_files(&case["completeGoldenFiles"], 2);
    check(
        &mut failures,
        "actual complete application and disk version2",
        &version2,
        &expected_version2,
    );
    check(
        &mut failures,
        "whole application refusal",
        &json!(application_error),
        &Value::Null,
    );
    let version3 = fixture.install(&case["completeGoldenFiles"], 3);
    let expected_version3 = fixture.expected_files(&case["completeGoldenFiles"], 3);
    check(
        &mut failures,
        "independent complete golden and disk version3",
        &version3,
        &expected_version3,
    );
    fixture.finish();
    let publications = fixture.publications();
    for publication in &publications {
        if !allowed_publications.contains(publication) {
            failures.push(json!({"context":"complete diagnostic publication stream","actual":publication,"allowedWholePublications":allowed_publications}));
        }
    }
    let actual = json!({"nativeGuard":guard,"initialDiagnostics":initial,"observations":observations,"originalDiskAfterQueries":original_disk,"applicationError":application_error,"selectedApplicationQueryIndex":0,"appliedFiles":version2,"independentGoldenFiles":version3,"allDiagnosticPublications":publications});
    let expected = json!({"queries":wanted,"selectedApplicationQueryIndex":0,"version2":expected_version2,"independentVersion3":expected_version3,"declarationOriginDefinitionsObservedOnly":8,"allowedWholeDiagnosticPublications":allowed_publications});
    fixture.capture(&context, case, &expected, &actual);
    (!failures.is_empty())
        .then(|| json!({"context":context,"failures":failures,"expected":expected,"actual":actual}))
}
