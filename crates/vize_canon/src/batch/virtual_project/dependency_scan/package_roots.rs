//! Logical package membership for one dependency walk; targets remain canonical.
use std::path::{Component, Path, PathBuf};

use vize_carton::FxHashSet;

pub(super) struct PackageRoots {
    roots: FxHashSet<PathBuf>,
}

impl PackageRoots {
    pub(super) fn new<'a>(roots: impl Iterator<Item = &'a Path>) -> Self {
        Self {
            roots: roots.map(Path::to_path_buf).collect(),
        }
    }

    pub(super) fn for_importer<'a>(&self, importer: &'a Path) -> Vec<&'a Path> {
        if self.roots.contains(Path::new("")) {
            // starts_with("") accepts both importer and canonical target.
            return vec![Path::new("")];
        }
        if self.roots.is_empty() {
            return Vec::new();
        }
        let mut matching = importer
            .ancestors()
            .filter(|ancestor| self.roots.contains(*ancestor))
            .collect::<Vec<_>>();
        // Windows ancestors stop at C:\, while starts_with("C:") also accepts
        // that importer. Keep this prefix-only root without canonicalizing it.
        if let Some(Component::Prefix(prefix)) = importer.components().next() {
            let prefix = Path::new(prefix.as_os_str());
            if self.roots.contains(prefix) && !matching.contains(&prefix) {
                matching.push(prefix);
            }
        }
        matching
    }
}

#[cfg(test)]
mod tests;
