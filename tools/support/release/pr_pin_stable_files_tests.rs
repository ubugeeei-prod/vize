use super::super::super::metadata;
use super::super::super::tests::Repo;
use super::*;
use serde_json::json;

fn verify_files(
    repo: &Repo,
    public: &str,
    cut: &str,
    merged: &str,
    parent: &str,
    files: Value,
) -> Result<(), String> {
    let raw = metadata::bytes(
        &[
            "--no-replace-objects",
            "diff",
            "--raw",
            "--no-renames",
            "--no-abbrev",
            public,
            cut,
        ],
        &repo.work,
    )?;
    let v = json!({"fixes":[{"merge":merged,"parent":parent}],"files":files});
    let c = Stable {
        policy: merged.into(),
        blob: String::new(),
        sha256: String::new(),
    };
    verify(
        &v,
        &c,
        public,
        cut,
        &[merged].into_iter().collect(),
        raw,
        &repo.work,
    )
}

#[test]
fn real_original_merge_cannot_launder_inherited_drift_as_a_changed_fix() {
    let repo = Repo::new();
    let public = repo.commit(&[
        ("src/code.rs", "original\n"),
        ("src/inherited.rs", "public\n"),
    ]);
    let drift = repo.commit(&[("src/inherited.rs", "unrelated later drift\n")]);
    let merged = repo.commit(&[("src/code.rs", "fixed\n")]);
    github::git(&["checkout", "--detach", &public], &repo.work).unwrap();
    let cut = repo.commit(&[
        ("src/code.rs", "fixed\n"),
        ("src/inherited.rs", "unrelated later drift\n"),
    ]);
    let files = json!([{"path":"src/code.rs","role":"fix","origin":merged},{"path":"src/inherited.rs","role":"fix","origin":merged}]);
    let error = verify_files(&repo, &public, &cut, &merged, &drift, files).unwrap_err();
    assert!(error.contains("never changed"), "{error}");
}

#[test]
fn actual_changed_body_requires_exact_applicable_preimage_and_a_shipping_fix_for_every_origin() {
    let repo = Repo::new();
    let public = repo.commit(&[("src/code.rs", "original\n")]);
    let drift = repo.commit(&[("src/code.rs", "different preimage\n")]);
    let merged = repo.commit(&[("src/code.rs", "fixed\n")]);
    github::git(&["checkout", "--detach", &public], &repo.work).unwrap();
    let cut = repo.commit(&[("src/code.rs", "fixed\n")]);
    let files = json!([{"path":"src/code.rs","role":"fix","origin":merged}]);
    let error = verify_files(&repo, &public, &cut, &merged, &drift, files.clone()).unwrap_err();
    assert!(error.contains("preimage"), "{error}");
    let mut files = files;
    files[0]["role"] = json!("corpus");
    // Even a literal matching body cannot make an unreferenced origin a shipping fix.
    let genuine = repo.commit(&[("src/code.rs", "fixed again\n")]);
    github::git(&["checkout", "--detach", &cut], &repo.work).unwrap();
    let after = repo.commit(&[("src/code.rs", "fixed again\n")]);
    files[0]["origin"] = json!(genuine);
    let error = verify_files(&repo, &cut, &after, &genuine, &cut, files).unwrap_err();
    assert!(error.contains("empty shipping"), "{error}");
}
