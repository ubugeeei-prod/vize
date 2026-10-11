#![cfg(test)]
#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_methods, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]

use serde_json::{Value, json};

// Reuse the existing whole-transaction application law without changing it.
#[path = "lsp_bound_event_casing_cli/application.rs"]
mod application;
#[path = "lsp_named_model_transactions_cli/contracts.rs"]
mod contracts;
#[path = "support/lsp_process.rs"]
mod lsp_process;
#[path = "lsp_named_model_transactions_cli/session.rs"]
mod session;
use session::Session;

const CONTRACTS: &str = include_str!(
    "../../../tests/_fixtures/differential/lsp/event-rename/4075/named-model/cases.json.txt"
);
const FILES: [&str; 3] = ["Child.vue", "Parent.vue", "Other.vue"];
const METHODS: [&str; 4] = ["prepareRename", "references", "definition", "rename"];

#[test]
fn named_model_child_transactions_preserve_every_authored_cursor_and_disk() {
    assert_origin("child");
}

#[test]
fn named_model_v_model_transactions_preserve_every_authored_cursor_and_disk() {
    assert_origin("v-model");
}

#[test]
fn named_model_update_transactions_preserve_every_authored_cursor_and_disk() {
    assert_origin("update");
}

fn assert_origin(origin: &str) {
    let cases = contracts::cases();
    let selected: Vec<_> = cases
        .iter()
        .filter(|case| case["origin"] == origin)
        .collect();
    assert_eq!(selected.len(), 4);
    let failures: Vec<_> = selected.into_iter().filter_map(assert_case).collect();
    assert_eq!(failures, Vec::<Value>::new(), "whole named-model contracts");
}

fn check(failures: &mut Vec<Value>, context: &str, actual: &Value, expected: &Value) {
    if actual != expected {
        failures.push(json!({"context":context,"actual":actual,"expected":expected}));
    }
}

fn assert_case(case: &Value) -> Option<Value> {
    let mut fixture = Session::new(case);
    let context = format!(
        "named model {}, newline={:?}, newName={}",
        case["origin"].as_str().unwrap(),
        case["newline"].as_str().unwrap(),
        case["newName"].as_str().unwrap(),
    );
    // Bind only the expected URI placeholders before the first native request.
    // Actual replies and their ordering are never normalized or filtered.
    let bound = contracts::bind(case, &fixture.uris());
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
    for query in bound["queries"].as_array().unwrap() {
        let mut replies = serde_json::Map::new();
        for method in METHODS {
            let request = &query["requests"][method];
            let actual = fixture.query(request);
            let allowed = &query["allowedWholeReplies"][method];
            if !allowed.as_array().unwrap().contains(&actual) {
                failures.push(json!({"context":"whole editor response","request":request,"actual":actual,"allowedWholeReplies":allowed}));
            }
            replies.insert(method.to_string(), actual);
        }
        observations.push(json!({"query":query,"replies":replies}));
    }
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
    assert_eq!(case["selectedApplicationQueryIndex"], 0);
    let selected = &observations[0]["replies"]["rename"];
    let applied = application::apply(&case["sources"], &fixture.uris(), selected);
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
    check(
        &mut failures,
        "unrelated component preserved by complete actual transaction",
        &actual_files["Other.vue"],
        &case["sources"]["Other.vue"],
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
    let expected = json!({"queries":bound["queries"],"selectedApplicationQueryIndex":0,"version2":expected_version2,"independentVersion3":expected_version3,"allDefinitionOriginsContracted":true,"allowedWholeDiagnosticPublications":allowed_publications});
    fixture.capture(&context, case, &expected, &actual);
    (!failures.is_empty())
        .then(|| json!({"context":context,"failures":failures,"expected":expected,"actual":actual}))
}
