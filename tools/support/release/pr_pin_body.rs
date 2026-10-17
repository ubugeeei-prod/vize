//! Exact generated integration policy and bounded, nonauthoritative PR decorations.
use serde_json::{Value, json};
use std::{fs, path::Path};

#[cfg(test)]
#[path = "pr_pin_body_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "pr_pin_body_observed_tests.rs"]
mod observed_tests;

const CODESMITH: &str = "\n<!-- codesmith:footer -->\n";
const CODESMITH_END: &str = "\n<!-- /codesmith:footer -->";
const CODERABBIT: &str = "\n<!-- This is an auto-generated comment: release notes by coderabbit.ai -->\n\n## Summary by CodeRabbit\n\n";
const CODERABBIT_END: &str =
    "\n<!-- end of auto-generated comment: release notes by coderabbit.ai -->";
const LEGACY_POLICY: &str = "The maintainer owns protected queue admission; this command never auto-merges this PR or writes main. Admit this integration only after immutable H/R qualification has succeeded: all required H checks, all five full H gates, every release build, and full preflight must be terminal green.";
const QUEUE_POLICY: &str = "The official fresh minor operator may request protected queue checks under its exact versioned lease and original absolute deadline; ordinary required PR checks still govern actual queue admission. Required integration checks must freshly authenticate the existing exact-H full qualification and every required job, plus the original Release candidate-ready join of every build and full preflight, before merge. Legacy commands and resumes retain serial admission. Tag creation and publication still require all original H checks, every release build, full preflight, and the actual signed protected integration merge. This command never writes main directly.";

pub(super) fn generated(source_number: u64, cut: &str, source_head: &str, tag: &str) -> String {
    format!(
        "Integrate only generated {} version metadata into moving main. Source PR #{} remains an immutable draft receipt at H={}. {QUEUE_POLICY} Actual delivery is checked against its final first parent, not the initial base.\n\n<!-- vize-release-pin-source: {} -->\n<!-- vize-release-pin-cut: {} -->\n<!-- vize-release-pin-head: {} -->\n<!-- vize-release-pin-tag: {} -->\n",
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
    // The bot omits the explanatory sentence in its current footer. The
    // explicit disabled marker and the repository/PR-bound links remain
    // mandatory in both observed formats.
    let suffixes = [" Autofix is disabled.", ""].map(|sentence| {
        format!(
            "\">{autofix}</a>\n<sup>Need help on this PR? Tag <code>@codesmith-bot</code> with what you need.{sentence}</sup>\n\n<!-- codesmith:autofix:disabled -->"
        )
    });
    let query = block
        .strip_prefix(&prefix)
        .and_then(|value| {
            suffixes
                .iter()
                .find_map(|suffix| value.strip_suffix(suffix))
        })
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
            .or_else(|| line.strip_prefix("- **"))
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
        } else if let Some(text) = line
            .strip_prefix("  * ")
            .or_else(|| line.strip_prefix("  - "))
        {
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
    // Immutable older H retains its exact original body and serial bytecode.
    // This recognizes one complete old canonical form, never arbitrary prose.
    let legacy = canonical.replacen(QUEUE_POLICY, LEGACY_POLICY, 1);
    let mut rest = raw
        .strip_prefix(canonical)
        .or_else(|| raw.strip_prefix(&legacy))
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
            // A complete closing marker may terminate the body. Consume
            // only its optional line ending, preserving the next envelope
            // and refusing unknown suffixes just as before.
            rest = tail.strip_prefix('\n').unwrap_or(tail);
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
