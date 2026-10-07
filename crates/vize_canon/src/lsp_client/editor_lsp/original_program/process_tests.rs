//! Actual owner retirement on the existing SessionMap idle and mode boundaries.

use super::super::CorsaProjectClient;
use crate::corsa_session_cache::{CorsaSessionKey, DEFAULT_SESSION_IDLE, SessionMap};
use std::{path::Path, time::Instant};

#[path = "../../../../tests/support/original_diagnosing_process.rs"]
mod control;

fn diagnose(client: &mut CorsaProjectClient, root: &Path, fixture: &serde_json::Value) {
    let path = root.join("source.ts").canonicalize().unwrap();
    let config = root.join("tsconfig.json").canonicalize().unwrap();
    let (report, effective, selected, custody) = client
        .original_program_diagnostics(
            crate::file_uri::path_to_file_uri(&path).as_str(),
            fixture["source"].as_str().unwrap(),
            &config,
            true,
        )
        .unwrap();
    assert_eq!(serde_json::to_value(report).unwrap(), fixture["expected"]);
    assert_eq!(selected, config);
    assert!(
        effective
            .file_names
            .iter()
            .any(|name| Path::new(name) == path)
    );
    for observed in [custody.before(), custody.after()] {
        assert_eq!(observed.project().compiler_options, effective.options);
        assert_eq!(Path::new(&observed.project().config_file_name), config);
    }
    assert_ne!(custody.before().handle(), custody.after().handle());
}

#[test]
fn native_original_idle_reap_retires_process_and_next_owner_checks_whole_answer() {
    if !control::enabled() {
        return;
    }
    let fixture = control::fixture();
    let root = tempfile::TempDir::new().unwrap();
    control::write_fixture(root.path(), &fixture);
    let native = control::runtime();
    let key = CorsaSessionKey::new(root.path().join("tsconfig.json"), "idle-native-control", "");
    let mut sessions = SessionMap::new(DEFAULT_SESSION_IDLE);
    let now = Instant::now();
    let client = CorsaProjectClient::new_for_workspace(native.to_str(), root.path()).unwrap();
    diagnose(
        sessions.insert_spawned(key.clone(), client, now),
        root.path(),
        &fixture,
    );
    let first = control::native_lsp(root.path(), &native);
    sessions.reap(now + DEFAULT_SESSION_IDLE);
    control::assert_reaped(&first);
    assert!(sessions.get_mut(&key, now + DEFAULT_SESSION_IDLE).is_none());
    let client = CorsaProjectClient::new_for_workspace(native.to_str(), root.path()).unwrap();
    diagnose(
        sessions.insert_spawned(key, client, now + DEFAULT_SESSION_IDLE),
        root.path(),
        &fixture,
    );
    let next = control::native_lsp(root.path(), &native);
    assert_ne!((next.pid, next.birth), (first.pid, first.birth));
    drop(sessions);
    control::assert_reaped(&next);
}

#[test]
fn native_original_materialized_mode_retires_diagnosing_process_before_refusal() {
    if !control::enabled() {
        return;
    }
    let fixture = control::fixture();
    let root = tempfile::TempDir::new().unwrap();
    control::write_fixture(root.path(), &fixture);
    let native = control::runtime();
    let mut client = CorsaProjectClient::new_for_workspace(native.to_str(), root.path()).unwrap();
    diagnose(&mut client, root.path(), &fixture);
    let life = control::native_lsp(root.path(), &native);
    client.activate_materialized_project_session().unwrap();
    control::assert_reaped(&life);
    assert!(client.original_diagnosing_session.is_none());
    assert!(matches!(
        client.original_program_diagnostics(
            crate::file_uri::path_to_file_uri(&root.path().join("source.ts")).as_str(),
            fixture["source"].as_str().unwrap(),
            &root.path().join("tsconfig.json"),
            true,
        ),
        Err(crate::OriginalProgramError::Backend(_))
    ));
    client.shutdown().unwrap();
}
