//! `vize lib outdated`: pulled versions vs. the installed and latest registries.

use serde::Serialize;
use vize_s0::String;

use super::LibContext;
use super::error::LibResult;
use super::fetch::latest_version;
use super::output::{json, line};
use super::resolve::{Origin, Source};

/// Per-item verdict.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum OutdatedState {
    /// The available registry ships identical sources.
    UpToDate,
    /// The available registry ships different sources (`vize lib update`).
    UpdateAvailable,
    /// A newer package version is published but not installed (content unknown).
    NewerRelease,
    /// The available registry no longer publishes the item.
    RemovedUpstream,
    /// No registry could be consulted.
    Unknown,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OutdatedItem {
    item: String,
    source: String,
    package: String,
    /// Version recorded in the lockfile.
    current: String,
    /// Version of the registry `update` would use (installed, or latest when not installed).
    wanted: Option<String>,
    /// Latest published npm version, when the source is an npm package and not --offline.
    latest: Option<String>,
    state: OutdatedState,
}

fn npm_package(context: &LibContext, source: &Source) -> Option<String> {
    match source {
        Source::Namespace(name) => match &context.resolver.namespace(name)?.origin {
            Origin::Npm { package, .. } => Some(package.clone()),
            _ => None,
        },
        _ => source.builtin_package().map(String::from),
    }
}

pub fn outdated(context: &mut LibContext) -> LibResult<String> {
    let lockfile = context.lockfile()?;
    let mut latest_cache: Vec<(String, Option<String>)> = Vec::new();
    let mut rows = Vec::with_capacity(lockfile.items.len());
    for locked in &lockfile.items {
        let source = Source::parse(&locked.kind);
        let mut wanted = None;
        let mut state = OutdatedState::Unknown;
        if let Some(source) = &source {
            let offline = context.resolver.offline();
            let available = if context.resolver.local_registry(source).is_some() || offline {
                context.resolver.local_registry(source)
            } else {
                context.resolver.registry(source, None).ok()
            };
            if let Some(registry) = available {
                wanted = Some(registry.manifest.package.version.clone());
                state = match registry.item(&locked.name) {
                    None => OutdatedState::RemovedUpstream,
                    Some(item) if item.content_hash == locked.content_hash => {
                        OutdatedState::UpToDate
                    }
                    Some(_) => OutdatedState::UpdateAvailable,
                };
            }
        }
        let package = source
            .as_ref()
            .and_then(|source| npm_package(context, source));
        let latest = match &package {
            Some(package) if !context.resolver.offline() => {
                match latest_cache.iter().find(|(seen, _)| seen == package) {
                    Some((_, version)) => version.clone(),
                    None => {
                        let version = latest_version(&context.resolver.tools().npm, package).ok();
                        latest_cache.push((package.clone(), version.clone()));
                        version
                    }
                }
            }
            _ => None,
        };
        if state == OutdatedState::UpToDate
            && latest
                .as_deref()
                .is_some_and(|latest| latest != locked.version && Some(latest) != wanted.as_deref())
        {
            state = OutdatedState::NewerRelease;
        }
        rows.push(OutdatedItem {
            item: locked.label(),
            source: locked.kind.clone(),
            package: locked.package.clone(),
            current: locked.version.clone(),
            wanted,
            latest,
            state,
        });
    }
    if context.json {
        return json(&serde_json::json!({ "items": rows }));
    }
    let mut out = String::default();
    let stale: Vec<&OutdatedItem> = rows
        .iter()
        .filter(|row| row.state != OutdatedState::UpToDate)
        .collect();
    if rows.is_empty() {
        line(&mut out, format_args!("nothing pulled yet"));
        return Ok(out);
    }
    if stale.is_empty() {
        line(
            &mut out,
            format_args!("all {} pulled items are up to date", rows.len()),
        );
        return Ok(out);
    }
    line(
        &mut out,
        format_args!(
            "{:<32} {:<12} {:<12} {:<12} state",
            "item", "current", "wanted", "latest"
        ),
    );
    for row in stale {
        let state = match row.state {
            OutdatedState::UpToDate => "up-to-date",
            OutdatedState::UpdateAvailable => "update-available",
            OutdatedState::NewerRelease => "newer-release",
            OutdatedState::RemovedUpstream => "removed-upstream",
            OutdatedState::Unknown => "unknown",
        };
        line(
            &mut out,
            format_args!(
                "{:<32} {:<12} {:<12} {:<12} {state}",
                row.item,
                row.current,
                row.wanted.as_deref().unwrap_or("-"),
                row.latest.as_deref().unwrap_or("-")
            ),
        );
    }
    Ok(out)
}
