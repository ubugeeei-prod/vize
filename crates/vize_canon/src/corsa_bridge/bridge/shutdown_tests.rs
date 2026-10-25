//! FIFO shutdown observes accepted startup; marker jobs grant no SDK credit.
#![expect(clippy::disallowed_types, reason = "worker ownership unit controls")]

use super::{CorsaBridge, CorsaBridgeConfig, CorsaBridgeError};
use futures::{FutureExt, executor::block_on};
use std::{sync::mpsc, time::Duration};

const SETTLE: Duration = Duration::from_secs(10);

fn abandoned_startup(bridge: &CorsaBridge) -> mpsc::Sender<()> {
    let (release, held) = mpsc::channel();
    let (entered, observed) = mpsc::channel();
    let mut startup = Box::pin(bridge.worker.submit_async(SETTLE, move |_| {
        entered.send(()).unwrap();
        held.recv().unwrap();
    }));
    assert!(startup.as_mut().now_or_never().is_none());
    observed.recv_timeout(SETTLE).unwrap();
    drop(startup);
    assert!(!bridge.is_initialized());
    assert!(bridge.is_draining());
    release
}

#[test]
fn shutdown_queues_behind_an_uninitialized_accepted_startup() {
    let bridge = CorsaBridge::new();
    let release = abandoned_startup(&bridge);
    let mut shutdown = Box::pin(bridge.shutdown());
    assert!(shutdown.as_mut().now_or_never().is_none());
    release.send(()).unwrap();
    assert!(block_on(shutdown).is_ok());
    assert!(!bridge.is_draining());
    assert!(!bridge.is_initialized());
}

#[test]
fn pending_startup_timeout_is_not_success_and_can_be_retried_after_drain() {
    let bridge = CorsaBridge::with_config(CorsaBridgeConfig {
        timeout_ms: 50,
        ..Default::default()
    });
    let release = abandoned_startup(&bridge);
    assert!(matches!(
        block_on(bridge.shutdown()),
        Err(CorsaBridgeError::Timeout)
    ));
    assert!(bridge.is_draining());
    release.send(()).unwrap();
    assert!(
        block_on(
            bridge
                .worker
                .submit_ready_async(SETTLE, |slot| slot.is_none())
        )
        .unwrap()
    );
    assert!(block_on(bridge.shutdown()).is_ok());
    assert!(!bridge.is_draining());
}

#[test]
fn empty_repeated_shutdown_never_spawns_a_client() {
    let bridge = CorsaBridge::new();
    assert!(block_on(bridge.shutdown()).is_ok());
    assert!(block_on(bridge.shutdown()).is_ok());
    assert!(
        block_on(
            bridge
                .worker
                .submit_ready_async(SETTLE, |slot| slot.is_none())
        )
        .unwrap()
    );
    assert!(!bridge.is_initialized());
}

#[test]
fn a_stopped_worker_is_not_a_successful_shutdown() {
    let bridge = CorsaBridge::new();
    assert!(
        block_on(
            bridge
                .worker
                .submit_async(SETTLE, |_| -> () { panic!("controlled worker panic") })
        )
        .is_err()
    );
    assert!(matches!(
        block_on(bridge.shutdown()),
        Err(CorsaBridgeError::ProcessTerminated)
    ));
}

#[test]
fn pinned_shutdown_ownership_cases_use_the_actual_worker_lane() {
    #[derive(serde::Deserialize)]
    #[serde(deny_unknown_fields, rename_all = "camelCase")]
    struct Fixture {
        schema: std::string::String,
        version: u8,
        bridge_cases: Vec<Case>,
        owner_cases: Vec<serde_json::Value>,
    }
    #[derive(serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Case {
        id: std::string::String,
        admission: std::string::String,
        release: std::string::String,
        expected: std::string::String,
    }
    let fixture: Fixture = serde_json::from_str(include_str!(
        "../../../../../tests/_fixtures/runtime/lsp-owned-shutdown-10760/ownership.json"
    ))
    .unwrap();
    assert_eq!(fixture.schema, "vize.owned-compiler-shutdown");
    assert_eq!(fixture.version, 1);
    assert_eq!(fixture.owner_cases.len(), 2);
    assert_eq!(
        fixture
            .bridge_cases
            .iter()
            .map(|case| case.id.as_str())
            .collect::<Vec<_>>(),
        [
            "absent-client",
            "pending-startup-release",
            "pending-startup-timeout",
            "stopped-worker",
        ]
    );
    for case in fixture.bridge_cases {
        let bridge = CorsaBridge::with_config(CorsaBridgeConfig {
            timeout_ms: if case.expected == "timeout" {
                50
            } else {
                10_000
            },
            ..Default::default()
        });
        let result = match (case.admission.as_str(), case.release.as_str()) {
            ("none", "none") => block_on(bridge.shutdown()),
            ("pending", "before-timeout") => {
                let release = abandoned_startup(&bridge);
                let mut shutdown = Box::pin(bridge.shutdown());
                assert!(shutdown.as_mut().now_or_never().is_none());
                release.send(()).unwrap();
                block_on(shutdown)
            }
            ("pending", "after-timeout") => {
                let release = abandoned_startup(&bridge);
                let result = block_on(bridge.shutdown());
                assert!(bridge.is_draining());
                release.send(()).unwrap();
                assert!(
                    block_on(
                        bridge
                            .worker
                            .submit_ready_async(SETTLE, |slot| slot.is_none())
                    )
                    .unwrap()
                );
                result
            }
            ("stopped", "none") => {
                assert!(
                    block_on(bridge.worker.submit_async(SETTLE, |_| -> () {
                        panic!("controlled fixture worker failure")
                    }))
                    .is_err()
                );
                block_on(bridge.shutdown())
            }
            _ => panic!("unregistered worker ownership input: {}", case.id),
        };
        let actual = match result {
            Ok(()) => "ok",
            Err(CorsaBridgeError::Timeout) => "timeout",
            Err(CorsaBridgeError::ProcessTerminated) => "process-terminated",
            Err(error) => panic!("unexpected ownership outcome: {error}"),
        };
        assert_eq!(actual, case.expected, "{}", case.id);
        assert!(!bridge.is_initialized());
    }
}
