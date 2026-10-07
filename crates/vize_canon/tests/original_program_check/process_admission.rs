//! Admission is independent of native process availability and runtime credit.

use super::{Allocator, Lang, OriginalProgramError, ProjectionError, block_on, file};
use vize_canon::{CorsaBridge, CorsaBridgeConfig};
use vize_l4::targets::ts::project_program;

fn fixture() -> serde_json::Value {
    serde_json::from_str(include_str!(
        "../../../../tests/_fixtures/differential/typechecker/original-diagnosing-process-7698/input.json"
    ))
    .unwrap()
}

#[test]
fn original_exported_identifier_initializer_retains_whole_typed_incomplete_refusal() {
    let fixture = fixture();
    let root = tempfile::TempDir::new().unwrap();
    let source = fixture["source"].as_str().unwrap();
    let shared = fixture["shared"].as_str().unwrap();
    let config = serde_json::to_vec(&fixture["config"]).unwrap();
    std::fs::write(root.path().join("source.ts"), source).unwrap();
    std::fs::write(root.path().join("shared.js"), shared).unwrap();
    std::fs::write(root.path().join("tsconfig.json"), &config).unwrap();
    let arena = Allocator::default();
    let original = file(&arena, source, Lang::Ts);
    let issues: Vec<_> = original
        .issues()
        .iter()
        .map(|issue| {
            serde_json::json!({
                "unit": issue.unit.index(),
                "start": issue.span.start,
                "end": issue.span.end,
                "kind": vize_l0::cstr!("{:?}", issue.kind)
            })
        })
        .collect();
    assert_eq!(
        serde_json::json!(issues),
        fixture["originalRefusal"]["issues"]
    );
    assert!(!original.is_complete());
    assert!(core::ptr::eq(original.artifact().source(), source));
    assert_eq!(fixture["originalRefusal"]["projection"], "IncompleteFile");
    let bridge = CorsaBridge::with_config(CorsaBridgeConfig {
        corsa_path: Some(root.path().join("absent-native-backend")),
        working_dir: Some(root.path().to_path_buf()),
        ..Default::default()
    });
    assert!(matches!(
        block_on(bridge.check_original_program(&original, &root.path().join("source.ts"))),
        Err(OriginalProgramError::Projection(
            ProjectionError::IncompleteFile
        ))
    ));
    assert!(!bridge.is_initialized());
    assert_eq!(
        std::fs::read(root.path().join("source.ts")).unwrap(),
        source.as_bytes()
    );
    assert_eq!(
        std::fs::read(root.path().join("shared.js")).unwrap(),
        shared.as_bytes()
    );
    assert_eq!(
        std::fs::read(root.path().join("tsconfig.json")).unwrap(),
        config
    );
}

#[test]
fn admitted_process_fixture_and_leaf_are_complete_genuine_modules() {
    let fixture = fixture();
    let admitted = &fixture["admitted"];
    for key in ["source", "leafEdit"] {
        let arena = Allocator::default();
        let source = admitted[key].as_str().unwrap();
        let original = file(&arena, source, Lang::Ts);
        assert!(original.is_complete(), "{key}: {:?}", original.issues());
        assert!(original.issues().is_empty());
        let projection = project_program(&original).unwrap();
        assert!(core::ptr::eq(projection.file(), &original));
        assert!(core::ptr::eq(original.artifact().source(), source));
        assert_eq!(original.artifact().source(), source);
    }
}
