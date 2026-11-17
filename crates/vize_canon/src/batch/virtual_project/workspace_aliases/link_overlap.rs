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

    pub(super) fn alias_overlap_path(&self, alias: &Path, target: &Path) -> Option<PathBuf> {
        if let Some(ancestor) = alias
            .ancestors()
            .skip(1)
            .find(|ancestor| self.set.contains(*ancestor))
        {
            return Some(ancestor.to_path_buf());
        }
        if let Some(descendant) = self.strict_descendant(alias) {
            return Some(descendant.clone());
        }
        if target != alias && self.set.contains(target) {
            return Some(target.to_path_buf());
        }
        // `target.starts_with(alias)` is not an overlap: the previous scan
        // ignored the alias itself via `path != alias`.
        target
            .ancestors()
            .skip(1)
            .find(|ancestor| *ancestor != alias && self.set.contains(*ancestor))
            .map(Path::to_path_buf)
    }

    fn strict_descendant(&self, prefix: &Path) -> Option<&PathBuf> {
        // The next path after `prefix` is the least strict descendant when
        // one exists; a later path cannot be a descendant if this one is not.
        let start = self.sorted.partition_point(|path| path.as_path() <= prefix);
        self.sorted
            .get(start)
            .filter(|path| path.starts_with(prefix))
    }
}

pub(super) fn retired_link_ancestor(retired: &FxHashSet<PathBuf>, path: &Path) -> bool {
    path.ancestors()
        .skip(1)
        .any(|ancestor| retired.contains(ancestor))
}
