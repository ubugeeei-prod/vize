//! `vize lib remove`.

use serde::Serialize;
use vize_s0::{String, cstr};

use super::error::{LibError, LibResult};
use super::fs_ops::{ensure_project_path, file_sha256, join_relative, remove_file_and_prune};
use super::lockfile::LockedItem;
use super::output::{json, line};
use super::plan::join_dir;
use super::{LibContext, RemoveArgs};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RemovedFile {
    path: String,
    /// `delete`, `missing`, or `modified` (kept unless `--force`).
    action: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RemovedItem {
    kind: String,
    name: String,
    orphan: bool,
    files: Vec<RemovedFile>,
}

/// Expand the requested items with non-direct dependencies nothing else needs.
fn removal_set(locked: &[LockedItem], requested: &[(String, String)]) -> Vec<(String, String)> {
    let mut removing: Vec<(String, String)> = requested.to_vec();
    loop {
        let orphan = locked.iter().find(|item| {
            let key = (item.kind.clone(), item.name.clone());
            !item.direct
                && !removing.contains(&key)
                && !locked.iter().any(|other| {
                    other.kind == item.kind
                        && !removing.contains(&(other.kind.clone(), other.name.clone()))
                        && other.name != item.name
                        && other.registry_dependencies.contains(&item.name)
                })
        });
        match orphan {
            Some(item) => removing.push((item.kind.clone(), item.name.clone())),
            None => return removing,
        }
    }
}

pub fn remove(context: &mut LibContext, args: &RemoveArgs) -> LibResult<String> {
    let mut lockfile = context.lockfile()?;
    let mut requested: Vec<(String, String)> = Vec::new();
    for query in &args.items {
        match lockfile.find(query).as_slice() {
            [item] => requested.push((item.kind.clone(), item.name.clone())),
            [] => return Err(LibError::new(cstr!("{query} is not pulled"))),
            _ => {
                return Err(LibError::new(cstr!(
                    "{query} is ambiguous; prefix it with ui: or composable:"
                )));
            }
        }
    }
    for (kind, name) in &requested {
        let blocking: Vec<String> = lockfile
            .dependents(kind, name)
            .into_iter()
            .filter(|dependent| {
                !requested.contains(&(dependent.kind.clone(), dependent.name.clone()))
            })
            .map(|dependent| dependent.name.clone())
            .collect();
        if !blocking.is_empty() && !args.force {
            return Err(LibError::new(cstr!(
                "{kind}:{name} is required by {} (remove those first or pass --force)",
                blocking.join(", ")
            )));
        }
    }
    let removing = removal_set(&lockfile.items, &requested);
    let mut report = Vec::new();
    let mut modified: Vec<String> = Vec::new();
    for (kind, name) in &removing {
        let Some(item) = lockfile.get(kind, name) else {
            continue;
        };
        let base = join_dir(&context.root, &item.dir)?;
        ensure_project_path(&context.root, &base)?;
        let mut files = Vec::new();
        for (path, locked) in &item.files {
            let target = join_relative(&base, path)?;
            ensure_project_path(&context.root, &target)?;
            let action = match file_sha256(&target)? {
                None => "missing",
                Some(local) if local == *locked => "delete",
                Some(_) => {
                    modified.push(cstr!("  {kind}:{name} {path}"));
                    "modified"
                }
            };
            files.push(RemovedFile {
                path: path.clone(),
                action,
            });
        }
        report.push(RemovedItem {
            kind: kind.clone(),
            name: name.clone(),
            orphan: !requested.contains(&(kind.clone(), name.clone())),
            files,
        });
    }
    if !modified.is_empty() && !args.force && !args.dry_run {
        return Err(LibError::new(cstr!(
            "refusing to delete locally modified files (re-run with --force):\n{}",
            modified.join("\n")
        )));
    }
    if !args.dry_run {
        for removed in &report {
            let Some(item) = lockfile.get(&removed.kind, &removed.name) else {
                continue;
            };
            let base = join_dir(&context.root, &item.dir)?;
            ensure_project_path(&context.root, &base)?;
            for file in &removed.files {
                if file.action == "delete" || (file.action == "modified" && args.force) {
                    let target = join_relative(&base, &file.path)?;
                    ensure_project_path(&context.root, &target)?;
                    remove_file_and_prune(&target, &base)?;
                }
            }
            // The kind directory itself goes once its last item is removed.
            if base != context.root {
                let _ = std::fs::remove_dir(&base);
            }
        }
        for (kind, name) in &removing {
            lockfile.remove(kind, name);
        }
        lockfile.write(&context.root, &context.lock_path)?;
    }
    if context.json {
        return json(&serde_json::json!({ "dryRun": args.dry_run, "items": report }));
    }
    let mut out = String::default();
    let verb = if args.dry_run {
        "would remove"
    } else {
        "removed"
    };
    for removed in &report {
        let orphan = if removed.orphan {
            " (orphaned dependency)"
        } else {
            ""
        };
        line(
            &mut out,
            format_args!(
                "{verb} {}{}{}{orphan}",
                removed.kind,
                if removed.kind.starts_with('@') {
                    "/"
                } else {
                    ":"
                },
                removed.name
            ),
        );
        for file in removed.files.iter().filter(|file| file.action != "delete") {
            line(&mut out, format_args!("  {} {}", file.action, file.path));
        }
    }
    Ok(out)
}
