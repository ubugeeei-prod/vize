//! `vize lib status` and `vize lib diff`.

use std::collections::BTreeSet;

use serde::Serialize;
use similar::TextDiff;
use vize_s0::{String, cstr};

use super::LibContext;
use super::error::{LibError, LibResult};
use super::fs_ops::{file_sha256, join_relative};
use super::lockfile::LockedItem;
use super::output::{json, line};
use super::plan::join_dir;
use super::resolve::RegistryKind;

/// Local state of one pulled file relative to the lockfile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LocalState {
    Clean,
    Modified,
    Missing,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct FileStatus {
    path: String,
    state: LocalState,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ItemStatus {
    kind: String,
    name: String,
    version: String,
    dir: String,
    direct: bool,
    state: LocalState,
    /// `up-to-date`, `update-available`, `removed-upstream`, or `unknown`.
    upstream: &'static str,
    registry_version: Option<String>,
    files: Vec<FileStatus>,
}

fn local_files(context: &LibContext, item: &LockedItem) -> LibResult<Vec<FileStatus>> {
    let base = join_dir(&context.root, &item.dir)?;
    let mut files = Vec::with_capacity(item.files.len());
    for (path, locked) in &item.files {
        let state = match file_sha256(&join_relative(&base, path)?)? {
            None => LocalState::Missing,
            Some(local) if local == *locked => LocalState::Clean,
            Some(_) => LocalState::Modified,
        };
        files.push(FileStatus {
            path: path.clone(),
            state,
        });
    }
    Ok(files)
}

pub fn status(context: &mut LibContext) -> LibResult<String> {
    let lockfile = context.lockfile()?;
    let mut statuses = Vec::with_capacity(lockfile.items.len());
    for item in &lockfile.items {
        let files = local_files(context, item)?;
        let state = if files.iter().any(|file| file.state == LocalState::Missing) {
            LocalState::Missing
        } else if files.iter().any(|file| file.state == LocalState::Modified) {
            LocalState::Modified
        } else {
            LocalState::Clean
        };
        let upstream = RegistryKind::parse(&item.kind)
            .and_then(|kind| context.resolver.local_registry(kind))
            .map(|registry| {
                let version = registry.manifest.package.version.clone();
                match registry.item(&item.name) {
                    None => ("removed-upstream", Some(version)),
                    Some(latest) if latest.content_hash == item.content_hash => {
                        ("up-to-date", Some(version))
                    }
                    Some(_) => ("update-available", Some(version)),
                }
            });
        let (upstream, registry_version) = upstream.unwrap_or(("unknown", None));
        statuses.push(ItemStatus {
            kind: item.kind.clone(),
            name: item.name.clone(),
            version: item.version.clone(),
            dir: item.dir.clone(),
            direct: item.direct,
            state,
            upstream,
            registry_version,
            files,
        });
    }
    if context.json {
        return json(&serde_json::json!({ "items": statuses }));
    }
    let mut out = String::default();
    if statuses.is_empty() {
        line(&mut out, format_args!("nothing pulled yet"));
    }
    for status in &statuses {
        let state = match status.state {
            LocalState::Clean => "clean",
            LocalState::Modified => "modified",
            LocalState::Missing => "missing files",
        };
        let upstream = match (status.upstream, status.registry_version.as_deref()) {
            ("update-available", Some(version)) => cstr!(", update available ({version})"),
            ("removed-upstream", Some(version)) => cstr!(", removed in {version}"),
            _ => String::default(),
        };
        line(
            &mut out,
            format_args!(
                "{}:{} {} {state}{upstream}{}",
                status.kind,
                status.name,
                status.version,
                if status.direct { "" } else { " (dependency)" }
            ),
        );
        for file in status
            .files
            .iter()
            .filter(|file| file.state != LocalState::Clean)
        {
            let marker = if file.state == LocalState::Missing {
                "missing"
            } else {
                "modified"
            };
            line(&mut out, format_args!("  {marker} {}", file.path));
        }
    }
    Ok(out)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct FileDiff {
    path: String,
    local: Option<LocalState>,
    upstream_changed: bool,
    diff: String,
}

/// Unified diff from the local copy of an item to a registry version.
pub fn diff(context: &mut LibContext, query: &str, to: Option<&str>) -> LibResult<String> {
    let lockfile = context.lockfile()?;
    let locked = match lockfile.find(query).as_slice() {
        [item] => (*item).clone(),
        [] => return Err(LibError::new(cstr!("{query} is not pulled"))),
        _ => {
            return Err(LibError::new(cstr!(
                "{query} is ambiguous; prefix it with ui: or composable:"
            )));
        }
    };
    let kind = RegistryKind::parse(&locked.kind)
        .ok_or_else(|| LibError::new(cstr!("unknown kind {} in lockfile", locked.kind)))?;
    let base = join_dir(&context.root, &locked.dir)?;
    let as_json = context.json;
    let registry = context.resolver.registry(kind, to)?;
    let label = registry.package_label();
    let item = registry.item(&locked.name);
    let mut paths: BTreeSet<String> = locked.files.keys().cloned().collect();
    paths.extend(
        item.iter()
            .flat_map(|item| item.files.iter().map(|file| file.path.clone())),
    );
    let mut diffs = Vec::new();
    for path in paths {
        let target = join_relative(&base, &path)?;
        let local_bytes = std::fs::read(&target).ok();
        let upstream_file = item.and_then(|item| item.files.iter().find(|file| file.path == path));
        let upstream_bytes = match upstream_file {
            Some(file) => Some(registry.read_file(file)?),
            None => None,
        };
        let locked_sha = locked.files.get(&path);
        let local_state = match file_sha256(&target)? {
            None => None,
            Some(local) if Some(&local) == locked_sha => Some(LocalState::Clean),
            Some(_) => Some(LocalState::Modified),
        };
        let upstream_changed = upstream_file.map(|file| &file.sha256) != locked_sha;
        let old = local_bytes
            .as_deref()
            .map(String::from_utf8_lossy)
            .unwrap_or_default();
        let new = upstream_bytes
            .as_deref()
            .map(String::from_utf8_lossy)
            .unwrap_or_default();
        if old == new {
            continue;
        }
        let local_header = cstr!("a/{path} (local)");
        let upstream_header = cstr!("b/{path} ({label})");
        let text_diff = TextDiff::from_lines(old.as_str(), new.as_str());
        let mut unified = text_diff.unified_diff();
        unified
            .context_radius(3)
            .header(&local_header, &upstream_header);
        let rendered = cstr!("{unified}");
        diffs.push(FileDiff {
            path,
            local: local_state,
            upstream_changed,
            diff: rendered,
        });
    }
    if as_json {
        return json(&serde_json::json!({
            "kind": locked.kind,
            "name": locked.name,
            "lockedVersion": locked.version,
            "registryVersion": registry.manifest.package.version,
            "files": diffs,
        }));
    }
    let mut out = String::default();
    if diffs.is_empty() {
        line(
            &mut out,
            format_args!("{}:{} matches {label}", locked.kind, locked.name),
        );
    }
    for file in diffs {
        out.push_str(&file.diff);
    }
    Ok(out)
}
