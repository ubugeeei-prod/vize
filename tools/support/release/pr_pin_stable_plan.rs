//! Complete finite authority supplied by a reviewed main commit, outside C.
use super::super::super::{pr_contract, pr_github as github};
use super::super::{first_parent, metadata};
use super::{PLAN_PATH, Stable};
use serde_json::Value;
use std::{collections::BTreeSet, path::Path};

pub(super) fn patch_target(base: &str, target: &str) -> Result<(), String> {
    fn parts(tag: &str) -> Result<Vec<u64>, String> {
        let raw = tag
            .strip_prefix('v')
            .ok_or("Missing stable version prefix")?;
        let fields: Vec<_> = raw.split('.').collect();
        if fields.len() != 3
            || fields.iter().any(|p| {
                p.is_empty()
                    || (p.len() > 1 && p.starts_with('0'))
                    || !p.bytes().all(|c| c.is_ascii_digit())
            })
        {
            return Err("Stable versions must be canonical non-prerelease semver".into());
        }
        fields
            .iter()
            .map(|s| s.parse().map_err(|_| "Stable version overflow".into()))
            .collect()
    }
    let b = parts(base)?;
    let t = parts(target)?;
    if b[0] != t[0] || b[1] != t[1] || b[2].checked_add(1) != Some(t[2]) {
        return Err("Stable releases must advance the published PATCH exactly once".into());
    }
    Ok(())
}

fn commit(sha: &str, root: &Path) -> Result<(String, Vec<String>), String> {
    pr_contract::sha(sha)?;
    let raw = github::git(&["--no-replace-objects", "cat-file", "commit", sha], root)?;
    let headers: Vec<_> = raw.lines().take_while(|l| !l.is_empty()).collect();
    let trees: Vec<_> = headers
        .iter()
        .filter_map(|l| l.strip_prefix("tree "))
        .collect();
    if trees.len() != 1 {
        return Err("Stable raw commit requires one tree".into());
    }
    pr_contract::sha(trees[0])?;
    Ok((
        trees[0].into(),
        headers
            .iter()
            .filter_map(|l| l.strip_prefix("parent ").map(str::to_owned))
            .collect(),
    ))
}

fn exact_keys(value: &Value, expected: &[&str]) -> Result<(), String> {
    let keys: BTreeSet<_> = value
        .as_object()
        .ok_or("Missing stable object")?
        .keys()
        .map(String::as_str)
        .collect();
    if keys != expected.iter().copied().collect() {
        return Err("Unknown or omitted stable plan fields".into());
    }
    Ok(())
}

fn field<'a>(v: &'a Value, p: &str) -> Result<&'a str, String> {
    pr_contract::field(v, p)
}

pub(super) fn verify(
    v: &Value,
    context: &Stable,
    cut: &str,
    repository: &str,
    root: &Path,
) -> Result<(), String> {
    verify_against(
        v,
        context,
        cut,
        repository,
        &github::fetch_main(root)?,
        root,
        &|resource| github::api(repository, resource, root),
    )
}

#[cfg(test)]
fn verify_with(
    v: &Value,
    context: &Stable,
    cut: &str,
    repository: &str,
    root: &Path,
    api: &dyn Fn(&str) -> Result<Value, String>,
) -> Result<(), String> {
    verify_against(
        v,
        context,
        cut,
        repository,
        &github::fetch_main(root)?,
        root,
        api,
    )
}

pub(super) fn verify_against(
    v: &Value,
    context: &Stable,
    cut: &str,
    repository: &str,
    main: &str,
    root: &Path,
    api: &dyn Fn(&str) -> Result<Value, String>,
) -> Result<(), String> {
    super::super::retire_guard::no_replacements(root)?;
    exact_keys(
        v,
        &[
            "schema",
            "repository",
            "published",
            "target",
            "cut",
            "tree",
            "commits",
            "fixes",
            "rawDiff",
            "files",
        ],
    )?;
    if field(v, "/schema")? != "stable-patch-v1"
        || field(v, "/repository")? != repository
        || field(v, "/cut")? != cut
    {
        return Err("Stable plan repository/cut/schema mismatch".into());
    }
    let p = &v["published"];
    exact_keys(p, &["tag", "tagObject", "head", "tree", "run", "sourcePr"])?;
    let tag = field(p, "/tag")?;
    let public = field(p, "/head")?;
    patch_target(tag, field(v, "/target")?)?;
    first_parent(&context.policy, main, root)?;
    let tag_ref = api(&format!("git/ref/tags/{tag}"))?;
    if field(&tag_ref, "/object/type")? != "tag"
        || field(&tag_ref, "/object/sha")? != field(p, "/tagObject")?
    {
        return Err("Published stable tag moved or is not annotated".into());
    }
    let object = api(&format!("git/tags/{}", field(p, "/tagObject")?))?;
    if field(&object, "/tag")? != tag
        || field(&object, "/object/type")? != "commit"
        || field(&object, "/object/sha")? != public
    {
        return Err("Published stable tag does not bind its declared H".into());
    }
    let message = field(&object, "/message")?;
    let one = |key: &str| -> Result<String, String> {
        let prefix = format!("{key}: ");
        let found: Vec<_> = message
            .lines()
            .filter_map(|l| l.strip_prefix(&prefix))
            .collect();
        match found.as_slice() {
            [s] => Ok((*s).into()),
            _ => Err(format!("Ambiguous public tag field {key}")),
        }
    };
    let run = p["run"]
        .as_u64()
        .filter(|n| *n > 0)
        .ok_or("Missing public Release R")?;
    let source = p["sourcePr"]
        .as_u64()
        .filter(|n| *n > 0)
        .ok_or("Missing public source PR")?;
    if one("Release-Mode")? != "pinned"
        || one("Validated-head")? != public
        || one("Validated-run")? != run.to_string()
        || one("Release-PR")? != format!("#{source}")
    {
        return Err("Public stable annotation and original H/R/source disagree".into());
    }
    let release = api(&format!("releases/tags/{tag}"))?;
    if release["draft"] != false
        || release["prerelease"] != false
        || field(&release, "/tag_name")? != tag
        || !release["published_at"].is_string()
    {
        return Err("Stable origin was never actually published".into());
    }
    let r = api(&format!("actions/runs/{run}"))?;
    if r["id"].as_u64() != Some(run)
        || r["event"] != "workflow_dispatch"
        || r["status"] != "completed"
        || r["conclusion"] != "success"
        || field(&r, "/head_sha")? != public
        || field(&r, "/head_branch")? != format!("release/{tag}")
        || field(&r, "/path")? != ".github/workflows/release.yml"
        || field(&r, "/head_repository/full_name")? != repository
    {
        return Err("Original public Release H/R identity is not terminal success".into());
    }
    if commit(public, root)?.0 != field(p, "/tree")? || commit(cut, root)?.0 != field(v, "/tree")? {
        return Err("The whole public/backport tree differs from the reviewed plan".into());
    }
    let base = tag.trim_start_matches('v');
    if github::version_text(&metadata::text(public, "Cargo.toml", root)?)? != base
        || github::version_text(&metadata::text(cut, "Cargo.toml", root)?)? != base
    {
        return Err("Backport C altered the published base version".into());
    }
    let commits = v["commits"]
        .as_array()
        .filter(|a| !a.is_empty())
        .ok_or("Missing ordered backport commits")?;
    let mut previous = public.to_string();
    let mut seen = BTreeSet::new();
    for row in commits {
        let sha = row.as_str().ok_or("Invalid backport commit")?;
        if !seen.insert(sha) || commit(sha, root)?.1 != vec![previous.clone()] {
            return Err(
                "Backport chain has missing, duplicate, reordered or foreign parents".into(),
            );
        }
        previous = sha.into();
    }
    if previous != cut {
        return Err("Backport chain does not terminate at exact C".into());
    }
    let fixes = v["fixes"]
        .as_array()
        .filter(|a| !a.is_empty())
        .ok_or("Empty stable fix closure")?;
    let mut merged = BTreeSet::new();
    for f in fixes {
        exact_keys(f, &["pr", "merge", "parent", "reporter"])?;
        let n = f["pr"]
            .as_u64()
            .filter(|n| *n > 0)
            .ok_or("Invalid original fix PR")?;
        let m = field(f, "/merge")?;
        if !merged.insert(m) || commit(m, root)?.1 != vec![field(f, "/parent")?.to_owned()] {
            return Err("Original fix SHA/parent closure changed".into());
        }
        first_parent(m, main, root)?;
        let pr = api(&format!("pulls/{n}"))?;
        if pr["merged"] != true
            || field(&pr, "/state")? != "closed"
            || field(&pr, "/merge_commit_sha")? != m
            || field(&pr, "/base/repo/full_name")? != repository
            || field(&pr, "/head/repo/full_name")? != repository
            || !pr["merged_at"].is_string()
        {
            return Err("Stable fix is not the actual merged own-repository PR".into());
        }
        let raw = github::git(&["--no-replace-objects", "cat-file", "commit", m], root)?;
        let trailer = format!("Co-authored-by: {}", field(f, "/reporter")?);
        if !raw.lines().any(|l| l.eq_ignore_ascii_case(&trailer)) {
            return Err("Original fix lacks its verified reporter trailer".into());
        }
    }
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
        root,
    )?;
    if raw.is_empty()
        || String::from_utf8(raw.clone()).map_err(|e| e.to_string())? != field(v, "/rawDiff")?
    {
        return Err("Complete backport diff changed or omitted bytes/modes/paths".into());
    }
    super::files::verify(v, context, public, cut, &merged, raw, root)?;
    Ok(())
}

#[cfg(test)]
#[path = "pr_pin_stable_archive_tests.rs"]
mod archive_tests;
#[cfg(test)]
#[path = "pr_pin_stable_plan_tests.rs"]
mod tests;
