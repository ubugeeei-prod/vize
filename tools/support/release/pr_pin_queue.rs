//! One owned native queue entry; no persistent auto-merge or mutation retries.
use super::super::pr_contract::{self, Candidate};
use serde_json::{Value, json};
use std::path::Path;

#[path = "pr_pin_process.rs"]
mod process;
#[cfg(test)]
pub(super) use process::COMPENSATION_RUNNER;
pub(super) use process::Compensation;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Authority {
    pub lease: String,
    pub operator_run: u64,
    pub operator_attempt: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct QueueOwner {
    pub entry_id: String,
    pub pull_request_id: String,
    pub number: u64,
    pub head: String,
    pub authority: Authority,
}

type Execute<'a> = dyn FnMut(&str, &[&str], &Path) -> Result<String, String> + 'a;
type Retain<'a> = dyn FnMut(&str, &Value) -> Result<(), String> + 'a;
pub(super) struct Adapter<'a> {
    pub root: &'a Path,
    pub execute: &'a mut Execute<'a>,
    pub retain: &'a mut Retain<'a>,
}

impl Adapter<'_> {
    pub(super) fn commit(
        &mut self,
        repository: &str,
        id: &str,
        parent: &str,
        expected: Option<(&str, &str)>,
    ) -> Result<Value, String> {
        let value = self.read("rollback-commit", repository, &format!("git/commits/{id}"))?;
        pr_contract::sha(pr_contract::field(&value, "/tree/sha")?)?;
        if value["sha"] != id
            || value["parents"]
                .as_array()
                .is_none_or(|p| p.len() != 1 || p[0]["sha"] != parent)
            || expected.is_some_and(|(tree, message)| {
                value["tree"]["sha"] != tree
                    || (value["message"] != message && value["message"] != format!("{message}\n"))
            })
        {
            return Err("Original rollback commit tree/parent/message changed".into());
        }
        Ok(value)
    }
    fn call(
        &mut self,
        stage: &str,
        args: &[String],
        observed: impl FnOnce(&Value) -> Result<(), String>,
    ) -> Result<Value, String> {
        let args: Vec<_> = args.iter().map(String::as_str).collect();
        let raw = match (self.execute)("gh", &args, self.root) {
            Ok(raw) => raw,
            Err(error) => {
                return match (self.retain)(stage, &json!({"commandError":error})) {
                    Ok(()) => Err(error),
                    Err(retention) => Err(format!("{error}\nQueue custody failed: {retention}")),
                };
            }
        };
        let value = serde_json::from_str::<Value>(&raw)
            .map_err(|error| format!("Invalid {stage} response: {error}"))
            .and_then(|value| {
                if value.get("errors").is_some_and(|errors| !errors.is_null()) {
                    return Err(format!(
                        "GraphQL {stage} refused; retained complete response"
                    ));
                }
                observed(&value)?;
                Ok(value)
            });
        let retained = (self.retain)(stage, &json!({"raw":raw}));
        match (value, retained) {
            (Err(error), Err(retention)) => {
                Err(format!("{error}\nQueue custody failed: {retention}"))
            }
            (Err(error), _) | (_, Err(error)) => Err(error),
            (Ok(value), Ok(())) => Ok(value),
        }
    }

    pub(super) fn read(
        &mut self,
        stage: &str,
        repo: &str,
        endpoint: &str,
    ) -> Result<Value, String> {
        let endpoint = format!("repos/{repo}/{endpoint}");
        self.call(stage, &["api".into(), endpoint], |_| Ok(()))
    }

    fn graphql(
        &mut self,
        stage: &str,
        query: &str,
        variables: &[(&str, String)],
        observed: impl FnOnce(&Value) -> Result<(), String>,
    ) -> Result<Value, String> {
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
        self.call(stage, &args, observed)
    }
}

fn authenticate(
    candidate: &Candidate,
    expected: &Authority,
    verify: &mut impl FnMut(&Candidate) -> Result<Authority, String>,
) -> Result<(), String> {
    pr_contract::sha(&candidate.head)?;
    pr_contract::sha(&expected.lease)?;
    if candidate.number == 0
        || candidate.merged
        || candidate.branch != format!("release-integration/{}", candidate.tag)
        || expected.operator_run == 0
        || expected.operator_attempt == 0
        || verify(candidate)? != *expected
    {
        return Err(
            "Exact integration/original operator attempt/lease authentication failed".into(),
        );
    }
    Ok(())
}

fn effective_policy(candidate: &Candidate, adapter: &mut Adapter<'_>) -> Result<(), String> {
    let rules = adapter.read("main-policy", &candidate.repository, "rules/branches/main")?;
    let rules = rules.as_array().ok_or("Missing effective main rules")?;
    let mut contexts = Vec::new();
    for rule in rules
        .iter()
        .filter(|rule| rule["type"] == "required_status_checks")
    {
        let checks = rule
            .pointer("/parameters/required_status_checks")
            .and_then(Value::as_array)
            .ok_or("Missing effective required contexts")?;
        for check in checks {
            if check["integration_id"].as_u64() != Some(15368) {
                return Err("Required context must belong to the reviewed Actions app".into());
            }
            contexts.push(pr_contract::field(check, "/context")?.to_owned());
        }
    }
    contexts.sort();
    if contexts != ["check-js", "check-vize-apps", "fmt-rust", "test-report"] {
        return Err("Effective main required contexts changed".into());
    }
    let queue: Vec<_> = rules
        .iter()
        .filter(|rule| rule["type"] == "merge_queue")
        .collect();
    let [queue] = queue.as_slice() else {
        return Err("Exactly one effective merge queue required".into());
    };
    if queue["ruleset_id"].as_u64() != Some(24013395) {
        return Err("Effective merge queue authority changed".into());
    }
    if queue.pointer("/parameters/grouping_strategy") != Some(&json!("ALLGREEN")) {
        return Err("Protected integration requires ALLGREEN grouping".into());
    }
    Ok(())
}

const SNAPSHOT: &str = "query($owner:String!,$name:String!,$number:Int!){repository(owner:$owner,name:$name){nameWithOwner pullRequest(number:$number){id number state isDraft headRefOid headRefName baseRefName autoMergeRequest{enabledAt} mergeQueueEntry{id}}}}";
const ENQUEUE: &str = "mutation($pr:ID!,$head:GitObjectID!){enqueuePullRequest(input:{pullRequestId:$pr,expectedHeadOid:$head}){mergeQueueEntry{id pullRequest{id number headRefOid}}}}";

fn snapshot(
    candidate: &Candidate,
    stage: &str,
    allow_consumed: bool,
    adapter: &mut Adapter<'_>,
) -> Result<Value, String> {
    let (owner, name) = candidate
        .repository
        .split_once('/')
        .filter(|(owner, name)| !owner.is_empty() && !name.is_empty() && !name.contains('/'))
        .ok_or("Invalid queue repository")?;
    let value = adapter.graphql(
        stage,
        SNAPSHOT,
        &[
            ("owner", owner.into()),
            ("name", name.into()),
            ("number", candidate.number.to_string()),
        ],
        |_| Ok(()),
    )?;
    if value
        .pointer("/data/repository/nameWithOwner")
        .and_then(Value::as_str)
        != Some(&candidate.repository)
    {
        return Err("Queue repository changed".into());
    }
    let pr = value
        .pointer("/data/repository/pullRequest")
        .ok_or("Missing integration queue snapshot")?;
    node_id(pr, "/id")?;
    let consumed = allow_consumed
        && pr["state"] == "MERGED"
        && pr.get("mergeQueueEntry") == Some(&Value::Null);
    if pr["number"].as_u64() != Some(candidate.number)
        || (pr["state"] != "OPEN" && !consumed)
        || pr["isDraft"] != false
        || pr["headRefOid"] != candidate.head
        || pr["headRefName"] != candidate.branch
        || pr["baseRefName"] != "main"
        || pr.get("autoMergeRequest") != Some(&Value::Null)
    {
        return Err("Integration identity or unknown persistent auto-merge changed".into());
    }
    Ok(pr.clone())
}

fn node_id(value: &Value, pointer: &str) -> Result<String, String> {
    let id = pr_contract::field(value, pointer)?;
    if id.is_empty() || id.len() > 512 || !id.bytes().all(|byte| byte.is_ascii_graphic()) {
        return Err("Invalid queue node identity".into());
    }
    Ok(id.to_owned())
}

/// Caller first earns all ordinary I required checks via existing checks_pass.
/// Authentication must reverify I/source/pin and the same live O/attempt/lease/D.
pub(super) fn enqueue(
    candidate: &Candidate,
    authority: &Authority,
    verify: &mut impl FnMut(&Candidate) -> Result<Authority, String>,
    adapter: &mut Adapter<'_>,
    acquired: &mut Option<QueueOwner>,
) -> Result<(), String> {
    if acquired.is_some() {
        return Err("Existing owned queue capability must not be overwritten".into());
    }
    authenticate(candidate, authority, verify)?;
    effective_policy(candidate, adapter)?;
    let pr = snapshot(candidate, "before-enqueue", false, adapter)?;
    if pr.get("mergeQueueEntry") != Some(&Value::Null) {
        return Err("An existing or unknown queue entry cannot be adopted".into());
    }
    let pull_request_id = node_id(&pr, "/id")?;
    authenticate(candidate, authority, verify)?;
    adapter.graphql(
        "enqueue",
        ENQUEUE,
        &[
            ("pr", pull_request_id.clone()),
            ("head", candidate.head.clone()),
        ],
        |response| {
            let entry = response
                .pointer("/data/enqueuePullRequest/mergeQueueEntry")
                .ok_or("Missing acquired queue entry")?;
            let entry_id = node_id(entry, "/id")?;
            if entry.pointer("/pullRequest/id").and_then(Value::as_str) != Some(&pull_request_id)
                || entry.pointer("/pullRequest/number").and_then(Value::as_u64)
                    != Some(candidate.number)
                || entry
                    .pointer("/pullRequest/headRefOid")
                    .and_then(Value::as_str)
                    != Some(&candidate.head)
            {
                return Err("Acquired queue entry does not bind the expected integration".into());
            }
            *acquired = Some(QueueOwner {
                entry_id,
                pull_request_id: pull_request_id.clone(),
                number: candidate.number,
                head: candidate.head.clone(),
                authority: authority.clone(),
            });
            Ok(())
        },
    )?;
    // Ownership is recorded before fallible custody, post-read or authentication.
    let owner = acquired
        .as_ref()
        .ok_or("Missing authenticated acquired entry")?;
    let current = snapshot(candidate, "after-enqueue", false, adapter)?;
    if current["id"] != owner.pull_request_id
        || current
            .pointer("/mergeQueueEntry/id")
            .and_then(Value::as_str)
            != Some(&owner.entry_id)
    {
        return Err(
            "The newly acquired queue entry changed; retained response requires inspection".into(),
        );
    }
    authenticate(candidate, authority, verify)?;
    Ok(())
}

/// Dequeue has no expected-entry CAS. Exact pre/post reads plus the exclusive
/// cooperative O lease bound this operation; a privileged concurrent rewrite
/// cannot be made atomic across GitHub APIs. Never retry an ambiguous mutation.
pub(super) fn cleanup(
    candidate: &Candidate,
    owner: &QueueOwner,
    verify: &mut impl FnMut(&Candidate) -> Result<Authority, String>,
    adapter: &mut Adapter<'_>,
) -> Result<(), String> {
    if owner.number != candidate.number || owner.head != candidate.head {
        return Err("Cleanup ownership does not bind this exact integration".into());
    }
    authenticate(candidate, &owner.authority, verify)?;
    let pr = snapshot(candidate, "before-dequeue", true, adapter)?;
    if pr["id"] != owner.pull_request_id {
        return Err("Cleanup PR node changed".into());
    }
    authenticate(candidate, &owner.authority, verify)?;
    if pr.get("mergeQueueEntry") == Some(&Value::Null) {
        return Ok(());
    }
    if pr.pointer("/mergeQueueEntry/id").and_then(Value::as_str) != Some(&owner.entry_id) {
        return Err("A foreign or unknown queue entry must remain untouched".into());
    }
    // Genuine retained 9363 API evidence: DequeuePullRequestInput uses `id`,
    // not enqueue's `pullRequestId`; no fallback mutation is permitted.
    let response = adapter.graphql(
        "dequeue",
        "mutation($pr:ID!){dequeuePullRequest(input:{id:$pr}){mergeQueueEntry{id}}}",
        &[("pr", owner.pull_request_id.clone())],
        |_| Ok(()),
    )?;
    if response
        .pointer("/data/dequeuePullRequest/mergeQueueEntry/id")
        .and_then(Value::as_str)
        != Some(&owner.entry_id)
    {
        return Err("Dequeue returned a different entry; preserve complete evidence".into());
    }
    let current = snapshot(candidate, "after-dequeue", true, adapter)?;
    if current["id"] != owner.pull_request_id
        || current.get("mergeQueueEntry") != Some(&Value::Null)
    {
        return Err("Queue cleanup did not leave the exact integration unqueued".into());
    }
    authenticate(candidate, &owner.authority, verify)
}

#[cfg(test)]
#[path = "pr_pin_queue_tests.rs"]
mod tests;
