//! Exact authored checks retain a physical native process, never claim Program identity.

use super::{assert_diagnosing_options, file};
use corsa::runtime::block_on;
use std::path::Path;
use vize_canon::{CorsaBridge, CorsaBridgeConfig};
use vize_l0::{Allocator, String};
use vize_l1::embed::Lang;

#[path = "../support/original_diagnosing_process.rs"]
mod control;

fn complete_check(
    bridge: &CorsaBridge,
    root: &Path,
    expected: &serde_json::Value,
) -> (String, serde_json::Value) {
    let path = root.join("source.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let arena = Allocator::default();
    let original = file(&arena, &source, Lang::Ts);
    assert!(original.is_complete(), "{:?}", original.issues());
    let result = block_on(bridge.check_original_program(&original, &path)).unwrap();
    assert_diagnosing_options(&result);
    assert!(std::ptr::eq(result.projection().file(), &original));
    assert_eq!(serde_json::to_value(result.report()).unwrap(), *expected);
    let spans: Vec<_> = result
        .authored_spans()
        .iter()
        .map(|span| {
            let span = span.as_ref().unwrap();
            source.get(span.start as usize..span.end as usize)
        })
        .collect();
    let authored: Vec<_> = expected["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|_| Some("return"))
        .collect();
    assert_eq!(spans, authored);
    assert_eq!(std::fs::read(&path).unwrap(), source.as_bytes());
    assert_eq!(result.source_path(), path.canonicalize().unwrap());
    assert_eq!(
        result.diagnostic_configuration_path(),
        root.join("tsconfig.json").canonicalize().unwrap()
    );
    let expected_membership =
        ["shared.js", "source.ts"].map(|name| root.join(name).canonicalize().unwrap());
    let membership: Vec<_> = result
        .configuration()
        .file_names
        .iter()
        .map(|name| Path::new(name))
        .collect();
    assert_eq!(membership, expected_membership);
    (
        result
            .diagnosing_configuration()
            .session()
            .session_id
            .as_str()
            .into(),
        result.configuration().options.clone(),
    )
}

#[test]
fn retained_native_process_preserves_whole_noop_leaf_shared_inverse_and_shutdown() {
    if !control::enabled() {
        return;
    }
    let fixture = control::fixture();
    let root = tempfile::TempDir::new().unwrap();
    control::write_fixture(root.path(), &fixture);
    let native = control::runtime();
    let bridge = CorsaBridge::with_config(CorsaBridgeConfig {
        corsa_path: Some(native.clone()),
        working_dir: Some(root.path().to_path_buf()),
        ..Default::default()
    });
    let config = std::fs::read(root.path().join("tsconfig.json")).unwrap();
    block_on(bridge.spawn()).unwrap();
    let session = complete_check(&bridge, root.path(), &fixture["expected"]);
    let life = control::native_lsp(root.path(), &native);
    for _ in 0..5 {
        assert_eq!(
            complete_check(&bridge, root.path(), &fixture["expected"]),
            session
        );
        assert_eq!(control::native_lsp(root.path(), &native), life);
    }
    for (path, edited, original) in [
        ("source.ts", "leafEdit", "source"),
        ("shared.js", "sharedEdit", "shared"),
    ] {
        std::fs::write(root.path().join(path), fixture[edited].as_str().unwrap()).unwrap();
        assert_eq!(
            complete_check(&bridge, root.path(), &fixture["editedExpected"]),
            session
        );
        assert_eq!(control::native_lsp(root.path(), &native), life);
        std::fs::write(root.path().join(path), fixture[original].as_str().unwrap()).unwrap();
        assert_eq!(
            complete_check(&bridge, root.path(), &fixture["expected"]),
            session
        );
        assert_eq!(control::native_lsp(root.path(), &native), life);
    }
    assert_eq!(
        std::fs::read(root.path().join("tsconfig.json")).unwrap(),
        config
    );
    assert_eq!(
        std::fs::read_to_string(root.path().join("source.ts")).unwrap(),
        fixture["source"].as_str().unwrap()
    );
    assert_eq!(
        std::fs::read_to_string(root.path().join("shared.js")).unwrap(),
        fixture["shared"].as_str().unwrap()
    );
    // Config bytes are a real cache input, even when the complete answer stays
    // the same. Retire the old owner before serving the revised configuration.
    let mut revised = fixture["config"].clone();
    revised["compilerOptions"]["noUnusedLocals"] = serde_json::json!(true);
    std::fs::write(
        root.path().join("tsconfig.json"),
        serde_json::to_vec(&revised).unwrap(),
    )
    .unwrap();
    let revised_session = complete_check(&bridge, root.path(), &fixture["expected"]);
    let revised_life = control::native_lsp(root.path(), &native);
    // Server-generated API IDs need the physical process to identify an owner.
    assert_ne!((&revised_session.0, &revised_life), (&session.0, &life));
    let mut revised_options = session.1.clone();
    revised_options["noUnusedLocals"] = serde_json::json!(true);
    assert_eq!(revised_session.1, revised_options);
    assert_ne!(
        (revised_life.pid, revised_life.birth),
        (life.pid, life.birth)
    );
    control::assert_reaped(&life);
    std::fs::write(root.path().join("tsconfig.json"), &config).unwrap();
    let inverse_session = complete_check(&bridge, root.path(), &fixture["expected"]);
    let inverse_life = control::native_lsp(root.path(), &native);
    assert_ne!(
        (&inverse_session.0, &inverse_life),
        (&revised_session.0, &revised_life)
    );
    assert_eq!(inverse_session.1, session.1);
    assert_ne!(
        (inverse_life.pid, inverse_life.birth),
        (revised_life.pid, revised_life.birth)
    );
    control::assert_reaped(&revised_life);
    assert_eq!(
        std::fs::read(root.path().join("tsconfig.json")).unwrap(),
        config
    );
    block_on(bridge.shutdown()).unwrap();
    control::assert_reaped(&inverse_life);
}

#[test]
fn dropping_original_bridge_reaps_its_physical_native_diagnosing_process() {
    if !control::enabled() {
        return;
    }
    let fixture = control::fixture();
    let root = tempfile::TempDir::new().unwrap();
    control::write_fixture(root.path(), &fixture);
    let native = control::runtime();
    let bridge = CorsaBridge::with_config(CorsaBridgeConfig {
        corsa_path: Some(native.clone()),
        working_dir: Some(root.path().to_path_buf()),
        ..Default::default()
    });
    block_on(bridge.spawn()).unwrap();
    complete_check(&bridge, root.path(), &fixture["expected"]);
    let life = control::native_lsp(root.path(), &native);
    drop(bridge);
    control::assert_reaped(&life);
}
