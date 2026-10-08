//! Trace the unchanged original partial-application law through the editor.
#![expect(
    clippy::disallowed_types,
    clippy::expect_used,
    reason = "complete test custody uses std strings and fails closed by panicking"
)]

use serde_json::{Value, json};
use tower_lsp::lsp_types::{TextEdit, WorkspaceEdit};

use crate::ide::{IdeContext, RenameService};

#[path = "editor_event_partial_diagnostics_tests/fixture.rs"]
mod fixture;
use fixture::{APP, Fixture, MISSING_CALL, TOGGLE, UPDATED_APP, UPDATED_TOGGLE};

fn expected_rename(fixture: &Fixture) -> Value {
    json!({"changes":{
        fixture.uri("App.vue"):[{"range":{"start":{"line":9,"character":11},"end":{"line":9,"character":17}},"newText":"update"}],
        fixture.uri("Toggle.vue"):[
            {"range":{"start":{"line":2,"character":2},"end":{"line":2,"character":8}},"newText":"update"},
            {"range":{"start":{"line":6,"character":8},"end":{"line":6,"character":14}},"newText":"update"},
        ],
    }})
}

fn apply(source: &str, edits: &[TextEdit]) -> String {
    let mut edits = edits.to_vec();
    edits.sort_by_key(|edit| std::cmp::Reverse(edit.range.start));
    let mut applied = source.to_owned();
    for edit in edits {
        let start = fixture::offset(source, edit.range.start);
        let end = fixture::offset(source, edit.range.end);
        assert!(start < end, "complete returned edit range");
        applied.replace_range(start..end, &edit.new_text);
    }
    applied
}

#[test]
fn original_partial_event_application_retains_complete_editor_native_diagnostic_custody() {
    for newline in ["\n", "\r\n"] {
        let toggle = TOGGLE.replace('\n', newline);
        let app = APP.replace('\n', newline);
        let missing_call = MISSING_CALL.replace('\n', newline);
        let golden_toggle = UPDATED_TOGGLE.replace('\n', newline);
        let golden_app = UPDATED_APP.replace('\n', newline);
        let fixture = Fixture::new(&toggle, &app);
        // Independently authored full answers precede every native request.
        let expected_edit = expected_rename(&fixture);
        let expected_guard = json!([{
            "range":{"start":{"line":1,"character":6},"end":{"line":1,"character":17}},
            "severity":1,"code":2322,"source":"vize/types",
            "message":"Type 'string' is not assignable to type 'number'."
        }]);
        let expected_partial = json!([{
            "range":{"start":{"line":6,"character":7},"end":{"line":6,"character":15}},
            "severity":1,"code":2345,"source":"vize/types",
            "message":"Argument of type '\"change\"' is not assignable to parameter of type '\"update\"'."
        }]);
        let expected = json!({"guard":[expected_guard,[]],"initial":[[],[]],
        "rename":expected_edit,"partialFiles":[
            {"file":"Toggle.vue","text":missing_call,"disk":missing_call,"version":2,"diagnostics":expected_partial},
            {"file":"App.vue","text":golden_app,"disk":golden_app,"version":2,"diagnostics":[]},
        ],"independentRepair":[
            {"file":"Toggle.vue","text":golden_toggle,"disk":golden_toggle,"version":3,"diagnostics":[]},
            {"file":"App.vue","text":golden_app,"disk":golden_app,"version":3,"diagnostics":[]},
        ]});
        let mut custody = Vec::new();
        let guard = fixture.prove_native(&mut custody);
        let mut initial = Vec::new();
        for (name, text) in [("Toggle.vue", &toggle), ("App.vue", &app)] {
            fixture.open(name, text);
            initial.push(fixture.collect(name, &mut custody));
        }
        let uri = fixture.uri("Toggle.vue");
        let offset = fixture::offset(&toggle, tower_lsp::lsp_types::Position::new(6, 9));
        let context = IdeContext::new(&fixture.state, &uri, offset).expect("original call cursor");
        let edit = crate::runtime::block_on(async {
            let _scope = fixture.state.corsa_request_scope().await;
            let bridge = fixture
                .state
                .get_corsa_bridge()
                .await
                .expect("required native bridge");
            RenameService::rename_with_corsa(&context, "update", Some(bridge)).await
        });
        let rename = serde_json::to_value(&edit).expect("complete workspace edit");
        let WorkspaceEdit {
            changes: Some(changes),
            document_changes: None,
            change_annotations: None,
        } = edit.expect("original atomic rename")
        else {
            panic!("plain complete authored changes")
        };
        let child = changes.get(&uri).expect("returned child edits");
        let first = child.first().expect("returned declaration edit");
        let partial_toggle = apply(&toggle, std::slice::from_ref(first));
        let partial_app = apply(
            &app,
            changes
                .get(&fixture.uri("App.vue"))
                .expect("returned parent edits"),
        );
        assert_eq!(
            (&partial_toggle, &partial_app),
            (&missing_call, &golden_app)
        );
        // Match the external law: write both files before either version2 change.
        fixture.write("Toggle.vue", &partial_toggle);
        fixture.write("App.vue", &partial_app);
        let mut partial = Vec::new();
        for (name, text) in [("Toggle.vue", &partial_toggle), ("App.vue", &partial_app)] {
            fixture.change(name, text, 2);
            let diagnostics = fixture.collect(name, &mut custody);
            partial.push(fixture.file(name, text, 2, diagnostics));
        }
        // Preserve the complete version2 observation before independent repair.
        fixture.write("Toggle.vue", &golden_toggle);
        fixture.write("App.vue", &golden_app);
        let mut repaired = Vec::new();
        for (name, text) in [("Toggle.vue", &golden_toggle), ("App.vue", &golden_app)] {
            fixture.change(name, text, 3);
            let diagnostics = fixture.collect(name, &mut custody);
            repaired.push(fixture.file(name, text, 3, diagnostics));
        }
        let actual = json!({"guard":guard,"initial":initial,"rename":rename,
            "partialFiles":partial,"independentRepair":repaired});
        fixture.capture(newline, &expected, &actual, &custody);
        crate::runtime::block_on(async {
            fixture
                .state
                .get_corsa_bridge()
                .await
                .expect("required live bridge")
                .shutdown()
                .await
                .expect("native shutdown");
        });
        assert_eq!(
            actual, expected,
            "complete original partial-application law; custody={custody:#?}"
        );
    }
}
