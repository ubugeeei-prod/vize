//! Physical in-root native assets stay read-only, including through a relay.

use std::path::Path;

use serde_json::json;

use super::{Fixture, byte_offset, range};

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
  "include": ["src/**/*.vue", "src/**/*.ts"]
}"#;
const NO_LIB_CONFIG: &str = r#"{
  "compilerOptions": {
    "noLib": true, "strict": true,
    "moduleResolution": "Bundler", "module": "ESNext", "target": "ESNext"
  },
  "include": ["src/**/*.vue", "runtime-distribution/lib.esnext.d.ts", "runtime-distribution/lib.dom.d.ts"]
}"#;
const CONTROL: &str = include_str!(
    "../../../../tests/_fixtures/differential/lsp/rename-runtime-distribution/8010/input.json"
);
const MUTABLE: &str = include_str!(
    "../../../../tests/_fixtures/differential/lsp/rename-runtime-distribution/8010/Toggle.mutable.vue.txt"
);
const PROBE: &str = include_str!(
    "../../../../tests/_fixtures/differential/lsp/rename-runtime-distribution/8010/Probe.vue.txt"
);
const DECLARED: &str = include_str!(
    "../../../../tests/_fixtures/differential/lsp/rename-runtime-distribution/8010/lib.custom.d.ts.txt"
);
const AFTER_FLIP: &str = include_str!(
    "../../../../tests/_fixtures/differential/lsp/rename-runtime-distribution/8010/Probe.flip.after.vue.txt"
);
const AFTER_ALIAS: &str = include_str!(
    "../../../../tests/_fixtures/differential/lsp/rename-runtime-distribution/8010/Probe.alias.after.vue.txt"
);

#[path = "runtime_distribution/assets.rs"]
mod assets;
pub(super) use assets::RuntimeDistribution;

fn native_property_reference(root: &Path, file: &str, interface: &str) -> serde_json::Value {
    let path = root.join(file);
    let text = std::fs::read_to_string(&path).unwrap();
    let start = text.find(&format!("interface {interface} {{")).unwrap();
    let end = start + text[start..].find('}').unwrap();
    let property = start + text[start..end].find("\"change\":").unwrap() + 1;
    json!({"uri":super::file_uri(&path),"range":range(&text,property,"change")})
}

fn complete_reference_vector(mut references: serde_json::Value) -> Vec<String> {
    let references = references.as_array_mut().expect("complete reference array");
    for location in references.iter_mut() {
        let path = url::Url::parse(location["uri"].as_str().unwrap())
            .unwrap()
            .to_file_path()
            .unwrap();
        location["uri"] = json!(super::file_uri(&path));
    }
    let mut result = references
        .iter()
        .map(serde_json::Value::to_string)
        .collect::<Vec<_>>();
    result.sort();
    result
}

fn unsafe_emit_case(config: &str, crlf: bool, relay: bool) {
    assert_eq!(TOGGLE.matches("const emit").count(), 1);
    let convert = |text: &str| {
        if crlf {
            text.replace('\n', "\r\n")
        } else {
            text.to_owned()
        }
    };
    assert_eq!(MUTABLE, TOGGLE.replacen("const emit", "let emit", 1));
    let control: serde_json::Value = serde_json::from_str(CONTROL).unwrap();
    let config_identity: serde_json::Value = serde_json::from_str(config).unwrap();
    let config_name = if config_identity["compilerOptions"]["noLib"] == true {
        "noLibRootedDistribution"
    } else {
        "default"
    };
    assert_eq!(config_identity, control["configs"][config_name]);
    let toggle = convert(MUTABLE);
    let app = convert(APP);
    // Authored independently before any native query; no returned edit is
    // applied to a runtime file, or used to manufacture the expected result.
    let expected = control["unsafeQuery"]["expectedRename"].clone();
    assert!(expected.is_null());
    let mut fixture = Fixture::with_runtime(
        &[("src/Toggle.vue", &toggle), ("src/App.vue", &app)],
        config,
        true,
        relay,
    );
    fixture.open("src/Toggle.vue", &toggle, 1);
    fixture.open("src/App.vue", &app, 1);
    let shifted = format!("<!-- 😀 -->{}{}", if crlf { "\r\n" } else { "\n" }, toggle);
    for (version, source) in [
        (1, toggle.as_str()),
        (2, shifted.as_str()),
        (3, toggle.as_str()),
    ] {
        if version > 1 {
            fixture.change("src/Toggle.vue", source, version, json!([]));
        }
        let offset = source.find("emit(\"change\"").unwrap() + "emit(\"".len();
        let position = range(source, offset, "change")["start"].clone();
        let distribution = &fixture.runtime_distribution.as_ref().unwrap().copied;
        let mut expected_references = Vec::new();
        for file in ["lib.dom.d.ts", "lib.webworker.d.ts"] {
            for interface in control["readonlyReferences"][file].as_array().unwrap() {
                expected_references.push(native_property_reference(
                    distribution,
                    file,
                    interface.as_str().unwrap(),
                ));
            }
        }
        expected_references.push(
            json!({"uri":fixture.uri("src/Toggle.vue"),"range":range(source,offset,"change")}),
        );
        println!(
            "{}",
            json!({"kind":"prequery-complete-copied-native-control","version":version,"expectedRename":expected,"expectedReferences":expected_references})
        );
        let references = fixture.request(
            "src/Toggle.vue",
            "textDocument/references",
            position.clone(),
            json!({"context":{"includeDeclaration":true}}),
        );
        assert_eq!(
            complete_reference_vector(references),
            complete_reference_vector(json!(expected_references)),
            "full read-only native reference vector must remain available"
        );
        let actual = fixture.request(
            "src/Toggle.vue",
            "textDocument/rename",
            position,
            json!({"newName":"update"}),
        );
        assert_eq!(
            actual, expected,
            "the COMPLETE unsafe transaction must be refused"
        );
        for (name, text) in [
            ("src/Toggle.vue", toggle.as_str()),
            ("src/App.vue", app.as_str()),
        ] {
            assert_eq!(
                std::fs::read(fixture.project.path().join(name)).unwrap(),
                text.as_bytes()
            );
        }
        fixture
            .runtime_distribution
            .as_ref()
            .unwrap()
            .assert_unchanged();
    }
    fixture.shutdown();
}

#[test]
fn physically_copied_native_distribution_refuses_whole_mutable_emit_transaction() {
    for config in [CONFIG, NO_LIB_CONFIG] {
        for crlf in [false, true] {
            unsafe_emit_case(config, crlf, false);
        }
    }
}

#[cfg(unix)]
#[test]
fn outside_root_native_relay_cannot_authorize_its_inside_root_distribution() {
    unsafe_emit_case(NO_LIB_CONFIG, false, true);
}

#[test]
fn copied_distribution_preserves_complete_sfc_and_imported_alias_packets() {
    for crlf in [false, true] {
        let convert = |text: &str| {
            if crlf {
                text.replace('\n', "\r\n")
            } else {
                text.to_owned()
            }
        };
        let probe = convert(PROBE);
        let declared = convert(DECLARED);
        let mut fixture = Fixture::with_runtime(
            &[
                ("src/Probe.vue", &probe),
                ("runtime-distribution/lib.custom.d.ts", &declared),
            ],
            CONFIG,
            true,
            false,
        );
        fixture.open("src/Probe.vue", &probe, 1);
        let mut version = 1;
        for (name, renamed, offsets) in [
            (
                "flip",
                "flipNext",
                vec![
                    probe.find("flip()").unwrap(),
                    probe.find("@click=\"flip").unwrap() + "@click=\"".len(),
                ],
            ),
            (
                "declaredValue",
                "nextDeclaredValue",
                vec![
                    probe.find("declaredValue").unwrap(),
                    probe.find("return declaredValue").unwrap() + "return ".len(),
                    probe.find("{{ declaredValue").unwrap() + 3,
                ],
            ),
        ] {
            let edits = offsets.iter().enumerate().map(|(index, &offset)| {
                json!({"range":range(&probe,offset,name),"newText":if name == "declaredValue" && index == 0 { format!("{name} as {renamed}") } else { renamed.to_owned() }})
            }).collect::<Vec<_>>();
            let expected = json!({"changes":{fixture.uri("src/Probe.vue"):edits}});
            let actual = fixture.request(
                "src/Probe.vue",
                "textDocument/rename",
                range(&probe, offsets[0], name)["start"].clone(),
                json!({"newName":renamed}),
            );
            assert_eq!(
                actual, expected,
                "full positive SFC packet, including alias rewrite"
            );
            let typed: lsp_types::WorkspaceEdit = serde_json::from_value(actual).unwrap();
            let uri: lsp_types::Uri = fixture.uri("src/Probe.vue").parse().unwrap();
            let mut returned = typed.changes.unwrap().remove(&uri).unwrap();
            returned.sort_by_key(|edit| std::cmp::Reverse(edit.range.start));
            let mut applied = probe.clone();
            for edit in returned {
                applied.replace_range(
                    byte_offset(&probe, edit.range.start)..byte_offset(&probe, edit.range.end),
                    &edit.new_text,
                );
            }
            let authored_after = convert(if name == "flip" {
                AFTER_FLIP
            } else {
                AFTER_ALIAS
            });
            assert_eq!(
                applied, authored_after,
                "independently authored application result"
            );
            std::fs::write(fixture.project.path().join("src/Probe.vue"), &applied).unwrap();
            version += 1;
            fixture.change("src/Probe.vue", &applied, version, json!([]));
            std::fs::write(fixture.project.path().join("src/Probe.vue"), &probe).unwrap();
            version += 1;
            fixture.change("src/Probe.vue", &probe, version, json!([]));
            assert_eq!(
                std::fs::read(
                    fixture
                        .project
                        .path()
                        .join("runtime-distribution/lib.custom.d.ts")
                )
                .unwrap(),
                declared.as_bytes()
            );
            fixture
                .runtime_distribution
                .as_ref()
                .unwrap()
                .assert_unchanged();
        }
        fixture.shutdown();
    }
}
