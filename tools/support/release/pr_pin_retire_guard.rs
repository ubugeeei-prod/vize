//! Live retirement authority; absence/error ambiguity never authorizes a reservation.
use super::super::{
    pr_budget::{self, Budget},
    pr_contract, pr_github as github,
};
use super::{
    Source, first_parent, marker, metadata, retire::Request, retire_archive as archive,
    retire_evidence as evidence,
};
use serde_json::{Value, json};
use std::{env, fs, path::Path};

pub(super) fn authorize(root: &Path) -> Result<String, String> {
    let repository = github::repository(root)?;
    let raw = github::output("gh", &["api", "user"], root)?;
    let user: Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    if user["type"] != "User" {
        return Err("Retirement requires a genuine maintainer User".into());
    }
    let login = user["login"]
        .as_str()
        .ok_or("Missing authenticated maintainer")?;
    pr_contract::maintainer(&github::author(&repository, login, root)?)?;
    Ok(repository)
}

pub(super) fn no_replacements(root: &Path) -> Result<(), String> {
    if !github::git(
        &["for-each-ref", "--format=%(refname)", "refs/replace"],
        root,
    )?
    .is_empty()
        || env::var_os("GIT_REPLACE_REF_BASE").is_some()
    {
        return Err("Retirement requires original Git objects without replacement refs".into());
    }
    Ok(())
}

#[cfg(test)]
pub(super) use super::retire_release::release_page;

fn not_queued(repository: &str, number: u64, root: &Path) -> Result<(), String> {
    let (owner, name) = repository
        .split_once('/')
        .ok_or("Invalid retirement repository")?;
    let query = "query($owner:String!,$name:String!,$number:Int!){repository(owner:$owner,name:$name){pullRequest(number:$number){mergeQueueEntry{id} autoMergeRequest{enabledAt}}}}";
    let raw = github::output(
        "gh",
        &[
            "api",
            "graphql",
            "-f",
            &format!("query={query}"),
            "-f",
            &format!("owner={owner}"),
            "-f",
            &format!("name={name}"),
            "-F",
            &format!("number={number}"),
        ],
        root,
    )?;
    let result: Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    let pr = result
        .pointer("/data/repository/pullRequest")
        .filter(|v| v.is_object())
        .ok_or("Missing authenticated integration queue state")?;
    if result.get("errors").is_some()
        || pr.get("mergeQueueEntry") != Some(&Value::Null)
        || pr.get("autoMergeRequest") != Some(&Value::Null)
    {
        return Err("Queued or auto-merge integration forbids retirement".into());
    }
    Ok(())
}

pub(super) fn absence(
    source: &Source,
    budget: Option<&Budget>,
    root: &Path,
) -> Result<Value, String> {
    pr_budget::check(budget)?;
    let directory = env::temp_dir().join(format!("vize-retirement-absence-{}", std::process::id()));
    fs::create_dir(&directory)
        .map_err(|e| format!("Ambiguous existing retirement absence output: {e}"))?;
    let path = directory.join("absence.json");
    let result = (|| {
        github::run(
            "node",
            &[
                "tools/support/release/retirement_absence.ts",
                "--root",
                root.to_str().ok_or("Invalid root")?,
                "--head",
                &source.candidate.head,
                "--tag",
                &source.candidate.tag,
                "--output",
                path.to_str().ok_or("Invalid absence output")?,
            ],
            root,
        )?;
        pr_budget::check(budget)?;
        let receipt: Value = serde_json::from_slice(&fs::read(&path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        if receipt["schema"] != "vize-retired-unpublished-registry-absence-v1"
            || receipt["head"].as_str() != Some(&source.candidate.head)
            || receipt["tag"].as_str() != Some(&source.candidate.tag)
            || receipt.pointer("/plan/parentCut").and_then(Value::as_str) != Some(&source.cut)
        {
            return Err("Raw-H registry absence receipt identity changed".into());
        }
        Ok(receipt)
    })();
    let _ = fs::remove_dir_all(&directory);
    result
}

/// Validate original immutable refs and live provider authority. Main may advance normally.
pub(super) fn capture(
    repository: &str,
    request: &Request,
    require_closed: bool,
    owned_lease: bool,
    budget: Option<&Budget>,
    root: &Path,
) -> Result<(Source, Value), String> {
    pr_budget::check(budget)?;
    no_replacements(root)?;
    pr_contract::sha(&request.head)?;
    pr_contract::tag(&request.tag)?;
    github::git(&["fetch", "--no-tags", "origin", &request.head], root)?;
    let source = super::source(
        repository,
        request.source,
        &request.head,
        &request.tag,
        true,
        root,
    )?;
    pr_budget::check(budget)?;
    let source_pr = github::api(repository, &format!("pulls/{}", request.source), root)?;
    if require_closed && !source.closed {
        return Err("A reservation is incomplete until its original source is closed".into());
    }
    let pin_ref = format!("refs/heads/release-pin/{}", request.tag);
    let pin = archive::remote(&pin_ref, root)?.ok_or("Original pin disappeared")?;
    pr_budget::check(budget)?;
    let integration = github::api(repository, &format!("pulls/{}", source.integration), root)?;
    let author = pr_contract::field(&integration, "/user/login")?;
    pr_contract::maintainer(&github::author(repository, author, root)?)?;
    let branch = format!("release-integration/{}", request.tag);
    for (path, expected) in [
        ("/base/ref", "main"),
        ("/base/repo/full_name", repository),
        ("/head/repo/full_name", repository),
        ("/head/ref", branch.as_str()),
    ] {
        if pr_contract::field(&integration, path)? != expected {
            return Err("Original M branch/repository identity changed".into());
        }
    }
    if integration["merged"] != false
        || !matches!(integration["state"].as_str(), Some("open" | "closed"))
        || (require_closed && integration["state"] != "closed")
    {
        return Err(
            "Original M must remain unmerged and a completed reservation requires it closed".into(),
        );
    }
    not_queued(repository, source.integration, root)?;
    let body = integration["body"]
        .as_str()
        .ok_or("Original M body missing")?;
    for (key, value) in [
        ("vize-release-pin-source", request.source.to_string()),
        ("vize-release-pin-head", request.head.clone()),
        ("vize-release-pin-cut", source.cut.clone()),
        ("vize-release-pin-tag", request.tag.clone()),
    ] {
        if marker(body, key)? != value {
            return Err("Original M source markers changed".into());
        }
    }
    let integration_head = pr_contract::field(&integration, "/head/sha")?;
    pr_contract::sha(integration_head)?;
    if archive::remote(&format!("refs/heads/{branch}"), root)?.as_deref() != Some(integration_head)
    {
        return Err("Original M ref and PR head disagree".into());
    }
    github::git(&["fetch", "--no-tags", "origin", integration_head], root)?;
    let parent = github::parent(integration_head, root)?;
    first_parent(&source.cut, &parent, root)?;
    metadata::verify_delta(
        &parent,
        integration_head,
        &source.base_version,
        request.tag.trim_start_matches('v'),
        root,
    )?;
    pr_budget::check(budget)?;
    let main = github::fetch_main(root)?;
    first_parent(&source.cut, &main, root)?;
    if github::version_text(&metadata::text(&main, "Cargo.toml", root)?)? != source.base_version {
        return Err(
            "Main has advanced its version; unpublished retirement cannot reserve this cut".into(),
        );
    }
    if github::tag_target(&request.tag, root)?.is_some() {
        return Err("Any original release tag permanently forbids retirement".into());
    }
    let releases = super::retire_release::absence(repository, &request.tag, budget, root)?;
    if !owned_lease
        && archive::remote(
            &format!("refs/heads/release-operator/{}", request.tag),
            root,
        )?
        .is_some()
    {
        return Err("Original release operator lease is live or ambiguous; never steal it".into());
    }
    let hosted: Vec<_> = [
        "GITHUB_RUN_ID",
        "GITHUB_SHA",
        "GITHUB_WORKFLOW_REF",
        "GITHUB_ACTOR",
        "GITHUB_TRIGGERING_ACTOR",
    ]
    .into_iter()
    .map(env::var)
    .collect();
    let context = if hosted.iter().all(Result::is_ok) {
        let values: Vec<_> = hosted
            .iter()
            .map(|value| value.as_ref().unwrap().as_str())
            .collect();
        let login = github::output("gh", &["api", "user", "--jq", ".login"], root)?;
        if values[3] != login || values[4] != login {
            return Err(
                "Current operator actor/trigger differs from the authenticated maintainer User"
                    .into(),
            );
        }
        pr_contract::sha(values[1])?;
        first_parent(values[1], &github::git(&["rev-parse", "HEAD"], root)?, root)?;
        Some(
            json!({"id":values[0].parse::<u64>().map_err(|_|"Invalid current operator run")?,"sha":values[1],"actor":login,"workflowRef":values[2]}),
        )
    } else {
        None
    };
    let operators = evidence::no_other_operator(repository, context.as_ref(), budget, root)?;
    let run = github::api(repository, &format!("actions/runs/{}", request.run), root)?;
    super::run_identity(&run, &source, request.run)?;
    evidence::terminal_failed(&run)?;
    let entry = github::api(
        repository,
        &format!("actions/runs/{}", request.operator),
        root,
    )?;
    evidence::original_operator(&entry, &source, request.operator)?;
    pr_budget::check(budget)?;
    let (release_pages, jobs) = evidence::inventory(
        repository,
        &format!("actions/runs/{}/jobs?filter=all", request.run),
        "jobs",
        budget,
        root,
    )?;
    let (operator_pages, operator_jobs) = evidence::inventory(
        repository,
        &format!("actions/runs/{}/jobs?filter=all", request.operator),
        "jobs",
        budget,
        root,
    )?;
    evidence::original_failure_jobs(&run, &jobs, &entry, &operator_jobs)?;
    evidence::publication_jobs(
        &jobs,
        &metadata::text(
            &source.candidate.head,
            ".github/workflows/release.yml",
            root,
        )?,
    )?;
    pr_budget::check(budget)?;
    let identity = json!({"repository":repository,"tag":request.tag,"sourcePr":request.source,"head":request.head,"cut":source.cut,"baseVersion":source.base_version,"pin":pin,"integrationPr":source.integration,"integrationHead":integration_head,"releaseRun":request.run,"releaseAttempt":run["run_attempt"],"operatorRun":request.operator,"operatorAttempt":entry["run_attempt"]});
    Ok((
        source,
        json!({"identity":identity,"sourcePr":source_pr,"integrationPr":integration,"releaseRun":run,"operatorRun":entry,"publicationJobs":jobs,"originalFailureJobPages":{"release":release_pages,"operator":operator_pages},"operatorInventory":operators,"mainAtGuard":main,"tagAbsent":true,"githubReleaseAbsent":true,"githubReleaseInventory":releases}),
    ))
}
