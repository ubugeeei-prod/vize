mod support;

#[cfg(feature = "experimental-distributed")]
use corsa::lsp::{VirtualChange, VirtualDocument};
#[cfg(feature = "experimental-distributed")]
use corsa::orchestrator::{DistributedApiOrchestrator, RaftCluster, ReplicatedCommand};
use corsa::{
    api::{ApiClient, ApiMode, UpdateSnapshotParams},
    observability::{CorsaEvent, CorsaObserver},
    orchestrator::{ApiOrchestrator, ApiOrchestratorConfig},
    runtime::block_on,
};
use serde_json::{Value, json};
use std::{
    future::Future,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
    time::Duration,
};

#[derive(Default)]
struct EventCollector {
    events: Mutex<Vec<CorsaEvent>>,
}

impl CorsaObserver for EventCollector {
    fn on_event(&self, event: &CorsaEvent) {
        self.events.lock().unwrap().push(event.clone());
    }
}

fn run_async_test<F>(future: F)
where
    F: Future<Output = ()> + Send + 'static,
{
    let handle = thread::Builder::new()
        .name("orchestrator-test".into())
        .stack_size(8 * 1024 * 1024)
        .spawn(move || block_on(future))
        .unwrap();
    if let Err(panic) = handle.join() {
        std::panic::resume_unwind(panic);
    }
}

#[test]
fn orchestrator_caches_snapshots_and_results() {
    run_async_test(async {
        let orchestrator = ApiOrchestrator::default();
        let profile = support::api_profile("async-cache", ApiMode::AsyncJsonRpcStdio);
        let snapshot_a = orchestrator
            .cached_snapshot(
                &profile,
                "workspace",
                UpdateSnapshotParams {
                    open_project: Some("/workspace/tsconfig.json".into()),
                    file_changes: None,
                    overlay_changes: None,
                },
            )
            .await
            .unwrap();
        let snapshot_b = orchestrator
            .cached_snapshot(
                &profile,
                "workspace",
                UpdateSnapshotParams {
                    open_project: Some("/workspace/tsconfig.json".into()),
                    file_changes: None,
                    overlay_changes: None,
                },
            )
            .await
            .unwrap();
        assert!(Arc::ptr_eq(&snapshot_a, &snapshot_b));

        let calls = Arc::new(AtomicUsize::new(0));
        let first = orchestrator
            .cached(&profile, "ping", Some(Duration::from_secs(30)), {
                let calls = calls.clone();
                move |client| async move {
                    calls.fetch_add(1, Ordering::SeqCst);
                    client.raw_json_request("ping", Value::Null).await
                }
            })
            .await
            .unwrap();
        let second = orchestrator
            .cached(&profile, "ping", Some(Duration::from_secs(30)), {
                let calls = calls.clone();
                move |client| async move {
                    calls.fetch_add(1, Ordering::SeqCst);
                    client.raw_json_request("ping", Value::Null).await
                }
            })
            .await
            .unwrap();
        assert_eq!(first, json!("pong"));
        assert_eq!(second, json!("pong"));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    });
}

#[test]
fn orchestrator_executes_parallel_batches() {
    run_async_test(async {
        let orchestrator = ApiOrchestrator::default();
        let profile = support::api_profile("async-batch", ApiMode::AsyncJsonRpcStdio);
        let values = orchestrator
            .execute_all(&profile, 2, [1_u32, 2, 3, 4], |client, value| async move {
                let echoed = client
                    .raw_json_request("echo", json!({ "value": value }))
                    .await?;
                Ok::<_, corsa::CorsaError>(echoed["value"].as_u64().unwrap() as u32)
            })
            .await
            .unwrap();
        assert_eq!(values.as_slice(), &[1, 2, 3, 4]);
    });
}

#[test]
fn orchestrator_executes_batches_larger_than_work_queue_capacity() {
    run_async_test(async {
        let orchestrator = ApiOrchestrator::new(ApiOrchestratorConfig {
            work_queue_capacity: 1,
            ..ApiOrchestratorConfig::default()
        });
        let profile = support::api_profile("async-batch-backpressure", ApiMode::AsyncJsonRpcStdio);
        let values = orchestrator
            .execute_all(&profile, 2, 0_u32..8, |_, value| async move {
                Ok::<_, corsa::CorsaError>(value)
            })
            .await
            .unwrap();
        assert_eq!(values.as_slice(), &[0, 1, 2, 3, 4, 5, 6, 7]);
    });
}

#[test]
fn orchestrator_reports_batch_task_panics() {
    run_async_test(async {
        let orchestrator = ApiOrchestrator::default();
        let profile = support::api_profile("async-batch-panic", ApiMode::AsyncJsonRpcStdio);
        let error = orchestrator
            .execute_all(&profile, 1, [1_u32], |_, _| async move {
                panic!("batch task exploded");
                #[allow(unreachable_code)]
                Ok::<_, corsa::CorsaError>(0_u32)
            })
            .await
            .unwrap_err();
        assert!(
            error
                .to_string()
                .contains("orchestrator batch worker panicked: batch task exploded"),
            "{error}"
        );
    });
}

#[test]
fn orchestrator_skips_worker_start_for_empty_batches() {
    run_async_test(async {
        let orchestrator = ApiOrchestrator::default();
        let profile = support::api_profile("async-empty-batch", ApiMode::AsyncJsonRpcStdio);
        let values = orchestrator
            .execute_all(
                &profile,
                4,
                std::iter::empty::<u32>(),
                |_, value| async move { Ok::<_, corsa::CorsaError>(value) },
            )
            .await
            .unwrap();
        assert!(values.is_empty());
        assert_eq!(orchestrator.stats().worker_count, 0);
    });
}

#[test]
fn orchestrator_recomputes_expired_cached_values() {
    run_async_test(async {
        let orchestrator = ApiOrchestrator::default();
        let profile = support::api_profile("async-expiring-cache", ApiMode::AsyncJsonRpcStdio);
        let calls = Arc::new(AtomicUsize::new(0));

        let _: Value = orchestrator
            .cached(&profile, "expiring", Some(Duration::from_millis(5)), {
                let calls = calls.clone();
                move |client| async move {
                    calls.fetch_add(1, Ordering::SeqCst);
                    client.raw_json_request("ping", Value::Null).await
                }
            })
            .await
            .unwrap();

        std::thread::sleep(Duration::from_millis(20));

        let _: Value = orchestrator
            .cached(&profile, "expiring", Some(Duration::from_millis(5)), {
                let calls = calls.clone();
                move |client| async move {
                    calls.fetch_add(1, Ordering::SeqCst);
                    client.raw_json_request("ping", Value::Null).await
                }
            })
            .await
            .unwrap();

        assert_eq!(calls.load(Ordering::SeqCst), 2);
    });
}

#[test]
#[cfg(feature = "experimental-distributed")]
fn raft_cluster_elects_a_leader_and_rejects_follower_writes() {
    let cluster = RaftCluster::new(["n1", "n2", "n3"]);
    let document =
        VirtualDocument::in_memory("cluster", "/main.ts", "typescript", "let value = 1;").unwrap();
    assert!(
        cluster
            .append(
                "n1",
                ReplicatedCommand::PutDocument {
                    document: document.clone(),
                },
            )
            .is_err()
    );
    assert_eq!(cluster.campaign("n2").unwrap(), 1);
    cluster
        .append(
            "n2",
            ReplicatedCommand::PutDocument {
                document: document.clone(),
            },
        )
        .unwrap();
    assert!(
        cluster
            .append(
                "n1",
                ReplicatedCommand::PutDocument {
                    document: document.clone(),
                },
            )
            .is_err()
    );
    for node in ["n1", "n2", "n3"] {
        let state = cluster.node_state(node).unwrap();
        assert_eq!(state.documents[document.uri.as_str()], document);
    }
}

#[test]
#[cfg(feature = "experimental-distributed")]
fn distributed_orchestrator_replicates_virtual_documents_and_results() {
    run_async_test(async {
        let orchestrator = DistributedApiOrchestrator::new(["n1", "n2", "n3"]);
        let profile = support::api_profile("dist-cache", ApiMode::AsyncJsonRpcStdio);
        let document =
            VirtualDocument::in_memory("cluster", "/main.ts", "typescript", "let value = 1;")
                .unwrap();
        orchestrator.campaign("n1").unwrap();
        orchestrator
            .open_virtual_document("n1", document.clone())
            .unwrap();
        let updated = orchestrator
            .change_virtual_document(
                "n1",
                &document.uri,
                [VirtualChange::splice(
                    lsp_types::Range::new(
                        lsp_types::Position::new(0, 12),
                        lsp_types::Position::new(0, 13),
                    ),
                    "2",
                )],
            )
            .unwrap();
        assert_eq!(updated.text, "let value = 2;");
        let calls = Arc::new(AtomicUsize::new(0));
        let first: Value = orchestrator
            .cached(&profile, "n1", "ping", Some(Duration::from_secs(30)), {
                let calls = calls.clone();
                move |client| async move {
                    calls.fetch_add(1, Ordering::SeqCst);
                    client.raw_json_request("ping", Value::Null).await
                }
            })
            .await
            .unwrap();
        let second: Value = orchestrator
            .cached(&profile, "n1", "ping", Some(Duration::from_secs(30)), {
                let calls = calls.clone();
                move |client| async move {
                    calls.fetch_add(1, Ordering::SeqCst);
                    client.raw_json_request("ping", Value::Null).await
                }
            })
            .await
            .unwrap();
        assert_eq!(first, json!("pong"));
        assert_eq!(second, json!("pong"));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        for node in ["n1", "n2", "n3"] {
            let state = orchestrator.node_state(node).unwrap();
            assert_eq!(
                state.documents[document.uri.as_str()].text,
                "let value = 2;"
            );
            assert_eq!(
                state.result::<Value>("ping").unwrap().unwrap(),
                json!("pong")
            );
        }
    });
}

#[test]
#[cfg(feature = "experimental-distributed")]
fn distributed_orchestrator_replicates_snapshot_records() {
    run_async_test(async {
        let orchestrator = DistributedApiOrchestrator::new(["leader", "follower-a", "follower-b"]);
        let profile = support::api_profile("dist-snapshot", ApiMode::AsyncJsonRpcStdio);
        orchestrator.campaign("leader").unwrap();
        let snapshot = orchestrator
            .cached_snapshot(
                &profile,
                "leader",
                "workspace",
                UpdateSnapshotParams {
                    open_project: Some("/workspace/tsconfig.json".into()),
                    file_changes: None,
                    overlay_changes: None,
                },
            )
            .await
            .unwrap();
        let record = orchestrator.snapshot_record("leader", "workspace").unwrap();
        assert_eq!(record.handle, snapshot.handle);
        for node in ["leader", "follower-a", "follower-b"] {
            let state = orchestrator.node_state(node).unwrap();
            assert_eq!(state.snapshots["workspace"].handle, snapshot.handle);
        }
    });
}

#[test]
fn orchestrator_enforces_cache_limits() {
    run_async_test(async {
        let orchestrator = ApiOrchestrator::new(ApiOrchestratorConfig {
            max_workers_per_profile: 2,
            max_cached_snapshots: 1,
            max_cached_results: 1,
            work_queue_capacity: 2,
            observer: None,
        });
        let profile = support::api_profile("limited-cache", ApiMode::AsyncJsonRpcStdio);

        let snapshot_a = orchestrator
            .cached_snapshot(
                &profile,
                "workspace-a",
                UpdateSnapshotParams {
                    open_project: Some("/workspace/a/tsconfig.json".into()),
                    file_changes: None,
                    overlay_changes: None,
                },
            )
            .await
            .unwrap();
        let _snapshot_b = orchestrator
            .cached_snapshot(
                &profile,
                "workspace-b",
                UpdateSnapshotParams {
                    open_project: Some("/workspace/b/tsconfig.json".into()),
                    file_changes: None,
                    overlay_changes: None,
                },
            )
            .await
            .unwrap();
        let snapshot_a_again = orchestrator
            .cached_snapshot(
                &profile,
                "workspace-a",
                UpdateSnapshotParams {
                    open_project: Some("/workspace/a/tsconfig.json".into()),
                    file_changes: None,
                    overlay_changes: None,
                },
            )
            .await
            .unwrap();
        assert!(!Arc::ptr_eq(&snapshot_a, &snapshot_a_again));

        let calls = Arc::new(AtomicUsize::new(0));
        let compute = |calls: Arc<AtomicUsize>, key: &'static str| {
            move |client: ApiClient| async move {
                calls.fetch_add(1, Ordering::SeqCst);
                client
                    .raw_json_request("echo", json!({ "value": key }))
                    .await
            }
        };
        let _: Value = orchestrator
            .cached(
                &profile,
                "result-a",
                Some(Duration::from_secs(30)),
                compute(calls.clone(), "a"),
            )
            .await
            .unwrap();
        let _: Value = orchestrator
            .cached(
                &profile,
                "result-b",
                Some(Duration::from_secs(30)),
                compute(calls.clone(), "b"),
            )
            .await
            .unwrap();
        let _: Value = orchestrator
            .cached(
                &profile,
                "result-a",
                Some(Duration::from_secs(30)),
                compute(calls.clone(), "a"),
            )
            .await
            .unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 3);

        let stats = orchestrator.stats();
        assert_eq!(stats.cached_snapshot_count, 1);
        assert_eq!(stats.cached_result_count, 1);
    });
}

#[test]
fn orchestrator_emits_eviction_events() {
    run_async_test(async {
        let observer = Arc::new(EventCollector::default());
        let orchestrator = ApiOrchestrator::new(
            ApiOrchestratorConfig {
                max_workers_per_profile: 2,
                max_cached_snapshots: 1,
                max_cached_results: 1,
                work_queue_capacity: 2,
                observer: None,
            }
            .with_observer(observer.clone()),
        );
        let profile = support::api_profile("observed-cache", ApiMode::AsyncJsonRpcStdio);

        let _ = orchestrator
            .cached_snapshot(
                &profile,
                "workspace-a",
                UpdateSnapshotParams {
                    open_project: Some("/workspace/a/tsconfig.json".into()),
                    file_changes: None,
                    overlay_changes: None,
                },
            )
            .await
            .unwrap();
        let _ = orchestrator
            .cached_snapshot(
                &profile,
                "workspace-b",
                UpdateSnapshotParams {
                    open_project: Some("/workspace/b/tsconfig.json".into()),
                    file_changes: None,
                    overlay_changes: None,
                },
            )
            .await
            .unwrap();
        let _: Value = orchestrator
            .cached(
                &profile,
                "ping-a",
                Some(Duration::from_secs(30)),
                |client| async move { client.raw_json_request("ping", Value::Null).await },
            )
            .await
            .unwrap();
        let _: Value = orchestrator
            .cached(
                &profile,
                "ping-b",
                Some(Duration::from_secs(30)),
                |client| async move { client.raw_json_request("ping", Value::Null).await },
            )
            .await
            .unwrap();

        let events = observer.events.lock().unwrap().clone();
        assert!(events.contains(&CorsaEvent::OrchestratorSnapshotEvicted {
            key: "workspace-a".into(),
        }));
        assert!(events.contains(&CorsaEvent::OrchestratorResultEvicted {
            key: "ping-a".into(),
        }));
    });
}

#[test]
fn orchestrator_rejects_worker_requests_above_limit() {
    run_async_test(async {
        let orchestrator = ApiOrchestrator::new(ApiOrchestratorConfig {
            max_workers_per_profile: 1,
            max_cached_snapshots: 4,
            max_cached_results: 4,
            work_queue_capacity: 4,
            observer: None,
        });
        let profile = support::api_profile("limited-workers", ApiMode::AsyncJsonRpcStdio);
        let error = orchestrator.prewarm(&profile, 2).await.unwrap_err();
        assert!(matches!(
            error,
            corsa::CorsaError::Protocol(message) if message.contains("exceeds the configured maximum")
        ));
    });
}
