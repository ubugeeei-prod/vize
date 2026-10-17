import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { repoRoot } from "../_helpers/moonbit.ts";

const cases = [
  "cold",
  "failed-check",
  "existing-run",
  "existing-merged-run",
  "merged-without-run",
  "concurrent-run",
  "main-drift",
  "failed-main-drift",
  "head-drift",
  "branch-drift",
  "local-drift",
  "remote-drift",
  "merged-drift",
  "parent-drift",
  "base-drift",
  "tag-appeared",
  "version-drift",
  "candidate-drift",
  "budget-before",
  "budget-after",
  "budget-final",
  "lease-before",
  "lease-after",
  "lease-final",
];

test("actual legacy driver qualifies a cold cut before R and retains resume and drift safety", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-legacy-check-"));
  try {
    const watch = path.join(repoRoot, "tools/support/release/pr_watch.rs");
    const contract = path.join(repoRoot, "tools/support/release/pr_contract.rs");
    // rust-script's entry source must change when either included production module changes.
    const digest = createHash("sha256")
      .update(fs.readFileSync(watch))
      .update(fs.readFileSync(contract))
      .digest("hex");
    const script = path.join(root, "legacy-check.rs");
    fs.writeFileSync(script, harness(watch, contract, digest));
    const result = spawnSync("rust-script", ["--test", script], {
      encoding: "utf8",
      // rust-script 0.34 uses cargo test directly and does not forward script arguments.
      env: { ...process.env, RUST_TEST_NOCAPTURE: "1" },
      timeout: 180_000,
    });
    assert.equal(
      result.status,
      0,
      `${String(result.error ?? "")}\n${result.stdout}\n${result.stderr}`,
    );
    assert.match(result.stdout, /1 passed; 0 failed/);
    for (const name of cases) assert.ok(result.stdout.includes(`scenario ${name}: passed`), name);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

function harness(watch: string, contract: string, digest: string): string {
  return `//! \`\`\`cargo
//! [dependencies]
//! serde_json = "1"
//! [package]
//! edition = "2024"
//! \`\`\`
#![allow(dead_code)]
fn main() {} // Keep rust-script from wrapping module items inside an implicit main.
use serde_json::{json, Value};
use std::{cell::RefCell, path::Path, time::Duration};
const SOURCE_DIGEST: &str = "${digest}";
const REPO: &str = "owner/repo";
const HEAD: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const CUT: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const OTHER: &str = "cccccccccccccccccccccccccccccccccccccccc";
const BRANCH: &str = "release/v1.2.3";
const TAG: &str = "v1.2.3";
#[path = ${JSON.stringify(watch)}] mod pr_watch;
#[path = ${JSON.stringify(contract)}] mod pr_contract;
#[derive(Default)]
struct State { mode: String, events: Vec<String>, lists: usize, dispatches: usize,
    budget_checks: usize, lease_checks: usize, parent_reads: usize }
thread_local! { static STATE: RefCell<State> = RefCell::new(State::default()); }
fn mode(expected: &str) -> bool { STATE.with(|s| s.borrow().mode == expected) }
fn event(name: &str) { STATE.with(|s| s.borrow_mut().events.push(name.into())); }
fn unused<T>() -> Result<T, String> { panic!("unexpected backend operation outside find_or_dispatch") }
mod pr_budget {
    use super::*;
    pub struct Budget;
    pub fn configured() -> Result<Option<Budget>, String> { unused() }
    impl Budget {
        pub fn new(_: Duration) -> Self { Self }
        pub fn check(&self) -> Result<(), String> {
            let count = STATE.with(|s| { let mut s = s.borrow_mut(); s.budget_checks += 1; s.budget_checks });
            event("budget");
            if (mode("budget-before") && count == 1) || (mode("budget-after") && count == 2)
                || (mode("budget-final") && count == 3) { Err("budget expired".into()) } else { Ok(()) }
        }
        pub fn sleep(&self, _: Duration) -> Result<(), String> { self.check() }
    }
}
mod pr_pin { pub mod lock {
    use super::super::*;
    pub struct Operator;
    impl Operator { pub fn verify(&self) -> Result<(), String> {
        let count = STATE.with(|s| { let mut s = s.borrow_mut(); s.lease_checks += 1; s.lease_checks });
        event("lease");
        if (mode("lease-before") && count == 1) || (mode("lease-after") && count == 2)
            || (mode("lease-final") && count == 3) { Err("lease lost".into()) } else { Ok(()) }
    } }
} }
mod pr_ci {
    use super::*;
    pub fn qualify_check(repo: &str, branch: &str, head: &str, cut: &str,
        budget: Option<&pr_budget::Budget>, _: &Path) -> Result<(), String> {
        assert_eq!((repo, branch, head, cut), (REPO, BRANCH, HEAD, CUT));
        assert!(budget.is_some());
        event("qualify");
        if mode("failed-check") || mode("failed-main-drift") { Err("Check failed".into()) } else { Ok(()) }
    }
    pub fn published_identity(_: &pr_contract::Candidate, _: &Path) -> Result<(), String> { unused() }
}
mod pr_promote {
    use super::*;
    pub fn refresh(_: &pr_contract::Candidate, _: &str, _: &str, _: &Path) -> Result<(), String> { unused() }
    pub fn checks_pass(_: &pr_contract::Candidate, _: &Path) -> Result<bool, String> { unused() }
    pub fn promote(_: &pr_contract::Candidate, _: u64, _: &Path) -> Result<bool, String> { unused() }
}
mod pr_github {
    use super::*;
    pub fn api(repo: &str, resource: &str, _: &Path) -> Result<Value, String> {
        assert_eq!(repo, REPO);
        if resource.starts_with("actions/workflows/release.yml/runs?") {
            assert_eq!(resource, format!("actions/workflows/release.yml/runs?event=workflow_dispatch&head_sha={HEAD}&per_page=100"));
            let (lists, dispatched) = STATE.with(|s| { let mut s = s.borrow_mut(); s.lists += 1; (s.lists, s.dispatches > 0) });
            event("find-run");
            let id = if mode("existing-run") || mode("existing-merged-run") { Some(101) }
                else if mode("concurrent-run") && lists == 2 { Some(202) }
                else if dispatched { Some(303) } else { None };
            // Attractive foreign-head/title/event runs must never be treated as this R.
            let mut runs = vec![
                json!({"id":999,"head_sha":OTHER,"display_title":format!("Release {TAG} PR #42 @ {HEAD}"),"event":"workflow_dispatch"}),
                json!({"id":998,"head_sha":HEAD,"display_title":"foreign title","event":"workflow_dispatch"}),
                json!({"id":997,"head_sha":HEAD,"display_title":format!("Release {TAG} PR #42 @ {HEAD}"),"event":"push"})];
            if let Some(id) = id { runs.push(json!({"id":id,"head_sha":HEAD,
                "display_title":format!("Release {TAG} PR #42 @ {HEAD}"),"event":"workflow_dispatch"})); }
            return Ok(json!({"workflow_runs":runs}));
        }
        assert_eq!(resource, "pulls/42"); event("reauth-pr");
        Ok(json!({"head":{"sha":if mode("head-drift") {OTHER} else {HEAD},
            "ref":if mode("branch-drift") {"release/v2.0.0"} else {BRANCH}},"merged":mode("merged-drift")}))
    }
    pub fn parent(head: &str, _: &Path) -> Result<String, String> {
        assert_eq!(head, HEAD); event("parent");
        let reads = STATE.with(|s| { let mut s = s.borrow_mut(); s.parent_reads += 1; s.parent_reads });
        Ok(if mode("parent-drift") && reads > 1 {OTHER} else {CUT}.into())
    }
    pub fn git(args: &[&str], _: &Path) -> Result<String, String> {
        if args == ["rev-parse", "HEAD"] { event("local-head"); return Ok(if mode("local-drift") {OTHER} else {HEAD}.into()); }
        assert_eq!(args, ["ls-remote", "--heads", "origin", &format!("refs/heads/{BRANCH}")]);
        event("remote-head");
        Ok(format!("{} refs/heads/{BRANCH}", if mode("remote-drift") {OTHER} else {HEAD}))
    }
    pub fn candidate(repo: &str, number: u64, head: &str, tag: &str, allow_merged: bool, _: &Path)
        -> Result<pr_contract::Candidate, String> {
        assert_eq!((repo,number,head,tag,allow_merged), (REPO,42,HEAD,TAG,false)); event("candidate");
        Ok(pr_contract::Candidate {number:if mode("candidate-drift") {43} else {42},
            repository:REPO.into(),author:"maintainer".into(),branch:BRANCH.into(),head:HEAD.into(),
            base:if mode("base-drift") {OTHER} else {CUT}.into(),tag:TAG.into(),merged:false})
    }
    pub fn fetch_main(_: &Path) -> Result<String, String> { event("main");
        Ok(if mode("main-drift") || mode("failed-main-drift") {OTHER} else {CUT}.into()) }
    pub fn version(_: &Path) -> Result<String, String> { event("version");
        Ok(if mode("version-drift") {"2.0.0"} else {"1.2.3"}.into()) }
    pub fn tag_target(tag: &str, _: &Path) -> Result<Option<String>, String> {
        assert_eq!(tag,TAG); event("tag"); Ok(if mode("tag-appeared") {Some(HEAD.into())} else {None}) }
    pub fn output(program: &str, args: &[&str], _: &Path) -> Result<String, String> {
        assert_eq!(program,"gh");
        assert_eq!(args, ["workflow","run","release.yml","--repo",REPO,"--ref",BRANCH,
            "--field",&format!("tag_name={TAG}"),"--field","release_pr=42","--field",&format!("expected_sha={HEAD}")]);
        event("dispatch"); STATE.with(|s| s.borrow_mut().dispatches += 1); Ok(String::new())
    }
    pub fn protect_release_pr(_: &str, _: u64, _: &Path) -> Result<Value, String> { unused() }
    pub fn jobs(_: &str, _: u64, _: &Path) -> Result<Vec<Value>, String> { unused() }
    pub fn clean(_: &Path) -> Result<(), String> { unused() }
}
#[test]
fn actual_legacy_driver_handoff() {
    for scenario in ${JSON.stringify(cases)} {
        STATE.with(|s| *s.borrow_mut() = State {mode:scenario.into(), ..State::default()});
        let result = pr_watch::find_or_dispatch(REPO,42,TAG,HEAD,BRANCH,
            scenario == "merged-without-run" || scenario == "existing-merged-run", &pr_pin::lock::Operator, &pr_budget::Budget, Path::new("."));
        STATE.with(|s| {
            let s = s.borrow();
            match scenario {
                "cold" => {
                    assert_eq!(result,Ok(Some(303)));
                    assert_eq!(s.dispatches,1);
                    assert_eq!(s.events, ["find-run","parent","budget","lease","qualify","budget","lease",
                        "reauth-pr","local-head","remote-head","candidate","main","parent","version","tag",
                        "find-run","budget","lease","dispatch","budget","find-run"]);
                }
                "existing-run" | "existing-merged-run" => { assert_eq!(result,Ok(Some(101))); assert_eq!(s.events,["find-run"]); }
                "concurrent-run" => { assert_eq!(result,Ok(Some(202))); assert_eq!(s.dispatches,0); }
                "failed-check" => { assert_eq!(result,Err("Check failed".into())); assert!(s.events.iter().any(|e| e == "reauth-pr")); }
                "merged-without-run" => { assert!(result.unwrap_err().contains("Merged release PR")); assert_eq!(s.events,["find-run"]); }
                "version-drift" | "candidate-drift" => { assert!(result.unwrap_err().contains("identity or version changed")); }
                value if value.starts_with("budget-") => { assert_eq!(result,Err("budget expired".into())); }
                value if value.starts_with("lease-") => { assert_eq!(result,Err("lease lost".into())); }
                _ => assert_eq!(result,Ok(None),"{scenario}"),
            }
            if scenario != "cold" { assert_eq!(s.dispatches,0,"{scenario} dispatched stale R"); }
            if scenario.ends_with("-before") { assert!(!s.events.iter().any(|e| e == "qualify")); }
            if scenario != "existing-run" && scenario != "existing-merged-run" && scenario != "merged-without-run" && !scenario.ends_with("-before") {
                assert_eq!(s.events.iter().filter(|e| *e == "qualify").count(),1);
            }
        });
        println!("scenario {scenario}: passed");
    }
}
`;
}
