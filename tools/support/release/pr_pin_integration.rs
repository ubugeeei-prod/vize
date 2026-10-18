use super::super::{
    pr_budget::{self, Budget},
    pr_github as github, pr_start,
};
use super::{Source, lock, marker, metadata};
use serde_json::Value;
use std::{fs, path::Path};

/// An in-process result of the initial normal preparation, never a saved receipt
/// or resume input. Private fields prevent other callers from asserting custody.
pub(super) struct PreparedNormalCut {
    source_number: u64,
    repository: String,
    cut: String,
    head: String,
    tree: String,
    base_version: String,
    tag: String,
}

fn raw_identity(revision: &str, root: &Path) -> Result<(String, Vec<String>), String> {
    super::super::pr_contract::sha(revision)?;
    let object = github::git(
        &["--no-replace-objects", "cat-file", "commit", revision],
        root,
    )?;
    let mut trees = Vec::new();
    let mut parents = Vec::new();
    for line in object.lines().take_while(|line| !line.is_empty()) {
        if let Some(tree) = line.strip_prefix("tree ") {
            super::super::pr_contract::sha(tree)?;
            trees.push(tree.to_owned());
        } else if let Some(parent) = line.strip_prefix("parent ") {
            super::super::pr_contract::sha(parent)?;
            parents.push(parent.to_owned());
        }
    }
    if trees.len() != 1 {
        return Err("Prepared integration requires one raw commit tree".into());
    }
    Ok((trees.remove(0), parents))
}

impl PreparedNormalCut {
    pub(super) fn capture(
        source: &Source,
        bump: &str,
        root: &Path,
    ) -> Result<Option<Self>, String> {
        if bump != "minor" || source.stable.is_some() || source.integration != 0 || source.closed {
            return Ok(None);
        }
        let candidate = &source.candidate;
        if candidate.merged
            || candidate.number == 0
            || candidate.base != source.cut
            || candidate.branch != format!("release/{}", candidate.tag)
            || github::git(&["rev-parse", "HEAD"], root)? != candidate.head
        {
            return Err("Initial prepared cut identity changed".into());
        }
        super::retire_guard::no_replacements(root)?;
        let (tree, parents) = raw_identity(&candidate.head, root)?;
        if parents != [source.cut.clone()] {
            return Err("Prepared H must have its exact single C parent".into());
        }
        metadata::verify_delta(
            &source.cut,
            &candidate.head,
            &source.base_version,
            candidate.tag.trim_start_matches('v'),
            root,
        )?;
        Ok(Some(Self {
            source_number: candidate.number,
            repository: candidate.repository.clone(),
            cut: source.cut.clone(),
            head: candidate.head.clone(),
            tree,
            base_version: source.base_version.clone(),
            tag: candidate.tag.clone(),
        }))
    }

    pub(super) fn project(
        self,
        source: &Source,
        main: &str,
        root: &Path,
        authorize: &dyn Fn() -> Result<(), String>,
    ) -> Result<Option<String>, String> {
        if main != self.cut {
            return Ok(None);
        }
        if source.stable.is_some()
            || source.integration != 0
            || source.closed
            || source.candidate.merged
            || source.candidate.number != self.source_number
            || source.candidate.repository != self.repository
            || source.candidate.base != self.cut
            || source.cut != self.cut
            || source.candidate.head != self.head
            || source.base_version != self.base_version
            || source.candidate.tag != self.tag
            || source.candidate.branch != format!("release/{}", self.tag)
        {
            return Err("Prepared result does not belong to this initial cut".into());
        }
        authorize()?;
        super::retire_guard::no_replacements(root)?;
        if raw_identity(&self.head, root)? != (self.tree.clone(), vec![self.cut.clone()]) {
            return Err("Prepared source raw identity changed".into());
        }
        let head = github::git(
            &[
                "commit-tree",
                &self.tree,
                "-p",
                main,
                "-m",
                &format!("chore(release): integrate {} version metadata", self.tag),
            ],
            root,
        )?;
        if raw_identity(&head, root)? != (self.tree, vec![main.to_owned()]) {
            return Err(
                "Integration did not preserve the entire prepared H tree and C parent".into(),
            );
        }
        metadata::verify_delta(
            main,
            &head,
            &self.base_version,
            self.tag.trim_start_matches('v'),
            root,
        )?;
        Ok(Some(head))
    }
}

pub(super) fn open_pr(
    repository: &str,
    branch: &str,
    title: &str,
    body: &str,
    draft: bool,
    root: &Path,
) -> Result<u64, String> {
    let path = root.join(".git-release-pinned-pr.md");
    fs::write(&path, body).map_err(|e| e.to_string())?;
    let mut args = vec![
        "pr",
        "create",
        "--repo",
        repository,
        "--base",
        "main",
        "--head",
        branch,
        "--title",
        title,
        "--body-file",
        path.to_str().ok_or("Invalid PR body path")?,
    ];
    if draft {
        args.push("--draft");
    }
    let created = github::output("gh", &args, root);
    let _ = fs::remove_file(path);
    let url = created.map_err(|e|format!("{e}\nPreserved branch {branch}; inspect its existing PR before retrying. No branch or remote tag was deleted."))?;
    println!("{url}");
    url.rsplit('/')
        .next()
        .and_then(|n| n.parse().ok())
        .ok_or("Invalid created PR URL".into())
}

pub(super) fn integration_pr(
    source: &Source,
    bump: &str,
    budget: Option<&Budget>,
    prepared: Option<PreparedNormalCut>,
    operator: &lock::Operator,
    root: &Path,
) -> Result<u64, String> {
    pr_budget::check(budget)?;
    let candidate = &source.candidate;
    let branch = format!("release-integration/{}", candidate.tag);
    let found = github::output(
        "gh",
        &[
            "pr",
            "list",
            "--repo",
            &candidate.repository,
            "--head",
            &branch,
            "--state",
            "open",
            "--json",
            "number,body,state",
        ],
        root,
    )?;
    let prs: Value = serde_json::from_str(&found).map_err(|e| e.to_string())?;
    let prs = prs.as_array().ok_or("Missing integration PR list")?;
    if let [pr] = prs.as_slice() {
        // A closed receipt whose branch is gone is an abandoned cut of this
        // version. The next cut may reuse the integration name.
        let abandoned = pr["state"].as_str() == Some("CLOSED")
            && github::git(
                &[
                    "ls-remote",
                    "--heads",
                    "origin",
                    &format!("refs/heads/{branch}"),
                ],
                root,
            )?
            .is_empty();
        if !abandoned {
            let body = pr["body"]
                .as_str()
                .ok_or("Missing existing integration body")?;
            if marker(body, "vize-release-pin-head")? != candidate.head
                || marker(body, "vize-release-pin-source")? != candidate.number.to_string()
            {
                return Err(
                    "Existing integration branch belongs to a different cut; preserve it for review."
                        .into(),
                );
            }
            return pr["number"]
                .as_u64()
                .ok_or("Missing integration number".into());
        }
    }
    if prs.len() > 1 {
        return Err("Ambiguous integration PRs; refusing to replace any branch.".into());
    }
    if !github::git(
        &[
            "ls-remote",
            "--heads",
            "origin",
            &format!("refs/heads/{branch}"),
        ],
        root,
    )?
    .is_empty()
    {
        return Err(
            "An orphan integration branch exists; inspect it instead of replacing it.".into(),
        );
    }
    let mut main = github::fetch_main(root)?;
    super::stable::main_anchor(source, &main, root)?;
    pr_budget::check(budget)?;
    if let Some(prepared) = prepared {
        if let Some(head) = prepared.project(source, &main, root, &|| {
            pr_budget::check(budget)?;
            operator.verify()
        })? {
            // A moving main must use the original main-based preparation. An
            // unreferenced local object is not a published integration.
            main = github::fetch_main(root)?;
            super::stable::main_anchor(source, &main, root)?;
            if main == source.cut {
                operator.verify()?;
                if !github::git(
                    &[
                        "ls-remote",
                        "--heads",
                        "origin",
                        &format!("refs/heads/{branch}"),
                    ],
                    root,
                )?
                .is_empty()
                {
                    return Err("Integration branch appeared during prepared-tree reuse".into());
                }
                return publish_integration(source, &branch, &head, budget, root);
            }
        }
    }
    let path = pr_start::pinned_integration_worktree(&main, root)?;
    let result = (|| {
        if github::version(&path)? != source.base_version {
            return Err("Another version owns main; no new integration is allowed.".into());
        }
        pr_budget::check(budget)?;
        if source.stable.is_some() {
            pr_start::prepare("patch", &path)?;
            if format!("v{}", github::version(&path)?) != candidate.tag {
                return Err("Main metadata integration produced a different stable PATCH".into());
            }
        } else {
            super::retire::prepare(bump, &candidate.tag, budget, &path)?;
        }
        pr_budget::check(budget)?;
        let head = github::git(&["rev-parse", "HEAD"], &path)?;
        metadata::verify_delta(
            &main,
            &head,
            &source.base_version,
            candidate.tag.trim_start_matches('v'),
            &path,
        )?;
        publish_integration(source, &branch, &head, budget, &path)
    })();
    match result {
        Ok(number) => {
            github::git(
                &[
                    "worktree",
                    "remove",
                    path.to_str().ok_or("Invalid integration path")?,
                ],
                root,
            )?;
            Ok(number)
        }
        Err(error) => Err(format!(
            "{error}\nPreserved generated integration worktree: {}",
            path.display()
        )),
    }
}

fn publish_integration(
    source: &Source,
    branch: &str,
    head: &str,
    budget: Option<&Budget>,
    root: &Path,
) -> Result<u64, String> {
    pr_budget::check(budget)?;
    github::git(
        &["push", "origin", &format!("{head}:refs/heads/{branch}")],
        root,
    )?;
    let candidate = &source.candidate;
    let body = super::body::generated(
        candidate.number,
        &source.cut,
        &candidate.head,
        &candidate.tag,
    );
    open_pr(
        &candidate.repository,
        branch,
        &format!(
            "chore(release): integrate {} version metadata",
            candidate.tag
        ),
        &body,
        false,
        root,
    )
}
