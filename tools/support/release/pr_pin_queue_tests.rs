use super::*;
use std::collections::VecDeque;
const HEAD: &str = "1111111111111111111111111111111111111111";
const LEASE: &str = "2222222222222222222222222222222222222222";
fn candidate() -> Candidate {
    Candidate {
        number: 42,
        repository: "owner/repo".into(),
        author: "maintainer".into(),
        branch: "release-integration/v0.463.0".into(),
        head: HEAD.into(),
        base: "3".repeat(40),
        tag: "v0.463.0".into(),
        merged: false,
    }
}
fn authority() -> Authority {
    Authority {
        lease: LEASE.into(),
        operator_run: 7,
        operator_attempt: 2,
    }
}
fn owner() -> QueueOwner {
    QueueOwner {
        entry_id: "MQE_owned".into(),
        pull_request_id: "PR_exact".into(),
        number: 42,
        head: HEAD.into(),
        authority: authority(),
    }
}
fn policy() -> Value {
    json!([
        {"type":"required_status_checks","parameters":{"required_status_checks":
            (["check-js","check-vize-apps","fmt-rust","test-report"].map(|context|json!({"context":context,"integration_id":15368})))}},
        {"type":"merge_queue","ruleset_id":24013395,"parameters":{"grouping_strategy":"ALLGREEN"}}
    ])
}
fn snapshot(entry: Option<&str>) -> Value {
    json!({"data":{"repository":{"nameWithOwner":"owner/repo","pullRequest":{
        "id":"PR_exact","number":42,"state":"OPEN","isDraft":false,
        "headRefOid":HEAD,"headRefName":"release-integration/v0.463.0","baseRefName":"main",
        "autoMergeRequest":null,"mergeQueueEntry":entry.map(|id|json!({"id":id}))
    }}}})
}
fn enqueued() -> Value {
    json!({"data":{"enqueuePullRequest":{"mergeQueueEntry":{"id":"MQE_owned",
        "pullRequest":{"id":"PR_exact","number":42,"headRefOid":HEAD}}}}})
}
fn dequeued() -> Value {
    // Genuine retained API response shape: 9363 owned-dequeue-for-legacy-ssr.
    json!({"data":{"dequeuePullRequest":{"mergeQueueEntry":{"id":"MQE_owned"}}}})
}
#[derive(Default)]
struct Harness {
    replies: VecDeque<Result<String, String>>,
    calls: Vec<Vec<String>>,
    proofs: VecDeque<Result<Authority, String>>,
    receipts: Vec<(String, Value)>,
    auth_calls: usize,
    acquired: Option<QueueOwner>,
    fail_enqueue_retention: bool,
}
impl Harness {
    fn new(replies: Vec<Value>) -> Self {
        Self {
            replies: replies
                .into_iter()
                .map(|value| Ok(value.to_string()))
                .collect(),
            ..Self::default()
        }
    }
    fn run(&mut self, cleanup_owner: Option<&QueueOwner>) -> Result<Option<QueueOwner>, String> {
        let Self {
            replies,
            calls,
            proofs,
            receipts,
            auth_calls,
            acquired,
            fail_enqueue_retention,
        } = self;
        let mut execute = |program: &str, args: &[&str], root: &Path| {
            assert_eq!(program, "gh");
            assert_eq!(root, Path::new("/fixture"));
            calls.push(args.iter().map(|arg| (*arg).to_owned()).collect());
            replies.pop_front().expect("unexpected additional API call")
        };
        let mut retain = |stage: &str, response: &Value| {
            receipts.push((stage.into(), response.clone()));
            if stage == "enqueue" && *fail_enqueue_retention {
                return Err("queue custody disk full".into());
            }
            Ok(())
        };
        let mut verify = |observed: &Candidate| {
            assert_eq!(*observed, candidate());
            *auth_calls += 1;
            proofs.pop_front().unwrap_or_else(|| Ok(authority()))
        };
        let mut adapter = Adapter {
            root: Path::new("/fixture"),
            execute: &mut execute,
            retain: &mut retain,
            batch: None,
        };
        if let Some(owner) = cleanup_owner {
            cleanup(&candidate(), owner, &mut verify, &mut adapter).map(|()| None)
        } else {
            enqueue(
                &candidate(),
                &authority(),
                &mut verify,
                &mut adapter,
                acquired,
            )?;
            Ok(acquired.clone())
        }
    }
    fn writes(&self) -> usize {
        self.calls
            .iter()
            .filter(|args| args.iter().any(|arg| arg.starts_with("query=mutation")))
            .count()
    }
}
fn expected_graphql(query: &str, variables: &[(&str, &str)]) -> Vec<String> {
    let mut args = vec![
        "api".into(),
        "graphql".into(),
        "-f".into(),
        format!("query={query}"),
    ];
    for (key, value) in variables {
        args.extend([
            if *key == "number" { "-F" } else { "-f" }.into(),
            format!("{key}={value}"),
        ]);
    }
    args
}

#[test]
fn native_queue_acquires_only_actual_returned_entry_with_expected_head_and_whole_custody() {
    let responses = vec![
        policy(),
        snapshot(None),
        enqueued(),
        snapshot(Some("MQE_owned")),
    ];
    let mut fixture = Harness::new(responses.clone());
    assert_eq!(fixture.run(None).unwrap(), Some(owner()));
    assert_eq!(fixture.auth_calls, 3);
    assert_eq!(fixture.writes(), 1);
    assert_eq!(
        fixture.calls[0],
        vec!["api", "repos/owner/repo/rules/branches/main"]
    );
    assert_eq!(
        fixture.calls[1],
        expected_graphql(
            SNAPSHOT,
            &[("owner", "owner"), ("name", "repo"), ("number", "42")]
        )
    );
    assert_eq!(
        fixture.calls[2],
        expected_graphql(
            "mutation($pr:ID!,$head:GitObjectID!){enqueuePullRequest(input:{pullRequestId:$pr,expectedHeadOid:$head}){mergeQueueEntry{id pullRequest{id number headRefOid}}}}",
            &[("pr", "PR_exact"), ("head", HEAD)]
        )
    );
    assert_eq!(fixture.calls[3], fixture.calls[1]);
    assert_eq!(
        fixture
            .receipts
            .iter()
            .map(|(_, v)| serde_json::from_str::<Value>(v["raw"].as_str().unwrap()).unwrap())
            .collect::<Vec<_>>(),
        responses
    );
    assert!(
        fixture
            .calls
            .iter()
            .all(|args| !args.iter().any(|arg| arg == "--auto" || arg == "--admin"))
    );
}

#[test]
fn owned_dequeue_uses_real_id_input_once_and_verifies_exact_entry_then_fresh_null() {
    let mut fixture = Harness::new(vec![
        snapshot(Some("MQE_owned")),
        dequeued(),
        snapshot(None),
    ]);
    fixture.run(Some(&owner())).unwrap();
    assert_eq!(fixture.auth_calls, 3);
    assert_eq!(fixture.writes(), 1);
    assert_eq!(
        fixture.calls[1],
        expected_graphql(
            "mutation($pr:ID!){dequeuePullRequest(input:{id:$pr}){mergeQueueEntry{id}}}",
            &[("pr", "PR_exact")]
        )
    );
    assert_eq!(fixture.calls[0], fixture.calls[2]);
    assert_eq!(fixture.receipts.len(), 3);
}

#[test]
fn already_removed_owned_entry_needs_fresh_identity_and_authority_but_no_write() {
    let mut fixture = Harness::new(vec![snapshot(None)]);
    fixture.run(Some(&owner())).unwrap();
    assert_eq!(fixture.auth_calls, 2);
    assert_eq!(fixture.writes(), 0);
}

#[test]
fn same_identity_merged_integration_allows_only_fresh_null_cleanup_noop() {
    for entry in [None, Some("MQE_owned"), Some("MQE_foreign")] {
        let mut state = snapshot(entry);
        state["data"]["repository"]["pullRequest"]["state"] = json!("MERGED");
        let mut fixture = Harness::new(vec![state]);
        let result = fixture.run(Some(&owner()));
        assert_eq!(result.is_ok(), entry.is_none());
        assert_eq!(fixture.writes(), 0);
        assert_eq!(fixture.calls.len(), 1);
        assert_eq!(fixture.auth_calls, if entry.is_none() { 2 } else { 1 });
    }
}

#[test]
fn changed_effective_required_contexts_app_or_queue_policy_refuse_before_admission() {
    let mut cases = Vec::new();
    for index in 0..4 {
        let mut changed = policy();
        changed[0]["parameters"]["required_status_checks"]
            .as_array_mut()
            .unwrap()
            .remove(index);
        cases.push(changed);
        let mut changed = policy();
        changed[0]["parameters"]["required_status_checks"][index]["integration_id"] = json!(999);
        cases.push(changed);
    }
    let mut duplicate = policy();
    let check = duplicate[0]["parameters"]["required_status_checks"][0].clone();
    duplicate[0]["parameters"]["required_status_checks"]
        .as_array_mut()
        .unwrap()
        .push(check);
    cases.push(duplicate);
    for (pointer, value) in [
        ("/1/parameters/grouping_strategy", json!("HEADGREEN")),
        ("/1/ruleset_id", json!(1)),
    ] {
        let mut changed = policy();
        *changed.pointer_mut(pointer).unwrap() = value;
        cases.push(changed);
    }
    let mut missing = policy();
    missing.as_array_mut().unwrap().pop();
    cases.push(missing);
    let mut duplicate = policy();
    let queue = duplicate[1].clone();
    duplicate.as_array_mut().unwrap().push(queue);
    cases.push(duplicate);
    for changed in cases {
        let mut fixture = Harness::new(vec![changed]);
        assert!(fixture.run(None).is_err());
        assert_eq!(fixture.calls.len(), 1);
        assert_eq!(fixture.writes(), 0);
        assert_eq!(fixture.receipts.len(), 1);
    }
}

#[test]
fn missing_effective_grouping_refuses_without_secondary_lookup_or_mutation() {
    let mut missing = policy();
    missing[1]["parameters"]
        .as_object_mut()
        .unwrap()
        .remove("grouping_strategy");
    for effective in [missing, {
        let mut value = policy();
        value[1]["parameters"]["grouping_strategy"] = Value::Null;
        value
    }] {
        let mut fixture = Harness::new(vec![effective]);
        assert!(fixture.run(None).is_err());
        assert_eq!(fixture.calls.len(), 1);
        assert_eq!(fixture.writes(), 0);
    }
}

#[test]
fn existing_entries_unknown_auto_or_changed_integration_are_never_adopted_or_mutated() {
    let mut snapshots = vec![snapshot(Some("MQE_foreign"))];
    for (pointer, value) in [
        ("/id", json!("")),
        ("/number", json!(43)),
        ("/state", json!("MERGED")),
        ("/isDraft", json!(true)),
        ("/headRefOid", json!("4".repeat(40))),
        ("/headRefName", json!("other")),
        ("/baseRefName", json!("other")),
        (
            "/autoMergeRequest",
            json!({"enabledAt":"2026-10-17T00:00:00Z"}),
        ),
    ] {
        let mut changed = snapshot(None);
        *changed
            .pointer_mut(&format!("/data/repository/pullRequest{pointer}"))
            .unwrap() = value;
        snapshots.push(changed);
    }
    for field in ["mergeQueueEntry", "autoMergeRequest"] {
        let mut missing = snapshot(None);
        missing
            .pointer_mut("/data/repository/pullRequest")
            .unwrap()
            .as_object_mut()
            .unwrap()
            .remove(field);
        snapshots.push(missing);
    }
    for state in snapshots {
        let mut fixture = Harness::new(vec![policy(), state]);
        assert!(fixture.run(None).is_err());
        assert_eq!(fixture.writes(), 0);
    }
}

#[test]
fn authentication_head_mismatch_and_graphql_failures_are_fatal_without_retry() {
    for error in [
        "HTTP 401 auth failed\nstdout packet\nstderr packet",
        "expectedHeadOid mismatch",
    ] {
        let mut fixture = Harness::new(vec![policy(), snapshot(None)]);
        fixture.replies.push_back(Err(error.into()));
        assert_eq!(fixture.run(None).unwrap_err(), error);
        assert_eq!(fixture.writes(), 1);
        assert_eq!(fixture.calls.len(), 3);
        assert_eq!(
            fixture.receipts.last().unwrap().1,
            json!({"commandError":error})
        );
    }
    let errors =
        json!({"data":{"enqueuePullRequest":null},"errors":[{"message":"strict guard refused"}]});
    let mut fixture = Harness::new(vec![policy(), snapshot(None), errors.clone()]);
    assert!(fixture.run(None).is_err());
    assert_eq!(fixture.writes(), 1);
    assert_eq!(
        serde_json::from_str::<Value>(fixture.receipts.last().unwrap().1["raw"].as_str().unwrap())
            .unwrap(),
        errors
    );
}

#[test]
fn foreign_cleanup_entries_changed_lease_attempt_or_unknown_auto_remain_untouched() {
    for proof in [
        Err("lease authentication unavailable".into()),
        Ok(Authority {
            operator_run: 8,
            ..authority()
        }),
        Ok(Authority {
            lease: "4".repeat(40),
            ..authority()
        }),
        Ok(Authority {
            operator_attempt: 3,
            ..authority()
        }),
    ] {
        let mut fixture = Harness::default();
        fixture.proofs.push_back(proof);
        assert!(fixture.run(Some(&owner())).is_err());
        assert!(fixture.calls.is_empty());
    }
    let mut states = vec![snapshot(Some("MQE_foreign"))];
    for (field, changed) in [
        ("autoMergeRequest", json!({"enabledAt":"unknown"})),
        ("headRefOid", json!("4".repeat(40))),
        ("id", json!("PR_foreign")),
    ] {
        let mut value = snapshot(Some("MQE_owned"));
        value["data"]["repository"]["pullRequest"][field] = changed;
        states.push(value);
    }
    for state in states {
        let mut fixture = Harness::new(vec![state]);
        assert!(fixture.run(Some(&owner())).is_err());
        assert_eq!(fixture.writes(), 0);
    }
    let mut fixture = Harness::new(vec![snapshot(Some("MQE_owned"))]);
    fixture.proofs.extend([
        Ok(authority()),
        Err("operator attempt changed before dequeue".into()),
    ]);
    assert!(fixture.run(Some(&owner())).is_err());
    assert_eq!(fixture.writes(), 0);
}

#[test]
fn ambiguous_dequeue_response_or_replacement_after_write_never_causes_second_mutation() {
    for response in [
        json!({"data":{"dequeuePullRequest":{"mergeQueueEntry":{"id":"MQE_foreign"}}}}),
        json!({"data":{"dequeuePullRequest":null},"errors":[{"message":"denied"}]}),
    ] {
        let mut fixture = Harness::new(vec![snapshot(Some("MQE_owned")), response]);
        assert!(fixture.run(Some(&owner())).is_err());
        assert_eq!(fixture.writes(), 1);
        assert_eq!(fixture.calls.len(), 2);
    }
    let mut fixture = Harness::new(vec![
        snapshot(Some("MQE_owned")),
        dequeued(),
        snapshot(Some("MQE_replacement")),
    ]);
    assert!(fixture.run(Some(&owner())).is_err());
    assert_eq!(fixture.writes(), 1);
    let mut fixture = Harness::new(vec![snapshot(Some("MQE_owned"))]);
    fixture
        .replies
        .push_back(Err("network failure after server response lost".into()));
    assert!(fixture.run(Some(&owner())).is_err());
    assert_eq!(fixture.writes(), 1);
    assert_eq!(fixture.receipts.len(), 2);
}

#[test]
fn enqueue_records_only_authenticated_response_owner_before_fallible_postwrite_reads() {
    for (pointer, value) in [
        ("/id", json!("")),
        ("/pullRequest/id", json!("PR_foreign")),
        ("/pullRequest/number", json!(43)),
        ("/pullRequest/headRefOid", json!("4".repeat(40))),
    ] {
        let mut response = enqueued();
        *response
            .pointer_mut(&format!(
                "/data/enqueuePullRequest/mergeQueueEntry{pointer}"
            ))
            .unwrap() = value;
        let mut fixture = Harness::new(vec![policy(), snapshot(None), response]);
        assert!(fixture.run(None).is_err(), "{pointer}");
        assert_eq!(fixture.writes(), 1);
        assert_eq!(fixture.calls.len(), 3);
        assert!(fixture.acquired.is_none());
    }
    let mut fixture = Harness::new(vec![
        policy(),
        snapshot(None),
        enqueued(),
        snapshot(Some("MQE_owned")),
    ]);
    fixture.proofs.extend([
        Ok(authority()),
        Ok(authority()),
        Err("lease changed after enqueue".into()),
    ]);
    assert!(fixture.run(None).is_err());
    assert_eq!(fixture.acquired, Some(owner()));
    assert_eq!(fixture.writes(), 1);
    assert_eq!(fixture.receipts.len(), 4);
    let mut fixture = Harness::new(vec![policy(), snapshot(None)]);
    fixture
        .replies
        .push_back(Ok("{malformed whole server packet\n".into()));
    assert!(fixture.run(None).is_err());
    assert!(fixture.acquired.is_none());
    assert_eq!(fixture.writes(), 1);
    assert_eq!(
        fixture.receipts.last().unwrap().1,
        json!({"raw":"{malformed whole server packet\n"})
    );
    let mut fixture = Harness::new(vec![policy(), snapshot(None), enqueued()]);
    fixture.replies.push_back(Err("GET failed".into()));
    assert!(fixture.run(None).is_err());
    assert_eq!(fixture.acquired, Some(owner()));
    assert_eq!(fixture.writes(), 1);
    fixture.replies.extend(
        [snapshot(Some("MQE_owned")), dequeued(), snapshot(None)]
            .map(|value| Ok(value.to_string())),
    );
    fixture.run(Some(&owner())).unwrap();
    assert_eq!(fixture.writes(), 2);
    let mut fixture = Harness::new(vec![policy(), snapshot(None), enqueued()]);
    fixture.fail_enqueue_retention = true;
    assert!(fixture.run(None).unwrap_err().contains("disk full"));
    assert_eq!(fixture.acquired, Some(owner()));
}
