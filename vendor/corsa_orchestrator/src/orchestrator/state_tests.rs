use super::*;
use lsp_types::{Position, Range};

#[test]
fn state_applies_document_changes() {
    let mut state = ReplicatedState::default();
    let document =
        VirtualDocument::in_memory("cluster", "/main.ts", "typescript", "let value = 1;").unwrap();
    let uri = document.key();
    state
        .apply(&ReplicatedCommand::PutDocument {
            document: document.clone(),
        })
        .unwrap();
    state
        .apply(&ReplicatedCommand::ApplyDocumentChange {
            uri: uri.clone(),
            changes: [VirtualChange::splice(
                Range::new(Position::new(0, 12), Position::new(0, 13)),
                "2",
            )]
            .into_iter()
            .collect(),
        })
        .unwrap();
    assert_eq!(state.documents[&uri].text, "let value = 2;");
}

#[test]
fn stale_results_do_not_decode() {
    let mut state = ReplicatedState::default();
    state.results.insert(
        "ping".into(),
        ReplicatedCacheEntry {
            expires_at_unix_ms: Some(0),
            bytes: serde_json::to_vec("pong").unwrap().into_iter().collect(),
        },
    );
    assert_eq!(state.result::<String>("ping").unwrap(), None);
}

#[test]
fn fresh_results_decode_successfully() {
    let mut state = ReplicatedState::default();
    state.results.insert(
        "ping".into(),
        ReplicatedCacheEntry::encode(&"pong", Some(Duration::from_secs(60))).unwrap(),
    );
    assert_eq!(
        state.result::<String>("ping").unwrap().as_deref(),
        Some("pong")
    );
}

#[test]
fn malformed_results_surface_decode_errors() {
    let mut state = ReplicatedState::default();
    state.results.insert(
        "broken".into(),
        ReplicatedCacheEntry::new((*b"{").into_iter().collect(), Some(Duration::from_secs(60))),
    );
    assert!(matches!(
        state.result::<String>("broken"),
        Err(CorsaError::Json(_))
    ));
}

#[test]
fn change_to_unknown_document_returns_protocol_error() {
    let err = ReplicatedState::default()
        .apply(&ReplicatedCommand::ApplyDocumentChange {
            uri: "missing".into(),
            changes: SmallVec::new(),
        })
        .unwrap_err();
    assert!(
        matches!(err, CorsaError::Protocol(message) if message.contains("unknown replicated document"))
    );
}

#[test]
fn remove_commands_clear_existing_entries() {
    let mut state = ReplicatedState::default();
    let document =
        VirtualDocument::in_memory("cluster", "/main.ts", "typescript", "let value = 1;").unwrap();
    let uri = document.key();
    state
        .apply(&ReplicatedCommand::PutDocument { document })
        .unwrap();
    state.snapshots.insert(
        "workspace".into(),
        ReplicatedSnapshot {
            handle: SnapshotHandle::from("snapshot-1"),
            projects: SmallVec::new(),
            changes: None,
        },
    );
    state.results.insert(
        "ping".into(),
        ReplicatedCacheEntry::encode(&"pong", Some(Duration::from_secs(60))).unwrap(),
    );

    state
        .apply(&ReplicatedCommand::RemoveDocument { uri: uri.clone() })
        .unwrap();
    state
        .apply(&ReplicatedCommand::RemoveSnapshot {
            key: "workspace".into(),
        })
        .unwrap();
    state
        .apply(&ReplicatedCommand::RemoveResult { key: "ping".into() })
        .unwrap();

    assert!(!state.documents.contains_key(uri.as_str()));
    assert!(state.snapshots.is_empty());
    assert!(state.results.is_empty());
}
