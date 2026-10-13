//! Each admitted whole body must be an actual changed original or reviewed transport.
use super::super::super::{pr_contract, pr_github as github};
use super::Stable;
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
fn field<'a>(v: &'a Value, p: &str) -> Result<&'a str, String> {
    pr_contract::field(v, p)
}

#[cfg(test)]
#[path = "pr_pin_stable_files_tests.rs"]
mod tests;
fn empty_origin(v: &Value) -> Result<&str, String> {
    v["origin"].as_str().ok_or("Missing origin string".into())
}
fn entry(revision: &str, path: &str, root: &Path) -> Result<String, String> {
    github::git(
        &["--no-replace-objects", "ls-tree", revision, "--", path],
        root,
    )
}

pub(super) fn verify(
    v: &Value,
    context: &Stable,
    public: &str,
    cut: &str,
    merged: &BTreeSet<&str>,
    raw: Vec<u8>,
    root: &Path,
) -> Result<(), String> {
    let files = v["files"]
        .as_array()
        .ok_or("Missing whole backport file authority")?;
    let mut names = BTreeSet::new();
    let mut shipping = 0;
    let mut used = BTreeSet::new();
    let parents: BTreeMap<_, _> = v["fixes"]
        .as_array()
        .ok_or("Missing fixes")?
        .iter()
        .map(|f| Ok((field(f, "/merge")?, field(f, "/parent")?)))
        .collect::<Result<_, String>>()?;
    for f in files {
        let keys: BTreeSet<_> = f
            .as_object()
            .ok_or("Missing file authority")?
            .keys()
            .map(String::as_str)
            .collect();
        if keys != ["path", "role", "origin"].into_iter().collect() {
            return Err("Unknown or omitted stable file fields".into());
        }
        let path = field(f, "/path")?;
        if !names.insert(path)
            || path == super::PLAN_PATH
            || path
                .split('/')
                .any(|s| s.is_empty() || s == "." || s == "..")
            || path.contains(['\n', '\t', '\\'])
        {
            return Err("Unknown, duplicate, escaped or self-referential stable path".into());
        }
        let after = github::git(&["--no-replace-objects", "ls-tree", cut, "--", path], root)?;
        match field(f, "/role")? {
            "fix" | "corpus" => {
                let origin = field(f, "/origin")?;
                let parent = parents.get(origin).ok_or("Unknown original fix")?;
                if entry(parent, path, root)? == entry(origin, path, root)? {
                    return Err("The claimed original fix never changed this whole path".into());
                }
                if entry(public, path, root)? != entry(parent, path, root)? {
                    return Err(
                        "Backport preimage differs from the original applicable whole body/mode"
                            .into(),
                    );
                }
                if !merged.contains(origin)
                    || after
                        != github::git(
                            &["--no-replace-objects", "ls-tree", origin, "--", path],
                            root,
                        )?
                {
                    return Err(
                        "Backported fix/corpus differs from the complete original merged body"
                            .into(),
                    );
                }
                if field(f, "/role")? == "fix" {
                    shipping += 1;
                    used.insert(origin);
                }
            }
            "qualification" => {
                let allowed = path.starts_with("tools/support/release/")
                    || path == "tools/commands/release/pr.rs"
                    || path == ".github/workflows/release.yml"
                    || path.starts_with(".github/actions/setup-moonbit/")
                    || path.starts_with(".github/actions/setup-rust-script/");
                if !allowed
                    || !empty_origin(f)?.is_empty()
                    || after
                        != github::git(
                            &[
                                "--no-replace-objects",
                                "ls-tree",
                                &context.policy,
                                "--",
                                path,
                            ],
                            root,
                        )?
                {
                    return Err("Unreviewed stable qualification transport".into());
                }
            }
            "corpus-adapter" => {
                if !path.starts_with("tests/")
                    && !(path.starts_with("crates/") && path.contains("/tests/"))
                {
                    return Err("An adapter cannot alter shipping source".into());
                }
                if !empty_origin(f)?.is_empty() {
                    return Err("Corpus adapter falsely claims an unchanged original body".into());
                }
            }
            _ => return Err("Unknown stable closure role".into()),
        }
    }
    let observed: BTreeSet<_> = String::from_utf8(raw)
        .map_err(|e| e.to_string())?
        .lines()
        .map(|l| {
            l.split_once('\t')
                .map(|(_, p)| p.to_owned())
                .ok_or("Invalid raw stable path")
        })
        .collect::<Result<_, _>>()?;
    if shipping == 0
        || used != *merged
        || observed != names.into_iter().map(str::to_owned).collect()
    {
        return Err("Extra/omitted backport paths or empty shipping fix closure".into());
    }
    Ok(())
}
