use super::super::{CorsaResolveError, discover_in_walk, platform_suffix};
use super::{resolve, write_file};
use std::{fs, path::Path};

#[test]
fn runtime_selection_fixture_covers_declared_graphs_and_overrides() {
    // These files are selection markers, never executable/TypeScript/API evidence.
    let fixture: serde_json::Value = serde_json::from_str(
        &include_str!("../../../../../tests/_fixtures/runtime/native-runtime-10694/selection.json")
            .replace("$PLATFORM", platform_suffix()),
    )
    .unwrap();
    assert_eq!(
        fixture.get("schema").unwrap(),
        "vize.native-runtime-selection"
    );
    assert_eq!(fixture.get("version").unwrap(), 1);
    let mut graph_ids = Vec::new();
    for case in fixture.get("graphCases").unwrap().as_array().unwrap() {
        let root = tempfile::tempdir().unwrap();
        graph_ids.push(case.get("id").unwrap().as_str().unwrap());
        for (name, manifest) in case.get("manifests").unwrap().as_object().unwrap() {
            let file = root.path().join(name);
            fs::create_dir_all(file.parent().unwrap()).unwrap();
            fs::write(file, serde_json::to_vec(manifest).unwrap()).unwrap();
        }
        for file in case.get("files").unwrap().as_array().unwrap() {
            write_file(&root.path().join(file.as_str().unwrap()));
        }
        let expected = case
            .get("expected")
            .unwrap()
            .as_str()
            .map(|file| root.path().join(file).canonicalize().unwrap());
        let actual = discover_in_walk(&[root.path().to_path_buf()], false)
            .map(|file| file.canonicalize().unwrap());
        assert_eq!(actual, expected, "graph {}", case.get("id").unwrap());
    }
    assert_eq!(
        graph_ids,
        [
            "canonical-beta",
            "typescript-six-beta-alias",
            "stale-bundle-refused",
            "unlinked-store-refused",
            "platform-mismatch-refused"
        ]
    );

    let mut override_ids = Vec::new();
    for case in fixture.get("overrideCases").unwrap().as_array().unwrap() {
        let root = tempfile::tempdir().unwrap();
        let source = case.get("source").unwrap().as_str().unwrap();
        let requested = Path::new(case.get("requested").unwrap().as_str().unwrap());
        let selected = root.path().join(requested);
        override_ids.push(case.get("id").unwrap().as_str().unwrap());
        if case.get("present").unwrap().as_bool().unwrap() {
            write_file(&selected);
        }
        let actual = match source {
            "configuration" => resolve(Some(requested), Some(root.path()), &[]),
            "CORSA_PATH" => resolve(None, Some(root.path()), &[("CORSA_PATH", requested)]),
            _ => panic!("unknown override source"),
        };
        if case.get("present").unwrap().as_bool().unwrap() {
            assert_eq!(actual.unwrap(), selected.canonicalize().unwrap());
        } else {
            let expected_source = if source == "configuration" {
                "configuration"
            } else {
                "CORSA_PATH"
            };
            assert_eq!(
                actual.unwrap_err(),
                CorsaResolveError::ExplicitNotFound {
                    source: expected_source,
                    path: selected,
                }
            );
        }
    }
    assert_eq!(
        override_ids,
        [
            "explicit-valid",
            "explicit-missing",
            "environment-valid",
            "environment-missing"
        ]
    );
}
