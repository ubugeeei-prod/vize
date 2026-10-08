//! Genuine unowned-emitter refusal and diagnostic failures for partial application.

use serde_json::{Value, json};

use super::{APP, CONFIG, Fixture, TOGGLE, safe_update};

const AMBIGUOUS: &str = include_str!(
    "../../../../tests/_fixtures/differential/lsp/rename-library-refusal/8010/supplemental/ambiguous-bound-emitter/Toggle.vue.txt"
);
const UPDATED_TOGGLE: &str = include_str!(
    "../../../../tests/_fixtures/differential/lsp/event-rename/8010/UpdatedToggle.vue.txt"
);
const UPDATED_APP: &str = include_str!(
    "../../../../tests/_fixtures/differential/lsp/event-rename/8010/UpdatedApp.vue.txt"
);
const MISSING_CALL: &str = include_str!(
    "../../../../tests/_fixtures/differential/lsp/rename-library-refusal/8010/supplemental/partial-application/MissingCallToggle.vue.txt"
);
const CALL_ONLY: &str = include_str!(
    "../../../../tests/_fixtures/differential/lsp/rename-library-refusal/8010/supplemental/partial-application/CallOnlyToggle.vue.txt"
);

#[test]
fn ambiguous_mutable_emitter_refuses_whole_transactions_and_preserves_native_asset_bytes() {
    for newline in ["\n", "\r\n"] {
        let toggle = AMBIGUOUS.replace('\n', newline);
        let app = APP.replace('\n', newline);
        for parent_open in [false, true] {
            let mut fixture = Fixture::with_config(
                &[("src/Toggle.vue", &toggle), ("src/App.vue", &app)],
                CONFIG,
            );
            let library = safe_update::library(&fixture);
            let library_bytes = std::fs::read(&library).unwrap();
            fixture.open("src/Toggle.vue", &toggle, 1);
            if parent_open {
                fixture.open("src/App.vue", &app, 1);
            }
            let mut actual = Vec::new();
            let mut expected = Vec::new();
            for name in ["update", "up\"date", "up'date", "up\\date"] {
                let rename = fixture.request(
                    "src/Toggle.vue",
                    "textDocument/rename",
                    json!({"line":6,"character":9}),
                    json!({"newName":name}),
                );
                actual.push(json!({"newName":name,"rename":rename,
                    "toggleDisk":std::fs::read_to_string(fixture.project.path().join("src/Toggle.vue")).unwrap(),
                    "appDisk":std::fs::read_to_string(fixture.project.path().join("src/App.vue")).unwrap(),
                    "libraryBytesUnchanged":std::fs::read(&library).unwrap()==library_bytes}));
                expected.push(json!({"newName":name,"rename":Value::Null,"toggleDisk":toggle,"appDisk":app,"libraryBytesUnchanged":true}));
            }
            let context = format!(
                "ambiguous mutable event whole refusal, parent_open={parent_open}, newline={newline:?}"
            );
            safe_update::capture(
                &fixture,
                &context,
                &json!({"toggle":toggle,"app":app,"library":library}),
                &json!(expected),
                &json!(actual),
            );
            assert_eq!(actual, expected, "{context}");
            assert_eq!(std::fs::read(&library).unwrap(), library_bytes);
            fixture.shutdown();
        }
    }
}

#[test]
fn missing_emit_and_partial_transaction_application_produce_complete_ts2345_before_repair() {
    for newline in ["\n", "\r\n"] {
        let toggle = TOGGLE.replace('\n', newline);
        let app = APP.replace('\n', newline);
        for (kind, partial_toggle, partial_app, argument, parameter) in [
            ("MissingCall", MISSING_CALL, UPDATED_APP, "change", "update"),
            ("CallOnly", CALL_ONLY, APP, "update", "change"),
        ] {
            let partial_toggle = partial_toggle.replace('\n', newline);
            let partial_app = partial_app.replace('\n', newline);
            let golden_toggle = UPDATED_TOGGLE.replace('\n', newline);
            let golden_app = UPDATED_APP.replace('\n', newline);
            let diagnostics = json!([{
                "range":{"start":{"line":6,"character":7},"end":{"line":6,"character":15}},
                "severity":1,"code":2345,"source":"vize/types",
                "message":format!("Argument of type '\"{argument}\"' is not assignable to parameter of type '\"{parameter}\"'."),
            }]);
            let mut fixture = Fixture::with_config(
                &[("src/Toggle.vue", &toggle), ("src/App.vue", &app)],
                CONFIG,
            );
            let asset = safe_update::library(&fixture);
            let asset_bytes = std::fs::read(&asset).unwrap();
            fixture.open("src/Toggle.vue", &toggle, 1);
            fixture.open("src/App.vue", &app, 1);
            // Both negative authored projects and every complete diagnostic field
            // precede the native query. The service must still return all three edits.
            let expected_edit = safe_update::rename(&fixture, &toggle, &app);
            let expected = json!({"rename":expected_edit,"partialFiles":[
                {"file":"src/Toggle.vue","text":partial_toggle,"disk":partial_toggle,"diagnostics":diagnostics},
                {"file":"src/App.vue","text":partial_app,"disk":partial_app,"diagnostics":[]},
            ],"independentRepair":[
                {"file":"src/Toggle.vue","text":golden_toggle,"disk":golden_toggle,"version":3,"diagnostics":[]},
                {"file":"src/App.vue","text":golden_app,"disk":golden_app,"version":3,"diagnostics":[]},
            ]});
            let rename = fixture.request(
                "src/Toggle.vue",
                "textDocument/rename",
                json!({"line":6,"character":9}),
                json!({"newName":"update"}),
            );
            assert_eq!(
                rename, expected_edit,
                "the provider returns the entire safe transaction"
            );
            // This deliberately incomplete editor application uses returned edits.
            // The complete provider response remains retained and unmodified.
            let child_edits = &rename["changes"][fixture.uri("src/Toggle.vue")];
            let selected_child = json!([child_edits[usize::from(kind == "CallOnly")].clone()]);
            let applied_toggle = safe_update::apply(&toggle, &selected_child);
            let applied_app = if kind == "MissingCall" {
                safe_update::apply(&app, &rename["changes"][fixture.uri("src/App.vue")])
            } else {
                app.clone()
            };
            assert_eq!(
                (&applied_toggle, &applied_app),
                (&partial_toggle, &partial_app)
            );
            for (name, text) in [
                ("src/Toggle.vue", &applied_toggle),
                ("src/App.vue", &applied_app),
            ] {
                std::fs::write(fixture.project.path().join(name), text).unwrap();
            }
            let child_diagnostics = safe_update::publish(
                &mut fixture,
                "src/Toggle.vue",
                &applied_toggle,
                2,
                false,
                diagnostics,
            );
            let app_diagnostics = safe_update::publish(
                &mut fixture,
                "src/App.vue",
                &applied_app,
                2,
                false,
                json!([]),
            );
            let actual_partial = json!([
                {"file":"src/Toggle.vue","text":applied_toggle,"disk":std::fs::read_to_string(fixture.project.path().join("src/Toggle.vue")).unwrap(),"diagnostics":child_diagnostics},
                {"file":"src/App.vue","text":applied_app,"disk":std::fs::read_to_string(fixture.project.path().join("src/App.vue")).unwrap(),"diagnostics":app_diagnostics},
            ]);
            // Retain the complete version2 editor observation first. This
            // separate public CLI route sees these exact partial disk bytes.
            let diagnostic_witness = super::diagnostic_witness::capture(&fixture);
            for (name, text) in [
                ("src/Toggle.vue", &golden_toggle),
                ("src/App.vue", &golden_app),
            ] {
                std::fs::write(fixture.project.path().join(name), text).unwrap();
            }
            let child_diagnostics = safe_update::publish(
                &mut fixture,
                "src/Toggle.vue",
                &golden_toggle,
                3,
                false,
                json!([]),
            );
            let app_diagnostics = safe_update::publish(
                &mut fixture,
                "src/App.vue",
                &golden_app,
                3,
                false,
                json!([]),
            );
            let actual = json!({"rename":rename,"partialFiles":actual_partial,"independentRepair":[
                {"file":"src/Toggle.vue","text":golden_toggle,"disk":std::fs::read_to_string(fixture.project.path().join("src/Toggle.vue")).unwrap(),"version":3,"diagnostics":child_diagnostics},
                {"file":"src/App.vue","text":golden_app,"disk":std::fs::read_to_string(fixture.project.path().join("src/App.vue")).unwrap(),"version":3,"diagnostics":app_diagnostics},
            ]});
            let context = format!("incomplete event application {kind}, newline={newline:?}");
            safe_update::capture_with_witness(
                &fixture,
                &context,
                &json!({"toggle":toggle,"app":app}),
                &expected,
                &actual,
                Some(diagnostic_witness),
            );
            assert_eq!(actual, expected, "{context}");
            assert_eq!(std::fs::read(&asset).unwrap(), asset_bytes);
            fixture.shutdown();
        }
    }
}
