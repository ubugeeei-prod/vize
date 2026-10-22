//! Package-link overlap without comparing every link to every other link.

use std::path::{Path, PathBuf};

use vize_carton::{FxHashMap, FxHashSet};

/// Link paths indexed for ancestor lookups and one descendant probe.
pub(super) struct LinkPathIndex {
    set: FxHashSet<PathBuf>,
    sorted: Vec<PathBuf>,
}

impl LinkPathIndex {
    pub(super) fn from_links(links: &FxHashMap<PathBuf, PathBuf>) -> Self {
        let mut sorted: Vec<PathBuf> = links.keys().cloned().collect();
        sorted.sort();
        let set = sorted.iter().cloned().collect();
        Self { set, sorted }
    }

    /// `path` is this link or sits under one. Ancestor walk, not a link scan.
    pub(super) fn covers(&self, path: &Path) -> bool {
        path.ancestors().any(|ancestor| self.set.contains(ancestor))
    }

    /// Another link is an ancestor or descendant of `alias`, or an ancestor of
    /// `target` (including an exact hit on `target`).
    pub(super) fn alias_overlaps(&self, alias: &Path, target: &Path) -> bool {
        if alias
            .ancestors()
            .skip(1)
            .any(|ancestor| self.set.contains(ancestor))
        {
            return true;
        }
        if self.has_strict_descendant(alias) {
            return true;
        }
        if target != alias && self.set.contains(target) {
            return true;
        }
        // `target.starts_with(alias)` is not an overlap: the previous scan
        // ignored the alias itself via `path != alias`.
        target
            .ancestors()
            .skip(1)
            .any(|ancestor| ancestor != alias && self.set.contains(ancestor))
    }

    fn has_strict_descendant(&self, prefix: &Path) -> bool {
        // The next path after `prefix` is the least strict descendant when
        // one exists; a later path cannot be a descendant if this one is not.
        let start = self.sorted.partition_point(|path| path.as_path() <= prefix);
        self.sorted
            .get(start)
            .is_some_and(|path| path.starts_with(prefix))
    }
}

pub(super) fn retired_link_ancestor(retired: &FxHashSet<PathBuf>, path: &Path) -> bool {
    path.ancestors()
        .skip(1)
        .any(|ancestor| retired.contains(ancestor))
}
