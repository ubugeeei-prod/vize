use super::super::tests::compensation_deadline as deadline;
use super::super::{pr_budget::Budget, pr_github as github};
use super::{
    body, lock, queue,
    tests::{Repo, source},
    watcher,
};
use serde_json::{Value, json};
use std::{
    cell::RefCell,
    fs, thread,
    time::{Duration, Instant},
};

const GITHUB_COMPENSATION_FIXTURE: &str = r#"
import fs from 'node:fs';
const request=JSON.parse(process.argv[1]);
const data=JSON.parse(fs.readFileSync(request.data,'utf8'));
const args=request.args;
fs.appendFileSync(request.log,JSON.stringify(args)+'\n');
if(args[0]!=='api') throw Error('Only native API fixture calls allowed');
if(args[1]!=='graphql') {
  if(!Object.hasOwn(data,args[1])) throw Error('Unknown read '+args[1]);
  process.stdout.write(JSON.stringify(data[args[1]]));
} else {
  const query=args.find(value=>value.startsWith('query='));
  let entry=JSON.parse(fs.readFileSync(request.state,'utf8'));
  if(query.includes('enqueuePullRequest')) {
    if(entry!==null) throw Error('Entry already exists');
    entry={id:'MQE_owned'}; fs.writeFileSync(request.state,JSON.stringify(entry));
    process.stdout.write(JSON.stringify({data:{enqueuePullRequest:{mergeQueueEntry:{...entry,pullRequest:{id:'PR_exact',number:43,headRefOid:data.I}}}}}));
  } else if(query.includes('dequeuePullRequest')) {
    fs.writeFileSync(request.state,'null');
    process.stdout.write(JSON.stringify({data:{dequeuePullRequest:{mergeQueueEntry:entry}}}));
  } else {
    process.stdout.write(JSON.stringify({data:{repository:{nameWithOwner:'owner/repo',pullRequest:{id:'PR_exact',number:43,state:'OPEN',isDraft:false,headRefOid:data.I,headRefName:'release-integration/v1.2.3',baseRefName:'main',autoMergeRequest:null,mergeQueueEntry:entry}}}}));
  }
}
"#;

#[test]
fn compensation_runner_sigkills_reaps_and_retains_whole_process_failure_packets() {
    let repo = Repo::new();
    let packet = |script: &str, d| {
        super::super::tests::direct_compensation_packet(
            queue::COMPENSATION_RUNNER,
            script,
            d,
            &repo.work,
        )
    };
    let started = Instant::now();
    let child = "process.on('SIGTERM',()=>{});process.stdout.write(Buffer.from([0,255,10]));process.stderr.write(Buffer.from([254,1]));setInterval(()=>{},1000)";
    let timed = packet(child, deadline(1000));
    assert!(started.elapsed() < Duration::from_secs(4));
    assert_eq!(timed["signal"], "SIGKILL");
    assert_eq!(timed["timeout"], true);
    assert_eq!(timed["stdout"], json!([0, 255, 10]));
    assert_eq!(timed["stderr"], json!([254, 1]));
    let pid = timed["pid"].as_u64().unwrap();
    assert!(pid > 0);
    let check = format!(
        "try{{process.kill({pid},0);process.exit(1)}}catch(error){{if(error.code!=='ESRCH')throw error;process.stdout.write('reaped')}}"
    );
    let reaped = packet(&check, deadline(1000));
    assert_eq!(reaped["status"], 0);
    assert_eq!(
        serde_json::from_value::<Vec<u8>>(reaped["stdout"].clone()).unwrap(),
        b"reaped"
    );
    let exited = packet(
        "process.stdout.write('whole stdout');process.stderr.write('whole stderr');process.exit(17)",
        deadline(1000),
    );
    assert_eq!(exited["status"], 17);
    assert_eq!(exited["timeout"], false);
    assert_eq!(exited["outputComplete"], true);
    assert_eq!(
        serde_json::from_value::<Vec<u8>>(exited["stdout"].clone()).unwrap(),
        b"whole stdout"
    );
    assert_eq!(
        serde_json::from_value::<Vec<u8>>(exited["stderr"].clone()).unwrap(),
        b"whole stderr"
    );
    let overflow = packet(
        "process.stdout.write(Buffer.alloc(3*1024*1024,97))",
        deadline(1000),
    );
    assert_eq!(overflow["outputComplete"], false);
    assert!(!overflow["stdout"].as_array().unwrap().is_empty());
    let d = deadline(600);
    assert_eq!(packet("setTimeout(()=>{},150)", d)["status"], 0);
    assert_eq!(packet("setInterval(()=>{},1000)", d)["timeout"], true);
    assert!(queue::Compensation::limited(15_001).is_err());
}

#[test]
fn gnu_outer_timeout_bounds_hung_node_and_inherited_stdio_descendants() {
    let repo = Repo::new();
    let pids = repo.work.join("processes.jsonl");
    let path = serde_json::to_string(&pids).unwrap();
    let outer = format!(
        "const fs=require('node:fs');fs.appendFileSync({path},JSON.stringify({{role:'launcher',pid:process.ppid}})+'\\n'+JSON.stringify({{role:'outer',pid:process.pid}})+'\\n');process.stdout.write(Buffer.from([0,255]));process.stderr.write(Buffer.from([254]));setInterval(()=>{{}},1000)"
    );
    let descendant = format!(
        "require('node:fs').appendFileSync({path},JSON.stringify({{role:'descendant',pid:process.pid}})+'\\n');setInterval(()=>{{}},1000)"
    );
    let inherited = format!(
        "const fs=require('node:fs');fs.appendFileSync({path},JSON.stringify({{role:'gh',pid:process.pid}})+'\\n'+JSON.stringify({{role:'bridge',pid:process.ppid}})+'\\n');require('node:child_process').spawn(process.execPath,['-e',{}],{{stdio:'inherit'}});process.exit(0)",
        serde_json::to_string(&descendant).unwrap()
    );
    for (index, script) in [outer, inherited].iter().enumerate() {
        fs::write(&pids, "").unwrap();
        let packets = RefCell::new(Vec::new());
        let mut retain = |_: &str, value: &Value| {
            packets.borrow_mut().push(value.clone());
            Ok(())
        };
        let clock = queue::Compensation::limited(1000).unwrap();
        let started = Instant::now();
        let result = if index == 0 {
            clock
                .outer_fixture(script, &repo.work, &mut retain)
                .map(|_| ())
        } else {
            clock
                .output("node", &["-e", script], &repo.work, &mut retain)
                .map(|_| ())
        };
        assert!(result.is_err());
        assert!(started.elapsed() < Duration::from_secs(4));
        let packets = packets.borrow();
        let packet = packets.last().unwrap();
        assert!(packet["signal"] == 9 || packet["status"] == 137);
        assert_eq!(packet["innerOutput"], "missing-or-partial");
        if index == 0 {
            assert_eq!(packet["stdout"], json!([0, 255]));
            assert_eq!(packet["stderr"], json!([254]));
        }
        assert_eq!(
            fs::read_to_string(&pids).unwrap().lines().count(),
            if index == 0 { 2 } else { 3 }
        );
        super::super::tests::assert_compensation_processes_stopped(&repo.work, &pids);
    }
}

#[test]
fn expired_original_budget_uses_actual_watch_finish_to_withdraw_only_acquired_entry() {
    let repo = Repo::new();
    let cut = repo.commit(&[("Cargo.toml", "[workspace.package]\nversion = \"1.2.2\"\n")]);
    github::git(&["push", "origin", "main"], &repo.work).unwrap();
    let head = repo.commit(&[("Cargo.toml", "[workspace.package]\nversion = \"1.2.3\"\n")]);
    github::git(
        &[
            "push",
            "origin",
            &format!("{head}:refs/heads/release/v1.2.3"),
        ],
        &repo.work,
    )
    .unwrap();
    let tree = github::git(&["rev-parse", "HEAD^{tree}"], &repo.work).unwrap();
    let i = github::git(
        &[
            "commit-tree",
            &tree,
            "-p",
            &cut,
            "-m",
            "version integration",
        ],
        &repo.work,
    )
    .unwrap();
    github::git(
        &[
            "push",
            "origin",
            &format!("{i}:refs/heads/release-integration/v1.2.3"),
        ],
        &repo.work,
    )
    .unwrap();
    let source = source(&head, &cut);
    let budget = Budget::new(Duration::from_secs(5));
    let deadline = budget.child_allowance().unwrap().1;
    let operator = lock::Operator::compensation_fixture(&source, deadline, &repo.work).unwrap();
    lock::install_pin(&source, &operator, &repo.work).unwrap();
    let lease = github::git(
        &["rev-parse", "refs/heads/release-operator/v1.2.3"],
        &repo.remote,
    )
    .unwrap();
    let pin = github::git(
        &["rev-parse", "refs/heads/release-pin/v1.2.3"],
        &repo.remote,
    )
    .unwrap();
    let owner_before = fs::read(
        repo.work
            .join(".git/vize-release-operator-v1.2.3.lock/owner.txt"),
    )
    .unwrap();
    let mut candidate = source.candidate.clone();
    candidate.number = 43;
    candidate.head = i.clone();
    candidate.branch = "release-integration/v1.2.3".into();
    let pr = |number, draft, branch: &str, head: &str, body: String| {
        json!({"number":number,"draft":draft,"state":"open","merged":false,"body":body,
        "head":{"sha":head,"ref":branch,"repo":{"full_name":"owner/repo"}},"base":{"ref":"main","repo":{"full_name":"owner/repo"}},"user":{"type":"User","login":"maintainer"}})
    };
    let source_body = format!(
        "<!-- vize-release-pin: immutable-v1 -->\n<!-- vize-release-bump: minor -->\n<!-- vize-release-pin-head: {head} -->\n<!-- vize-release-pin-cut: {cut} -->\n<!-- vize-release-base-version: 1.2.2 -->\n<!-- vize-release-integration: 43 -->\n"
    );
    let reference = |branch: &str, id: &str| json!({"ref":format!("refs/heads/{branch}"),"object":{"type":"commit","sha":id}});
    let mut data = json!({"I":i,
        "repos/owner/repo/pulls/42":pr(42,true,"release/v1.2.3",&head,source_body),
        "repos/owner/repo/pulls/43":pr(43,false,"release-integration/v1.2.3",&i,body::generated(42,&cut,&head,"v1.2.3")),
        "repos/owner/repo/collaborators/maintainer/permission":{"role_name":"maintain"},
        "repos/owner/repo/actions/runs/7":{"id":7,"run_attempt":2,"head_sha":cut,"head_branch":"main","event":"workflow_dispatch","path":".github/workflows/release-operator.yml","status":"in_progress","conclusion":null,"head_repository":{"full_name":"owner/repo"},"actor":{"login":"maintainer"},"triggering_actor":{"login":"maintainer"}},
        "repos/owner/repo/rules/branches/main":[{"type":"required_status_checks","parameters":{"required_status_checks":["check-js","check-vize-apps","fmt-rust","test-report"].map(|context|json!({"context":context,"integration_id":15368}))}},{"type":"merge_queue","ruleset_id":24013395,"parameters":{"grouping_strategy":"ALLGREEN"}}]
    });
    for (branch, id) in [
        ("release/v1.2.3", &head),
        ("release-integration/v1.2.3", &i),
        ("release-operator/v1.2.3", &lease),
        ("release-pin/v1.2.3", &pin),
    ] {
        data[format!("repos/owner/repo/git/ref/heads/{branch}")] = reference(branch, id);
    }
    for id in [&head, &lease, &pin] {
        data[format!("repos/owner/repo/git/commits/{id}")] =
            super::super::tests::git_commit_packet(&repo.work, id);
    }
    let data_path = repo.work.join("github.json");
    let state = repo.work.join("entry.json");
    let log = repo.work.join("requests.jsonl");
    fs::write(&data_path, data.to_string()).unwrap();
    fs::write(&state, "null").unwrap();
    let packets = RefCell::new(Vec::new());
    let retain = |_: &str, value: &Value| {
        packets.borrow_mut().push(value.clone());
        Ok(())
    };
    let execute =
        |clock: &queue::Compensation, program: &str, args: &[&str], root: &std::path::Path| {
            assert_eq!(program, "gh");
            let input = json!({"data":data_path,"state":state,"log":log,"args":args}).to_string();
            clock.output(
                "node",
                &[
                    "--input-type=module-typescript",
                    "-e",
                    GITHUB_COMPENSATION_FIXTURE,
                    &input,
                ],
                root,
                &mut &retain,
            )
        };
    let admission = queue::Compensation::new().unwrap();
    let mut run = |program: &str, args: &[&str], root: &std::path::Path| {
        execute(&admission, program, args, root)
    };
    let mut retain_api = |_: &str, _: &Value| Ok(());
    let authority = queue::Authority {
        lease: lease.clone(),
        operator_run: 7,
        operator_attempt: 2,
    };
    let mut verify = |_: &super::super::pr_contract::Candidate| Ok(authority.clone());
    let mut acquired = None;
    queue::enqueue(
        &candidate,
        &authority,
        &mut verify,
        &mut queue::Adapter {
            root: &repo.work,
            execute: &mut run,
            retain: &mut retain_api,
        },
        &mut acquired,
    )
    .unwrap();
    assert!(
        budget.check().is_ok(),
        "actual lease and native queue acquisition occur before original D"
    );
    while budget.check().is_ok() {
        thread::sleep(Duration::from_millis(5));
    }
    let failure = budget.check().unwrap_err();
    let owner = acquired.as_ref().unwrap();
    let context = watcher::Rollback {
        pinned: &source,
        candidate: &candidate,
        owner,
        operator: &operator,
        actor: "maintainer",
        root: &repo.work,
    };
    for (endpoint, pointer, replacement) in [
        ("pulls/42".into(), "/head/sha", json!("4".repeat(40))),
        ("pulls/43".into(), "/head/sha", json!("4".repeat(40))),
        ("actions/runs/7".into(), "/run_attempt", json!(3)),
        (
            "git/ref/heads/release-operator/v1.2.3".into(),
            "/object/sha",
            json!("4".repeat(40)),
        ),
        (
            "collaborators/maintainer/permission".into(),
            "/role_name",
            json!("write"),
        ),
        (
            format!("git/commits/{pin}"),
            "/message",
            json!("forged pin"),
        ),
    ] {
        let mut changed = data.clone();
        *changed[format!("repos/owner/repo/{endpoint}")]
            .pointer_mut(pointer)
            .unwrap() = replacement;
        fs::write(&data_path, changed.to_string()).unwrap();
        let clock = queue::Compensation::new().unwrap();
        let error = watcher::finish_owned_watch(&failure, &context, |program, args, root| {
            execute(&clock, program, args, root)
        })
        .unwrap_err();
        assert!(error.starts_with(&failure));
        assert!(error.contains("withdrawal refused"));
        assert_eq!(
            fs::read_to_string(&state).unwrap(),
            "{\"id\":\"MQE_owned\"}"
        );
        assert_eq!(super::super::tests::queue_mutations(&log), 1);
    }
    fs::write(&data_path, data.to_string()).unwrap();
    fs::write(&state, "{\"id\":\"MQE_foreign\"}").unwrap();
    let clock = queue::Compensation::new().unwrap();
    assert!(
        watcher::finish_owned_watch(&failure, &context, |program, args, root| execute(
            &clock, program, args, root
        ))
        .unwrap_err()
        .contains("foreign")
    );
    assert_eq!(
        fs::read_to_string(&state).unwrap(),
        "{\"id\":\"MQE_foreign\"}"
    );
    assert_eq!(super::super::tests::queue_mutations(&log), 1);
    fs::write(&state, "{\"id\":\"MQE_owned\"}").unwrap();
    let compensation = queue::Compensation::new().unwrap();
    let offset = packets.borrow().len();
    let started = Instant::now();
    let result = watcher::finish_owned_watch(&failure, &context, |program, args, root| {
        execute(&compensation, program, args, root)
    });
    assert_eq!(result.unwrap_err(), failure);
    assert!(started.elapsed() < Duration::from_secs(15));
    assert_eq!(fs::read_to_string(&state).unwrap(), "null");
    assert_eq!(
        fs::read(
            repo.work
                .join(".git/vize-release-operator-v1.2.3.lock/owner.txt")
        )
        .unwrap(),
        owner_before
    );
    let observed: Vec<Value> = packets.borrow()[offset..]
        .iter()
        .map(|value| {
            value["raw"]
                .as_str()
                .map_or_else(|| value.clone(), |raw| serde_json::from_str(raw).unwrap())
        })
        .collect();
    let d = observed[0]["deadline"].clone();
    assert!(observed.iter().all(|value| value["deadline"] == d));
    let mut outer = observed.iter().filter(|v| v.get("innerOutput").is_some());
    assert!(outer.all(|v| v["status"] == 0 && v["signal"].is_null()));
    assert_eq!(super::super::tests::queue_mutations(&log), 2);
    assert!(budget.check().is_err());
    lock::verify_pin(&source, &repo.work).unwrap();
    assert_eq!(
        github::git(&["rev-parse", "refs/heads/main"], &repo.remote).unwrap(),
        cut
    );
    assert!(github::tag_target("v1.2.3", &repo.work).unwrap().is_none());
}

#[test]
fn expected_hosted_cut_is_strict_and_does_not_restrict_unconfigured_local_cuts() {
    let cut = "1111111111111111111111111111111111111111";
    let verify = super::start::expected_cut;
    assert!(verify(None, cut).is_ok());
    assert!(verify(Some(cut), cut).is_ok());
    for expected in [
        "",
        "main",
        "1111111",
        "2222222222222222222222222222222222222222",
    ] {
        assert!(verify(Some(expected), cut).is_err());
    }
}

fn expire() -> Result<(), String> {
    let budget = Budget::new(Duration::from_millis(1));
    thread::sleep(Duration::from_millis(2));
    budget.check()
}

#[test]
fn operator_timeout_releases_its_real_lease_and_preserves_resumable_source_and_pin() {
    let repo = Repo::new();
    let cut = repo.commit(&[("Cargo.toml", "[workspace.package]\nversion = \"1.2.2\"\n")]);
    github::git(&["push", "origin", "main"], &repo.work).unwrap();
    let head = repo.commit(&[("Cargo.toml", "[workspace.package]\nversion = \"1.2.3\"\n")]);
    github::git(
        &[
            "push",
            "origin",
            &format!("{head}:refs/heads/release/v1.2.3"),
        ],
        &repo.work,
    )
    .unwrap();
    let source = source(&head, &cut);
    let directory = repo.work.join(".git/vize-release-operator-v1.2.3.lock");
    let result = (|| {
        let operator = lock::acquire("v1.2.3", &head, &repo.work)?;
        lock::install_pin(&source, &operator, &repo.work)?;
        assert!(directory.join("remote-head.txt").is_file());
        expire()
    })();
    assert!(result.unwrap_err().contains("budget expired"));
    assert!(!directory.exists());
    assert!(
        github::git(
            &[
                "ls-remote",
                "--refs",
                "origin",
                "refs/heads/release-operator/v1.2.3"
            ],
            &repo.work
        )
        .unwrap()
        .is_empty()
    );
    lock::verify_pin(&source, &repo.work).unwrap();
    assert_eq!(
        github::git(&["rev-parse", "refs/heads/main"], &repo.remote).unwrap(),
        cut
    );
    assert!(github::tag_target("v1.2.3", &repo.work).unwrap().is_none());
    let resumed = lock::acquire("v1.2.3", &head, &repo.work).unwrap();
    resumed.verify().unwrap();
    lock::verify_pin(&source, &repo.work).unwrap();
    drop(resumed);
    assert!(!directory.exists());
}

#[test]
fn operator_timeout_never_deletes_a_changed_remote_lease() {
    let repo = Repo::new();
    let head = repo.commit(&[("source.rs", "// original source\n")]);
    github::git(&["push", "origin", "main"], &repo.work).unwrap();
    let directory = repo.work.join(".git/vize-release-operator-v1.2.3.lock");
    let result = (|| {
        let _operator = lock::acquire("v1.2.3", &head, &repo.work)?;
        github::git(
            &["update-ref", "refs/heads/release-operator/v1.2.3", &head],
            &repo.remote,
        )?;
        expire()
    })();
    assert!(result.is_err());
    assert_eq!(
        github::git(
            &["rev-parse", "refs/heads/release-operator/v1.2.3"],
            &repo.remote
        )
        .unwrap(),
        head
    );
    assert!(directory.join("owner.txt").is_file());
    assert!(directory.join("remote-head.txt").is_file());
    assert!(lock::acquire("v1.2.3", &head, &repo.work).is_err());
    // Test fixture cleanup has no remote ownership recovery semantics.
    fs::remove_dir_all(directory).unwrap();
}
