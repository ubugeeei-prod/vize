//! Incremental virtual documents for one checker session.
//!
//! The first time a path is published it is `created`. A later edit of that
//! same path is `changed`. A path is `deleted` only after its file is removed.
//! Opening a different filename does not delete the previous virtual file.

use corsa::api::FileChangeSummary;
use std::path::{Path, PathBuf};
use vize_l0::{FxHashSet, String, ToCompactString};

use super::{errors::io_error_message, paths::path_to_wire};

/// Projection coordinates belong to the same retained document as its input hash.
/// Insert only after the checker has successfully accepted the document.
pub(super) struct OpenedDocument {
    pub(super) input_hash: u64,
    pub(super) rewritten_source: Option<String>,
    pub(super) import_source_map: vize_canon::ImportSourceMap,
}

/// Created, changed, and deleted paths sent on the last refresh.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct RefreshCounts {
    pub(super) created: usize,
    pub(super) changed: usize,
    pub(super) deleted: usize,
}

impl RefreshCounts {
    pub(super) fn from_summary(summary: &FileChangeSummary) -> Self {
        Self {
            created: summary.created.len(),
            changed: summary.changed.len(),
            deleted: summary.deleted.len(),
        }
    }
}

#[derive(Default)]
pub(super) struct PublishedDocuments {
    wires: FxHashSet<String>,
}

impl PublishedDocuments {
    /// The spawn already opened `wire`, so its first edit is `changed`.
    pub(super) fn primed(wire: String) -> Self {
        let mut published = Self::default();
        published.wires.insert(wire);
        published
    }

    fn seen(&self, wire: &str) -> bool {
        self.wires.contains(wire)
    }

    /// Returns whether this is the first time `wire` is published.
    fn first_sight(&mut self, wire: &str) -> bool {
        self.wires.insert(wire.to_compact_string())
    }

    fn take(&mut self, wire: &str) -> bool {
        self.wires.remove(wire)
    }
}

/// Write `content` to `path` and classify that document.
///
/// `removed` lists files that were actually deleted. Other virtual files stay
/// on disk. Overlay sessions still upsert text separately, so a later edit is
/// not also sent as a disk `changed` entry.
pub(super) fn sync_stable_virtual_file(
    published: &mut PublishedDocuments,
    path: &Path,
    content: &str,
    overlay: bool,
    removed: &[PathBuf],
) -> Result<FileChangeSummary, String> {
    let mut summary = FileChangeSummary::default();
    for old in removed {
        if old == path {
            continue;
        }
        remove_one(published, old, &mut summary)?;
    }
    let wire = path_to_wire(path);
    let already = published.seen(wire.as_str());
    if !already || !overlay {
        write_virtual_file(path, content)?;
    }
    let created = published.first_sight(wire.as_str());
    if created {
        summary.created.push(wire.as_str().into());
    } else if !overlay {
        summary.changed.push(wire.as_str().into());
    }
    Ok(summary)
}

/// Delete `path` and record `deleted` only when this session had published it.
#[cfg(test)]
pub(super) fn remove_stable_virtual_file(
    published: &mut PublishedDocuments,
    path: &Path,
) -> Result<FileChangeSummary, String> {
    let mut summary = FileChangeSummary::default();
    remove_one(published, path, &mut summary)?;
    Ok(summary)
}

fn remove_one(
    published: &mut PublishedDocuments,
    path: &Path,
    summary: &mut FileChangeSummary,
) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }
    std::fs::remove_file(path).map_err(|error| {
        io_error_message("Failed to remove patina virtual TypeScript", path, &error)
    })?;
    let wire = path_to_wire(path);
    if published.take(wire.as_str()) {
        summary.deleted.push(wire.as_str().into());
    }
    Ok(())
}

fn write_virtual_file(path: &Path, content: &str) -> Result<(), String> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent).map_err(|error| {
            io_error_message("Failed to create patina virtual directory", path, &error)
        })?;
    }
    std::fs::write(path, content).map_err(|error| {
        io_error_message("Failed to write patina virtual TypeScript", path, &error)
    })
}

#[cfg(test)]
mod tests {
    use super::super::paths::path_to_wire;
    use super::{
        PublishedDocuments, RefreshCounts, remove_stable_virtual_file, sync_stable_virtual_file,
    };
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_CASE_ID: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn second_virtual_file_stays_and_counts_follow_sight_and_removal() {
        let root = case_dir();
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let first = root.join("src").join("A.vue.patina.ts");
        let second = root.join("src").join("B.ts.patina.ts");
        let mut published = PublishedDocuments::default();

        let created = sync_stable_virtual_file(
            &mut published,
            &first,
            "export const first = 1;\n",
            false,
            &[],
        )
        .expect("first virtual file should be created");
        assert_eq!(RefreshCounts::from_summary(&created), counts(1, 0, 0));

        let next = sync_stable_virtual_file(
            &mut published,
            &second,
            "export const second = 2;\n",
            false,
            &[],
        )
        .expect("second virtual file should be created");
        assert_eq!(RefreshCounts::from_summary(&next), counts(1, 0, 0));
        let first_text = std::fs::read_to_string(&first).unwrap();
        assert_eq!(first_text, "export const first = 1;\n");
        let first_remains = first.is_file();
        assert!(first_remains);

        let edited = sync_stable_virtual_file(
            &mut published,
            &first,
            "export const first = 3;\n",
            false,
            &[],
        )
        .expect("later edit should be changed");
        assert_eq!(RefreshCounts::from_summary(&edited), counts(0, 1, 0));
        let second_remains = second.is_file();
        assert!(second_remains);

        let deleted = remove_stable_virtual_file(&mut published, &second)
            .expect("removing a published file should be deleted");
        assert_eq!(RefreshCounts::from_summary(&deleted), counts(0, 0, 1));
        let second_gone = !second.exists();
        assert!(second_gone);
        let first_after_delete = first.is_file();
        assert!(first_after_delete);

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn primed_spawn_file_is_changed_not_created() {
        let root = case_dir();
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("src").join("A.vue.patina.ts");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, "").unwrap();
        let mut published = PublishedDocuments::primed(path_to_wire(&path));

        let edited = sync_stable_virtual_file(
            &mut published,
            &path,
            "export const first = 1;\n",
            false,
            &[],
        )
        .expect("the primed file should be edited in place");
        assert_eq!(RefreshCounts::from_summary(&edited), counts(0, 1, 0));
        let kept = path.is_file();
        assert!(kept);
        let text = std::fs::read_to_string(&path).unwrap();
        assert_eq!(text, "export const first = 1;\n");

        let _ = std::fs::remove_dir_all(&root);
    }

    fn counts(created: usize, changed: usize, deleted: usize) -> RefreshCounts {
        RefreshCounts {
            created,
            changed,
            deleted,
        }
    }

    fn case_dir() -> std::path::PathBuf {
        let id = NEXT_CASE_ID.fetch_add(1, Ordering::Relaxed);
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("vize-tests")
            .join(format!("patina-virtual-docs-{}-{id}", std::process::id()))
    }
}
