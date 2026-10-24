use super::*;

fn receipt(output: PathBuf) -> FailureReceipt {
    FailureReceipt::new(
        output,
        "fixture",
        "immutable-revision".into(),
        "src/Oracle.vue",
        BatchIncrementalBudget {
            cold_ms: 15_000,
            warm_ms: 5_000,
            max_requested_files: 681,
            max_changed_files: 1,
        },
        1.0,
    )
}

#[test]
#[expect(clippy::disallowed_types, reason = "Rust panic payload is std String")]
fn budget_unwind_preserves_complete_cold_metrics_and_unexecuted_warms() {
    let temp = tempfile::tempdir().unwrap();
    let output = temp.path().join("receipt");
    let failure = std::panic::catch_unwind(|| {
        let mut guard = receipt(output.clone());
        guard.begin(0);
        guard.complete(
            0,
            27_374,
            IncrementalCheckMetrics {
                checks: 1,
                session_starts: 1,
                last_session_started: true,
                last_requested_files: 681,
                last_tree_entries_scanned: 700,
                last_materialized_entries_considered: 700,
                last_full_rebuild: true,
                ..Default::default()
            },
            681,
        );
        crate::budget::assert_within_budget("cold", 27_374, 15_000, 1.0);
    })
    .unwrap_err();
    assert_eq!(
        failure.downcast_ref::<std::string::String>().unwrap(),
        "cold took 27374ms, budget is 15000ms at scale 1 (15000ms)"
    );
    let raw: Value =
        serde_json::from_slice(&fs::read(output.join("failure.json")).unwrap()).unwrap();
    assert_eq!(raw["fileCount"], 681);
    assert_eq!(raw["lanes"][0]["durationMs"], 27_374);
    let metrics = raw["lanes"][0]["metrics"].as_object().unwrap();
    assert_eq!(metrics.len(), 19);
    assert_eq!(metrics["lastTreeEntriesScanned"], 700);
    assert_eq!(metrics["lastFullRebuild"], true);
    for lane in &raw["lanes"].as_array().unwrap()[1..] {
        assert_eq!(lane["state"], "NOT_EXECUTED");
        assert!(lane["durationMs"].is_null() && lane["metrics"].is_null());
    }
    assert!(!output.join("metrics.json").exists());
}

#[test]
fn incomplete_warm_is_distinct_from_unexecuted_repair_and_io_failure_keeps_original_panic() {
    let temp = tempfile::tempdir().unwrap();
    let output = temp.path().join("receipt");
    let _ = std::panic::catch_unwind(|| {
        let mut guard = receipt(output.clone());
        guard.complete(0, 10, Default::default(), 681);
        guard.begin(1);
        panic!("warm check failed");
    });
    let raw: Value =
        serde_json::from_slice(&fs::read(output.join("failure.json")).unwrap()).unwrap();
    assert_eq!(raw["lanes"][1]["state"], "IN_PROGRESS");
    assert_eq!(raw["lanes"][2]["state"], "NOT_EXECUTED");
    let blocked = temp.path().join("ordinary-file");
    fs::write(&blocked, "not a directory").unwrap();
    let failure = std::panic::catch_unwind(|| {
        let _guard = receipt(blocked);
        panic!("original failure");
    })
    .unwrap_err();
    assert_eq!(failure.downcast_ref::<&str>(), Some(&"original failure"));
}

#[test]
fn successful_or_disarmed_drop_does_not_add_a_metrics_stage() {
    let temp = tempfile::tempdir().unwrap();
    let output = temp.path().join("absent");
    drop(receipt(output.clone()));
    assert!(!output.exists());
    let _ = std::panic::catch_unwind(|| {
        let mut guard = receipt(output.clone());
        guard.disarm();
        panic!("after successful publication");
    });
    assert!(!output.exists());
}

#[test]
fn scan_unwind_retains_whole_root_order_and_each_materialized_identity() {
    let temp = tempfile::tempdir().unwrap();
    let output = temp.path().join("receipt");
    let root = temp.path().join("fixture");
    let first = root.join("First.vue");
    let second = root.join("Second.vue");
    let support = temp.path().join("outside.ts");
    let first_virtual = temp.path().join("mirror/First.vue.ts");
    let support_virtual = temp.path().join("mirror/outside.ts");
    let selected = vec![second.clone(), first.clone()];
    let failure = std::panic::catch_unwind(|| {
        let mut guard = receipt(output.clone());
        guard.begin(0);
        guard.scanned_inputs(
            &root,
            "source-revision",
            &selected,
            [
                (first.as_path(), first_virtual.as_path(), "alpha"),
                (support.as_path(), support_virtual.as_path(), "support\n"),
                (first.as_path(), first_virtual.as_path(), "beta"),
            ],
        );
        panic!("unchanged scale assertion");
    })
    .unwrap_err();
    assert_eq!(
        failure.downcast_ref::<&str>(),
        Some(&"unchanged scale assertion")
    );
    let raw: Value =
        serde_json::from_slice(&fs::read(output.join("failure.json")).unwrap()).unwrap();
    assert_eq!(
        raw["scannedInputs"],
        json!({
            "sourceRevision": "source-revision",
            "compiledGithubSha": option_env!("GITHUB_SHA"),
            "fixtureRoot": root,
            "selectedRoots": selected,
            "materializedFiles": [
                {"originalPath":first,"fixtureRelativePath":"First.vue","virtualPath":first_virtual,
                 "contentBytes":5,"contentSha256":"8ed3f6ad685b959ead7022518e1af76cd816f8e8ec7ccdda1ed4018e8f2223f8"},
                {"originalPath":support,"fixtureRelativePath":null,"virtualPath":support_virtual,
                 "contentBytes":8,"contentSha256":"c05b56ab6d3d07ccca6778ebd799bf98dfa2b7f10dd43601f0938f04e5070878"},
                {"originalPath":first,"fixtureRelativePath":"First.vue","virtualPath":first_virtual,
                 "contentBytes":4,"contentSha256":"f44e64e75f3948e9f73f8dfa94721c4ce8cbb4f265c4790c702b2d41cfbf2753"},
            ],
        })
    );
    assert_eq!(raw["lanes"][0]["state"], "IN_PROGRESS");
    assert!(raw["fileCount"].is_null());
    assert!(!output.join("metrics.json").exists());
}
