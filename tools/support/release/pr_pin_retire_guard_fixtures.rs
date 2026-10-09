use super::super::super::retire_source;
use super::super::archive;
use super::{Repo, github};
use serde_json::{Value, json};

/// These are inert API controls; mutations must never change any original private remote ref.
pub(super) fn guard_controls(
    receipt: &Value,
    logs: &std::collections::BTreeMap<String, Vec<u8>>,
    repo: &Repo,
) {
    super::absence::controls(receipt, logs, repo);
    let before = github::git(&["ls-remote", "--refs", "origin"], &repo.work).unwrap();
    let row = json!({"id":77,"tag_name":"v0.6.0","draft":true,"prerelease":false,"published_at":null,
        "url":"https://api.github.com/repos/owner/repo/releases/77"});
    let mut valid = receipt.clone();
    valid["guards"]["sourcePr"]["state"] = json!("closed");
    valid["guards"]["integrationPr"]["state"] = json!("closed");
    valid["guards"]["sourcePr"]["base"]["sha"] = receipt["identity"]["head"].clone();
    valid["guards"]["githubReleaseInventory"]["pages"] = json!([[row]]);
    retire_source::validate(&valid, &repo.work).unwrap();
    let mut current = receipt["guards"]["operatorRun"].clone();
    current["id"] = json!(9);
    current["status"] = json!("in_progress");
    current["conclusion"] = Value::Null;
    valid["guards"]["operatorInventory"] = json!({"pages":[{"total_count":2,"workflow_runs":[receipt["guards"]["operatorRun"],current]}],
        "currentOperatorContext":{"id":9,"sha":current["head_sha"],"actor":"maintainer","workflowRef":"owner/repo/.github/workflows/release-operator.yml@refs/heads/main"}});
    retire_source::validate(&valid, &repo.work).unwrap();
    let mut duplicate_body = receipt["guards"]["sourcePr"]["body"]
        .as_str()
        .unwrap()
        .to_string();
    duplicate_body.push_str(&format!(
        "<!-- vize-release-pin-head: {} -->\n",
        receipt["identity"]["head"].as_str().unwrap()
    ));
    for (pointer, value) in [
        ("/guards/sourcePr/number", json!(43)),
        ("/guards/sourcePr/user/login", json!("")),
        ("/guards/sourcePr/draft", json!(false)),
        ("/guards/sourcePr/merged", json!(true)),
        ("/guards/sourcePr/state", json!("unknown")),
        (
            "/guards/sourcePr/head/sha",
            receipt["identity"]["cut"].clone(),
        ),
        ("/guards/sourcePr/head/ref", json!("foreign")),
        (
            "/guards/sourcePr/head/repo/full_name",
            json!("foreign/repo"),
        ),
        ("/guards/sourcePr/base/ref", json!("develop")),
        ("/guards/sourcePr/body", json!(duplicate_body)),
        ("/guards/integrationPr/number", json!(100)),
        ("/guards/integrationPr/user/login", json!("")),
        ("/guards/integrationPr/head/ref", json!("foreign")),
        (
            "/guards/integrationPr/head/sha",
            receipt["identity"]["head"].clone(),
        ),
        (
            "/guards/integrationPr/body",
            json!("missing source markers"),
        ),
        ("/guards/mainAtGuard", json!("a".repeat(40))),
        ("/guards/tagAbsent", json!(false)),
        ("/guards/githubReleaseAbsent", json!(false)),
        ("/guards/publicationJobs", json!([])),
        ("/guards/operatorInventory/pages", json!([])),
        ("/guards/operatorInventory/pages/0/total_count", json!(3)),
        (
            "/guards/operatorInventory/pages/0/workflow_runs/1/triggering_actor/login",
            json!("foreign"),
        ),
        (
            "/guards/operatorInventory/currentOperatorContext/workflowRef",
            json!("owner/repo/.github/workflows/release-operator.yml@refs/heads/foreign"),
        ),
        (
            "/guards/githubReleaseInventory/publishedTagLookup",
            json!("assumed 404"),
        ),
        ("/guards/githubReleaseInventory/pages", json!([])),
        ("/guards/githubReleaseInventory/pages", json!([[row, row]])),
        (
            "/guards/githubReleaseInventory/pages/0/0/tag_name",
            json!("v0.8.0"),
        ),
        (
            "/guards/githubReleaseInventory/pages/0/0/prerelease",
            Value::Null,
        ),
        (
            "/guards/githubReleaseInventory/pages/0/0/published_at",
            json!(false),
        ),
        (
            "/guards/githubReleaseInventory/pages/0/0/url",
            json!("https://api.github.com/repos/foreign/repo/releases/77"),
        ),
    ] {
        let mut bad = valid.clone();
        *bad.pointer_mut(pointer).unwrap() = value;
        assert!(
            archive::install(bad, logs, &repo.work).is_err(),
            "{pointer}"
        );
        assert_eq!(
            github::git(&["ls-remote", "--refs", "origin"], &repo.work).unwrap(),
            before
        );
    }
}
