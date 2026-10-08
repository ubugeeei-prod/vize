//! Real original-config transactions, actual application, and independent repair.

use serde_json::{Value, json};

use super::{APP, CONFIG, Fixture, TOGGLE, range};

const UPDATED_TOGGLE: &str = include_str!(
    "../../../../tests/_fixtures/differential/lsp/event-rename/8010/UpdatedToggle.vue.txt"
);
const UPDATED_APP: &str = include_str!(
    "../../../../tests/_fixtures/differential/lsp/event-rename/8010/UpdatedApp.vue.txt"
);

pub(super) fn references(fixture: &Fixture, toggle: &str, app: &str) -> Value {
    json!([
        {"uri":fixture.uri("src/App.vue"),"range":site(app,"change=\"onChange\"",6)},
        {"uri":fixture.uri("src/Toggle.vue"),"range":site(toggle,"change:",6)},
        {"uri":fixture.uri("src/Toggle.vue"),"range":site(toggle,"change\", true",6)},
    ])
}

pub(super) fn rename(fixture: &Fixture, toggle: &str, app: &str) -> Value {
    json!({"changes":{
        fixture.uri("src/App.vue"):[{"range":site(app,"change=\"onChange\"",6),"newText":"update"}],
        fixture.uri("src/Toggle.vue"):[
            {"range":site(toggle,"change:",6),"newText":"update"},
            {"range":site(toggle,"change\", true",6),"newText":"update"},
        ],
    }})
}

fn site(source: &str, needle: &str, length: usize) -> Value {
    let offset = source.find(needle).unwrap();
    range(source, offset, &needle[..length])
}

pub(super) fn library(fixture: &Fixture) -> std::path::PathBuf {
    fixture
        .runtime
        .ancestors()
        .flat_map(|ancestor| {
            [
                ancestor.join("lib.dom.d.ts"),
                ancestor.join("lib/lib.dom.d.ts"),
            ]
        })
        .find(|path| path.is_file())
        .expect("real native DOM asset")
}

pub(super) fn publish(
    fixture: &mut Fixture,
    file: &str,
    text: &str,
    version: i64,
    opening: bool,
    expected_diagnostics: Value,
) -> Value {
    let uri = fixture.uri(file);
    // The unchanged lint+typecheck contract publishes prompt sync feedback,
    // then the complete native answer. Author both packets before didChange.
    let complete = json!({"jsonrpc":"2.0","method":"textDocument/publishDiagnostics",
        "params":{"uri":uri,"version":version,"diagnostics":expected_diagnostics}});
    let expected_packets = if opening {
        vec![complete]
    } else {
        vec![
            json!({"jsonrpc":"2.0","method":"textDocument/publishDiagnostics",
            "params":{"uri":uri,"version":version,"diagnostics":[]}}),
            complete,
        ]
    };
    let Some(first) = expected_packets.first() else {
        panic!("the authored publication sequence must be nonempty")
    };
    fixture.expect_publication(&uri, version, first["params"]["diagnostics"].clone());
    let notification = if opening {
        json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{
            "textDocument":{"uri":uri,"languageId":"vue","version":version,"text":text}
        }})
    } else {
        json!({"jsonrpc":"2.0","method":"textDocument/didChange","params":{
            "textDocument":{"uri":uri,"version":version},"contentChanges":[{"text":text}]
        }})
    };
    fixture.lsp.send(notification);
    let actual_packets = fixture.recv_publication_sequence(&uri, version, &expected_packets);
    fixture
        .publication_sequences
        .push(json!({"file":file,"version":version,"opening":opening,
        "expected":expected_packets,"actual":actual_packets}));
    assert_eq!(
        actual_packets, expected_packets,
        "whole prompt/native publication sequence"
    );
    let Some(complete) = actual_packets.last() else {
        panic!("the complete native publication must be retained")
    };
    complete["params"]["diagnostics"].clone()
}

pub(super) fn apply(source: &str, entries: &Value) -> String {
    apply_checked(source, entries).unwrap()
}

pub(super) fn apply_checked(source: &str, entries: &Value) -> Result<String, String> {
    let mut edits: Vec<lsp_types::TextEdit> =
        serde_json::from_value(entries.clone()).map_err(|error| error.to_string())?;
    edits.sort_by_key(|edit| std::cmp::Reverse(edit.range.start));
    let mut applied = source.to_owned();
    for edit in edits {
        let start = super::super::byte_offset(source, edit.range.start);
        let end = super::super::byte_offset(source, edit.range.end);
        if start >= end {
            return Err(format!("incomplete native edit range: {:?}", edit.range));
        }
        applied.replace_range(start..end, &edit.new_text);
    }
    Ok(applied)
}

fn files(toggle: &str, app: &str, version: Option<i64>) -> Value {
    let rows: Vec<_> = [("src/Toggle.vue", toggle), ("src/App.vue", app)]
        .into_iter()
        .map(|(name, text)| {
            let mut row = json!({"file":name,"text":text,"disk":text,"diagnostics":[]});
            if let Some(version) = version {
                row["version"] = json!(version);
            }
            row
        })
        .collect();
    json!(rows)
}

#[test]
fn original_config_safe_updates_apply_all_owned_sites_and_validate_independent_repairs() {
    for newline in ["\n", "\r\n"] {
        let original = TOGGLE.replace('\n', newline);
        let app = APP.replace('\n', newline);
        let shifted = format!("<!-- 😀 -->{newline}{original}");
        for parent_open in [false, true] {
            for (snapshot, source) in [
                ("Original", original.as_str()),
                ("UnsavedEmoji", shifted.as_str()),
                ("Restored", original.as_str()),
            ] {
                let mut fixture = Fixture::with_config(
                    &[("src/Toggle.vue", &original), ("src/App.vue", &app)],
                    CONFIG,
                );
                let asset = library(&fixture);
                let asset_bytes = std::fs::read(&asset).unwrap();
                fixture.open("src/Toggle.vue", source, 1);
                if parent_open {
                    fixture.open("src/App.vue", &app, 1);
                }
                let repaired = UPDATED_TOGGLE.replace('\n', newline);
                let golden_toggle = if snapshot == "UnsavedEmoji" {
                    format!("<!-- 😀 -->{newline}{repaired}")
                } else {
                    repaired
                };
                let golden_app = UPDATED_APP.replace('\n', newline);
                // Full literal source, expected geometry, and goldens precede queries.
                let expected = json!({"references":references(&fixture,source,&app),"rename":rename(&fixture,source,&app),"applicationErrors":[],
                    "files":files(&golden_toggle,&golden_app,None),"independentRepair":files(&golden_toggle,&golden_app,Some(3))});
                let position = site(source, "hange\", true", 1)["start"].clone();
                let edit = fixture.request(
                    "src/Toggle.vue",
                    "textDocument/rename",
                    position.clone(),
                    json!({"newName":"update"}),
                );
                let references = fixture.request(
                    "src/Toggle.vue",
                    "textDocument/references",
                    position,
                    json!({"context":{"includeDeclaration":true}}),
                );
                let mut application_errors = Vec::new();
                let mut apply_file = |file: &str, text: &str| match apply_checked(
                    text,
                    &edit["changes"][fixture.uri(file)],
                ) {
                    Ok(applied) => applied,
                    Err(error) => {
                        application_errors.push(json!({"file":file,"error":error}));
                        text.to_owned()
                    }
                };
                let applied_toggle = apply_file("src/Toggle.vue", source);
                let applied_app = apply_file("src/App.vue", &app);
                let applied = [
                    ("src/Toggle.vue", &applied_toggle),
                    ("src/App.vue", &applied_app),
                ];
                for (name, text) in applied {
                    std::fs::write(fixture.project.path().join(name), text).unwrap();
                }
                let mut actual_files = Vec::new();
                for (name, text) in applied {
                    let diagnostics = publish(
                        &mut fixture,
                        name,
                        text,
                        2,
                        name == "src/App.vue" && !parent_open,
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
                    let diagnostics = publish(&mut fixture, name, text, 3, false, json!([]));
                    independent.push(json!({"file":name,"text":text,"disk":std::fs::read_to_string(fixture.project.path().join(name)).unwrap(),"version":3,"diagnostics":diagnostics}));
                }
                let actual = json!({"references":references,"rename":edit,"applicationErrors":application_errors,"files":actual_files,"independentRepair":independent});
                let context = format!(
                    "original strict event safe update {snapshot}, parent_open={parent_open}, newline={newline:?}"
                );
                capture(
                    &fixture,
                    &context,
                    &json!({"toggleBuffer":source,"toggleDisk":original,"app":app}),
                    &expected,
                    &actual,
                );
                assert_eq!(actual, expected, "{context}");
                assert_eq!(
                    std::fs::read(&asset).unwrap(),
                    asset_bytes,
                    "native asset bytes unchanged"
                );
                fixture.shutdown();
            }
        }
    }
}

pub(super) fn capture(
    fixture: &Fixture,
    context: &str,
    inputs: &Value,
    expected: &Value,
    actual: &Value,
) {
    capture_with_witness(fixture, context, inputs, expected, actual, None);
}

pub(super) fn capture_with_witness(
    fixture: &Fixture,
    context: &str,
    inputs: &Value,
    expected: &Value,
    actual: &Value,
    diagnostic_witness: Option<Value>,
) {
    let root = std::env::var_os("VIZE_TEST_FIX_HISTORY_CAPTURE_DIR")
        .map(std::path::PathBuf::from)
        .or_else(|| {
            let profile = std::env::var_os("NEXTEST_PROFILE")?;
            Some(
                std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .parent()?
                    .parent()?
                    .join("target/nextest")
                    .join(profile),
            )
        });
    if let Some(root) = root {
        let root = root.join("event-library-authority-transactions");
        std::fs::create_dir_all(&root).unwrap();
        let packet = json!({"context":context,"sourceSha":std::env::var("SOURCE_SHA").ok(),"inputs":inputs,"expected":expected,"actual":actual,
            "diagnosticWitness":diagnostic_witness,"publicationSequences":fixture.publication_sequences,
            "cliBinary":env!("CARGO_BIN_EXE_vize"),"requireTsgo":std::env::var("VIZE_TEST_REQUIRE_TSGO").ok(),"disableTsgo":std::env::var("VIZE_TEST_DISABLE_TSGO").ok(),
            "runtime":fixture.runtime,"tsconfig":std::fs::read_to_string(fixture.project.path().join("tsconfig.json")).unwrap(),
            "vizeConfig":std::fs::read_to_string(fixture.project.path().join("vize.config.json")).unwrap()});
        std::fs::write(
            root.join(fixture.project.path().file_name().unwrap())
                .with_extension("json"),
            serde_json::to_vec_pretty(&packet).unwrap(),
        )
        .unwrap();
    }
}
