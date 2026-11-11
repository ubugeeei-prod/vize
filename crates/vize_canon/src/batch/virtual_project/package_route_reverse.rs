//! Changed path -> package route bindings that consulted it.
//!
//! Every importer of one package route carries the same reachability inputs.
//! Indexing them per binding cost bindings x inputs hash inserts (#9727), so a
//! shared set is indexed once and owns the keys of the bindings that use it.
//!
//! Only an incremental refresh asks which bindings a changed path affects. A
//! one-shot check never does, yet indexing every input of every binding was
//! most of its serial route setup, and freeing the index most of its
//! teardown. The index is therefore built from the authoritative bindings on
//! the first query and maintained from then on; until then an insert or a
//! removal has nothing to update.

#![expect(
    clippy::disallowed_types,
    reason = "shared reachability sets are indexed by allocation identity"
)]

use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

use vize_carton::{FxHashMap, FxHashSet};

use crate::package_route::PackageRouteKey;

/// A set is alive while any binding holds it, so its address is a stable id.
type SetId = usize;

fn set_id(set: &Arc<[PathBuf]>) -> SetId {
    Arc::as_ptr(set).cast::<PathBuf>() as usize
}

struct SharedSet {
    paths: Arc<[PathBuf]>,
    keys: FxHashSet<PackageRouteKey>,
}

/// Reverse index over the bindings of one project, built on first use.
#[derive(Default)]
pub(crate) struct PackageRouteReverse {
    index: OnceLock<ReverseIndex>,
}

impl PackageRouteReverse {
    /// Forget the index; the next query rebuilds it from the bindings then.
    pub(crate) fn clear(&mut self) {
        self.index = OnceLock::new();
    }

    pub(crate) fn insert(&mut self, key: &PackageRouteKey, binding: &crate::PackageRouteBinding) {
        if let Some(index) = self.index.get_mut() {
            index.insert(key, binding);
        }
    }

    pub(crate) fn remove(&mut self, key: &PackageRouteKey, binding: &crate::PackageRouteBinding) {
        if let Some(index) = self.index.get_mut() {
            index.remove(key, binding);
        }
    }

    /// The index over `bindings`, which must be every binding inserted and
    /// not removed since the last [`Self::clear`].
    pub(crate) fn index<'a>(
        &self,
        bindings: impl IntoIterator<Item = (&'a PackageRouteKey, &'a crate::PackageRouteBinding)>,
    ) -> &ReverseIndex {
        self.index.get_or_init(|| {
            let mut index = ReverseIndex::default();
            for (key, binding) in bindings {
                index.insert(key, binding);
            }
            index
        })
    }

    #[cfg(test)]
    fn is_built(&self) -> bool {
        self.index.get().is_some()
    }
}

#[derive(Default)]
pub(crate) struct ReverseIndex {
    own: FxHashMap<PathBuf, FxHashSet<PackageRouteKey>>,
    sets: FxHashMap<SetId, SharedSet>,
    shared: FxHashMap<PathBuf, FxHashSet<SetId>>,
}

impl ReverseIndex {
    fn insert(&mut self, key: &PackageRouteKey, binding: &crate::PackageRouteBinding) {
        for path in &binding.invalidation_paths {
            self.own
                .entry(path.clone())
                .or_default()
                .insert(key.clone());
        }
        for set in &binding.shared_invalidation_paths {
            let id = set_id(set);
            let entry = self.sets.entry(id).or_insert_with(|| {
                for path in set.iter() {
                    self.shared.entry(path.clone()).or_default().insert(id);
                }
                SharedSet {
                    paths: Arc::clone(set),
                    keys: FxHashSet::default(),
                }
            });
            entry.keys.insert(key.clone());
        }
    }

    fn remove(&mut self, key: &PackageRouteKey, binding: &crate::PackageRouteBinding) {
        for path in &binding.invalidation_paths {
            let empty = self.own.get_mut(path).is_some_and(|keys| {
                keys.remove(key);
                keys.is_empty()
            });
            if empty {
                self.own.remove(path);
            }
        }
        for set in &binding.shared_invalidation_paths {
            let id = set_id(set);
            let unused = self.sets.get_mut(&id).is_some_and(|entry| {
                entry.keys.remove(key);
                entry.keys.is_empty()
            });
            if !unused {
                continue;
            }
            let Some(entry) = self.sets.remove(&id) else {
                continue;
            };
            for path in entry.paths.iter() {
                let empty = self.shared.get_mut(path).is_some_and(|ids| {
                    ids.remove(&id);
                    ids.is_empty()
                });
                if empty {
                    self.shared.remove(path);
                }
            }
        }
    }

    pub(crate) fn extend_keys_for(&self, path: &Path, out: &mut FxHashSet<PackageRouteKey>) {
        if let Some(keys) = self.own.get(path) {
            out.extend(keys.iter().cloned());
        }
        for id in self.shared.get(path).into_iter().flatten() {
            if let Some(entry) = self.sets.get(id) {
                out.extend(entry.keys.iter().cloned());
            }
        }
    }

    /// Every indexed path with its bindings, own and shared alike.
    #[cfg(test)]
    pub(crate) fn flattened(
        &self,
    ) -> std::collections::BTreeMap<PathBuf, std::collections::BTreeSet<PackageRouteKey>> {
        let mut flat = std::collections::BTreeMap::<_, std::collections::BTreeSet<_>>::new();
        for path in self.own.keys().chain(self.shared.keys()) {
            let mut keys = FxHashSet::default();
            self.extend_keys_for(path, &mut keys);
            flat.entry(path.clone()).or_default().extend(keys);
        }
        flat
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn binding(
        importer: &str,
        own: &[&str],
        shared: &[&Arc<[PathBuf]>],
    ) -> crate::PackageRouteBinding {
        crate::PackageRouteBinding {
            importer_path: PathBuf::from(importer),
            specifier: "pkg".into(),
            occurrence_mode: crate::PackageResolutionMode::Import,
            context: crate::PackageResolutionContext::new(
                Some("bundler"),
                crate::PackageResolutionMode::Import,
                ["import"],
            ),
            route: None,
            invalidation_paths: own.iter().map(PathBuf::from).collect(),
            shared_invalidation_paths: shared.iter().map(|set| Arc::clone(set)).collect(),
        }
    }

    fn keys_for(index: &ReverseIndex, path: &str) -> Vec<PathBuf> {
        let mut keys = FxHashSet::default();
        index.extend_keys_for(Path::new(path), &mut keys);
        let mut importers = keys
            .into_iter()
            .map(|key| key.importer_path)
            .collect::<Vec<_>>();
        importers.sort();
        importers
    }

    #[test]
    fn shared_sets_are_indexed_once_and_released_with_their_last_binding() {
        let set: Arc<[PathBuf]> = ["/w/pkg/package.json", "/w/pkg/src/index.ts"]
            .map(PathBuf::from)
            .into();
        let a = binding("/w/a.ts", &["/w/a.ts"], &[&set]);
        let b = binding("/w/b.ts", &["/w/b.ts"], &[&set]);
        let mut index = ReverseIndex::default();
        index.insert(&a.key(), &a);
        index.insert(&b.key(), &b);

        assert_eq!(index.sets.len(), 1);
        assert_eq!(
            keys_for(&index, "/w/pkg/src/index.ts"),
            [PathBuf::from("/w/a.ts"), PathBuf::from("/w/b.ts")]
        );
        assert_eq!(keys_for(&index, "/w/a.ts"), [PathBuf::from("/w/a.ts")]);

        index.remove(&a.key(), &a);
        assert_eq!(
            keys_for(&index, "/w/pkg/package.json"),
            [PathBuf::from("/w/b.ts")]
        );
        assert!(keys_for(&index, "/w/a.ts").is_empty());

        index.remove(&b.key(), &b);
        assert!(index.sets.is_empty() && index.shared.is_empty() && index.own.is_empty());
    }

    #[test]
    fn the_index_is_built_from_the_bindings_on_first_query_and_maintained_after() {
        let set: Arc<[PathBuf]> = ["/w/pkg/package.json"].map(PathBuf::from).into();
        let a = binding("/w/a.ts", &["/w/a.ts", "/w/tsconfig.json"], &[&set]);
        let b = binding("/w/b.ts", &["/w/b.ts", "/w/tsconfig.json"], &[]);
        let c = binding("/w/c.ts", &["/w/c.ts", "/w/tsconfig.json"], &[&set]);
        let mut bindings = FxHashMap::default();
        let mut reverse = PackageRouteReverse::default();
        for binding in [&a, &b] {
            reverse.insert(&binding.key(), binding);
            bindings.insert(binding.key(), binding.clone());
        }
        assert!(!reverse.is_built());

        let index = reverse.index(&bindings);
        assert_eq!(
            keys_for(index, "/w/tsconfig.json"),
            [PathBuf::from("/w/a.ts"), PathBuf::from("/w/b.ts")]
        );
        assert_eq!(
            keys_for(index, "/w/pkg/package.json"),
            [PathBuf::from("/w/a.ts")]
        );

        // Once built, inserts and removals keep it equal to a fresh build.
        reverse.insert(&c.key(), &c);
        bindings.insert(c.key(), c.clone());
        reverse.remove(&a.key(), &a);
        bindings.remove(&a.key());
        let fresh = PackageRouteReverse::default();
        assert_eq!(
            reverse.index(&bindings).flattened(),
            fresh.index(&bindings).flattened()
        );
        assert_eq!(
            keys_for(reverse.index(&bindings), "/w/pkg/package.json"),
            [PathBuf::from("/w/c.ts")]
        );

        // A cleared index is rebuilt from whatever the bindings are then.
        reverse.clear();
        assert!(!reverse.is_built());
        bindings.remove(&b.key());
        assert_eq!(
            keys_for(reverse.index(&bindings), "/w/tsconfig.json"),
            [PathBuf::from("/w/c.ts")]
        );
    }
}
