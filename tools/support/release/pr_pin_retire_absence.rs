//! Structural custody of retained absence metadata; fresh raw-H absence remains authoritative.
use super::{retire_archive as archive, retire_ledger::text};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

fn token(value: &str, extra: &[u8]) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || extra.contains(&b))
}
fn digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn stamp(value: &Value, field: &str) -> Result<(), String> {
    let value = text(value, field)?;
    if !value.contains('T') || !value.ends_with('Z') {
        return Err("Retained absence timestamp missing/invalid".into());
    }
    Ok(())
}
fn targets(plan: &Value, identity: &Value) -> Result<BTreeMap<(String, String), String>, String> {
    let version = text(plan, "/version")?;
    if plan["head"] != identity["head"]
        || plan["parentCut"] != identity["cut"]
        || format!("v{version}") != text(identity, "/tag")?
    {
        return Err("Retained publication plan H/C/version differs".into());
    }
    let mut result = BTreeMap::new();
    for channel in ["npm", "crates"] {
        let rows = plan[channel]
            .as_array()
            .filter(|r| !r.is_empty() && r.len() <= 1000)
            .ok_or("Missing bounded retained registry plan")?;
        for row in rows {
            let name = text(row, "/name")?;
            let valid = if channel == "npm" {
                matches!(name, "vize" | "oxlint-plugin-vize")
                    || name
                        .strip_prefix("@vizejs/")
                        .is_some_and(|n| token(n, b"-"))
            } else {
                name == "vize" || name.strip_prefix("vize_").is_some_and(|n| token(n, b"_"))
            };
            if !valid || text(row, "/version")? != version {
                return Err("Retained registry plan name/version differs".into());
            }
            let url = if channel == "npm" {
                format!(
                    "https://registry.npmjs.org/{}/{version}",
                    name.replace('@', "%40").replace('/', "%2F")
                )
            } else {
                format!("https://crates.io/api/v1/crates/{name}/{version}")
            };
            if result.insert((channel.into(), name.into()), url).is_some() {
                return Err("Duplicate retained registry target".into());
            }
        }
    }
    let editor = &plan["editor"];
    let publisher = text(editor, "/publisher")?;
    let extension = text(editor, "/name")?;
    if !token(publisher, b"-") || !token(extension, b"-") || text(editor, "/version")? != version {
        return Err("Retained editor plan differs".into());
    }
    let name = format!("{publisher}.{extension}");
    result.insert(("marketplace".into(), name.clone()), format!("https://marketplace.visualstudio.com/_apis/gallery/publishers/{publisher}/extensions/{extension}?flags=1&api-version=7.2-preview.2"));
    result.insert(
        ("openvsx".into(), name),
        format!("https://open-vsx.org/api/{publisher}/{extension}/{version}"),
    );
    let authority = &plan["authority"];
    super::super::pr_contract::sha(text(authority, "/tree")?)?;
    if !digest(text(authority, "/commitSha256")?) {
        return Err("Missing retained raw-H digest metadata".into());
    }
    let blobs = authority["blobs"]
        .as_array()
        .filter(|r| !r.is_empty() && r.len() <= 1000)
        .ok_or("Missing retained raw-H blob metadata")?;
    let mut paths = BTreeMap::new();
    for blob in blobs {
        let path = text(blob, "/path")?;
        if path.starts_with('/')
            || path
                .split('/')
                .any(|p| p.is_empty() || p == "." || p == "..")
            || !path
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"_./-".contains(&b))
            || !digest(text(blob, "/sha256")?)
        {
            return Err("Invalid/duplicate retained raw blob metadata".into());
        }
        let oid = text(blob, "/oid")?;
        super::super::pr_contract::sha(oid)?;
        // The raw-H planner retains repeated reads (native manifest is read twice).
        let metadata = (oid, text(blob, "/sha256")?);
        if paths
            .insert(path, metadata)
            .is_some_and(|prior| prior != metadata)
        {
            return Err("Contradictory repeated raw blob metadata".into());
        }
    }
    Ok(result)
}

fn missing(channel: &str, name: &str, version: &str, body: &Value) -> Result<(), String> {
    let object = body.as_object().ok_or("Missing retained response object")?;
    let valid = match channel {
        "npm" => {
            object.len() == 1 && matches!(body["error"].as_str(), Some("Not found"))
                || (object.len() == 1 && body["error"] == format!("version not found: {version}"))
        }
        "crates" => {
            object.len() == 1
                && body["errors"].as_array().is_some_and(|rows| {
                    rows.len() == 1
                        && rows[0].as_object().is_some_and(|row| row.len() == 1)
                        && [
                            format!("crate `{name}` does not exist"),
                            format!("crate `{name}` does not have a version `{version}`"),
                        ]
                        .iter()
                        .any(|detail| rows[0]["detail"].as_str() == Some(detail.as_str()))
                })
        }
        "openvsx" => {
            object.len() == 1 && body["error"] == format!("Extension not found: {name} {version}")
        }
        "marketplace" => {
            let kind = text(body, "/typeKey")?;
            let message = text(body, "/message")?;
            let lower = message.to_lowercase();
            matches!(
                kind,
                "ExtensionNotFoundException"
                    | "ExtensionVersionNotFoundException"
                    | "ExtensionDoesNotExistException"
                    | "ExtensionVersionDoesNotExistException"
                    | "VersionNotFoundException"
            ) && message.contains(name)
                && (!kind.contains("Version") || message.contains(version))
                && (lower.contains("not found")
                    || lower.contains("does not exist")
                    || (lower.contains("does not have") && lower.contains("version")))
                && !["publisher", "extensionName", "version", "versions"]
                    .iter()
                    .any(|key| object.contains_key(*key))
        }
        _ => false,
    };
    if valid {
        Ok(())
    } else {
        Err("Unsupported retained typed missing response".into())
    }
}
fn marketplace(name: &str, version: &str, body: &Value) -> Result<(), String> {
    let (publisher, extension) = name.split_once('.').ok_or("Invalid retained editor name")?;
    if text(body, "/publisher/publisherName")? != publisher
        || text(body, "/extensionName")? != extension
    {
        return Err("Retained Marketplace identity differs".into());
    }
    text(body, "/publisher/publisherId")?;
    text(body, "/extensionId")?;
    let rows = body["versions"]
        .as_array()
        .filter(|r| !r.is_empty() && r.len() <= 10_000)
        .ok_or("Missing bounded complete Marketplace versions")?;
    for key in [
        "pagingToken",
        "continuationToken",
        "nextLink",
        "@odata.nextLink",
        "truncated",
        "hasMore",
        "latestOnly",
        "includeLatestVersionOnly",
    ] {
        if body.get(key).is_some() {
            return Err("Retained Marketplace inventory is partial".into());
        }
    }
    for key in ["versionCount", "totalVersions", "totalCount", "count"] {
        if body
            .get(key)
            .is_some_and(|v| v.as_u64() != Some(rows.len() as u64))
        {
            return Err("Retained Marketplace count differs".into());
        }
    }
    let mut seen = BTreeSet::new();
    for row in rows {
        let published = text(row, "/version")?;
        let (base, suffix) = published
            .find(['-', '+'])
            .map_or((published, None), |index| {
                (&published[..index], Some(&published[index + 1..]))
            });
        let pieces: Vec<_> = base.split('.').collect();
        let platform = match row.get("targetPlatform") {
            None | Some(Value::Null) => "",
            Some(Value::String(value)) => value,
            _ => return Err("Malformed retained Marketplace platform".into()),
        };
        if suffix.is_some_and(|value| {
            value.is_empty()
                || !value
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b".-".contains(&b))
        }) || pieces.len() != 3
            || pieces
                .iter()
                .any(|p| p.is_empty() || !p.bytes().all(|b| b.is_ascii_digit()))
            || !published
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b".-+".contains(&b))
            || published == version
            || !seen.insert((published, platform))
        {
            return Err(
                "Malformed/duplicate/already published retained Marketplace version".into(),
            );
        }
    }
    Ok(())
}

pub(super) fn validate(receipt: &Value) -> Result<(), String> {
    let identity = archive::identity(receipt)?;
    let absence = &receipt["registryAbsence"];
    if absence["schema"] != "vize-retired-unpublished-registry-absence-v1"
        || absence["head"] != identity["head"]
        || absence["tag"] != identity["tag"]
    {
        return Err("Missing/foreign retained registry absence receipt".into());
    }
    stamp(absence, "/observedAt")?;
    let plan = &absence["plan"];
    let expected = targets(plan, identity)?;
    let observations = absence["observations"]
        .as_array()
        .filter(|r| r.len() == expected.len())
        .ok_or("Incomplete retained registry observations")?;
    let version = text(plan, "/version")?;
    let mut seen = BTreeSet::new();
    for observation in observations {
        let channel = text(observation, "/channel")?;
        let name = text(observation, "/name")?;
        let key = (channel.to_string(), name.to_string());
        if !seen.insert(key.clone())
            || expected.get(&key).map(String::as_str) != Some(text(observation, "/url")?)
            || text(observation, "/version")? != version
            || !digest(text(observation, "/responseSha256")?)
            || observation
                .get("date")
                .is_none_or(|v| !v.is_null() && v.as_str().is_none_or(str::is_empty))
        {
            return Err("Retained registry observation identity/metadata differs".into());
        }
        stamp(observation, "/observedAt")?;
        let status = observation["status"]
            .as_u64()
            .ok_or("Missing retained HTTP status")?;
        let limit = if channel == "marketplace" && status == 200 {
            2 * 1024 * 1024
        } else {
            65_536
        };
        if observation["responseBytes"]
            .as_u64()
            .is_none_or(|n| n == 0 || n > limit)
        {
            return Err("Retained HTTP body byte metadata out of bounds".into());
        }
        let body = &observation["response"];
        if status == 404 && observation["evidence"] == "typed-exact-version-missing" {
            missing(channel, name, version, body)?;
        } else if channel == "marketplace"
            && status == 200
            && observation["evidence"] == "complete-marketplace-version-inventory"
        {
            marketplace(name, version, body)?;
        } else {
            return Err("Retained registry status/evidence is not supported absence".into());
        }
    }
    Ok(())
}
