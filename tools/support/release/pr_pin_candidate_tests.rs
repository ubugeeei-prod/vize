use super::super::tests::{Repo, source};
use super::*;

fn bump(repo: &Repo, parent: &str) -> String {
    for path in metadata::paths(parent, &repo.work).unwrap() {
        let before = metadata::text(parent, &path, &repo.work).unwrap();
        let after = metadata::rewrite(&path, &before, "1.2.2", "1.2.3");
        if before != after {
            fs::write(repo.work.join(path), after).unwrap();
        }
    }
    repo.commit(&[])
}

#[test]
fn original_v0436_script_drift_is_rejected_before_metadata_delivery() {
    let packet: Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/release/pinned-catalog-script-drift.json"
    ))
    .unwrap();
    assert_eq!(packet["H"], "3390cb6044d53d655a9d64e112d2618375cff5ca");
    assert_eq!(packet["V"], "f9cfdaa0e84152ab9f11382e1b49e501d1ea44b6");
    assert_eq!(packet["path"], "/npm/vize/manifest/scripts/test");
    let repo = Repo::new();
    let manifest = |script: &Value| {
        format!(
            "{{\n  \"name\": \"vize\",\n  \"version\": \"1.2.2\",\n  \"scripts\": {{ \"test\": {script} }}\n}}\n"
        )
    };
    repo.commit(&[
        ("Cargo.toml", "[workspace]\nmembers = [\"pkg\"]\nresolver = \"2\"\n[workspace.package]\nversion = \"1.2.2\"\nedition = \"2024\"\n"),
        ("pkg/Cargo.toml", "[package]\nname = \"vize_law\"\nversion.workspace = true\nedition.workspace = true\n"),
        ("pkg/src/lib.rs", "pub fn law() {}\n"),
        ("npm/cli/package.json", &manifest(&packet["before"])),
        ("pnpm-workspace.yaml", "catalogs:\n  native-binaries:\n    \"@vizejs/native-test\": \"1.2.2\"\n"),
        ("tools/moon/cmd/publish_crates/main.mbt", "let published_crates = [\"vize_law\"]\n"),
        (".github/workflows/release.yml", "publish npm/cli\n"),
        ("tools/commands/ci/github/release-platforms.rs", "native-test\n"),
        ("tools/moon/cmd/publish_npm_package_dirs/main.mbt", "all native directories\n"),
        ("tools/moon/cmd/publish_npm_package/main.mbt", "publish exact version\n"),
    ]);
    github::output("cargo", &["generate-lockfile", "--offline"], &repo.work).unwrap();
    let cut = repo.commit(&[]);
    let head = bump(&repo, &cut);
    let mut source = source(&head, &cut);
    source.base_version = "1.2.2".into();
    github::git(&["reset", "--hard", &cut], &repo.work).unwrap();
    let ordinary = repo.commit(&[(
        "pkg/src/lib.rs",
        "pub fn law() { /* later source fix */ }\n",
    )]);
    let matching = bump(&repo, &ordinary);
    projection(&source, &matching, &ordinary, &repo.work).unwrap();
    let hidden = repo.commit(&[("pkg/src/lib.rs", "pub fn smuggled() {}\n")]);
    assert!(
        projection(&source, &hidden, &ordinary, &repo.work)
            .unwrap_err()
            .contains("exact generated")
    );
    github::git(&["reset", "--hard", &cut], &repo.work).unwrap();
    let predecessor = repo.commit(&[("npm/cli/package.json", &manifest(&packet["after"]))]);
    let candidate = bump(&repo, &predecessor);
    // A valid generated metadata delta used to enter and complete the queue.
    metadata::verify_delta(&predecessor, &candidate, "1.2.2", "1.2.3", &repo.work).unwrap();
    let before = metadata::catalog(&head, "1.2.3", &repo.work).unwrap();
    let mut after = metadata::catalog(&candidate, "1.2.3", &repo.work).unwrap();
    assert_eq!(
        before.pointer(packet["path"].as_str().unwrap()),
        Some(&packet["before"])
    );
    assert_eq!(
        after.pointer(packet["path"].as_str().unwrap()),
        Some(&packet["after"])
    );
    *after.pointer_mut(packet["path"].as_str().unwrap()).unwrap() = packet["before"].clone();
    assert_eq!(
        before, after,
        "the real regression contains exactly one field difference"
    );
    assert!(
        projection(&source, &candidate, &predecessor, &repo.work)
            .unwrap_err()
            .contains("pre-merge candidate registry closure differs")
    );
    candidate_parents(&candidate, &predecessor, &head, true, &repo.work).unwrap();
    assert!(candidate_parents(&candidate, &cut, &head, true, &repo.work).is_err());
    assert!(candidate_parents(&candidate, &predecessor, &head, false, &repo.work).is_err());
    let tree = github::git(&["rev-parse", &format!("{candidate}^{{tree}}")], &repo.work).unwrap();
    let synthetic = github::git(
        &[
            "commit-tree",
            &tree,
            "-p",
            &predecessor,
            "-p",
            &head,
            "-m",
            "synthetic PR merge",
        ],
        &repo.work,
    )
    .unwrap();
    candidate_parents(&synthetic, &predecessor, &head, false, &repo.work).unwrap();
    assert!(candidate_parents(&synthetic, &predecessor, &head, true, &repo.work).is_err());
}

#[test]
fn premerge_integration_fields_fail_closed_on_changed_custody() {
    let head = "a".repeat(40);
    let cut = "b".repeat(40);
    let valid = json!({"number":99,"state":"open","merged":false,"draft":false,"body":format!("<!-- vize-release-pin-source: 42 -->\n<!-- vize-release-pin-head: {head} -->\n<!-- vize-release-pin-cut: {cut} -->\n<!-- vize-release-pin-tag: v1.2.3 -->\n"),"head":{"sha":head,"ref":"release-integration/v1.2.3","repo":{"full_name":"owner/repo"}},"base":{"ref":"main","repo":{"full_name":"owner/repo"}}});
    assert!(integration_fields(&valid, "owner/repo", 99, &head).is_ok());
    for (pointer, value) in [
        ("/number", json!(98)),
        ("/state", json!("closed")),
        ("/merged", json!(true)),
        ("/draft", json!(true)),
        ("/head/sha", json!(cut)),
        ("/head/ref", json!("other")),
        ("/head/repo/full_name", json!("fork/repo")),
        ("/base/ref", json!("other")),
        ("/base/repo/full_name", json!("fork/repo")),
        ("/body", json!("<!-- vize-release-pin-source: 42 -->")),
        (
            "/body",
            json!(format!(
                "{}<!-- vize-release-pin-tag: v1.2.3 -->\n",
                valid["body"].as_str().unwrap()
            )),
        ),
    ] {
        let mut changed = valid.clone();
        *changed.pointer_mut(pointer).unwrap() = value;
        assert!(
            integration_fields(&changed, "owner/repo", 99, &head).is_err(),
            "{pointer}"
        );
    }
    let candidate = "c".repeat(40);
    let mut pr = valid.clone();
    pr["base"]["sha"] = json!(cut);
    let event = json!({"repository":{"full_name":"owner/repo"},"number":99,"pull_request":pr,"merge_group":{"head_sha":candidate,"base_sha":cut,"base_ref":"refs/heads/main","head_ref":format!("refs/heads/gh-readonly-queue/main/pr-99-{cut}")}});
    assert!(
        !event_fields(
            Some(("pull_request", &event)),
            &pr,
            "owner/repo",
            99,
            &candidate,
            &cut,
            &head
        )
        .unwrap()
    );
    assert!(
        event_fields(
            Some(("merge_group", &event)),
            &pr,
            "owner/repo",
            99,
            &candidate,
            &cut,
            &head
        )
        .unwrap()
    );
    for (name, pointer, value) in [
        (
            "pull_request",
            "/repository/full_name",
            json!("foreign/repo"),
        ),
        ("pull_request", "/number", json!(98)),
        (
            "pull_request",
            "/pull_request/body",
            json!(format!("{}changed prose", pr["body"].as_str().unwrap())),
        ),
        ("pull_request", "/pull_request/head/sha", json!(cut)),
        (
            "pull_request",
            "/pull_request/head/repo/full_name",
            json!("fork/repo"),
        ),
        ("pull_request", "/pull_request/base/sha", json!(candidate)),
        (
            "merge_group",
            "/repository/full_name",
            json!("foreign/repo"),
        ),
        ("merge_group", "/merge_group/head_sha", json!(head)),
        ("merge_group", "/merge_group/base_sha", json!(head)),
        (
            "merge_group",
            "/merge_group/base_ref",
            json!("refs/heads/other"),
        ),
        (
            "merge_group",
            "/merge_group/head_ref",
            json!(format!("refs/heads/gh-readonly-queue/main/pr-98-{cut}")),
        ),
    ] {
        let mut changed = event.clone();
        *changed.pointer_mut(pointer).unwrap() = value;
        assert!(
            event_fields(
                Some((name, &changed)),
                &pr,
                "owner/repo",
                99,
                &candidate,
                &cut,
                &head
            )
            .is_err(),
            "{name} {pointer}"
        );
    }
    assert!(
        event_fields(
            Some(("push", &event)),
            &pr,
            "owner/repo",
            99,
            &candidate,
            &cut,
            &head
        )
        .is_err()
    );
}
