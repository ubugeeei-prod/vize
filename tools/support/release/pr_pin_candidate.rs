//! Read-only, pre-merge qualification of the actual version delivery candidate.
use super::super::{pr_contract, pr_github as github};
use super::{Source, first_parent, marker, metadata, source};
use serde_json::{Value, json};
use std::{env, fs, path::Path};

fn integration_fields(
    pr: &Value,
    repository: &str,
    number: u64,
    head: &str,
) -> Result<(u64, String, String, String), String> {
    pr_contract::sha(head)?;
    if number == 0
        || pr.get("number").and_then(Value::as_u64) != Some(number)
        || pr.get("merged").and_then(Value::as_bool) != Some(false)
        || pr.get("draft").and_then(Value::as_bool) != Some(false)
        || pr_contract::field(pr, "/state")? != "open"
        || pr_contract::field(pr, "/base/ref")? != "main"
        || pr_contract::field(pr, "/head/sha")? != head
        || pr_contract::field(pr, "/base/repo/full_name")? != repository
        || pr_contract::field(pr, "/head/repo/full_name")? != repository
    {
        return Err(
            "Pre-merge qualification requires the exact open official integration PR.".into(),
        );
    }
    let body = pr_contract::field(pr, "/body")?;
    let source_number = marker(body, "vize-release-pin-source")?
        .parse::<u64>()
        .map_err(|_| "Invalid integration source PR number")?;
    if source_number == 0 || source_number == number {
        return Err("Integration requires a separate immutable source PR.".into());
    }
    let head = marker(body, "vize-release-pin-head")?;
    let cut = marker(body, "vize-release-pin-cut")?;
    let tag = marker(body, "vize-release-pin-tag")?;
    pr_contract::sha(&head)?;
    pr_contract::sha(&cut)?;
    pr_contract::tag(&tag)?;
    if pr_contract::field(pr, "/head/ref")? != format!("release-integration/{tag}") {
        return Err("Integration branch and pinned body tag disagree.".into());
    }
    Ok((source_number, head, cut, tag))
}

fn candidate_parents(
    candidate: &str,
    base: &str,
    head: &str,
    queue: bool,
    root: &Path,
) -> Result<(), String> {
    let row = github::git(&["rev-list", "--parents", "-n", "1", candidate], root)?;
    let expected = if queue {
        format!("{candidate} {base}")
    } else {
        format!("{candidate} {base} {head}")
    };
    if row != expected {
        return Err("Candidate parents do not bind the exact PR merge or own queue base.".into());
    }
    Ok(())
}

fn projection(source: &Source, candidate: &str, base: &str, root: &Path) -> Result<(), String> {
    first_parent(&source.cut, base, root)?;
    let version = source.candidate.tag.trim_start_matches('v');
    metadata::verify_delta(base, candidate, &source.base_version, version, root)?;
    // This is the same complete catalog used after actual signed delivery.
    // Manifest scripts and every other shipping field stay authoritative.
    if metadata::catalog(&source.candidate.head, version, root)?
        != metadata::catalog(candidate, version, root)?
    {
        return Err("The pre-merge candidate registry closure differs from the qualified H catalog; prepare a new source cut.".into());
    }
    Ok(())
}

fn event_fields(
    event: Option<(&str, &Value)>,
    pr: &Value,
    repository: &str,
    number: u64,
    candidate: &str,
    base: &str,
    head: &str,
) -> Result<bool, String> {
    match event {
        Some(("merge_group", event)) => {
            if pr_contract::field(event, "/repository/full_name")? != repository
                || pr_contract::field(event, "/merge_group/head_sha")? != candidate
                || pr_contract::field(event, "/merge_group/base_sha")? != base
                || pr_contract::field(event, "/merge_group/base_ref")? != "refs/heads/main"
                || pr_contract::field(event, "/merge_group/head_ref")?
                    != format!("refs/heads/gh-readonly-queue/main/pr-{number}-{base}")
            {
                return Err(
                    "Integration must qualify its own exact main merge-group candidate.".into(),
                );
            }
            Ok(true)
        }
        Some(("pull_request", event)) => {
            integration_fields(
                event
                    .get("pull_request")
                    .ok_or("Missing integration PR event")?,
                repository,
                number,
                head,
            )?;
            if event.get("number").and_then(Value::as_u64) != Some(number)
                || pr_contract::field(event, "/repository/full_name")? != repository
                || event.pointer("/pull_request/body") != pr.get("body")
                || pr_contract::field(event, "/pull_request/base/sha")? != base
            {
                return Err("Integration PR body/head/base changed since this event.".into());
            }
            Ok(false)
        }
        Some(_) => Err("Unsupported integration qualification event.".into()),
        None => Ok(false),
    }
}

fn verify(
    number: u64,
    candidate: &str,
    base: &str,
    head: &str,
    event: Option<(&str, &Value)>,
    root: &Path,
) -> Result<(), String> {
    for sha in [candidate, base, head] {
        pr_contract::sha(sha)?;
    }
    if github::git(&["rev-parse", "HEAD"], root)? != candidate
        || env::var("GITHUB_SHA").is_ok_and(|sha| sha != candidate)
    {
        return Err("Pre-merge qualification must check out its exact candidate SHA.".into());
    }
    let repository = github::repository(root)?;
    let pr = github::api(&repository, &format!("pulls/{number}"), root)?;
    let (source_number, source_head, cut, tag) =
        integration_fields(&pr, &repository, number, head)?;
    let remote = github::git(
        &[
            "ls-remote",
            "--heads",
            "origin",
            &format!("refs/heads/release-integration/{tag}"),
        ],
        root,
    )?;
    if remote.split_whitespace().next() != Some(head) {
        return Err("Integration branch and PR head disagree.".into());
    }
    pr_contract::maintainer(&github::author(
        &repository,
        pr_contract::field(&pr, "/user/login")?,
        root,
    )?)?;
    let queue = event_fields(event, &pr, &repository, number, candidate, base, head)?;
    if !queue && pr_contract::field(&pr, "/merge_commit_sha")? != candidate {
        return Err("The PR candidate is not its current GitHub synthetic merge SHA.".into());
    }
    github::git(
        &[
            "fetch",
            "--no-tags",
            "origin",
            &source_head,
            &cut,
            head,
            base,
        ],
        root,
    )?;
    candidate_parents(candidate, base, head, queue, root)?;
    let source = source(&repository, source_number, &source_head, &tag, false, root)?;
    if source.integration != number || source.cut != cut {
        return Err("Source and integration bodies do not bind the same immutable cut.".into());
    }
    let head_parent = github::parent(head, root)?;
    first_parent(&source.cut, &head_parent, root)?;
    metadata::verify_delta(
        &head_parent,
        head,
        &source.base_version,
        tag.trim_start_matches('v'),
        root,
    )?;
    projection(&source, candidate, base, root)?;
    println!(
        "{}",
        json!({"integration":number,"source":source_number,"H":source_head,"C":cut,"tag":tag,"candidate":candidate,"base":base,"integrationHead":head,"event":if queue {"merge_group"} else {"pull_request"},"catalogEqual":true,"metadataExact":true})
    );
    Ok(())
}

/// A local receipt qualifies only the current synthetic PR merge snapshot.
pub fn verify_candidate(
    number: u64,
    candidate: &str,
    base: &str,
    head: &str,
    root: &Path,
) -> Result<(), String> {
    verify(number, candidate, base, head, None, root)
}

pub fn check_candidate(number: u64, root: &Path) -> Result<(), String> {
    let event_name = env::var("GITHUB_EVENT_NAME").map_err(|_| "Missing integration event name")?;
    let event: Value = serde_json::from_slice(
        &fs::read(env::var("GITHUB_EVENT_PATH").map_err(|_| "Missing integration event payload")?)
            .map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let candidate = env::var("GITHUB_SHA").map_err(|_| "Missing integration candidate SHA")?;
    let (base, head) = match event_name.as_str() {
        "pull_request" => (
            pr_contract::field(&event, "/pull_request/base/sha")?.to_string(),
            pr_contract::field(&event, "/pull_request/head/sha")?.to_string(),
        ),
        "merge_group" => {
            let repository = github::repository(root)?;
            let pr = github::api(&repository, &format!("pulls/{number}"), root)?;
            (
                pr_contract::field(&event, "/merge_group/base_sha")?.to_string(),
                pr_contract::field(&pr, "/head/sha")?.to_string(),
            )
        }
        _ => return Err("Unsupported integration qualification event.".into()),
    };
    verify(
        number,
        &candidate,
        &base,
        &head,
        Some((&event_name, &event)),
        root,
    )
}

#[cfg(test)]
mod tests {
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
        let tree =
            github::git(&["rev-parse", &format!("{candidate}^{{tree}}")], &repo.work).unwrap();
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
}
