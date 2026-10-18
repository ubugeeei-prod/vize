use super::super::tests::compensation_deadline as deadline;
use super::super::{pr_budget::Budget, pr_github as github};
use super::{
    lock, queue,
    tests::{Repo, source},
};
use serde_json::{Value, json};
use std::{
    cell::RefCell,
    fs, thread,
    time::{Duration, Instant},
};

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
