//! Complete #8010 originals, atomic owned rename, and unchanged authored/lib bytes.

use serde_json::{Value, json};

use super::{Fixture, range};

#[path = "cold_collision.rs"]
mod cold_collision;
#[path = "safe_update.rs"]
mod safe_update;
#[path = "safety_controls.rs"]
mod safety_controls;

const TOGGLE: &str = include_str!(
    "../../../../tests/_fixtures/differential/lsp/rename-library-refusal/8010/Toggle.vue.txt"
);
const APP: &str = include_str!(
    "../../../../tests/_fixtures/differential/lsp/rename-library-refusal/8010/App.vue.txt"
);
const CONFIG: &str = r#"{
  "compilerOptions": {
    "lib": ["ESNext", "DOM"], "strict": true,
    "moduleResolution": "Bundler", "module": "ESNext", "target": "ESNext"
  },
  "include": ["src/**/*.vue"]
}"#;

#[test]
fn original_emit_string_keeps_atomic_owned_updates_and_unsafe_name_refusals_without_file_writes() {
    for crlf in [false, true] {
        for parent_open in [false, true] {
            let convert = |text: &str| {
                if crlf {
                    text.replace('\n', "\r\n")
                } else {
                    text.to_owned()
                }
            };
            let toggle = convert(TOGGLE);
            let app = convert(APP);
            let mut fixture = Fixture::with_config(
                &[("src/Toggle.vue", &toggle), ("src/App.vue", &app)],
                CONFIG,
            );
            let library = fixture
                .runtime
                .ancestors()
                .flat_map(|ancestor| {
                    [
                        ancestor.join("lib.dom.d.ts"),
                        ancestor.join("lib/lib.dom.d.ts"),
                    ]
                })
                .find(|path| path.is_file())
                .expect("the actual native runtime must expose its bundled DOM library");
            let library_bytes = std::fs::read(&library).unwrap();
            println!(
                "actual native runtime/library: {:?} {:?}",
                fixture.runtime, library
            );
            fixture.open("src/Toggle.vue", &toggle, 1);
            if parent_open {
                fixture.open("src/App.vue", &app, 1);
            }
            let shifted = format!("<!-- 😀 -->{}{}", if crlf { "\r\n" } else { "\n" }, toggle);
            for (version, source) in [
                (1, toggle.as_str()),
                (2, shifted.as_str()),
                (3, toggle.as_str()),
            ] {
                if version > 1 {
                    fixture.change("src/Toggle.vue", source, version, json!([]));
                }
                let position = if version == 2 {
                    json!({"line":7,"character":9})
                } else {
                    json!({"line":6,"character":9})
                };
                for new_name in ["update", "up\"date", "up'date", "up\\date"] {
                    let actual = fixture.request(
                        "src/Toggle.vue",
                        "textDocument/rename",
                        position.clone(),
                        json!({"newName":new_name}),
                    );
                    let _: Option<lsp_types::WorkspaceEdit> =
                        serde_json::from_value(actual.clone()).unwrap();
                    if new_name == "update" {
                        let references = fixture.request(
                            "src/Toggle.vue",
                            "textDocument/references",
                            position.clone(),
                            json!({"context":{"includeDeclaration":true}}),
                        );
                        assert_eq!(
                            references,
                            safe_update::references(&fixture, source, &app),
                            "all three complete owned sites, without library references"
                        );
                        assert_eq!(
                            actual,
                            safe_update::rename(&fixture, source, &app),
                            "the exact atomic transaction has no library or generated target"
                        );
                    } else {
                        assert_eq!(actual, Value::Null, "unsafe names refuse the whole rename");
                    }
                    assert_eq!(
                        std::fs::read(fixture.project.path().join("src/Toggle.vue")).unwrap(),
                        toggle.as_bytes()
                    );
                    assert_eq!(
                        std::fs::read(fixture.project.path().join("src/App.vue")).unwrap(),
                        app.as_bytes()
                    );
                    assert_eq!(std::fs::read(&library).unwrap(), library_bytes);
                }
                let edits = ["function flip()", "@click=\"flip\""].map(|selector| {
                    let offset = source.find(selector).unwrap() + selector.find("flip").unwrap();
                    json!({"range":range(source,offset,"flip"),"newText":"flipNext"})
                });
                let offset = source.find("function flip()").unwrap() + "function ".len();
                let query = range(source, offset, "flip")["start"].clone();
                let actual = fixture.request(
                    "src/Toggle.vue",
                    "textDocument/rename",
                    query,
                    json!({"newName":"flipNext"}),
                );
                let expected = json!({"changes":{fixture.uri("src/Toggle.vue"):edits}});
                let edit: lsp_types::WorkspaceEdit =
                    serde_json::from_value(actual.clone()).unwrap();
                assert_eq!(
                    actual, expected,
                    "ordinary local binding rename remains complete"
                );
                let mut applied = source.to_owned();
                let mut edits = edit
                    .changes
                    .unwrap()
                    .remove(&lsp_process_uri(&fixture))
                    .unwrap();
                edits.sort_by_key(|edit| std::cmp::Reverse(edit.range.start));
                for edit in edits {
                    let start = super::byte_offset(source, edit.range.start);
                    let end = super::byte_offset(source, edit.range.end);
                    applied.replace_range(start..end, &edit.new_text);
                }
                let expected_source = source
                    .replace("function flip()", "function flipNext()")
                    .replace("@click=\"flip\"", "@click=\"flipNext\"");
                assert_eq!(applied, expected_source);
            }
            fixture.shutdown();
        }
    }
}

fn lsp_process_uri(fixture: &Fixture) -> lsp_types::Uri {
    fixture.uri("src/Toggle.vue").parse().unwrap()
}
