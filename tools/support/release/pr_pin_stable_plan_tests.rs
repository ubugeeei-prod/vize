use super::super::super::tests::Repo;
use super::*;
use serde_json::json;
use std::collections::BTreeMap;

fn fixture() -> (Repo, Value, Stable, BTreeMap<String, Value>) {
    let repo = Repo::new();
    let public = repo.commit(&[
        ("Cargo.toml", "[workspace.package]\nversion = \"0.443.0\"\n"),
        ("src/code.rs", "original\n"),
    ]);
    let original = repo.commit(&[
        ("src/code.rs", "fixed\n"),
        ("tests/original.txt", "whole original oracle\n"),
    ]);
    github::git(
        &[
            "-c",
            "core.hooksPath=/dev/null",
            "commit",
            "--amend",
            "-m",
            "fix: original\n\nCo-authored-by: Reporter <123+reporter@users.noreply.github.com>",
        ],
        &repo.work,
    )
    .unwrap();
    let merged = github::git(&["rev-parse", "HEAD"], &repo.work).unwrap();
    assert_ne!(original, merged);
    github::git(&["push", "origin", "main"], &repo.work).unwrap();
    github::git(&["checkout", "--detach", &public], &repo.work).unwrap();
    let cut = repo.commit(&[
        ("src/code.rs", "fixed\n"),
        ("tests/original.txt", "whole original oracle\n"),
    ]);
    let raw = String::from_utf8(
        metadata::bytes(
            &[
                "--no-replace-objects",
                "diff",
                "--raw",
                "--no-renames",
                "--no-abbrev",
                &public,
                &cut,
            ],
            &repo.work,
        )
        .unwrap(),
    )
    .unwrap();
    let tag_body = format!(
        "Release v0.443.0\n\nRelease-Mode: pinned\nValidated-head: {public}\nValidated-run: 7\nRelease-PR: #42"
    );
    github::git(
        &["tag", "-a", "v0.443.0", &public, "-m", &tag_body],
        &repo.work,
    )
    .unwrap();
    let object = github::git(&["rev-parse", "refs/tags/v0.443.0"], &repo.work).unwrap();
    let v = json!({"schema":"stable-patch-v1","repository":"owner/repo","published":{"tag":"v0.443.0","tagObject":object,"head":public,"tree":commit(&public,&repo.work).unwrap().0,"run":7,"sourcePr":42},"target":"v0.443.1","cut":cut,"tree":commit(&cut,&repo.work).unwrap().0,"commits":[cut],"fixes":[{"pr":99,"merge":merged,"parent":public,"reporter":"Reporter <123+reporter@users.noreply.github.com>"}],"rawDiff":raw,"files":[{"path":"src/code.rs","role":"fix","origin":merged},{"path":"tests/original.txt","role":"corpus","origin":merged}]});
    github::git(&["checkout", "main"], &repo.work).unwrap();
    let policy = repo.commit(&[
        (PLAN_PATH, &format!("{v}\n")),
        ("main-only.css", "future nominal migration\n"),
    ]);
    github::git(&["push", "origin", "main"], &repo.work).unwrap();
    let context = Stable {
        policy: policy.clone(),
        blob: github::git(&["rev-parse", &format!("{policy}:{PLAN_PATH}")], &repo.work).unwrap(),
        sha256: super::super::digest(format!("{v}\n").as_bytes()).unwrap(),
    };
    let api = BTreeMap::from([
        (
            "git/ref/tags/v0.443.0".into(),
            json!({"object":{"type":"tag","sha":object}}),
        ),
        (
            format!("git/tags/{object}"),
            json!({"tag":"v0.443.0","object":{"type":"commit","sha":public},"message":tag_body}),
        ),
        (
            "releases/tags/v0.443.0".into(),
            json!({"draft":false,"prerelease":false,"tag_name":"v0.443.0","published_at":"2026-10-10T22:24:22Z"}),
        ),
        (
            "actions/runs/7".into(),
            json!({"id":7,"event":"workflow_dispatch","status":"completed","conclusion":"success","head_sha":public,"head_branch":"release/v0.443.0","path":".github/workflows/release.yml","head_repository":{"full_name":"owner/repo"}}),
        ),
        (
            "pulls/99".into(),
            json!({"merged":true,"state":"closed","merge_commit_sha":merged,"merged_at":"2026-10-11T00:00:00Z","base":{"repo":{"full_name":"owner/repo"}},"head":{"repo":{"full_name":"owner/repo"}}}),
        ),
    ]);
    (repo, v, context, api)
}

fn verify_fixture(
    repo: &Repo,
    v: &Value,
    c: &Stable,
    api: &BTreeMap<String, Value>,
) -> Result<(), String> {
    verify_with(
        v,
        c,
        field(v, "/cut")?,
        "owner/repo",
        &repo.work,
        &|resource| {
            api.get(resource)
                .cloned()
                .ok_or(format!("Unexpected API or mutation: {resource}"))
        },
    )
}

#[test]
fn exact_patch_only() {
    assert!(patch_target("v0.443.0", "v0.443.1").is_ok());
    for target in [
        "v0.444.0",
        "v0.443.0",
        "v0.443.2",
        "v0.443.1-rc.1",
        "v00.443.1",
        "v1.443.1",
    ] {
        assert!(patch_target("v0.443.0", target).is_err(), "{target}");
    }
}

#[test]
fn actual_detached_stable_chain_and_complete_bodies_qualify_without_main_source_substitution() {
    let (repo, v, c, api) = fixture();
    let main = github::git(&["rev-parse", "origin/main"], &repo.work).unwrap();
    assert!(first_parent(field(&v, "/cut").unwrap(), &main, &repo.work).is_err());
    assert!(metadata::text(field(&v, "/cut").unwrap(), "main-only.css", &repo.work).is_err());
    verify_fixture(&repo, &v, &c, &api).unwrap();
    assert_eq!(
        github::git(
            &["rev-parse", &format!("{}:{PLAN_PATH}", c.policy)],
            &repo.work
        )
        .unwrap(),
        c.blob
    );
}

#[test]
fn complete_real_git_plan_refuses_omission_substitution_extra_roles_and_changed_parent_identity() {
    let (repo, v, c, api) = fixture();
    let mutations = [
        ("/schema", json!("unknown")),
        ("/repository", json!("foreign/repo")),
        ("/cut", json!(c.policy)),
        ("/tree", json!("a".repeat(40))),
        ("/published/head", json!(c.policy)),
        ("/published/tree", json!("a".repeat(40))),
        ("/published/run", json!(0)),
        ("/published/sourcePr", json!(0)),
        ("/target", json!("v0.444.0")),
        ("/target", json!("v0.443.2")),
        ("/commits", json!([])),
        ("/commits", json!([v["cut"], v["cut"]])),
        ("/fixes", json!([])),
        ("/fixes/0/parent", json!(c.policy)),
        ("/fixes/0/reporter", json!("Forged <wrong@example.invalid>")),
        ("/rawDiff", json!("")),
        ("/files", json!([])),
        ("/files/0/role", json!("ignored")),
        ("/files/0/role", json!("qualification")),
        ("/files/0/role", json!("corpus-adapter")),
        ("/files/0/origin", json!(c.policy)),
        ("/files/1/path", json!("../tests/original.txt")),
    ];
    for (pointer, value) in mutations {
        let mut bad = v.clone();
        *bad.pointer_mut(pointer).unwrap() = value;
        assert!(verify_fixture(&repo, &bad, &c, &api).is_err(), "{pointer}");
    }
    let mut bad = v.clone();
    bad["files"]
        .as_array_mut()
        .unwrap()
        .push(v["files"][0].clone());
    assert!(verify_fixture(&repo, &bad, &c, &api).is_err());
    let mut bad = v.clone();
    bad["unknownOverride"] = json!(true);
    assert!(verify_fixture(&repo, &bad, &c, &api).is_err());
}

#[test]
fn actual_publication_fix_and_tag_responses_refuse_old_green_or_changed_public_authority() {
    let (repo, v, c, api) = fixture();
    for (route, pointer, value) in [
        (
            "git/ref/tags/v0.443.0",
            "/object/sha",
            json!("a".repeat(40)),
        ),
        ("git/ref/tags/v0.443.0", "/object/type", json!("commit")),
        ("actions/runs/7", "/conclusion", json!("failure")),
        ("actions/runs/7", "/head_sha", json!(c.policy)),
        (
            "actions/runs/7",
            "/path",
            json!(".github/workflows/check.yml"),
        ),
        ("releases/tags/v0.443.0", "/draft", json!(true)),
        ("releases/tags/v0.443.0", "/prerelease", json!(true)),
        ("pulls/99", "/merged", json!(false)),
        ("pulls/99", "/merge_commit_sha", json!(v["cut"])),
        ("pulls/99", "/head/repo/full_name", json!("foreign/repo")),
    ] {
        let mut bad = api.clone();
        *bad.get_mut(route).unwrap().pointer_mut(pointer).unwrap() = value;
        assert!(
            verify_fixture(&repo, &v, &c, &bad).is_err(),
            "{route}{pointer}"
        );
    }
}

#[test]
fn actual_replacement_ref_cannot_invent_stable_inclusion_or_change_whole_source_bodies() {
    let (repo, v, c, api) = fixture();
    github::git(
        &["replace", field(&v, "/cut").unwrap(), &c.policy],
        &repo.work,
    )
    .unwrap();
    let error = verify_fixture(&repo, &v, &c, &api).unwrap_err();
    assert!(error.contains("replacement"), "{error}");
    assert_eq!(
        commit(field(&v, "/cut").unwrap(), &repo.work).unwrap().0,
        field(&v, "/tree").unwrap()
    );
}
