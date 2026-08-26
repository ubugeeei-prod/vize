use std::sync::{Arc, Barrier};

use harness::RealCorsaRenameSession;
use vize_s0::cstr;

mod direct_first;
pub(in crate::ide) mod harness;

#[test]
fn concurrent_real_corsa_rename_sessions_are_isolated() {
    const SESSION_COUNT: usize = 20;
    const ASSERTED_ROUNDS: usize = 4;

    let corsa_path = resolve_tsgo_binary_required();
    // Setup and root validation happen before the only barrier. A setup
    // failure therefore reports immediately instead of stranding sibling
    // lanes at a barrier until the outer CI timeout.
    let sessions = (0..SESSION_COUNT)
        .map(|session_index| {
            let session = RealCorsaRenameSession::new(&corsa_path)
                .map_err(|error| cstr!("session {session_index} setup: {error}"))?;
            session
                .assert_root_identity()
                .map_err(|error| cstr!("session {session_index} root identity: {error}"))?;
            Ok((session_index, session))
        })
        .collect::<Result<Vec<_>, String>>()
        .unwrap_or_else(|error| panic!("{error}"));
    let mut roots = sessions
        .iter()
        .map(|(_, session)| session.logical_root().to_path_buf())
        .collect::<Vec<_>>();

    let rendezvous = Arc::new(Barrier::new(SESSION_COUNT));
    let sessions = std::thread::scope(|scope| {
        sessions
            .into_iter()
            .map(|(session_index, session)| {
                let rendezvous = Arc::clone(&rendezvous);
                scope.spawn(move || -> Result<_, String> {
                    rendezvous.wait();
                    session
                        .assert_direct_first_child_rename()
                        .map_err(|error| {
                            cstr!(
                                "session {session_index} direct-first rename: {error}; {}",
                                harness::evidence::describe_server_frames(&session)
                            )
                        })?;
                    for round in 0..ASSERTED_ROUNDS {
                        session.assert_prepare_ranges().map_err(|error| {
                            cstr!("session {session_index} round {round} prepare: {error}")
                        })?;
                        session.assert_parent_and_child_renames().map_err(|error| {
                            cstr!("session {session_index} round {round} rename: {error}")
                        })?;
                    }
                    Ok((session_index, session))
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|handle| handle.join().expect("real Corsa session thread panicked"))
            .collect::<Result<Vec<_>, _>>()
    })
    .unwrap_or_else(|error| panic!("{error}"));

    shutdown_sessions(sessions);
    assert_during_sibling_shutdown(&corsa_path, &mut roots);

    for (index, root) in roots.iter().enumerate() {
        assert!(
            roots.iter().enumerate().all(|(other_index, other)| {
                index == other_index || (root != other && !root.starts_with(other))
            }),
            "session roots must be pairwise disjoint: {roots:#?}"
        );
        assert!(
            !root.exists(),
            "session TempDir leaked after shutdown: {}",
            root.display()
        );
    }
}

fn shutdown_sessions(sessions: Vec<(usize, RealCorsaRenameSession)>) {
    let results = std::thread::scope(|scope| {
        sessions
            .into_iter()
            .map(|(session_index, mut session)| {
                scope.spawn(move || {
                    session
                        .shutdown()
                        .map_err(|error| cstr!("session {session_index} shutdown: {error}"))
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|handle| handle.join().expect("real Corsa shutdown thread panicked"))
            .collect::<Result<Vec<_>, _>>()
    });
    results.unwrap_or_else(|error| panic!("{error}"));
}

fn assert_during_sibling_shutdown(
    corsa_path: &std::path::Path,
    roots: &mut Vec<std::path::PathBuf>,
) {
    let mut closing = RealCorsaRenameSession::new_with_shutdown_gate(corsa_path)
        .unwrap_or_else(|error| panic!("closing session setup: {error}"));
    let mut survivor = RealCorsaRenameSession::new(corsa_path)
        .unwrap_or_else(|error| panic!("survivor session setup: {error}"));
    for (name, session) in [("closing", &closing), ("survivor", &survivor)] {
        session
            .assert_root_identity()
            .unwrap_or_else(|error| panic!("{name} session root identity: {error}"));
        session
            .assert_prepare_ranges()
            .unwrap_or_else(|error| panic!("{name} session initial prepare: {error}"));
        session
            .assert_parent_and_child_renames()
            .unwrap_or_else(|error| panic!("{name} session initial rename: {error}"));
        roots.push(session.logical_root().to_path_buf());
    }
    let mut gate = closing
        .arm_shutdown_gate()
        .unwrap_or_else(|error| panic!("closing session shutdown gate: {error}"));

    std::thread::scope(|scope| {
        let shutdown = scope.spawn(move || closing.shutdown());
        gate.wait_until_observed()
            .unwrap_or_else(|error| panic!("{error}"));
        // The closing editor has forwarded its protocol shutdown request and
        // is held before exit while the surviving sibling proves exact ranges
        // and edits on its independent transport.
        survivor
            .assert_prepare_ranges()
            .unwrap_or_else(|error| panic!("survivor during sibling shutdown: {error}"));
        survivor
            .assert_parent_and_child_renames()
            .unwrap_or_else(|error| panic!("survivor rename during sibling shutdown: {error}"));
        gate.release().unwrap_or_else(|error| panic!("{error}"));
        shutdown
            .join()
            .expect("real Corsa shutdown thread panicked")
            .unwrap_or_else(|error| panic!("closing session shutdown: {error}"));
    });
    survivor
        .shutdown()
        .unwrap_or_else(|error| panic!("survivor session shutdown: {error}"));
}

fn resolve_tsgo_binary_required() -> std::path::PathBuf {
    assert!(
        std::env::var_os("VIZE_TEST_DISABLE_TSGO").is_none(),
        "real-Corsa isolation coverage cannot be skipped with VIZE_TEST_DISABLE_TSGO",
    );
    super::corsa_tests::resolve_tsgo_binary()
        .unwrap_or_else(|| panic!("real-Corsa isolation requires pinned tsgo"))
}
