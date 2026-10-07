use super::super::{pr_contract, pr_github as github};
use super::tests::{Repo, source};
use super::{delivery, metadata};
use serde_json::json;
use std::fs;

#[test]
fn complete_shipment_catalog_rejects_exports_bins_and_feature_changes() {
    let repo = Repo::new();
    let cargo = "[workspace]\nmembers = [\"pkg\"]\nresolver = \"2\"\n[workspace.package]\nversion = \"1.2.3\"\nedition = \"2024\"\n";
    let package = "[package]\nname = \"vize_law\"\nversion.workspace = true\nedition.workspace = true\n[features]\ndefault = []\n";
    let npm = "{\"name\":\"vize-law\",\"version\":\"1.2.3\",\"bin\":{\"vize\":\"bin.cjs\"},\"exports\":\"./index.mjs\"}\n";
    repo.commit(&[
        ("Cargo.toml", cargo),
        ("pkg/Cargo.toml", package),
        ("pkg/src/lib.rs", "pub fn law() {}\n"),
        ("npm/law/package.json", npm),
        ("editors/vscode/package.json", "{\"name\":\"vize-law\",\"publisher\":\"maintainer\",\"version\":\"1.2.3\",\"main\":\"./extension.cjs\"}\n"),
        (
            "pnpm-workspace.yaml",
            "catalogs:\n  native-binaries:\n    \"@vizejs/native-test\": \"1.2.3\"\n",
        ),
        (
            "tools/moon/cmd/publish_crates/main.mbt",
            "let published_crates = [\"vize_law\"]\n",
        ),
    ]);
    github::output("cargo", &["generate-lockfile", "--offline"], &repo.work).unwrap();
    let head = repo.commit(&[]);
    let original = metadata::catalog(&head, "1.2.3", &repo.work).unwrap();
    assert!(original["npm"]["vize-law"].is_object());
    assert!(original["editors"]["maintainer.vize-law"].is_object());
    let duplicate = repo.commit(&[("npm/duplicate/package.json", npm)]);
    assert!(metadata::catalog(&duplicate, "1.2.3", &repo.work).is_err());
    github::git(&["reset", "--hard", &head], &repo.work).unwrap();
    let ordinary = repo.commit(&[(
        "pkg/src/lib.rs",
        "pub fn law() { /* ordinary source fix */ }\n",
    )]);
    assert_eq!(
        original,
        metadata::catalog(&ordinary, "1.2.3", &repo.work).unwrap()
    );
    // All temporary checkouts have different absolute paths; the canonical
    // publication descriptor must still compare equal (including /var aliases).
    assert_eq!(
        original,
        metadata::catalog(&head, "1.2.3", &repo.work).unwrap()
    );
    let exports=repo.commit(&[("npm/law/package.json","{\"name\":\"vize-law\",\"version\":\"1.2.3\",\"bin\":{\"vize\":\"other.cjs\"},\"exports\":\"./other.mjs\"}\n")]);
    assert_ne!(
        original,
        metadata::catalog(&exports, "1.2.3", &repo.work).unwrap()
    );
    github::git(&["reset", "--hard", &head], &repo.work).unwrap();
    let feature = repo.commit(&[(
        "pkg/Cargo.toml",
        &package.replace("default = []", "default = [\"new\"]\nnew = []"),
    )]);
    assert_ne!(
        original,
        metadata::catalog(&feature, "1.2.3", &repo.work).unwrap()
    );
}

#[test]
fn partial_source_contract_rejects_forks_merged_ready_and_unauthorized_sources() {
    let head = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let cut = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    let valid = json!({"number":42,"state":"open","merged":false,"draft":true,"body":format!("<!-- vize-release-pin: immutable-v1 -->\n<!-- vize-release-pin-head: {head} -->\n<!-- vize-release-pin-cut: {cut} -->\n"),"user":{"login":"maintainer"},"head":{"sha":head,"ref":"release/v1.2.3","repo":{"full_name":"owner/repo"}},"base":{"sha":cut,"ref":"main","repo":{"full_name":"owner/repo"}}});
    let permission = json!({"role_name":"maintain"});
    assert!(
        pr_contract::candidate_fields(&valid, &permission, "owner/repo", head, "v1.2.3", false)
            .is_ok()
    );
    for (pointer, value) in [
        ("/head/repo/full_name", json!("fork/repo")),
        ("/base/ref", json!("other")),
        ("/merged", json!(true)),
        ("/draft", json!(false)),
        ("/state", json!("closed")),
        ("/head/sha", json!(cut)),
    ] {
        let mut mutated = valid.clone();
        *mutated.pointer_mut(pointer).unwrap() = value;
        assert!(
            pr_contract::candidate_fields(
                &mutated,
                &permission,
                "owner/repo",
                head,
                "v1.2.3",
                false
            )
            .is_err(),
            "{pointer}"
        );
    }
    assert!(
        pr_contract::candidate_fields(
            &valid,
            &json!({"role_name":"write"}),
            "owner/repo",
            head,
            "v1.2.3",
            false
        )
        .is_err()
    );
    assert!(
        pr_contract::candidate(&valid, &permission, "owner/repo", head, "v1.2.3", false).is_err(),
        "legacy refresh rejects even an interrupted pin marker"
    );
}

#[test]
fn annotated_tag_receipt_binds_original_run_and_integration_and_rejects_lightweight_tags() {
    let repo = Repo::new();
    let cut = repo.commit(&[("Cargo.toml", "[workspace.package]\nversion = \"1.2.2\"\n")]);
    let head = repo.commit(&[("Cargo.toml", "[workspace.package]\nversion = \"1.2.3\"\n")]);
    let source = source(&head, &cut);
    let receipt = delivery::Receipt {
        head: head.clone(),
        merge: cut.clone(),
        parent: cut.clone(),
        gate: cut.clone(),
    };
    github::git(
        &[
            "tag",
            "-a",
            "v1.2.3",
            &head,
            "-m",
            &delivery::annotation(&source, 7, &receipt),
        ],
        &repo.work,
    )
    .unwrap();
    delivery::push_tag(&repo.work, "refs/tags/v1.2.3").unwrap();
    assert_eq!(delivery::tag_receipt(&source, &repo.work).unwrap().0, 7);
    let mut foreign = source.clone();
    foreign.integration = 100;
    assert!(delivery::tag_receipt(&foreign, &repo.work).is_err());
    github::git(&["push", "origin", ":refs/tags/v1.2.3"], &repo.work).unwrap();
    github::git(&["tag", "-d", "v1.2.3"], &repo.work).unwrap();
    github::git(&["tag", "v1.2.3", &head], &repo.work).unwrap();
    delivery::push_tag(&repo.work, "refs/tags/v1.2.3").unwrap();
    assert!(delivery::tag_receipt(&source, &repo.work).is_err());
    assert!(
        fs::metadata(repo.work.join("Cargo.toml"))
            .unwrap()
            .is_file()
    );
}
