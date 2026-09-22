//! Immutable identities shared by the release command and the Actions gate.

use serde_json::Value;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Candidate {
    pub number: u64,
    pub repository: String,
    pub author: String,
    pub branch: String,
    pub head: String,
    pub base: String,
    pub tag: String,
    pub merged: bool,
}

pub fn maintainer(permission: &Value) -> Result<(), String> {
    match permission.get("role_name").and_then(Value::as_str) {
        Some("maintain" | "admin") => Ok(()),
        _ => Err("Release PR authors must have the maintain or admin repository role.".into()),
    }
}

pub fn sha(value: &str) -> Result<(), String> {
    if value.len() == 40
        && value
            .bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
    {
        Ok(())
    } else {
        Err(format!("Expected a full lowercase commit SHA, got {value}"))
    }
}

pub fn tag(value: &str) -> Result<(), String> {
    let Some(version) = value.strip_prefix('v') else {
        return Err("Release tags must start with v.".into());
    };
    let (core, suffix) = version
        .split_once('-')
        .map_or((version, None), |(a, b)| (a, Some(b)));
    let parts: Vec<_> = core.split('.').collect();
    if parts.len() != 3
        || parts
            .iter()
            .any(|p| p.is_empty() || !p.bytes().all(|c| c.is_ascii_digit()))
        || suffix.is_some_and(|s| {
            s.is_empty()
                || !s
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || c == b'.' || c == b'-')
        })
    {
        return Err(format!("Invalid release tag {value}"));
    }
    Ok(())
}

pub fn candidate(
    pr: &Value,
    permission: &Value,
    repository: &str,
    expected_head: &str,
    release_tag: &str,
    allow_merged: bool,
) -> Result<Candidate, String> {
    maintainer(permission)?;
    sha(expected_head)?;
    tag(release_tag)?;
    let number = pr
        .get("number")
        .and_then(Value::as_u64)
        .filter(|n| *n > 0)
        .ok_or("Missing release PR number")?;
    let merged = pr.get("merged").and_then(Value::as_bool).unwrap_or(false);
    if (!merged && field(pr, "/state")? != "open") || (merged && !allow_merged) {
        return Err("Release PR must be open until promotion.".into());
    }
    if pr.get("draft").and_then(Value::as_bool) != Some(false) {
        return Err("A draft PR cannot be released.".into());
    }
    for pointer in ["/base/repo/full_name", "/head/repo/full_name"] {
        if field(pr, pointer)? != repository {
            return Err(
                "Release PR must come from the same repository; forks are rejected.".into(),
            );
        }
    }
    let branch = format!("release/{release_tag}");
    if field(pr, "/base/ref")? != "main" || field(pr, "/head/ref")? != branch {
        return Err("Release PR must target main from its release/vVERSION branch.".into());
    }
    if field(pr, "/head/sha")? != expected_head {
        return Err("Release PR head changed; discard the old validation result.".into());
    }
    let base = field(pr, "/base/sha")?;
    sha(base)?;
    Ok(Candidate {
        number,
        repository: repository.into(),
        author: field(pr, "/user/login")?.into(),
        branch,
        head: expected_head.into(),
        base: base.into(),
        tag: release_tag.into(),
        merged,
    })
}

pub fn current_parent(parent: &str, main: &str) -> Result<(), String> {
    sha(parent)?;
    sha(main)?;
    if parent == main {
        Ok(())
    } else {
        Err(
            "main advanced: refresh the release commit and repeat validation before merging."
                .into(),
        )
    }
}

pub fn ready_job(jobs: &[Value]) -> Result<bool, String> {
    let ready: Vec<_> = jobs
        .iter()
        .filter(|job| job.get("name").and_then(Value::as_str) == Some("Release candidate ready"))
        .collect();
    match ready.as_slice() {
        [] => Ok(false),
        [job] if job.get("status").and_then(Value::as_str) != Some("completed") => Ok(false),
        [job] if job.get("conclusion").and_then(Value::as_str) == Some("success") => Ok(true),
        [_] => Err("Release validation did not succeed; no tag was created.".into()),
        _ => Err("Ambiguous release readiness evidence.".into()),
    }
}

pub fn field<'a>(value: &'a Value, pointer: &str) -> Result<&'a str, String> {
    value
        .pointer(pointer)
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| format!("Missing GitHub field {pointer}"))
}
