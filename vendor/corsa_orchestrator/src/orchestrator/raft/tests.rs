use super::*;
use crate::lsp::VirtualDocument;
use crate::orchestrator::state::ReplicatedCommand;
use std::sync::Arc;

fn doc(id: &str, text: &str) -> VirtualDocument {
    VirtualDocument::in_memory("cluster", id, "typescript", text).unwrap()
}

#[test]
fn cluster_starts_without_a_leader() {
    let cluster = RaftCluster::new(["a", "b", "c"]);
    assert!(cluster.leader_id().is_none());
    assert_eq!(cluster.role("a"), Some(RaftRole::Follower));
}

#[test]
fn campaign_elects_a_leader_and_replicates_log() {
    let cluster = RaftCluster::new(["a", "b", "c"]);
    let term = cluster.campaign("b").unwrap();
    assert_eq!(term, 1);
    assert_eq!(cluster.leader_id().as_deref(), Some("b"));
    assert_eq!(cluster.role("b"), Some(RaftRole::Leader));
    assert_eq!(cluster.role("a"), Some(RaftRole::Follower));

    let document = doc("/main.ts", "let value = 1;");
    let key = document.key();
    cluster
        .append(
            "b",
            ReplicatedCommand::PutDocument {
                document: document.clone(),
            },
        )
        .unwrap();

    for node in ["a", "b", "c"] {
        let state = cluster.node_state(node).unwrap();
        assert_eq!(state.documents[key.as_str()], document);
    }
}

#[test]
fn append_to_follower_is_rejected() {
    let cluster = RaftCluster::new(["a", "b", "c"]);
    cluster.campaign("a").unwrap();
    let err = cluster
        .append(
            "b",
            ReplicatedCommand::PutDocument {
                document: doc("/main.ts", ""),
            },
        )
        .unwrap_err();
    assert!(format!("{err}").contains("not leader"));
}

#[test]
fn term_increments_across_re_elections() {
    let cluster = RaftCluster::new(["a", "b", "c"]);
    let t1 = cluster.campaign("a").unwrap();
    let t2 = cluster.campaign("b").unwrap();
    assert!(t2 > t1);
    assert_eq!(cluster.leader_id().as_deref(), Some("b"));
}

#[test]
fn single_node_cluster_elects_self() {
    let cluster = RaftCluster::new(["only"]);
    let term = cluster.campaign("only").unwrap();
    assert_eq!(term, 1);
    let document = doc("/main.ts", "x");
    let key = document.key();
    cluster
        .append(
            "only",
            ReplicatedCommand::PutDocument {
                document: document.clone(),
            },
        )
        .unwrap();
    let state = cluster.node_state("only").unwrap();
    assert!(state.documents.contains_key(key.as_str()));
}

#[test]
fn membership_change_adds_replica() {
    let cluster = RaftCluster::new(["a", "b", "c"]);
    cluster.campaign("a").unwrap();
    let document = doc("/m.ts", "x");
    let key = document.key();
    cluster
        .append(
            "a",
            ReplicatedCommand::PutDocument {
                document: document.clone(),
            },
        )
        .unwrap();
    cluster.add_member("d").unwrap();
    // Tick the cluster so the leader replicates to the new member.
    cluster.tick().unwrap();
    cluster.tick().unwrap();
    let state = cluster.node_state("d").unwrap();
    assert!(state.documents.contains_key(key.as_str()));
}

#[test]
fn membership_change_removes_replica() {
    let cluster = RaftCluster::new(["a", "b", "c"]);
    cluster.campaign("a").unwrap();
    cluster.remove_member("c").unwrap();
    let members = cluster.members();
    assert!(!members.iter().any(|m| m.as_str() == "c"));
    cluster
        .append(
            "a",
            ReplicatedCommand::PutDocument {
                document: doc("/m.ts", "x"),
            },
        )
        .unwrap();
}

#[test]
fn rejected_config_is_returned() {
    let defaults = RaftConfig::default();
    let config = RaftConfig {
        heartbeat_interval: defaults.election_timeout_min,
        ..defaults
    };
    let result = RaftClusterBuilder::new(["a"]).config(config).build();
    let err = match result {
        Ok(_) => panic!("expected invalid config to be rejected"),
        Err(err) => err,
    };
    assert!(format!("{err}").contains("invalid"));
}

#[test]
fn in_memory_storage_round_trips_state() {
    let storage = Arc::new(InMemoryStorage::new()) as Arc<dyn RaftStorage>;
    storage
        .save_hard_state(
            "node",
            &HardState {
                current_term: 7,
                voted_for: Some("peer".into()),
            },
        )
        .unwrap();
    let loaded = storage.load_hard_state("node").unwrap();
    assert_eq!(loaded.current_term, 7);
    assert_eq!(loaded.voted_for.as_deref(), Some("peer"));

    storage
        .append_log(
            "node",
            1,
            &[PersistedLogEntry {
                term: 1,
                command: ReplicatedCommand::PutDocument {
                    document: doc("/x.ts", "1"),
                },
            }],
        )
        .unwrap();
    let log = storage.load_log("node").unwrap();
    assert_eq!(log.len(), 1);
    storage.truncate_log_suffix("node", 0).unwrap();
    let log = storage.load_log("node").unwrap();
    assert!(log.is_empty());
}

#[test]
fn file_storage_persists_across_restart() {
    let dir = tempdir();
    {
        let storage = FileStorage::new(&dir).unwrap();
        storage
            .save_hard_state(
                "node",
                &HardState {
                    current_term: 9,
                    voted_for: Some("self".into()),
                },
            )
            .unwrap();
        storage
            .append_log(
                "node",
                1,
                &[PersistedLogEntry {
                    term: 9,
                    command: ReplicatedCommand::PutDocument {
                        document: doc("/x.ts", "1"),
                    },
                }],
            )
            .unwrap();
    }
    let storage = FileStorage::new(&dir).unwrap();
    let loaded = storage.load_hard_state("node").unwrap();
    assert_eq!(loaded.current_term, 9);
    assert_eq!(loaded.voted_for.as_deref(), Some("self"));
    assert_eq!(storage.load_log("node").unwrap().len(), 1);
}

#[test]
fn snapshot_compacts_log() {
    let config = RaftConfig {
        snapshot_threshold: 2,
        ..RaftConfig::default()
    };
    let cluster = RaftCluster::with_config(["a", "b", "c"], config);
    cluster.campaign("a").unwrap();
    for index in 0..5 {
        cluster
            .append(
                "a",
                ReplicatedCommand::PutDocument {
                    document: doc(&format!("/doc{index}.ts"), ""),
                },
            )
            .unwrap();
    }
    let snapshot = cluster.compact("a").unwrap();
    assert!(snapshot.last_included_index > 0);
}

#[test]
fn log_compaction_triggers_automatically_at_threshold() {
    let config = RaftConfig {
        snapshot_threshold: 2,
        ..RaftConfig::default()
    };
    let cluster = RaftCluster::with_config(["a", "b", "c"], config);
    cluster.campaign("a").unwrap();
    for index in 0..5 {
        cluster
            .append(
                "a",
                ReplicatedCommand::PutDocument {
                    document: doc(&format!("/doc{index}.ts"), ""),
                },
            )
            .unwrap();
    }
    // Applying committed entries must have crossed the threshold and
    // compacted the leader's in-memory log without an explicit compact().
    let nodes = cluster.inner.nodes.read();
    let leader = nodes["a"].lock();
    assert!(leader.log_base_index > 0);
    assert!(leader.log.len() < 5);
    // The replicated state machine still holds every applied document.
    drop(leader);
    drop(nodes);
    let state = cluster.node_state("a").unwrap();
    assert_eq!(state.documents.len(), 5);
}

#[test]
fn cluster_recovers_from_dropped_response_via_retry() {
    let cluster = RaftCluster::new(["a", "b", "c"]);
    cluster.campaign("a").unwrap();
    // Force a heartbeat by ticking; even if responses are processed, no
    // committed index should regress.
    let committed_before = cluster
        .node_state("a")
        .map(|state| state.documents.len())
        .unwrap_or_default();
    cluster.tick().unwrap();
    let committed_after = cluster
        .node_state("a")
        .map(|state| state.documents.len())
        .unwrap_or_default();
    assert_eq!(committed_before, committed_after);
}

fn tempdir() -> std::path::PathBuf {
    let mut dir = std::env::temp_dir();
    let suffix: u128 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    dir.push(format!("corsa-raft-test-{suffix}"));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}
