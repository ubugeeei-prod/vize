use std::sync::{Arc, Barrier};

use vize_s0::cstr;

use super::{harness::RealCorsaRenameSession, resolve_tsgo_binary_required};

#[test]
fn dependency_change_rearms_the_shared_editor_transport_once() {
    let corsa_path = resolve_tsgo_binary_required();
    let mut session = RealCorsaRenameSession::new(&corsa_path)
        .unwrap_or_else(|error| panic!("session setup: {error}"));
    session
        .assert_parent_queries_before_child_change()
        .unwrap_or_else(|error| panic!("generation one: {error}"));
    session
        .assert_child_change_rearms_parent_rename()
        .unwrap_or_else(|error| panic!("generation two: {error}"));
    session
        .shutdown()
        .unwrap_or_else(|error| panic!("shutdown: {error}"));
}

#[test]
fn direct_first_renames_survive_twenty_session_shutdown_overlap() {
    const PAIR_COUNT: usize = 10;

    let corsa_path = resolve_tsgo_binary_required();
    let mut pairs = (0..PAIR_COUNT)
        .map(|index| {
            let closing = RealCorsaRenameSession::new_with_shutdown_gate(&corsa_path)
                .map_err(|error| cstr!("closing session {index}: {error}"))?;
            closing
                .assert_prepare_ranges()
                .map_err(|error| cstr!("closing session {index} prepare: {error}"))?;
            closing
                .assert_parent_and_child_renames()
                .map_err(|error| cstr!("closing session {index} rename: {error}"))?;
            let survivor = RealCorsaRenameSession::new(&corsa_path)
                .map_err(|error| cstr!("survivor session {index}: {error}"))?;
            Ok((index, closing, survivor))
        })
        .collect::<Result<Vec<_>, String>>()
        .unwrap_or_else(|error| panic!("{error}"));
    let mut gates = pairs
        .iter_mut()
        .map(|(index, closing, _)| {
            closing
                .arm_shutdown_gate()
                .map_err(|error| cstr!("closing session {index} gate: {error}"))
        })
        .collect::<Result<Vec<_>, _>>()
        .unwrap_or_else(|error| panic!("{error}"));

    // Own the gates inside the scoped closure so unwinding drops them and
    // releases blocked shutdown threads before `scope` joins those threads.
    std::thread::scope(move |scope| {
        let shutdowns = pairs
            .into_iter()
            .map(|(index, mut closing, survivor)| {
                let shutdown = scope.spawn(move || {
                    closing
                        .shutdown()
                        .map_err(|error| cstr!("closing session {index}: {error}"))
                });
                (index, survivor, shutdown)
            })
            .collect::<Vec<_>>();
        for (index, gate) in gates.iter_mut().enumerate() {
            gate.wait_until_observed()
                .unwrap_or_else(|error| panic!("closing session {index}: {error}"));
        }

        let rendezvous = Arc::new(Barrier::new(PAIR_COUNT));
        let survivors = shutdowns
            .into_iter()
            .map(|(index, survivor, shutdown)| {
                let rendezvous = Arc::clone(&rendezvous);
                scope.spawn(move || {
                    rendezvous.wait();
                    survivor
                        .assert_direct_first_child_rename()
                        .map_err(|error| cstr!("survivor session {index}: {error}"))?;
                    Ok::<_, String>((survivor, shutdown))
                })
            })
            .collect::<Vec<_>>();
        let mut survivors = survivors
            .into_iter()
            .map(|survivor| survivor.join().expect("survivor thread panicked"))
            .collect::<Result<Vec<_>, _>>()
            .unwrap_or_else(|error| panic!("{error}"));

        for (index, gate) in gates.iter_mut().enumerate() {
            gate.release()
                .unwrap_or_else(|error| panic!("closing session {index}: {error}"));
        }
        for (mut survivor, shutdown) in survivors.drain(..) {
            shutdown
                .join()
                .expect("closing session thread panicked")
                .unwrap_or_else(|error| panic!("{error}"));
            survivor
                .shutdown()
                .unwrap_or_else(|error| panic!("survivor shutdown: {error}"));
        }
    });
}
