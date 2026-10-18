use super::super::pr_github as github;
use super::catalog::publication_targets;
use super::integration::PreparedNormalCut;
use super::tests::Repo;
use super::{Source, metadata};
use serde_json::{Value, json};

#[test]
fn target_projection_preserves_all_shipping_fields_and_unknown_kinds() {
    let shipping = json!([
        {"kind":["lib"],"crate_types":["lib"],"name":"law","test":true,"src_path":"src/lib.rs"},
        {"kind":["bin"],"crate_types":["bin"],"name":"cli","test":true},
        {"kind":["proc-macro"],"crate_types":["proc-macro"],"name":"derive"},
        {"kind":["custom-build"],"crate_types":["bin"],"name":"build-script-build"}
    ]);
    let shipping = shipping.as_array().unwrap();
    assert_eq!(publication_targets(shipping), *shipping);
    for kinds in [
        json!(["test"]),
        json!(["bench"]),
        json!(["example"]),
        json!(["test", "example"]),
    ] {
        let mut targets = shipping.clone();
        targets.push(json!({"kind":kinds,"crate_types":["bin"],"name":"observer"}));
        assert_eq!(publication_targets(&targets), *shipping);
    }
    for kinds in [
        json!(["test", "bin"]),
        json!(["example", "custom-build"]),
        json!(["future"]),
        json!(["test", "future"]),
        json!([]),
        json!(null),
        json!("test"),
        json!([null]),
    ] {
        let mut targets = shipping.clone();
        targets.push(json!({"kind":kinds,"crate_types":["bin"],"name":"unqualified"}));
        assert_eq!(publication_targets(&targets), targets);
        assert_ne!(publication_targets(&targets), *shipping);
    }
    for (field, value) in [
        ("name", json!("other")),
        ("src_path", json!("src/other.rs")),
        ("crate_types", json!(["cdylib"])),
        ("required-features", json!(["new"])),
    ] {
        let mut targets = shipping.clone();
        targets[0][field] = value;
        assert_eq!(publication_targets(&targets), targets);
        assert_ne!(publication_targets(&targets), *shipping, "{field}");
    }
}

#[test]
fn actual_h_v_eight_discovered_tests_preserve_the_complete_shipping_targets() {
    let packet: Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/release/pinned-catalog-dev-targets.json"
    ))
    .unwrap();
    assert_eq!(packet["H"], "fd6241bf8ea5466794cc955138a59a9b75a8aac2");
    assert_eq!(packet["V"], "18ce7841e693550334c81584bae2f9782d27f786");
    let crates = packet["crates"].as_array().unwrap();
    assert_eq!(crates.len(), 5);
    let mut added = Vec::new();
    for package in crates {
        let before = package["H"].as_array().unwrap();
        let after = package["V"].as_array().unwrap();
        assert!(before.iter().all(|target| after.contains(target)));
        assert_eq!(
            publication_targets(before),
            publication_targets(after),
            "{}",
            package["crate"]
        );
        assert!(!publication_targets(before).is_empty());
        for target in after.iter().filter(|target| !before.contains(target)) {
            assert_eq!(target["kind"], json!(["test"]));
            added.push(target["name"].as_str().unwrap());
            for kind in ["bin", "custom-build", "proc-macro", "future"] {
                let mut changed = after.clone();
                let index = changed.iter().position(|value| value == target).unwrap();
                changed[index]["kind"] = json!(["test", kind]);
                assert_ne!(
                    publication_targets(before),
                    publication_targets(&changed),
                    "{kind}"
                );
            }
        }
    }
    added.sort_unstable();
    assert_eq!(
        added,
        [
            "browser_ssr_context",
            "css_global_ownership",
            "custom_directive_identity",
            "keyed_fragment",
            "scoped_slotted_where",
            "slot_key_mappings",
            "vapor_component_refs",
            "vapor_keyed_fragment",
        ]
    );
}

#[test]
fn normal_cached_driver_module_changes_require_new_entry_source_pin() {
    let pin = include_str!("../../commands/release/pr.rs")
        .lines()
        .find(|line| line.starts_with("//! Driver source pin:"))
        .unwrap();
    assert_eq!(
        pin,
        "//! Driver source pin: release-integration-qualification-overlap-v19-prepared-tree."
    );
    let repo = Repo::new();
    let body = "#[path = \"dependency.rs\"] mod dependency;\nfn main() { println!(\"{}\", dependency::catalog_contract()); }\n";
    let original = format!("//! Driver source pin: original-catalog.\n{body}");
    repo.commit(&[
        ("driver.rs", &original),
        ("dependency.rs", "pub fn catalog_contract() -> u8 { 1 }\n"),
    ]);
    let driver = repo.work.join("driver.rs");
    let args = [driver.to_str().unwrap()];
    assert_eq!(
        github::output("rust-script", &args, &repo.work).unwrap(),
        "1"
    );
    repo.commit(&[("dependency.rs", "pub fn catalog_contract() -> u8 { 2 }\n")]);
    assert_eq!(
        github::output("rust-script", &args, &repo.work).unwrap(),
        "1"
    );
    repo.commit(&[("driver.rs", &format!("{pin}\n{body}"))]);
    assert_eq!(
        github::output("rust-script", &args, &repo.work).unwrap(),
        "2"
    );
}

fn prepared_fixture() -> (Repo, Source) {
    let repo = Repo::new();
    let cut = repo.commit(&[
        ("Cargo.toml", "[workspace.package]\nversion = \"1.2.2\"\n"),
        ("npm/native/package.json", "{\"version\":\"1.2.2\"}\n"),
        ("README.md", "literal 1.2.2\n"),
        ("src/code.rs", "unchanged product\n"),
    ]);
    let head = repo.commit(&[
        ("Cargo.toml", "[workspace.package]\nversion = \"1.2.3\"\n"),
        ("npm/native/package.json", "{\"version\":\"1.2.3\"}\n"),
        ("README.md", "literal 1.2.3\n"),
    ]);
    let mut source = super::tests::source(&head, &cut);
    source.integration = 0;
    (repo, source)
}

#[test]
fn prepared_normal_cut_reuses_whole_tree_without_another_worktree_or_graph_install() {
    let (repo, source) = prepared_fixture();
    let before = github::git(&["show-ref"], &repo.work).unwrap();
    let worktrees = github::git(&["worktree", "list", "--porcelain"], &repo.work).unwrap();
    let prepared = PreparedNormalCut::capture(&source, "minor", &repo.work)
        .unwrap()
        .unwrap();
    let authorizations = std::cell::Cell::new(0);
    let head = prepared
        .project(&source, &source.cut, &repo.work, &|| {
            authorizations.set(authorizations.get() + 1);
            Ok(())
        })
        .unwrap()
        .unwrap();
    assert_eq!(authorizations.get(), 1);
    assert_ne!(head, source.candidate.head);
    assert_eq!(github::parent(&head, &repo.work).unwrap(), source.cut);
    assert_eq!(
        github::git(&["rev-parse", &format!("{head}^{{tree}}")], &repo.work).unwrap(),
        github::git(
            &["rev-parse", &format!("{}^{{tree}}", source.candidate.head)],
            &repo.work
        )
        .unwrap()
    );
    metadata::verify_delta(&source.cut, &head, "1.2.2", "1.2.3", &repo.work).unwrap();
    assert_eq!(github::git(&["show-ref"], &repo.work).unwrap(), before);
    assert_eq!(
        github::git(&["worktree", "list", "--porcelain"], &repo.work).unwrap(),
        worktrees
    );
    assert_eq!(
        github::git(&["rev-parse", "HEAD"], &repo.work).unwrap(),
        source.candidate.head
    );
    assert_eq!(
        github::git(&["status", "--porcelain"], &repo.work).unwrap(),
        ""
    );
    assert!(!repo.work.join("node_modules").exists());
}

#[test]
fn prepared_normal_cut_keeps_noninitial_and_moved_main_on_the_original_route() {
    let (repo, source) = prepared_fixture();
    for bump in ["patch", "major", "alpha", "beta", "rc", "release"] {
        assert!(
            PreparedNormalCut::capture(&source, bump, &repo.work)
                .unwrap()
                .is_none()
        );
    }
    for index in 0..3 {
        let mut changed = source.clone();
        match index {
            0 => changed.integration = 99,
            1 => changed.closed = true,
            _ => {
                changed.stable = Some(super::stable::Stable {
                    policy: source.cut.clone(),
                    blob: source.cut.clone(),
                    sha256: "a".repeat(64),
                })
            }
        }
        assert!(
            PreparedNormalCut::capture(&changed, "minor", &repo.work)
                .unwrap()
                .is_none()
        );
    }
    let prepared = PreparedNormalCut::capture(&source, "minor", &repo.work)
        .unwrap()
        .unwrap();
    let advanced = repo.commit(&[("src/code.rs", "new main product\n")]);
    assert!(
        prepared
            .project(&source, &advanced, &repo.work, &|| {
                panic!("moved main must fall through before authorizing prepared-tree creation")
            })
            .unwrap()
            .is_none()
    );
    assert_eq!(
        github::git(&["rev-parse", "HEAD"], &repo.work).unwrap(),
        advanced
    );
}

#[test]
fn prepared_normal_cut_rejects_changed_owner_identity_and_revoked_lease_before_creation() {
    let (repo, source) = prepared_fixture();
    for index in 0..11 {
        let prepared = PreparedNormalCut::capture(&source, "minor", &repo.work)
            .unwrap()
            .unwrap();
        let mut changed = source.clone();
        match index {
            0 => changed.candidate.number += 1,
            1 => changed.candidate.repository = "other/repo".into(),
            2 => changed.candidate.head = source.cut.clone(),
            3 => changed.candidate.base = source.candidate.head.clone(),
            4 => changed.cut = source.candidate.head.clone(),
            5 => changed.base_version = "1.2.1".into(),
            6 => changed.candidate.tag = "v1.2.4".into(),
            7 => changed.candidate.branch = "main".into(),
            8 => changed.integration = 99,
            9 => changed.closed = true,
            _ => changed.candidate.merged = true,
        }
        assert!(
            prepared
                .project(&changed, &source.cut, &repo.work, &|| {
                    panic!("changed custody cannot reach operator authorization")
                })
                .is_err(),
            "mutation {index}"
        );
    }
    let prepared = PreparedNormalCut::capture(&source, "minor", &repo.work)
        .unwrap()
        .unwrap();
    let before = github::git(&["count-objects", "-v"], &repo.work).unwrap();
    assert_eq!(
        prepared
            .project(&source, &source.cut, &repo.work, &|| {
                Err("original lease was revoked".into())
            })
            .unwrap_err(),
        "original lease was revoked"
    );
    assert_eq!(
        github::git(&["count-objects", "-v"], &repo.work).unwrap(),
        before
    );
}

#[test]
fn prepared_normal_cut_refuses_hidden_changes_omitted_versions_and_raw_parent_substitution() {
    for change in ["product", "omitted", "parent", "replacement"] {
        let (repo, mut source) = prepared_fixture();
        match change {
            "product" => source.candidate.head = repo.commit(&[("src/code.rs", "hidden edit\n")]),
            "omitted" => {
                source.candidate.head =
                    repo.commit(&[("npm/native/package.json", "{\"version\":\"1.2.2\"}\n")])
            }
            "parent" => {
                let tree = github::git(&["rev-parse", "HEAD^{tree}"], &repo.work).unwrap();
                source.candidate.head = github::git(
                    &[
                        "commit-tree",
                        &tree,
                        "-p",
                        &source.cut,
                        "-p",
                        &source.candidate.head,
                        "-m",
                        "two parents",
                    ],
                    &repo.work,
                )
                .unwrap();
                github::git(
                    &["checkout", "--detach", &source.candidate.head],
                    &repo.work,
                )
                .unwrap();
            }
            _ => {
                github::git(
                    &["replace", &source.candidate.head, &source.cut],
                    &repo.work,
                )
                .unwrap();
            }
        }
        if matches!(change, "product" | "omitted") {
            // Keep a legitimate C parent so these failures must come from the
            // complete generated-delta check, rather than the parent guard.
            let tree = github::git(&["rev-parse", "HEAD^{tree}"], &repo.work).unwrap();
            source.candidate.head = github::git(
                &[
                    "commit-tree",
                    &tree,
                    "-p",
                    &source.cut,
                    "-m",
                    "tampered version tree",
                ],
                &repo.work,
            )
            .unwrap();
            github::git(
                &["checkout", "--detach", &source.candidate.head],
                &repo.work,
            )
            .unwrap();
            assert_eq!(
                github::parent(&source.candidate.head, &repo.work).unwrap(),
                source.cut
            );
        }
        assert!(
            PreparedNormalCut::capture(&source, "minor", &repo.work).is_err(),
            "{change}"
        );
    }
}
