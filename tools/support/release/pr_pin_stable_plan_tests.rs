use super::super::super::tests::Repo;
use super::*;
use serde_json::json;
use std::collections::BTreeMap;

pub(super) fn fixture() -> (Repo, Value, Stable, BTreeMap<String, Value>) {
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
    let mut cut_inputs = vec![
        ("src/code.rs", "fixed\n"),
        ("tests/original.txt", "whole original oracle\n"),
        ("tests/wired.rs", "complete independent source adapter\n"),
        (
            "tools/support/release/transport.rs",
            "complete reviewed transport\n",
        ),
    ];
    cut_inputs.extend(
        super::super::protocol_paths()
            .iter()
            .map(|p| (*p, "whole inert reviewed protocol fixture\n")),
    );
    let cut = repo.commit(&cut_inputs);
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
    let mut v = json!({"schema":"stable-patch-v1","repository":"owner/repo","published":{"tag":"v0.443.0","tagObject":object,"head":public,"tree":commit(&public,&repo.work).unwrap().0,"run":7,"sourcePr":42},"target":"v0.443.1","cut":cut,"tree":commit(&cut,&repo.work).unwrap().0,"commits":[cut],"fixes":[{"pr":99,"merge":merged,"parent":public,"reporter":"Reporter <123+reporter@users.noreply.github.com>"}],"rawDiff":raw,"files":[{"path":"src/code.rs","role":"fix","origin":merged},{"path":"tests/original.txt","role":"corpus","origin":merged},{"path":"tests/wired.rs","role":"corpus-adapter","origin":""},{"path":"tools/support/release/transport.rs","role":"qualification","origin":""}]});
    for path in super::super::protocol_paths() {
        v["files"]
            .as_array_mut()
            .unwrap()
            .push(json!({"path":path,"role":"qualification","origin":""}));
    }
    github::git(&["checkout", "main"], &repo.work).unwrap();
    let plan_text = format!("{v}\n");
    let mut policy_inputs = vec![
        (PLAN_PATH, plan_text.as_str()),
        ("main-only.css", "future nominal migration\n"),
        (
            "tools/support/release/transport.rs",
            "complete reviewed transport\n",
        ),
    ];
    policy_inputs.extend(
        super::super::protocol_paths()
            .iter()
            .map(|p| (*p, "whole inert reviewed protocol fixture\n")),
    );
    let policy = repo.commit(&policy_inputs);
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

#[test]
fn partial_stable_resume_authenticates_before_any_real_operator_ref_or_integration_mutation() {
    let (repo, v, c, api) = fixture();
    let cut = field(&v, "/cut").unwrap();
    let head = cut;
    let body = format!(
        "<!-- vize-release-bump: patch -->\n{}",
        super::super::markers(Some(&c))
    );
    let authenticate = |body: &str, tag: &str| {
        super::super::resume_authority(body, tag, cut, &repo.work, &|| {
            super::super::body_with(body, &|policy| {
                assert_eq!(policy, c.policy);
                verify_fixture(&repo, &v, &c, &api)?;
                Ok(c.clone())
            })
        })
    };
    let before = github::git(&["show-ref"], &repo.remote).unwrap();
    for (bad, tag) in [
        (body.replace("bump: patch", "bump: minor"), "v0.443.1"),
        (body.clone(), "v0.443.2"),
        (
            format!("{body}<!-- vize-release-stable-unknown: yes -->\n"),
            "v0.443.1",
        ),
        (
            format!("{body}{}", super::super::markers(Some(&c))),
            "v0.443.1",
        ),
        (body.replace(&c.sha256, &"a".repeat(64)), "v0.443.1"),
        ("<!-- vize-release-bump: patch -->\n".into(), "v0.443.1"),
    ] {
        let result = super::super::super::start::resume_operator(tag, head, &repo.work, &|| {
            authenticate(&bad, tag)
        });
        assert!(result.is_err(), "{tag}: {bad}");
        assert_eq!(github::git(&["show-ref"], &repo.remote).unwrap(), before);
        assert!(
            !repo
                .work
                .join(format!(".git/vize-release-operator-{tag}.lock"))
                .exists()
        );
    }
    let (context, operator) =
        super::super::super::start::resume_operator("v0.443.1", head, &repo.work, &|| {
            authenticate(&body, "v0.443.1")
        })
        .unwrap();
    assert_eq!(context.unwrap().blob, c.blob);
    assert!(
        github::git(
            &["show-ref", "refs/heads/release-operator/v0.443.1"],
            &repo.remote
        )
        .is_ok()
    );
    drop(operator);
    assert_eq!(github::git(&["show-ref"], &repo.remote).unwrap(), before);
}
