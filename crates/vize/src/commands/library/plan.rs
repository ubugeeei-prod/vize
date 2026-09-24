//! Three-way install planning shared by `pull` and `update`.
//!
//! For every file the plan compares the local bytes, the digest recorded in
//! the lockfile at pull time (the merge base), and the incoming registry
//! digest, and never writes anything until the whole plan is conflict-free
//! (or the caller forces it).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Serialize;
use vize_s0::{String, cstr};

use super::error::{LibError, LibResult};
use super::fs_ops::{file_sha256, join_relative, remove_file_and_prune, write_file};
use super::lockfile::{LockedItem, Lockfile};
use super::registry::{LoadedRegistry, NpmDependency, RegistryItem};

/// What happens to one file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FileAction {
    /// Absent locally; written from the registry.
    Create,
    /// Pristine locally and changed upstream; replaced.
    Update,
    /// Already identical to the registry.
    Unchanged,
    /// Edited locally, unchanged upstream; the local edit is preserved.
    KeepLocal,
    /// Pristine locally and dropped upstream; deleted.
    Delete,
    /// Edited locally (or untracked) and changed upstream; needs `--force`.
    Conflict,
    /// Edited locally and dropped upstream; needs `--force` to delete.
    ConflictDelete,
}

impl FileAction {
    pub fn is_conflict(self) -> bool {
        matches!(self, Self::Conflict | Self::ConflictDelete)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Update => "update",
            Self::Unchanged => "unchanged",
            Self::KeepLocal => "keep-local",
            Self::Delete => "delete",
            Self::Conflict => "conflict",
            Self::ConflictDelete => "conflict-delete",
        }
    }
}

/// One file in a plan.
#[derive(Debug, Clone, Serialize)]
pub struct FilePlan {
    pub path: String,
    pub action: FileAction,
    #[serde(skip)]
    pub target: PathBuf,
    #[serde(skip)]
    pub contents: Option<Vec<u8>>,
}

/// One item in a plan.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemPlan {
    pub kind: String,
    pub name: String,
    pub direct: bool,
    pub from_version: Option<String>,
    pub to_version: String,
    pub dir: String,
    pub files: Vec<FilePlan>,
    pub npm_dependencies: Vec<NpmDependency>,
    #[serde(skip)]
    pub lock: LockedItem,
}

impl ItemPlan {
    pub fn conflicts(&self) -> impl Iterator<Item = &FilePlan> {
        self.files.iter().filter(|file| file.action.is_conflict())
    }

    pub fn changes_anything(&self) -> bool {
        self.lock_changed()
            || self
                .files
                .iter()
                .any(|file| !matches!(file.action, FileAction::Unchanged | FileAction::KeepLocal))
    }

    fn lock_changed(&self) -> bool {
        self.from_version.as_deref() != Some(self.to_version.as_str())
    }
}

/// Plan one registry item into `root/dir`, three-way against `existing`.
pub fn plan_item(
    root: &Path,
    dir: &str,
    registry: &LoadedRegistry,
    item: &RegistryItem,
    existing: Option<&LockedItem>,
    direct: bool,
) -> LibResult<ItemPlan> {
    let base = join_dir(root, dir)?;
    let mut files = Vec::with_capacity(item.files.len());
    let mut locked_files = BTreeMap::new();
    for file in &item.files {
        let target = join_relative(&base, &file.path)?;
        let local = file_sha256(&target)?;
        let base_sha = existing.and_then(|locked| locked.files.get(&file.path));
        let action = match local.as_deref() {
            None => FileAction::Create,
            Some(local) if local == file.sha256 => FileAction::Unchanged,
            Some(_) if base_sha == Some(&file.sha256) => FileAction::KeepLocal,
            Some(local) if base_sha.is_some_and(|base| base == local) => FileAction::Update,
            Some(_) => FileAction::Conflict,
        };
        let contents = match action {
            FileAction::Create | FileAction::Update | FileAction::Conflict => {
                Some(registry.read_file(file)?)
            }
            _ => None,
        };
        locked_files.insert(file.path.clone(), file.sha256.clone());
        files.push(FilePlan {
            path: file.path.clone(),
            action,
            target,
            contents,
        });
    }
    for (path, base_sha) in existing.map(|locked| &locked.files).into_iter().flatten() {
        if locked_files.contains_key(path) {
            continue;
        }
        let target = join_relative(&base, path)?;
        let action = match file_sha256(&target)? {
            None => continue,
            Some(local) if local == *base_sha => FileAction::Delete,
            Some(_) => FileAction::ConflictDelete,
        };
        files.push(FilePlan {
            path: path.clone(),
            action,
            target,
            contents: None,
        });
    }
    files.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(ItemPlan {
        kind: item.kind.clone(),
        name: item.name.clone(),
        direct,
        from_version: existing.map(|locked| locked.version.clone()),
        to_version: registry.manifest.package.version.clone(),
        dir: dir.into(),
        files,
        npm_dependencies: item.dependencies.clone(),
        lock: LockedItem {
            name: item.name.clone(),
            kind: item.kind.clone(),
            package: registry.manifest.package.name.clone(),
            version: registry.manifest.package.version.clone(),
            content_hash: item.content_hash.clone(),
            dir: dir.into(),
            direct,
            registry_dependencies: item.registry_dependencies.clone(),
            files: locked_files,
        },
    })
}

/// Project-relative directory joined below the root (`.` is the root itself).
pub fn join_dir(root: &Path, dir: &str) -> LibResult<PathBuf> {
    if dir == "." {
        Ok(root.to_path_buf())
    } else {
        join_relative(root, dir)
    }
}

/// Human-readable list of every conflicting file in `plans`.
pub fn conflict_summary(plans: &[ItemPlan]) -> Option<String> {
    let mut lines: Vec<String> = Vec::new();
    for plan in plans {
        for file in plan.conflicts() {
            lines.push(cstr!(
                "  {}:{} {} ({})",
                plan.kind,
                plan.name,
                file.path,
                file.action.as_str()
            ));
        }
    }
    (!lines.is_empty()).then(|| String::from(lines.join("\n")))
}

/// Write every planned change, then record the new lock entries.
///
/// Refuses (writing nothing) when a conflict exists and `force` is false.
pub fn apply(
    plans: &[ItemPlan],
    root: &Path,
    lockfile: &mut Lockfile,
    lock_path: &Path,
    force: bool,
    force_flag: &str,
) -> LibResult<()> {
    if !force && let Some(summary) = conflict_summary(plans) {
        return Err(LibError::new(cstr!(
            "refusing to overwrite locally modified files (re-run with {force_flag}):\n{summary}"
        )));
    }
    for plan in plans {
        let base = join_dir(root, &plan.dir)?;
        for file in &plan.files {
            match (file.action, &file.contents) {
                (FileAction::Create | FileAction::Update | FileAction::Conflict, Some(bytes)) => {
                    write_file(&file.target, bytes)?;
                }
                (FileAction::Delete | FileAction::ConflictDelete, _) => {
                    remove_file_and_prune(&file.target, &base)?;
                }
                _ => {}
            }
        }
        lockfile.upsert(plan.lock.clone());
    }
    lockfile.write(lock_path)
}
