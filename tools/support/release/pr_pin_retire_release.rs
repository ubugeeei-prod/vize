//! Authenticated complete published/draft Release absence with cooperative pagination.
use super::super::{
    pr_budget::{self, Budget},
    pr_github as github,
};
use serde_json::{Value, json};
use std::{collections::BTreeSet, path::Path, process::Command};

pub(super) fn absent_response(output: &std::process::Output) -> Result<(), String> {
    let text = std::str::from_utf8(&output.stdout).map_err(|e| e.to_string())?;
    let status = text
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1));
    let body = text
        .split_once("\r\n\r\n")
        .or_else(|| text.split_once("\n\n"))
        .map(|(_, body)| body)
        .ok_or("Missing absence response headers")?;
    let body: Value = serde_json::from_str(body).map_err(|e| e.to_string())?;
    if output.status.success() || status != Some("404") || body["message"] != "Not Found" {
        return Err(
            "Genuine GitHub 404 required; authentication/network/other errors are not absence"
                .into(),
        );
    }
    Ok(())
}

pub(super) fn release_page(
    value: &Value,
    repository: &str,
    tag: &str,
    ids: &mut BTreeSet<u64>,
) -> Result<bool, String> {
    let entries = value
        .as_array()
        .filter(|rows| rows.len() <= 100)
        .ok_or("Missing bounded authenticated Release inventory")?;
    for release in entries {
        let id = release["id"]
            .as_u64()
            .filter(|id| *id > 0 && ids.insert(*id))
            .ok_or("Missing or duplicate Release inventory identity")?;
        let name = release["tag_name"]
            .as_str()
            .filter(|name| !name.is_empty())
            .ok_or("Missing typed Release tag identity")?;
        if release["draft"].as_bool().is_none()
            || release["prerelease"].as_bool().is_none()
            || !(release.get("published_at") == Some(&Value::Null)
                || release["published_at"]
                    .as_str()
                    .is_some_and(|s| !s.is_empty()))
            || release["url"] != format!("https://api.github.com/repos/{repository}/releases/{id}")
        {
            return Err("Missing draft/public Release state".into());
        }
        if name == tag {
            return Err(
                "Any original GitHub Release, including a draft, forbids retirement".into(),
            );
        }
    }
    Ok(entries.len() < 100)
}

pub(super) fn absence(
    repository: &str,
    tag: &str,
    budget: Option<&Budget>,
    root: &Path,
) -> Result<Value, String> {
    pr_budget::check(budget)?;
    let output = Command::new("gh")
        .args([
            "api",
            "--include",
            &format!("repos/{repository}/releases/tags/{tag}"),
        ])
        .current_dir(root)
        .output()
        .map_err(|e| e.to_string())?;
    absent_response(&output)?;
    let mut pages = Vec::new();
    let mut ids = BTreeSet::new();
    for page in 1..=100 {
        pr_budget::check(budget)?;
        let value = github::api(
            repository,
            &format!("releases?per_page=100&page={page}"),
            root,
        )?;
        pr_budget::check(budget)?;
        let last = release_page(&value, repository, tag, &mut ids)?;
        pages.push(value);
        if last {
            return Ok(json!({"publishedTagLookup":"authenticated typed HTTP 404", "pages":pages}));
        }
    }
    Err("Authenticated Release inventory exceeded its pagination bound".into())
}
