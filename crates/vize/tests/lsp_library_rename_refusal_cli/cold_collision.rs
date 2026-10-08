//! A literal public event keeps its identity beside a same-name value binding.

use serde_json::{Value, json};

use super::{APP, CONFIG, Fixture, range, safe_update};

const TOGGLE: &str = include_str!(
    "../../../../tests/_fixtures/differential/lsp/rename-library-refusal/8010/supplemental/cold-value-collision/Toggle.vue.txt"
);
const UPDATED_TOGGLE: &str = include_str!(
    "../../../../tests/_fixtures/differential/lsp/rename-library-refusal/8010/supplemental/cold-value-collision/UpdatedToggle.vue.txt"
);
const UPDATED_APP: &str = include_str!(
    "../../../../tests/_fixtures/differential/lsp/event-rename/8010/UpdatedApp.vue.txt"
);

fn site(source: &str, needle: &str) -> Value {
    range(source, source.find(needle).unwrap(), "change")
}

#[test]
fn cold_literal_event_preserves_the_same_name_value_and_renames_every_owned_site() {
    for newline in ["\n", "\r\n"] {
        let toggle = TOGGLE.replace('\n', newline);
        let app = APP.replace('\n', newline);
        let golden_toggle = UPDATED_TOGGLE.replace('\n', newline);
        let golden_app = UPDATED_APP.replace('\n', newline);
        let mut fixture = Fixture::with_config(
            &[("src/Toggle.vue", &toggle), ("src/App.vue", &app)],
            CONFIG,
        );
        let asset = safe_update::library(&fixture);
        let asset_bytes = std::fs::read(&asset).unwrap();
        fixture.open("src/Toggle.vue", &toggle, 1);
        let declaration = site(&toggle, "change:");
        let call = site(&toggle, "change\", change");
        let listener = site(&app, "change=\"onChange\"");
        // Complete literal sources, all endpoints, and independent repaired
        // files are authored before the first native rename query.
        let expected = json!({"references":[
            {"uri":fixture.uri("src/App.vue"),"range":listener},
            {"uri":fixture.uri("src/Toggle.vue"),"range":declaration},
            {"uri":fixture.uri("src/Toggle.vue"),"range":call},
        ],"rename":{"changes":{
            fixture.uri("src/App.vue"):[{"range":listener,"newText":"update"}],
            fixture.uri("src/Toggle.vue"):[
                {"range":declaration,"newText":"update"},
                {"range":call,"newText":"update"},
            ],
        }},"applicationErrors":[],"files":[
            {"file":"src/Toggle.vue","text":golden_toggle,"disk":golden_toggle,"diagnostics":[]},
            {"file":"src/App.vue","text":golden_app,"disk":golden_app,"diagnostics":[]},
        ],"independentRepair":[
            {"file":"src/Toggle.vue","text":golden_toggle,"disk":golden_toggle,"version":3,"diagnostics":[]},
            {"file":"src/App.vue","text":golden_app,"disk":golden_app,"version":3,"diagnostics":[]},
        ]});
        // App is unopened. Rename must materialize its configured consumer
        // before any reference request can warm the workspace.
        let rename = fixture.request(
            "src/Toggle.vue",
            "textDocument/rename",
            call["start"].clone(),
            json!({"newName":"update"}),
        );
        let references = fixture.request(
            "src/Toggle.vue",
            "textDocument/references",
            call["start"].clone(),
            json!({"context":{"includeDeclaration":true}}),
        );
        let mut application_errors = Vec::new();
        let mut applied = Vec::new();
        for (name, text) in [("src/Toggle.vue", &toggle), ("src/App.vue", &app)] {
            let text = match safe_update::apply_checked(text, &rename["changes"][fixture.uri(name)])
            {
                Ok(applied) => applied,
                Err(error) => {
                    application_errors.push(json!({"file":name,"error":error}));
                    text.clone()
                }
            };
            std::fs::write(fixture.project.path().join(name), &text).unwrap();
            applied.push((name, text));
        }
        let mut actual_files = Vec::new();
        for (name, text) in &applied {
            let diagnostics = safe_update::publish(
                &mut fixture,
                name,
                text,
                2,
                *name == "src/App.vue",
                json!([]),
            );
            actual_files.push(json!({"file":name,"text":text,"disk":std::fs::read_to_string(fixture.project.path().join(name)).unwrap(),"diagnostics":diagnostics}));
        }
        let goldens = [
            ("src/Toggle.vue", &golden_toggle),
            ("src/App.vue", &golden_app),
        ];
        for (name, text) in goldens {
            std::fs::write(fixture.project.path().join(name), text).unwrap();
        }
        let mut independent = Vec::new();
        for (name, text) in goldens {
            let diagnostics = safe_update::publish(&mut fixture, name, text, 3, false, json!([]));
            independent.push(json!({"file":name,"text":text,"disk":std::fs::read_to_string(fixture.project.path().join(name)).unwrap(),"version":3,"diagnostics":diagnostics}));
        }
        let actual = json!({"references":references,"rename":rename,"applicationErrors":application_errors,"files":actual_files,"independentRepair":independent});
        let context = format!("cold event/value collision, unopened parent, newline={newline:?}");
        safe_update::capture(
            &fixture,
            &context,
            &json!({"toggle":toggle,"app":app}),
            &expected,
            &actual,
        );
        assert_eq!(actual, expected, "{context}");
        assert_eq!(std::fs::read(&asset).unwrap(), asset_bytes);
        fixture.shutdown();
    }
}
