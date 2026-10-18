use super::*;
use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
};

fn pinned() -> Source {
    super::super::tests::source(&"a".repeat(40), &"b".repeat(40))
}
fn run(source: &Source) -> Value {
    json!({"id":7,"run_attempt":1,"head_sha":source.candidate.head,
        "head_branch":source.candidate.branch,"event":"workflow_dispatch",
        "display_title":format!("Pinned Release {} PR #{} @ {}",source.candidate.tag,source.candidate.number,source.candidate.head),
        "path":".github/workflows/release.yml","head_repository":{"full_name":source.candidate.repository},
        "status":"in_progress","conclusion":null})
}
fn job(source: &Source, conclusion: Value) -> Value {
    json!({"id":11,"run_id":7,"run_attempt":1,"head_sha":source.candidate.head,
        "name":"Release preflight / Wait for the original canonical Check",
        "status":if conclusion.is_null(){"in_progress"}else{"completed"},"conclusion":conclusion})
}

#[test]
fn failed_original_wait_is_fatal_while_whole_release_and_other_builds_are_pending() {
    let source = pinned();
    let observed = run(&source);
    let healthy = vec![
        job(&source, Value::Null),
        json!({"id":12,"run_id":7,"run_attempt":1,
        "head_sha":source.candidate.head,"name":"policy-disabled channel","status":"completed","conclusion":"skipped"}),
    ];
    assert!(release_progress(&observed, &healthy, &source, 7, 1).is_ok());
    for failure in [
        "failure",
        "cancelled",
        "timed_out",
        "action_required",
        "stale",
        "startup_failure",
        "neutral",
        "unknown",
    ] {
        let mut jobs = healthy.clone();
        jobs.push(job(&source, json!(failure)));
        jobs[2]["id"] = json!(13);
        let error = release_progress(&observed, &jobs, &source, 7, 1).unwrap_err();
        assert!(error.contains("original canonical Check"));
        assert!(error.contains(failure));
    }
    let mut complete = observed.clone();
    complete["status"] = json!("completed");
    complete["conclusion"] = json!("failure");
    assert!(release_progress(&complete, &healthy, &source, 7, 1).is_err());
}

#[test]
fn progress_observations_refuse_foreign_run_head_attempt_workflow_and_job_inventories() {
    let source = pinned();
    let observed = run(&source);
    let jobs = vec![job(&source, Value::Null)];
    for (pointer, value) in [
        ("/id", json!(8)),
        ("/run_attempt", json!(2)),
        ("/head_sha", json!("c".repeat(40))),
        ("/head_branch", json!("main")),
        ("/path", json!(".github/workflows/check.yml")),
        ("/head_repository/full_name", json!("foreign/repo")),
    ] {
        let mut changed = observed.clone();
        *changed.pointer_mut(pointer).unwrap() = value;
        assert!(
            release_progress(&changed, &jobs, &source, 7, 1).is_err(),
            "{pointer}"
        );
    }
    for (key, value) in [
        ("id", json!(0)),
        ("run_id", json!(8)),
        ("run_attempt", json!(2)),
        ("head_sha", json!("c".repeat(40))),
    ] {
        let mut changed = jobs.clone();
        changed[0][key] = value;
        assert!(
            release_progress(&observed, &changed, &source, 7, 1).is_err(),
            "{key}"
        );
    }
    assert!(
        release_progress(
            &observed,
            &[jobs[0].clone(), jobs[0].clone()],
            &source,
            7,
            1
        )
        .is_err()
    );
}

#[test]
fn typed_pending_states_remain_nonblocking_and_unknown_progress_never_grants_admission() {
    let source = pinned();
    for status in ["queued", "in_progress", "requested", "waiting", "pending"] {
        let mut observed = run(&source);
        observed["status"] = json!(status);
        assert!(release_progress(&observed, &[], &source, 7, 1).is_ok());
        observed["conclusion"] = json!("failure");
        assert!(release_progress(&observed, &[], &source, 7, 1).is_err());
    }
    for status in [Value::Null, json!("foreign")] {
        let mut observed = run(&source);
        observed["status"] = status.clone();
        assert!(release_progress(&observed, &[], &source, 7, 1).is_err());
        let mut invalid = job(&source, Value::Null);
        invalid["status"] = status;
        assert!(release_progress(&run(&source), &[invalid], &source, 7, 1).is_err());
    }
    // --failed reruns retain earlier successful jobs in filter=latest. They do
    // not permit a future job attempt or a changed current Release attempt.
    let mut resumed = run(&source);
    resumed["run_attempt"] = json!(2);
    assert!(release_progress(&resumed, &[job(&source, json!("success"))], &source, 7, 2).is_ok());
}

/// Exercise the same guard inside the real enqueue authority callback. Failure
/// can appear after ordinary checks or after acquisition; cleanup must preserve
/// the original failure and touch only the returned owned entry once.
#[test]
fn failure_during_admission_prevents_enqueue_or_withdraws_its_exact_acquired_entry() {
    let source = pinned();
    let mut candidate = source.candidate.clone();
    candidate.number = 43;
    candidate.branch = format!("release-integration/{}", candidate.tag);
    candidate.head = "c".repeat(40);
    let authority = queue::Authority {
        lease: "d".repeat(40),
        operator_run: 9,
        operator_attempt: 1,
    };
    let snapshot = |queued: bool| {
        json!({"data":{"repository":{"nameWithOwner":candidate.repository,"pullRequest":{
        "id":"PR_owned","number":43,"state":"OPEN","isDraft":false,"headRefOid":candidate.head,
        "headRefName":candidate.branch,"baseRefName":"main","autoMergeRequest":null,
        "mergeQueueEntry":if queued{json!({"id":"MQE_owned"})}else{Value::Null}}}}})
    };
    let policy = json!([
        {"type":"required_status_checks","parameters":{"required_status_checks":
            (["check-js","check-vize-apps","fmt-rust","test-report"].map(|context|json!({"context":context,"integration_id":15368})))}},
        {"type":"merge_queue","ruleset_id":24013395,"parameters":{"grouping_strategy":"ALLGREEN"}}]);
    let enqueued = json!({"data":{"enqueuePullRequest":{"mergeQueueEntry":{"id":"MQE_owned",
        "pullRequest":{"id":"PR_owned","number":43,"headRefOid":candidate.head}}}}});
    let dequeued = json!({"data":{"dequeuePullRequest":{"mergeQueueEntry":{"id":"MQE_owned"}}}});
    for failure_at in [0, 1, 2, 3, 4] {
        let calls = Cell::new(0);
        let writes = Cell::new(0);
        let replies = RefCell::new(VecDeque::from([
            policy.clone(),
            snapshot(false),
            enqueued.clone(),
            snapshot(true),
        ]));
        let mut execute = |_: &str, args: &[&str], _: &Path| {
            if args.iter().any(|arg| arg.starts_with("query=mutation")) {
                writes.set(writes.get() + 1);
            }
            Ok(replies
                .borrow_mut()
                .pop_front()
                .expect("unexpected request")
                .to_string())
        };
        let mut retain = |_: &str, _: &Value| Ok(());
        let mut verify = |_: &pr_contract::Candidate| {
            calls.set(calls.get() + 1);
            let failure = calls.get() == failure_at;
            release_progress(
                &run(&source),
                &[job(
                    &source,
                    if failure {
                        json!("failure")
                    } else {
                        Value::Null
                    },
                )],
                &source,
                7,
                1,
            )?;
            Ok(authority.clone())
        };
        let mut owned = None;
        // The top-of-watch observation can fail before any admission work.
        let result = release_progress(
            &run(&source),
            &[job(
                &source,
                if failure_at == 0 {
                    json!("failure")
                } else {
                    Value::Null
                },
            )],
            &source,
            7,
            1,
        )
        .and_then(|()| {
            queue::enqueue(
                &candidate,
                &authority,
                &mut verify,
                &mut queue::Adapter {
                    root: Path::new("/fixture"),
                    execute: &mut execute,
                    retain: &mut retain,
                    batch: None,
                },
                &mut owned,
            )
        });
        if failure_at < 3 {
            assert!(result.is_err());
            assert_eq!(writes.get(), 0);
            assert!(owned.is_none());
        } else if failure_at == 3 {
            let failure = result.unwrap_err();
            assert!(failure.contains("original canonical Check"));
            assert_eq!(writes.get(), 1);
            let owner = owned.as_ref().unwrap();
            replies
                .borrow_mut()
                .extend([snapshot(true), dequeued.clone(), snapshot(false)]);
            queue::cleanup(
                &candidate,
                owner,
                &mut |_| Ok(authority.clone()),
                &mut queue::Adapter {
                    root: Path::new("/fixture"),
                    execute: &mut execute,
                    retain: &mut retain,
                    batch: None,
                },
            )
            .unwrap();
            assert_eq!(writes.get(), 2);
            assert!(failure.contains("failure"));
        } else {
            result.unwrap();
            assert_eq!(writes.get(), 1);
            assert!(owned.is_some());
        }
    }
}

#[test]
fn acknowledged_rerun_waits_only_for_exact_next_attempt_within_original_ten_seconds() {
    let source = pinned();
    let mut previous = run(&source);
    previous["status"] = json!("completed");
    previous["conclusion"] = json!("failure");
    let mut next = run(&source);
    next["run_attempt"] = json!(2);
    let replies = RefCell::new(VecDeque::from([
        previous.clone(),
        previous.clone(),
        next.clone(),
    ]));
    let clock = Cell::new(Duration::ZERO);
    let reads = Cell::new(0);
    let actual = await_resumed_attempt(
        &previous,
        &source,
        7,
        || {
            reads.set(reads.get() + 1);
            Ok(replies.borrow_mut().pop_front().unwrap())
        },
        || clock.get(),
        |delay| {
            assert!(delay <= Duration::from_secs(1));
            clock.set(clock.get() + delay);
            Ok(())
        },
    )
    .unwrap();
    assert_eq!(actual, 2);
    assert_eq!(reads.get(), 3);
    assert_eq!(clock.get(), Duration::from_secs(2));
    // A stale failed job never becomes successful merely because the next run
    // attempt appeared; the complete job guard remains unchanged.
    assert!(
        release_progress(&next, &[job(&source, json!("failure"))], &source, 7, actual).is_err()
    );
    for (key, value) in [
        ("run_attempt", json!(0)),
        ("run_attempt", json!(3)),
        ("head_sha", json!("e".repeat(40))),
        ("path", json!(".github/workflows/foreign.yml")),
    ] {
        let mut changed = next.clone();
        changed[key] = value;
        let count = Cell::new(0);
        assert!(
            await_resumed_attempt(
                &previous,
                &source,
                7,
                || {
                    count.set(count.get() + 1);
                    Ok(changed.clone())
                },
                || Duration::ZERO,
                |_| panic!("foreign identity must not poll")
            )
            .is_err()
        );
        assert_eq!(count.get(), 1);
    }
    let clock = Cell::new(Duration::ZERO);
    let reads = Cell::new(0);
    assert!(
        await_resumed_attempt(
            &previous,
            &source,
            7,
            || {
                reads.set(reads.get() + 1);
                Ok(previous.clone())
            },
            || clock.get(),
            |delay| {
                clock.set(clock.get() + delay);
                Ok(())
            }
        )
        .unwrap_err()
        .contains("existing resume window")
    );
    assert_eq!(clock.get(), Duration::from_secs(10));
    assert_eq!(reads.get(), 10);
    let clock = Cell::new(Duration::ZERO);
    assert!(
        await_resumed_attempt(
            &previous,
            &source,
            7,
            || {
                clock.set(Duration::from_secs(11));
                Ok(next.clone())
            },
            || clock.get(),
            |_| panic!("late acknowledgement cannot sleep or grant admission")
        )
        .is_err()
    );
}
