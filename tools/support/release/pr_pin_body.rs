//! Exact generated integration policy and bounded, nonauthoritative PR decorations.
use serde_json::{Value, json};
use std::{fs, path::Path};

#[cfg(test)]
#[path = "pr_pin_body_tests.rs"]
mod tests;

const CODESMITH: &str = "\n<!-- codesmith:footer -->\n";
const CODESMITH_END: &str = "\n<!-- /codesmith:footer -->\n";
const CODERABBIT: &str = "\n<!-- This is an auto-generated comment: release notes by coderabbit.ai -->\n\n## Summary by CodeRabbit\n\n";
const CODERABBIT_END: &str =
    "\n<!-- end of auto-generated comment: release notes by coderabbit.ai -->";

pub(super) fn generated(source_number: u64, cut: &str, source_head: &str, tag: &str) -> String {
    format!(
        "Integrate only generated {} version metadata into moving main. Source PR #{} remains an immutable draft receipt at H={}. The maintainer owns protected queue admission; this command never auto-merges this PR or writes main. Admit this integration only after immutable H/R qualification has succeeded: all required H checks, all five full H gates, every release build, and full preflight must be terminal green. Actual delivery is checked against its final first parent, not the initial base.\n\n<!-- vize-release-pin-source: {} -->\n<!-- vize-release-pin-cut: {} -->\n<!-- vize-release-pin-head: {} -->\n<!-- vize-release-pin-tag: {} -->\n",
        tag, source_number, source_head, source_number, cut, source_head, tag
    )
}

fn picture(dark: &str, light: &str, alt: &str) -> String {
    let asset = "https://pr-comments-assets.blacksmith.sh/codesmith";
    format!(
        "<picture><source media=\"(prefers-color-scheme: dark)\" srcset=\"{asset}/{dark}\"><source media=\"(prefers-color-scheme: light)\" srcset=\"{asset}/{light}\"><img alt=\"{alt}\" src=\"{asset}/{dark}\"></picture>"
    )
}

fn positive_integer(value: &str) -> bool {
    value
        .parse::<u64>()
        .is_ok_and(|number| number > 0 && number.to_string() == value)
}

fn codesmith(block: &str, repository: &str, number: u64) -> Result<(), String> {
    let (owner, repo) = repository.split_once('/').ok_or("Invalid repository")?;
    let view = picture(
        "view-with-codesmith-dark-v3.svg",
        "view-with-codesmith-light-v3.svg",
        "View with [code]smith",
    );
    let autofix = picture(
        "autofix-with-codesmith-dark.svg",
        "autofix-with-codesmith-light.svg",
        "Autofix with [code]smith",
    );
    let prefix = format!(
        "---\n<a href=\"https://app.blacksmith.sh/{owner}/codesmith/{repo}/pr/{number}?autoLogin=true&ref=codesmith_pr_footer\">{view}</a> <a href=\"https://backend.blacksmith.sh/track/enable-autofix?"
    );
    let suffix = format!(
        "\">{autofix}</a>\n<sup>Need help on this PR? Tag <code>@codesmith-bot</code> with what you need. Autofix is disabled.</sup>\n\n<!-- codesmith:autofix:disabled -->"
    );
    let query = block
        .strip_prefix(&prefix)
        .and_then(|value| value.strip_suffix(&suffix))
        .ok_or("Unknown Codesmith decoration structure")?;
    let fields: Vec<_> = query
        .split('&')
        .map(|field| field.split_once('='))
        .collect();
    let encoded = repository.replace('/', "%2F");
    if fields.len() != 7
        || !matches!(fields[0], Some(("expires", value)) if positive_integer(value))
        || !matches!(fields[1], Some(("installation_model_id", value)) if positive_integer(value))
        || fields[2] != Some(("pr_number", number.to_string().as_str()))
        || fields[3] != Some(("ref", "codesmith_pr_footer"))
        || fields[4] != Some(("repository", encoded.as_str()))
        || fields[5]
            != Some((
                "return_to",
                format!("https%3A%2F%2Fgithub.com%2F{encoded}%2Fpull%2F{number}").as_str(),
            ))
        || !matches!(fields[6], Some(("signature", value)) if value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)))
    {
        return Err("Codesmith decoration does not bind the same own repository/PR".into());
    }
    Ok(())
}

fn coderabbit(block: &str) -> Result<(), String> {
    let mut group = false;
    let mut item = false;
    for line in block.lines() {
        if line.is_empty() {
            continue;
        }
        if let Some(name) = line
            .strip_prefix("* **")
            .and_then(|value| value.strip_suffix("**"))
        {
            if name.is_empty()
                || !name
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b' ' || byte == b'-')
            {
                return Err("Unknown CodeRabbit summary heading".into());
            }
            group = true;
        } else if let Some(text) = line.strip_prefix("  * ") {
            if !group || text.is_empty() || text.contains(['<', '>', '\r']) || text.contains("```")
            {
                return Err("Unknown CodeRabbit summary item".into());
            }
            item = true;
        } else {
            return Err("Unknown CodeRabbit summary structure".into());
        }
    }
    if !item {
        return Err("Empty CodeRabbit release summary".into());
    }
    Ok(())
}

/// Only the complete canonical policy is authoritative. Decoration text is never a release decision.
pub(super) fn qualify(
    raw: &str,
    canonical: &str,
    repository: &str,
    number: u64,
) -> Result<Vec<&'static str>, String> {
    let mut rest = raw
        .strip_prefix(canonical)
        .ok_or("Integration generated policy or pin fields changed")?;
    let mut seen = Vec::new();
    while !rest.is_empty() {
        if let Some(value) = rest.strip_prefix(CODESMITH) {
            if seen.contains(&"codesmith") {
                return Err("Repeated Codesmith decoration".into());
            }
            let (block, tail) = value
                .split_once(CODESMITH_END)
                .ok_or("Unterminated Codesmith decoration")?;
            codesmith(block, repository, number)?;
            seen.push("codesmith");
            rest = tail;
        } else if let Some(value) = rest.strip_prefix(CODERABBIT) {
            if seen.contains(&"coderabbit") {
                return Err("Repeated CodeRabbit decoration".into());
            }
            let (block, tail) = value
                .split_once(CODERABBIT_END)
                .ok_or("Unterminated CodeRabbit decoration")?;
            coderabbit(block)?;
            seen.push("coderabbit");
            rest = if tail == "\n" {
                ""
            } else {
                tail.strip_prefix('\n').unwrap_or(tail)
            };
        } else {
            return Err("Unknown integration body suffix".into());
        }
    }
    Ok(seen)
}

/// Preserve exactly the body observed by the validator, before any refusal can discard it.
pub(super) fn retain(
    directory: &Path,
    event: Option<(&str, &Value)>,
    current: Option<&Value>,
) -> Result<(), String> {
    fs::create_dir_all(directory).map_err(|error| error.to_string())?;
    let receipt = json!({"schema":"vize.release.integration-body-snapshot","version":1,"eventName":event.map(|(name, _)|name),"eventBody":event.and_then(|(_, value)|value.pointer("/pull_request/body")),"eventPullRequest":event.and_then(|(_, value)|value.get("pull_request")),"currentBody":current.and_then(|value|value.get("body")),"currentPullRequest":current,"qualification":"Raw input custody only; no catalog, merge, publication or editor/time claim."});
    fs::write(
        directory.join("bodies.json"),
        serde_json::to_vec_pretty(&receipt).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())
}
